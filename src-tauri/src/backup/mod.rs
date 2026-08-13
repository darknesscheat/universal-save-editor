use crate::core::error::{Error, Result};
use crate::core::model::BackupEntry;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Copies of save files taken before they were modified.
///
/// Layout on disk:
/// ```text
/// backups/
///   pathogenic/
///     2026-08-12_01-30-42/
///       meta.json        <- where it came from, when
///       run_save.json    <- the original bytes, byte for byte
/// ```
/// The original filename is kept so a backup folder is understandable even if
/// this app is long gone from the machine.
pub struct BackupManager {
    root: PathBuf,
}

#[derive(Serialize, Deserialize)]
struct Meta {
    game_id: String,
    original_path: String,
    original_file_name: String,
    created: String,
}

impl BackupManager {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Copy `save_path` into the backup store before anything modifies it.
    ///
    /// Returns the backup id. Any failure here is fatal to the write that
    /// prompted it, we would rather refuse to edit than edit unprotected.
    pub fn create(&self, game_id: &str, save_path: &Path) -> Result<String> {
        let bytes = std::fs::read(save_path)
            .map_err(|e| Error::BackupFailed(format!("could not read the save: {e}")))?;

        let file_name = save_path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "save".into());

        let now = chrono::Local::now();
        let stamp = now.format("%Y-%m-%d_%H-%M-%S").to_string();

        // A second edit inside the same second must not clobber the first.
        let mut dir = self.game_dir(game_id).join(&stamp);
        let mut suffix = 1;
        while dir.exists() {
            dir = self.game_dir(game_id).join(format!("{stamp}_{suffix}"));
            suffix += 1;
        }

        std::fs::create_dir_all(&dir).map_err(|e| Error::BackupFailed(e.to_string()))?;
        std::fs::write(dir.join(&file_name), &bytes)
            .map_err(|e| Error::BackupFailed(e.to_string()))?;

        let meta = Meta {
            game_id: game_id.to_string(),
            original_path: save_path.to_string_lossy().into_owned(),
            original_file_name: file_name,
            created: now.format("%Y-%m-%d %H:%M:%S").to_string(),
        };
        std::fs::write(
            dir.join("meta.json"),
            serde_json::to_vec_pretty(&meta).map_err(|e| Error::BackupFailed(e.to_string()))?,
        )
        .map_err(|e| Error::BackupFailed(e.to_string()))?;

        let id = format!(
            "{game_id}/{}",
            dir.file_name().unwrap_or_default().to_string_lossy()
        );

        // Verify the copy is byte-identical before letting the caller proceed.
        let written = std::fs::read(dir.join(&meta.original_file_name))
            .map_err(|e| Error::BackupFailed(e.to_string()))?;
        if written != bytes {
            let _ = std::fs::remove_dir_all(&dir);
            return Err(Error::BackupFailed(
                "the copy did not match the original".into(),
            ));
        }

        Ok(id)
    }

    /// Newest first.
    pub fn list(&self, game_id: Option<&str>) -> Vec<BackupEntry> {
        let mut out = Vec::new();
        let games: Vec<PathBuf> = match game_id {
            Some(g) => vec![self.game_dir(g)],
            None => std::fs::read_dir(&self.root)
                .map(|it| {
                    it.flatten()
                        .map(|e| e.path())
                        .filter(|p| p.is_dir())
                        .collect()
                })
                .unwrap_or_default(),
        };

        for game in games {
            let Ok(entries) = std::fs::read_dir(&game) else {
                continue;
            };
            for entry in entries.flatten() {
                let dir = entry.path();
                if !dir.is_dir() {
                    continue;
                }
                let Some(meta) = read_meta(&dir) else {
                    continue;
                };
                let size = std::fs::metadata(dir.join(&meta.original_file_name))
                    .map(|m| m.len())
                    .unwrap_or(0);
                out.push(BackupEntry {
                    id: format!(
                        "{}/{}",
                        meta.game_id,
                        dir.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    game_id: meta.game_id,
                    original_path: meta.original_path,
                    created: meta.created,
                    size_bytes: size,
                });
            }
        }

        // Newest first, and `created` alone cannot decide that, because it has
        // second precision and several backups can share a second. The folder
        // name carries the disambiguating suffix `create` added, so it breaks
        // the tie in the same order the backups were made. Without this,
        // pruning could keep an older copy and drop a newer one.
        out.sort_by(|a, b| b.created.cmp(&a.created).then(b.id.cmp(&a.id)));
        out
    }

    /// Put a backup back where it came from.
    ///
    /// The save being replaced is itself backed up first, so restoring is as
    /// undoable as editing.
    pub fn restore(&self, id: &str) -> Result<String> {
        let dir = self.resolve(id)?;
        let meta = read_meta(&dir).ok_or_else(|| Error::BackupNotFound(id.to_string()))?;
        let source = dir.join(&meta.original_file_name);
        let bytes =
            std::fs::read(&source).map_err(|e| Error::BackupNotFound(format!("{id}: {e}")))?;

        let target = PathBuf::from(&meta.original_path);
        if target.exists() {
            self.create(&meta.game_id, &target)?;
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }

        crate::save::io::write_atomically(&target, &bytes)?;
        Ok(meta.original_path)
    }

    /// Drop the oldest backups for a game beyond `keep`.
    ///
    /// Every edit leaves a copy and nothing ever removed them, so the folder
    /// grew without limit. Pruning runs after a successful write, and only
    /// ever deletes this app's own backups, never anything the game made.
    ///
    /// Returns how many were removed.
    pub fn prune(&self, game_id: &str, keep: usize) -> usize {
        if keep == 0 {
            return 0;
        }
        let entries = self.list(Some(game_id));
        if entries.len() <= keep {
            return 0;
        }

        // `list` is newest-first, so everything past `keep` is fair game.
        entries
            .into_iter()
            .skip(keep)
            .filter(|e| self.delete(&e.id).is_ok())
            .count()
    }

    pub fn delete(&self, id: &str) -> Result<()> {
        let dir = self.resolve(id)?;
        std::fs::remove_dir_all(dir).map_err(|e| Error::Io(e.to_string()))
    }

    fn game_dir(&self, game_id: &str) -> PathBuf {
        self.root.join(sanitise(game_id))
    }

    /// Turn `"pathogenic/2026-08-12_01-30-42"` into a path, refusing anything
    /// that tries to climb out of the backup folder.
    fn resolve(&self, id: &str) -> Result<PathBuf> {
        let mut parts = id.split('/');
        let (Some(game), Some(stamp), None) = (parts.next(), parts.next(), parts.next()) else {
            return Err(Error::BackupNotFound(id.to_string()));
        };
        if game.is_empty() || stamp.is_empty() {
            return Err(Error::BackupNotFound(id.to_string()));
        }
        let dir = self.root.join(sanitise(game)).join(sanitise(stamp));
        if !dir.is_dir() {
            return Err(Error::BackupNotFound(id.to_string()));
        }
        Ok(dir)
    }
}

fn read_meta(dir: &Path) -> Option<Meta> {
    serde_json::from_slice(&std::fs::read(dir.join("meta.json")).ok()?).ok()
}

/// Keep ids to characters that cannot express a path traversal.
fn sanitise(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
        .collect::<String>()
        .replace("..", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (tempfile::TempDir, BackupManager, PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let mgr = BackupManager::new(tmp.path().join("backups"));
        let save = tmp.path().join("run_save.json");
        std::fs::write(&save, r#"{"player":{"money":100}}"#).unwrap();
        (tmp, mgr, save)
    }

    #[test]
    fn creates_a_byte_identical_backup() {
        let (_t, mgr, save) = setup();
        let id = mgr.create("demo", &save).unwrap();
        let listed = mgr.list(Some("demo"));
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].id, id);
        assert!(listed[0].size_bytes > 0);
    }

    #[test]
    fn restore_brings_the_original_bytes_back() {
        let (_t, mgr, save) = setup();
        let original = std::fs::read(&save).unwrap();
        let id = mgr.create("demo", &save).unwrap();

        std::fs::write(&save, r#"{"player":{"money":999999}}"#).unwrap();
        mgr.restore(&id).unwrap();

        assert_eq!(std::fs::read(&save).unwrap(), original);
    }

    #[test]
    fn restoring_also_backs_up_what_it_replaces() {
        let (_t, mgr, save) = setup();
        let id = mgr.create("demo", &save).unwrap();
        std::fs::write(&save, "modified").unwrap();
        mgr.restore(&id).unwrap();
        // The original backup plus the pre-restore snapshot.
        assert_eq!(mgr.list(Some("demo")).len(), 2);
    }

    #[test]
    fn two_backups_in_the_same_second_do_not_collide() {
        let (_t, mgr, save) = setup();
        let a = mgr.create("demo", &save).unwrap();
        let b = mgr.create("demo", &save).unwrap();
        assert_ne!(a, b);
        assert_eq!(mgr.list(Some("demo")).len(), 2);
    }

    #[test]
    fn backups_are_listed_newest_first() {
        let (_t, mgr, save) = setup();
        mgr.create("demo", &save).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1100));
        mgr.create("demo", &save).unwrap();
        let l = mgr.list(Some("demo"));
        assert!(l[0].created >= l[1].created);
    }

    #[test]
    fn path_traversal_ids_are_refused() {
        let (_t, mgr, save) = setup();
        mgr.create("demo", &save).unwrap();
        assert!(mgr.restore("../../etc/passwd").is_err());
        assert!(mgr.restore("demo/../../secrets").is_err());
        assert!(mgr.restore("demo").is_err());
    }

    #[test]
    fn pruning_keeps_the_newest_and_drops_the_rest() {
        let (_t, mgr, save) = setup();
        let ids: Vec<String> = (0..5).map(|_| mgr.create("demo", &save).unwrap()).collect();
        assert_eq!(mgr.list(Some("demo")).len(), 5);

        let removed = mgr.prune("demo", 2);
        assert_eq!(removed, 3);

        // Specifically the two most recent, not any two. All five land in the
        // same second, so this only holds because ties break on the folder
        // name rather than being left to whatever order the directory is read
        // in.
        let left: Vec<String> = mgr.list(Some("demo")).into_iter().map(|b| b.id).collect();
        assert_eq!(left, vec![ids[4].clone(), ids[3].clone()]);
    }

    #[test]
    fn pruning_does_nothing_when_there_is_room() {
        let (_t, mgr, save) = setup();
        mgr.create("demo", &save).unwrap();
        assert_eq!(mgr.prune("demo", 20), 0);
        assert_eq!(mgr.list(Some("demo")).len(), 1);
    }

    #[test]
    fn a_keep_of_zero_is_treated_as_no_pruning() {
        let (_t, mgr, save) = setup();
        mgr.create("demo", &save).unwrap();
        assert_eq!(mgr.prune("demo", 0), 0);
        assert_eq!(mgr.list(Some("demo")).len(), 1, "pruning wiped everything");
    }

    #[test]
    fn pruning_one_game_leaves_another_alone() {
        let (_t, mgr, save) = setup();
        for _ in 0..3 {
            mgr.create("demo", &save).unwrap();
            mgr.create("other", &save).unwrap();
        }
        mgr.prune("demo", 1);
        assert_eq!(mgr.list(Some("demo")).len(), 1);
        assert_eq!(mgr.list(Some("other")).len(), 3);
    }

    #[test]
    fn deleting_removes_it_from_the_history() {
        let (_t, mgr, save) = setup();
        let id = mgr.create("demo", &save).unwrap();
        mgr.delete(&id).unwrap();
        assert!(mgr.list(Some("demo")).is_empty());
    }
}
