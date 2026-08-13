use crate::core::error::{Error, Result};
use std::io::Write;
use std::path::Path;

/// Replace a file's contents without ever leaving it half-written.
///
/// The sequence matters:
/// 1. write the new bytes to a temporary file **in the same folder** (a rename
///    is only atomic within one filesystem),
/// 2. flush and fsync so the data is really on disk, not just in a buffer,
/// 3. read the temporary file back and compare it byte for byte,
/// 4. rename it over the target, which the OS performs atomically.
///
/// If anything fails before step 4 the target still holds its original bytes
/// and the temporary file is cleaned up. A reader of the save file sees either
/// the whole old version or the whole new one, never a truncated mixture.
pub fn write_atomically(target: &Path, bytes: &[u8]) -> Result<()> {
    let dir = target
        .parent()
        .ok_or_else(|| Error::WriteFailed("the save has no parent folder".into()))?;

    let file_name = target
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "save".into());
    let temp = dir.join(format!(".{file_name}.use-tmp"));

    // Clean up anything a previous crashed run may have left behind.
    let _ = std::fs::remove_file(&temp);

    let result = (|| -> Result<()> {
        let mut f = std::fs::File::create(&temp).map_err(|e| Error::WriteFailed(e.to_string()))?;
        f.write_all(bytes)
            .map_err(|e| Error::WriteFailed(e.to_string()))?;
        f.flush().map_err(|e| Error::WriteFailed(e.to_string()))?;
        f.sync_all()
            .map_err(|e| Error::WriteFailed(e.to_string()))?;
        drop(f);

        let readback = std::fs::read(&temp).map_err(|e| Error::WriteFailed(e.to_string()))?;
        if readback != bytes {
            return Err(Error::WriteFailed(
                "the data written to disk did not match what was expected".into(),
            ));
        }

        // On both Windows and Unix this replaces an existing file atomically.
        std::fs::rename(&temp, target).map_err(|e| Error::WriteFailed(e.to_string()))
    })();

    if result.is_err() {
        let _ = std::fs::remove_file(&temp);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_new_content() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("save.json");
        write_atomically(&f, b"hello").unwrap();
        assert_eq!(std::fs::read(&f).unwrap(), b"hello");
    }

    #[test]
    fn replaces_existing_content() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("save.json");
        std::fs::write(&f, b"old contents that are longer").unwrap();
        write_atomically(&f, b"new").unwrap();
        assert_eq!(std::fs::read(&f).unwrap(), b"new");
    }

    #[test]
    fn leaves_no_temporary_files_behind() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("save.json");
        write_atomically(&f, b"x").unwrap();
        let leftovers: Vec<_> = std::fs::read_dir(tmp.path())
            .unwrap()
            .flatten()
            .filter(|e| e.file_name().to_string_lossy().contains("use-tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn a_failed_write_leaves_the_original_intact() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("gone");
        std::fs::create_dir(&dir).unwrap();
        let f = dir.join("save.json");
        std::fs::write(&f, b"original").unwrap();

        // Target a path whose parent does not exist: creation of the temp file
        // fails, and nothing else is touched.
        let bad = tmp.path().join("missing-dir").join("save.json");
        assert!(write_atomically(&bad, b"new").is_err());
        assert_eq!(std::fs::read(&f).unwrap(), b"original");
    }

    #[test]
    fn recovers_from_a_stale_temporary_file() {
        let tmp = tempfile::tempdir().unwrap();
        let f = tmp.path().join("save.json");
        std::fs::write(tmp.path().join(".save.json.use-tmp"), b"junk from a crash").unwrap();
        write_atomically(&f, b"fresh").unwrap();
        assert_eq!(std::fs::read(&f).unwrap(), b"fresh");
    }
}
