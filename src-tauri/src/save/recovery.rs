//! Copies of a save that the *game* made, not this app.
//!
//! Games keep their own safety nets and never mention them. Pathogenic writes
//! `save.json.bak` and `.bak2` beside the live file, and quarantines anything
//! it refuses to load as `corrupted_<unix-time>_save.json`. A player whose save
//! has gone wrong is standing next to three or four working copies with no way
//! to reach them.
//!
//! These are only ever read. Restoring one goes through the ordinary backup
//! manager, so the file being replaced is preserved first.

use crate::core::model::RecoveryFile;
use crate::core::paths;
use crate::plugins::manifest::Manifest;
use std::path::Path;

/// Find the game's own copies sitting beside `save_path`.
///
/// Newest first, and the live save itself is never included.
pub fn find_for(manifest: &Manifest, save_path: &Path) -> Vec<RecoveryFile> {
    let Some(dir) = save_path.parent() else {
        return Vec::new();
    };
    if manifest.recovery_patterns.is_empty() {
        return Vec::new();
    }

    let mut out = Vec::new();
    for pattern in &manifest.recovery_patterns {
        let full = dir.join(pattern);
        let Ok(hits) = glob::glob(&full.to_string_lossy()) else {
            continue;
        };
        for path in hits.flatten() {
            if !path.is_file() || path == save_path {
                continue;
            }
            let Ok(meta) = std::fs::metadata(&path) else {
                continue;
            };
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();

            out.push(RecoveryFile {
                path: path.to_string_lossy().into_owned(),
                name: name.clone(),
                // A quarantine file carries the moment it was set aside in its
                // own name; anything else falls back to the file's timestamp.
                created: timestamp_in(&name)
                    .or_else(|| modified_time(&meta))
                    .unwrap_or_default(),
                size_bytes: meta.len(),
            });
        }
    }

    out.sort_by(|a, b| b.created.cmp(&a.created));
    out.dedup_by(|a, b| a.path == b.path);
    out
}

/// Pull a Unix timestamp out of a name like `corrupted_1786483054_save.json`.
fn timestamp_in(name: &str) -> Option<String> {
    let digits: String = name
        .split(|c: char| !c.is_ascii_digit())
        .max_by_key(|part| part.len())?
        .to_string();

    // Ten digits is a plausible second-precision Unix time; shorter runs are
    // version numbers and slot indices.
    if digits.len() < 9 || digits.len() > 11 {
        return None;
    }
    let secs: i64 = digits.parse().ok()?;
    let dt = chrono::DateTime::from_timestamp(secs, 0)?.with_timezone(&chrono::Local);
    Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
}

fn modified_time(meta: &std::fs::Metadata) -> Option<String> {
    let t = meta.modified().ok()?;
    let dt: chrono::DateTime<chrono::Local> = t.into();
    Some(dt.format("%Y-%m-%d %H:%M:%S").to_string())
}

/// Confine a recovery path to the folder of the save it belongs to.
///
/// The GUI only ever sends back a path this module produced, but a command is
/// a public boundary.
pub fn is_beside(save_path: &Path, candidate: &Path) -> bool {
    let (Some(a), Some(b)) = (save_path.parent(), candidate.parent()) else {
        return false;
    };
    let canon = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
    canon(a) == canon(b)
}

/// Where a plugin's saves may live at all, used to reject a path that names a
/// folder this plugin has nothing to do with.
pub fn within_plugin_reach(manifest: &Manifest, candidate: &Path) -> bool {
    let platform = paths::current_platform();
    manifest.save_locations.iter().any(|loc| {
        if !loc.platforms.is_empty() && !loc.platforms.iter().any(|p| p == platform) {
            return false;
        }
        match paths::expand(&loc.root) {
            Ok(root) => {
                let canon = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
                canon(candidate).starts_with(canon(&root))
            }
            Err(_) => false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        serde_json::from_str(
            r#"{"id":"t","name":"T","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
              "recovery_patterns":["save.json.bak","save.json.bak2","corrupted_*_save.json"],
              "groups":[{"id":"g","label":"G","fields":[
                {"id":"a","label":"A","pointer":"/a","type":"integer"}]}]}"#,
        )
        .unwrap()
    }

    #[test]
    fn finds_the_games_own_copies() {
        let tmp = tempfile::tempdir().unwrap();
        let save = tmp.path().join("save.json");
        for name in [
            "save.json",
            "save.json.bak",
            "save.json.bak2",
            "corrupted_1786483054_save.json",
            "unrelated.txt",
        ] {
            std::fs::write(tmp.path().join(name), "{}").unwrap();
        }

        let found = find_for(&manifest(), &save);
        let names: Vec<&str> = found.iter().map(|f| f.name.as_str()).collect();

        assert_eq!(found.len(), 3, "got {names:?}");
        assert!(names.contains(&"save.json.bak"));
        assert!(names.contains(&"corrupted_1786483054_save.json"));
        assert!(!names.contains(&"save.json"), "the live save was offered");
        assert!(!names.contains(&"unrelated.txt"));
    }

    #[test]
    fn a_quarantine_timestamp_is_read_from_the_name() {
        let when = timestamp_in("corrupted_1786483054_save.json").unwrap();
        assert!(when.starts_with("2026-"), "got {when}");
    }

    #[test]
    fn short_numbers_in_a_name_are_not_mistaken_for_a_timestamp() {
        assert!(timestamp_in("save.json.bak2").is_none());
        assert!(timestamp_in("profile_1.json").is_none());
    }

    #[test]
    fn a_plugin_with_no_patterns_finds_nothing() {
        let m: Manifest = serde_json::from_str(
            r#"{"id":"t","name":"T","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
              "groups":[{"id":"g","label":"G","fields":[
                {"id":"a","label":"A","pointer":"/a","type":"integer"}]}]}"#,
        )
        .unwrap();
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("save.json.bak"), "{}").unwrap();
        assert!(find_for(&m, &tmp.path().join("save.json")).is_empty());
    }

    #[test]
    fn a_file_from_another_folder_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let a = tmp.path().join("a");
        let b = tmp.path().join("b");
        std::fs::create_dir_all(&a).unwrap();
        std::fs::create_dir_all(&b).unwrap();

        assert!(is_beside(&a.join("save.json"), &a.join("save.json.bak")));
        assert!(!is_beside(&a.join("save.json"), &b.join("save.json.bak")));
    }
}
