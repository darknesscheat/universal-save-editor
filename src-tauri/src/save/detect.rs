use crate::core::error::{Error, Result};
use crate::core::model::SaveSummary;
use crate::core::paths;
use crate::plugins::adapter;
use crate::plugins::manifest::{IdentifyRule, Manifest};
use serde_json::Value;
use std::path::{Path, PathBuf};

/// Find every save file this plugin knows how to open.
///
/// Files that match the glob but fail to parse or fail the plugin's `identify`
/// rules are skipped silently: a folder often holds `.bak` copies and
/// unrelated config files, and listing those as broken saves would be noise.
pub fn find_saves(manifest: &Manifest, locale: &str) -> Result<Vec<SaveSummary>> {
    let platform = paths::current_platform();
    let mut out = Vec::new();

    for location in &manifest.save_locations {
        if !location.platforms.is_empty() && !location.platforms.iter().any(|p| p == platform) {
            continue;
        }
        let root = match paths::expand(&location.root) {
            Ok(r) => r,
            // A manifest may legitimately describe a folder that does not
            // exist on this machine; that is not an error.
            Err(_) => continue,
        };
        if !root.is_dir() {
            continue;
        }

        let pattern = root.join(&location.pattern);
        let entries = match glob::glob(&pattern.to_string_lossy()) {
            Ok(e) => e,
            Err(e) => {
                return Err(Error::PluginLoad(format!(
                    "invalid save pattern '{}': {e}",
                    location.pattern
                )))
            }
        };

        let rules = location.identify.as_ref().unwrap_or(&manifest.identify);
        for path in entries.flatten() {
            if !path.is_file() {
                continue;
            }
            if let Some(summary) = summarise(manifest, location, rules, &path, locale) {
                out.push(summary);
            }
        }
    }

    out.sort_by(|a, b| b.modified.cmp(&a.modified));
    out.dedup_by(|a, b| a.path == b.path);
    Ok(out)
}

/// Read a candidate file and describe it, or return `None` if it is not a save
/// for this game.
fn summarise(
    manifest: &Manifest,
    location: &crate::plugins::manifest::SaveLocation,
    rules: &[IdentifyRule],
    path: &Path,
    locale: &str,
) -> Option<SaveSummary> {
    let bytes = std::fs::read(path).ok()?;
    let adapter = adapter::adapter_for(&manifest.format).ok()?;
    let doc = adapter.parse(&bytes).ok()?;

    if !matches_rules(rules, &doc) {
        return None;
    }

    let meta = std::fs::metadata(path).ok()?;
    let modified = meta
        .modified()
        .ok()
        .map(|t| {
            let dt: chrono::DateTime<chrono::Local> = t.into();
            dt.format("%Y-%m-%d %H:%M").to_string()
        })
        .unwrap_or_default();

    // A location's own label wins: it distinguishes a run-in-progress from a
    // profile when both live in the same folder.
    let title = location
        .label
        .as_ref()
        .map(|l| crate::core::i18n::pick(l, &location.label_i18n, locale).to_string())
        .or_else(|| {
            manifest
                .label
                .title_pointer
                .as_ref()
                .and_then(|p| doc.pointer(p))
                .map(render_scalar)
                .filter(|s| !s.is_empty())
        })
        .unwrap_or_else(|| default_title(path));

    // Prefix with the containing folder so several profiles are told apart.
    let folder = path
        .parent()
        .and_then(|p| p.file_name())
        .map(|s| s.to_string_lossy().into_owned());

    let detail = manifest
        .label
        .subtitle_pointer
        .as_ref()
        .and_then(|p| doc.pointer(p))
        .map(render_scalar)
        .filter(|s| !s.is_empty())
        .map(|s| format!("{}{}", manifest.label.subtitle_prefix, s));

    let subtitle = match (folder, detail) {
        (Some(f), Some(d)) => format!("{f} · {d}"),
        (Some(f), None) => f,
        (None, Some(d)) => d,
        (None, None) => String::new(),
    };

    Some(SaveSummary {
        path: path.to_string_lossy().into_owned(),
        title,
        subtitle,
        modified,
        size_bytes: meta.len(),
    })
}

/// Every rule's pointer must resolve, otherwise this is someone else's file.
pub fn matches_rules(rules: &[IdentifyRule], doc: &Value) -> bool {
    rules
        .iter()
        .all(|rule| doc.pointer(&rule.pointer).is_some())
}

/// Does this document look like a save this plugin handles?
///
/// A plugin may cover several kinds of file, a run in progress and a permanent
/// profile, say, so the document only has to satisfy one location's rules.
pub fn identifies_as(manifest: &Manifest, doc: &Value) -> bool {
    let mut any_location_rule = false;
    for loc in &manifest.save_locations {
        if let Some(rules) = &loc.identify {
            any_location_rule = true;
            if matches_rules(rules, doc) {
                return true;
            }
        }
    }
    // Locations without their own rules fall back to the manifest-level set.
    let has_fallback_location = manifest.save_locations.iter().any(|l| l.identify.is_none());

    if has_fallback_location || !any_location_rule {
        return matches_rules(&manifest.identify, doc);
    }
    false
}

/// Fall back to a human-ish name built from the path, e.g. `profile_1 /
/// run_save`.
fn default_title(path: &Path) -> String {
    let file = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Save".into());
    match path.parent().and_then(|p| p.file_name()) {
        Some(dir) => format!("{} / {}", dir.to_string_lossy(), file),
        None => file,
    }
}

fn render_scalar(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Cheap identity of the file as it is on disk right now.
///
/// Used to notice that the game rewrote a save between the editor opening it
/// and the player pressing Save. Modification time alone is not enough, some
/// filesystems have coarse timestamps, so the size travels with it.
pub fn stamp_of(path: &Path) -> crate::core::model::SaveStamp {
    use crate::core::model::SaveStamp;

    let Ok(meta) = std::fs::metadata(path) else {
        return SaveStamp {
            modified_ms: 0,
            size_bytes: 0,
        };
    };
    let modified_ms = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0);

    SaveStamp {
        modified_ms,
        size_bytes: meta.len(),
    }
}

/// Does this save folder look like Steam keeps a cloud copy of it?
///
/// Steam drops `steam_autocloud.vdf` beside the files it syncs. When it is
/// there, a local edit can be reverted by the cloud on the next launch, worth
/// telling the player before they wonder why their change vanished.
pub fn is_cloud_synced(save_path: &Path) -> bool {
    save_path
        .parent()
        .map(|dir| {
            dir.join("steam_autocloud.vdf").is_file()
                // Godot keeps saves one level down, in profile_N/.
                || dir
                    .parent()
                    .map(|up| up.join("steam_autocloud.vdf").is_file())
                    .unwrap_or(false)
        })
        .unwrap_or(false)
}

/// Load and verify a single save the user picked.
pub fn load_document(manifest: &Manifest, path: &Path) -> Result<Value> {
    if !path.is_file() {
        return Err(Error::SaveMissing);
    }
    let bytes = std::fs::read(path).map_err(|e| Error::SaveRead(e.to_string()))?;
    let adapter = adapter::adapter_for(&manifest.format)?;
    let doc = adapter.parse(&bytes)?;

    if !identifies_as(manifest, &doc) {
        return Err(Error::SaveFormat {
            game: manifest.name.clone(),
        });
    }
    Ok(doc)
}

/// Resolve a user-supplied path against the plugin's declared save folders.
///
/// The GUI can only ever send back a path we handed it, but a command is a
/// public boundary: confining writes to the folders a plugin declares means a
/// malformed request cannot make the app overwrite an arbitrary file.
pub fn is_within_declared_locations(manifest: &Manifest, path: &Path) -> bool {
    let platform = paths::current_platform();
    manifest.save_locations.iter().any(|loc| {
        if !loc.platforms.is_empty() && !loc.platforms.iter().any(|p| p == platform) {
            return false;
        }
        match paths::expand(&loc.root) {
            Ok(root) => canonical_or_self(path).starts_with(canonical_or_self(&root)),
            Err(_) => false,
        }
    })
}

fn canonical_or_self(p: &Path) -> PathBuf {
    p.canonicalize().unwrap_or_else(|_| p.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest_with_identify() -> Manifest {
        serde_json::from_str(
            r#"{"id":"t","name":"Test Game","version":"1","format":"json",
                "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
                "identify":[{"pointer":"/player/hp"}],
                "groups":[{"id":"g","label":"G","fields":[
                  {"id":"hp","label":"HP","pointer":"/player/hp","type":"integer"}]}]}"#,
        )
        .unwrap()
    }

    #[test]
    fn identify_accepts_a_matching_document() {
        let m = manifest_with_identify();
        let doc = serde_json::json!({"player":{"hp":6}});
        assert!(identifies_as(&m, &doc));
    }

    #[test]
    fn identify_rejects_a_foreign_document() {
        let m = manifest_with_identify();
        let doc = serde_json::json!({"settings":{"volume":1}});
        assert!(!identifies_as(&m, &doc));
    }

    #[test]
    fn loading_a_foreign_file_is_an_error_not_an_empty_editor() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("other.json");
        std::fs::write(&f, r#"{"unrelated":true}"#).unwrap();
        let err = load_document(&manifest_with_identify(), &f).unwrap_err();
        assert!(err.to_string().contains("Test Game"));
    }

    #[test]
    fn loading_a_missing_file_reports_it_clearly() {
        let tmp = tempfile::tempdir().unwrap();
        let err =
            load_document(&manifest_with_identify(), &tmp.path().join("nope.json")).unwrap_err();
        assert!(matches!(err, Error::SaveMissing));
    }

    #[test]
    fn default_title_uses_parent_folder() {
        let t = default_title(Path::new("/x/profile_1/run_save.json"));
        assert_eq!(t, "profile_1 / run_save");
    }
}
