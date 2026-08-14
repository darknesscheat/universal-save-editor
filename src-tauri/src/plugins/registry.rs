use crate::core::error::{Error, Result};
use crate::plugins::manifest::Manifest;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// All plugins currently known to the app.
///
/// Plugins are plain folders containing a `manifest.json`. The registry scans
/// one or more roots: the bundled `plugins/` folder and the user's own plugin
/// folder, so a community plugin is installed by dropping in a directory.
#[derive(Default)]
pub struct Registry {
    plugins: BTreeMap<String, LoadedPlugin>,
    /// Folders that failed to load, surfaced in Settings rather than silently
    /// swallowed, a broken plugin should be visible, not invisible.
    problems: Vec<PluginProblem>,
}

#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    pub manifest: Manifest,
    pub dir: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct PluginProblem {
    pub path: String,
    pub reason: String,
}

impl Registry {
    /// Scan every root in order. Later roots win on id collision, which lets a
    /// user override a bundled plugin with their own copy.
    pub fn load(roots: &[PathBuf]) -> Self {
        let mut reg = Registry::default();
        for root in roots {
            if !root.is_dir() {
                continue;
            }
            let entries = match std::fs::read_dir(root) {
                Ok(e) => e,
                Err(e) => {
                    reg.problems.push(PluginProblem {
                        path: root.display().to_string(),
                        reason: e.to_string(),
                    });
                    continue;
                }
            };
            for entry in entries.flatten() {
                let dir = entry.path();
                if !dir.is_dir() {
                    continue;
                }
                match load_one(&dir) {
                    Ok(m) => {
                        reg.plugins
                            .insert(m.id.clone(), LoadedPlugin { manifest: m, dir });
                    }
                    Err(e) => reg.problems.push(PluginProblem {
                        path: dir.display().to_string(),
                        reason: e.to_string(),
                    }),
                }
            }
        }
        reg
    }

    pub fn get(&self, id: &str) -> Result<&LoadedPlugin> {
        self.plugins
            .get(id)
            .ok_or_else(|| Error::PluginNotFound(id.to_string()))
    }

    pub fn all(&self) -> impl Iterator<Item = &LoadedPlugin> {
        self.plugins.values()
    }

    pub fn problems(&self) -> &[PluginProblem] {
        &self.problems
    }
}

fn load_one(dir: &Path) -> Result<Manifest> {
    let file = dir.join("manifest.json");
    if !file.is_file() {
        return Err(Error::PluginLoad("no manifest.json in this folder".into()));
    }
    let text = std::fs::read_to_string(&file).map_err(|e| Error::PluginLoad(e.to_string()))?;
    let manifest: Manifest = serde_json::from_str(&text)
        .map_err(|e| Error::PluginLoad(format!("manifest.json: {e}")))?;

    validate(&manifest)?;
    Ok(manifest)
}

/// Catch the mistakes a plugin author is most likely to make, at load time,
/// rather than letting them surface as a confusing empty editor screen.
fn validate(m: &Manifest) -> Result<()> {
    if m.id.trim().is_empty() {
        return Err(Error::PluginLoad("'id' must not be empty".into()));
    }
    crate::plugins::adapter::adapter_for(&m.format)?;

    if m.save_locations.is_empty() {
        return Err(Error::PluginLoad(
            "'save_locations' must not be empty".into(),
        ));
    }
    if m.groups.is_empty() {
        return Err(Error::PluginLoad("'groups' must not be empty".into()));
    }

    let mut seen = std::collections::HashSet::new();
    for group in &m.groups {
        for f in &group.fields {
            check_pointer(&f.pointer)?;
            if !seen.insert(f.pointer.clone()) {
                return Err(Error::PluginLoad(format!(
                    "two fields share the pointer '{}'",
                    f.pointer
                )));
            }
            check_choices(m, f)?;
        }
        for l in &group.lists {
            check_pointer(&l.pointer)?;

            match l.source {
                crate::plugins::manifest::ListSource::Array => {
                    if l.fields.is_empty() {
                        return Err(Error::PluginLoad(format!(
                            "list '{}' has no fields to edit",
                            l.id
                        )));
                    }
                    for f in &l.fields {
                        check_pointer(&f.pointer)?;
                        check_choices(m, f)?;
                    }
                }
                crate::plugins::manifest::ListSource::Object => {
                    // Either the value is one thing, `entry` addresses it
                    // directly and needs no pointer of its own, or it is a
                    // record, described by `fields` exactly as an array row is.
                    match (&l.entry, l.fields.is_empty()) {
                        (Some(entry), _) => check_choices(m, entry)?,
                        (None, false) => {
                            for f in &l.fields {
                                check_pointer(&f.pointer)?;
                                check_choices(m, f)?;
                            }
                        }
                        (None, true) => {
                            return Err(Error::PluginLoad(format!(
                                "list '{}' reads an object but declares neither \
                                 'entry' nor 'fields'",
                                l.id
                            )))
                        }
                    }
                }
            }
        }
    }

    check_presets(m)?;
    Ok(())
}

/// A preset that changes nothing is always a mistake, and a silent one: it
/// appears as a button, the button works, and the save comes back untouched.
/// The way it happens is a misspelled key, which serde ignores.
fn check_presets(m: &Manifest) -> Result<()> {
    for p in &m.presets {
        if p.set.is_empty() && p.set_in_lists.is_empty() {
            return Err(Error::PluginLoad(format!(
                "preset '{}' would change nothing: it declares neither 'set' nor \
                 'set_in_lists'",
                p.id
            )));
        }
        for item in &p.set {
            check_pointer(&item.pointer)?;
        }
    }
    Ok(())
}

fn check_pointer(p: &str) -> Result<()> {
    if p.is_empty() || !p.starts_with('/') {
        return Err(Error::PluginLoad(format!(
            "pointer '{p}' must be a JSON pointer starting with '/'"
        )));
    }
    Ok(())
}

fn check_choices(m: &Manifest, f: &crate::plugins::manifest::Field) -> Result<()> {
    use crate::plugins::manifest::FieldKind;
    // Check the reference first: "unknown option set 'rarities'" tells a plugin
    // author exactly what to fix, where "has no options" would send them
    // looking in the wrong place.
    if let Some(r) = &f.options_ref {
        if !m.option_sets.contains_key(r) {
            return Err(Error::PluginLoad(format!(
                "field '{}' references unknown option set '{r}'",
                f.id
            )));
        }
    }
    if f.kind == FieldKind::Choice && m.choices(f).is_empty() {
        return Err(Error::PluginLoad(format!(
            "field '{}' is a choice but has no options",
            f.id
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_plugin(root: &Path, name: &str, body: &str) {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("manifest.json"), body).unwrap();
    }

    const GOOD: &str = r#"{
        "id":"demo","name":"Demo","version":"1.0.0","format":"json",
        "save_locations":[{"root":"{HOME}/demo","pattern":"*.json"}],
        "groups":[{"id":"g","label":"G","fields":[
            {"id":"money","label":"Money","pointer":"/money","type":"integer"}
        ]}]
    }"#;

    #[test]
    fn loads_a_valid_plugin() {
        let tmp = tempfile::tempdir().unwrap();
        write_plugin(tmp.path(), "demo", GOOD);
        let reg = Registry::load(&[tmp.path().to_path_buf()]);
        assert_eq!(reg.all().count(), 1);
        assert!(reg.problems().is_empty());
        assert_eq!(reg.get("demo").unwrap().manifest.name, "Demo");
    }

    #[test]
    fn broken_plugin_is_reported_not_swallowed() {
        let tmp = tempfile::tempdir().unwrap();
        write_plugin(tmp.path(), "bad", "{ not json");
        let reg = Registry::load(&[tmp.path().to_path_buf()]);
        assert_eq!(reg.all().count(), 0);
        assert_eq!(reg.problems().len(), 1);
    }

    #[test]
    fn one_broken_plugin_does_not_stop_the_others() {
        let tmp = tempfile::tempdir().unwrap();
        write_plugin(tmp.path(), "good", GOOD);
        write_plugin(tmp.path(), "bad", "{ not json");
        let reg = Registry::load(&[tmp.path().to_path_buf()]);
        assert_eq!(reg.all().count(), 1);
        assert_eq!(reg.problems().len(), 1);
    }

    #[test]
    fn duplicate_pointers_are_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        write_plugin(
            tmp.path(),
            "dup",
            r#"{"id":"d","name":"D","version":"1","format":"json",
               "save_locations":[{"root":"{HOME}/d","pattern":"*.json"}],
               "groups":[{"id":"g","label":"G","fields":[
                 {"id":"a","label":"A","pointer":"/x","type":"integer"},
                 {"id":"b","label":"B","pointer":"/x","type":"integer"}]}]}"#,
        );
        let reg = Registry::load(&[tmp.path().to_path_buf()]);
        assert!(reg.problems()[0].reason.contains("share the pointer"));
    }

    #[test]
    fn unknown_option_set_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        write_plugin(
            tmp.path(),
            "opt",
            r#"{"id":"o","name":"O","version":"1","format":"json",
               "save_locations":[{"root":"{HOME}/o","pattern":"*.json"}],
               "groups":[{"id":"g","label":"G","fields":[
                 {"id":"a","label":"A","pointer":"/x","type":"choice","options_ref":"nope"}]}]}"#,
        );
        assert!(reg_reason(&tmp).contains("unknown option set"));
    }

    /// Regression: two bundled plugins shipped a preset whose edits were under
    /// a key named `edits`, which the schema calls `set`. Serde ignored it, the
    /// button appeared, and pressing it changed nothing at all.
    #[test]
    fn a_preset_that_would_change_nothing_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        write_plugin(
            tmp.path(),
            "preset",
            r#"{"id":"p","name":"P","version":"1","format":"json",
               "save_locations":[{"root":"{HOME}/p","pattern":"*.json"}],
               "groups":[{"id":"g","label":"G","fields":[
                 {"id":"money","label":"Money","pointer":"/money","type":"integer"}]}],
               "presets":[{"id":"rich","label":"Rich",
                 "edits":[{"pointer":"/money","value":999}]}]}"#,
        );
        assert!(reg_reason(&tmp).contains("would change nothing"));
    }

    #[test]
    fn a_preset_that_sets_something_is_accepted() {
        let tmp = tempfile::tempdir().unwrap();
        write_plugin(
            tmp.path(),
            "preset",
            r#"{"id":"p","name":"P","version":"1","format":"json",
               "save_locations":[{"root":"{HOME}/p","pattern":"*.json"}],
               "groups":[{"id":"g","label":"G","fields":[
                 {"id":"money","label":"Money","pointer":"/money","type":"integer"}]}],
               "presets":[{"id":"rich","label":"Rich",
                 "set":[{"pointer":"/money","value":999}]}]}"#,
        );
        let reg = Registry::load(&[tmp.path().to_path_buf()]);
        assert!(reg.problems().is_empty(), "{:?}", reg.problems());
    }

    fn reg_reason(tmp: &tempfile::TempDir) -> String {
        Registry::load(&[tmp.path().to_path_buf()]).problems()[0]
            .reason
            .clone()
    }
}
