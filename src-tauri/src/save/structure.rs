//! Adding and removing rows of a list.
//!
//! Kept apart from ordinary field edits. Inserting or deleting an
//! element renumbers everything after it, so a batch that mixed
//! `/player/loadout/2/rarity` with "delete row 1" would write the rarity to
//! the wrong item. Structural changes therefore travel as their own request,
//! are written immediately through the same safety pipeline, and the editor
//! reloads afterwards with fresh indices.

use crate::backup::BackupManager;
use crate::core::error::{Error, Result};
use crate::core::model::{SaveStamp, WriteReport};
use crate::plugins::manifest::{ListField, ListSource, Manifest};
use crate::save::{detect, io};
use serde_json::Value;
use std::path::Path;

/// Which way a row is being changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RowChange {
    Add,
    Remove,
}

/// Add or remove one row of a list, then write the file.
///
/// `index` is ignored for [`RowChange::Add`], new rows go on the end, which is
/// the only position that does not disturb what the player is looking at.
pub fn change_row(
    manifest: &Manifest,
    backups: &BackupManager,
    save_path: &Path,
    list_id: &str,
    change: RowChange,
    index: usize,
    expected: Option<&SaveStamp>,
) -> Result<WriteReport> {
    if let Some(expected) = expected {
        if detect::stamp_of(save_path) != *expected {
            return Err(Error::SaveChangedOnDisk);
        }
    }

    let mut doc = detect::load_document(manifest, save_path)?;

    // Only a list the manifest declares *and* this document actually shows.
    let list = manifest
        .groups_for(&doc)
        .flat_map(|g| g.lists.iter())
        .find(|l| l.id == list_id)
        .ok_or_else(|| Error::UnknownField(list_id.to_string()))?
        .clone();

    if list.source != ListSource::Array {
        return Err(Error::ListNotEditable {
            list: list.label.clone(),
        });
    }

    let Some(Value::Array(items)) = doc.pointer_mut(&list.pointer) else {
        return Err(Error::UnknownField(list.pointer.clone()));
    };

    match change {
        RowChange::Add => add_row(&list, items)?,
        RowChange::Remove => remove_row(&list, items, index)?,
    }

    // Everything from here is the ordinary write pipeline: prove the result is
    // still a save the game would accept, back up, then swap atomically.
    let adapter = crate::plugins::adapter::adapter_for(&manifest.format)?;
    let bytes = adapter.write(&doc)?;
    let verify = adapter
        .parse(&bytes)
        .map_err(|e| Error::WriteFailed(format!("the rebuilt save did not parse ({e})")))?;
    if !detect::identifies_as(manifest, &verify) {
        return Err(Error::WriteFailed(
            "the rebuilt save no longer looks like a valid save file".into(),
        ));
    }

    let backup_id = backups.create(&manifest.id, save_path)?;
    io::write_atomically(save_path, &bytes)?;

    Ok(WriteReport {
        backup_id,
        changed_fields: 1,
        save_path: save_path.to_string_lossy().into_owned(),
        stamp: detect::stamp_of(save_path),
    })
}

fn add_row(list: &ListField, items: &mut Vec<Value>) -> Result<()> {
    if !list.allow_add {
        return Err(Error::ListNotEditable {
            list: list.label.clone(),
        });
    }
    let Some(template) = &list.new_item else {
        return Err(Error::PluginLoad(format!(
            "list '{}' allows adding but declares no 'new_item'",
            list.id
        )));
    };
    // Count only what the player can see, so a filtered list's limit means
    // what it looks like it means.
    if let Some(max) = list.max_items {
        if visible(list, items) >= max {
            return Err(Error::ListFull {
                list: list.label.clone(),
                max,
            });
        }
    }
    items.push(template.clone());
    Ok(())
}

fn remove_row(list: &ListField, items: &mut Vec<Value>, index: usize) -> Result<()> {
    if !list.allow_remove {
        return Err(Error::ListNotEditable {
            list: list.label.clone(),
        });
    }
    if index >= items.len() {
        return Err(Error::UnknownField(format!("{}/{index}", list.pointer)));
    }
    // A hidden row is not the player's to delete: they never saw it.
    if let Some(filter) = &list.item_filter {
        if !filter.matches(&items[index]) {
            return Err(Error::UnknownField(format!("{}/{index}", list.pointer)));
        }
    }
    if let Some(min) = list.min_items {
        if visible(list, items) <= min {
            return Err(Error::ListAtMinimum {
                list: list.label.clone(),
                min,
            });
        }
    }
    items.remove(index);
    Ok(())
}

fn visible(list: &ListField, items: &[Value]) -> usize {
    match &list.item_filter {
        Some(f) => items.iter().filter(|i| f.matches(i)).count(),
        None => items.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn manifest() -> Manifest {
        serde_json::from_str(
            r#"{"id":"demo","name":"Demo","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/demo","pattern":"*.json"}],
              "identify":[{"pointer":"/mutations"}],
              "groups":[{"id":"g","label":"G","lists":[
                {"id":"mutations","label":"Mutations","pointer":"/mutations",
                 "allow_add":true,"allow_remove":true,"min_items":1,"max_items":3,
                 "new_item":{"path":"res://new.tres"},
                 "item_label_pointer":"/path",
                 "fields":[{"id":"path","label":"Mutation","pointer":"/path","type":"text"}]},
                {"id":"fixed","label":"Fixed","pointer":"/loadout",
                 "item_label_pointer":"/slot",
                 "fields":[{"id":"slot","label":"Slot","pointer":"/slot","type":"text"}]}]}]}"#,
        )
        .unwrap()
    }

    fn setup() -> (tempfile::TempDir, BackupManager, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let mgr = BackupManager::new(tmp.path().join("backups"));
        let save = tmp.path().join("save.json");
        std::fs::write(
            &save,
            json!({
                "mutations": [{"path":"a"},{"path":"b"}],
                "loadout": [{"slot":"ESlot1"}]
            })
            .to_string(),
        )
        .unwrap();
        (tmp, mgr, save)
    }

    fn read(path: &Path) -> Value {
        serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
    }

    #[test]
    fn adds_a_row_from_the_template() {
        let (_t, mgr, save) = setup();
        change_row(
            &manifest(),
            &mgr,
            &save,
            "mutations",
            RowChange::Add,
            0,
            None,
        )
        .unwrap();

        let after = read(&save);
        assert_eq!(after["mutations"].as_array().unwrap().len(), 3);
        assert_eq!(after["mutations"][2]["path"], json!("res://new.tres"));
    }

    #[test]
    fn removes_the_row_asked_for_and_leaves_the_rest() {
        let (_t, mgr, save) = setup();
        change_row(
            &manifest(),
            &mgr,
            &save,
            "mutations",
            RowChange::Remove,
            0,
            None,
        )
        .unwrap();

        let after = read(&save);
        assert_eq!(after["mutations"].as_array().unwrap().len(), 1);
        assert_eq!(after["mutations"][0]["path"], json!("b"));
    }

    #[test]
    fn refuses_to_add_past_the_maximum() {
        let (_t, mgr, save) = setup();
        let m = manifest();
        change_row(&m, &mgr, &save, "mutations", RowChange::Add, 0, None).unwrap();
        let err = change_row(&m, &mgr, &save, "mutations", RowChange::Add, 0, None).unwrap_err();
        assert!(err.to_string().contains("3"), "got: {err}");
        assert_eq!(read(&save)["mutations"].as_array().unwrap().len(), 3);
    }

    #[test]
    fn refuses_to_remove_below_the_minimum() {
        let (_t, mgr, save) = setup();
        let m = manifest();
        change_row(&m, &mgr, &save, "mutations", RowChange::Remove, 0, None).unwrap();
        let err = change_row(&m, &mgr, &save, "mutations", RowChange::Remove, 0, None).unwrap_err();
        assert!(err.to_string().contains("1"), "got: {err}");
        assert_eq!(read(&save)["mutations"].as_array().unwrap().len(), 1);
    }

    /// Equipment slots are defined by the game; inventing or destroying one
    /// would break the save, so the plugin does not allow it.
    #[test]
    fn a_list_that_does_not_allow_it_is_refused() {
        let (_t, mgr, save) = setup();
        let m = manifest();
        assert!(change_row(&m, &mgr, &save, "fixed", RowChange::Add, 0, None).is_err());
        assert!(change_row(&m, &mgr, &save, "fixed", RowChange::Remove, 0, None).is_err());
        assert_eq!(read(&save)["loadout"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn an_unknown_list_is_refused() {
        let (_t, mgr, save) = setup();
        assert!(change_row(&manifest(), &mgr, &save, "nope", RowChange::Add, 0, None).is_err());
    }

    #[test]
    fn an_out_of_range_index_is_refused() {
        let (_t, mgr, save) = setup();
        assert!(change_row(
            &manifest(),
            &mgr,
            &save,
            "mutations",
            RowChange::Remove,
            99,
            None
        )
        .is_err());
    }

    #[test]
    fn a_backup_is_taken_before_the_row_changes() {
        let (_t, mgr, save) = setup();
        let before = std::fs::read(&save).unwrap();
        let report = change_row(
            &manifest(),
            &mgr,
            &save,
            "mutations",
            RowChange::Add,
            0,
            None,
        )
        .unwrap();

        mgr.restore(&report.backup_id).unwrap();
        assert_eq!(std::fs::read(&save).unwrap(), before);
    }

    #[test]
    fn a_stale_file_is_refused() {
        let (_t, mgr, save) = setup();
        let stamp = detect::stamp_of(&save);
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&save, json!({"mutations":[{"path":"z"}]}).to_string()).unwrap();

        let err = change_row(
            &manifest(),
            &mgr,
            &save,
            "mutations",
            RowChange::Add,
            0,
            Some(&stamp),
        )
        .unwrap_err();
        assert!(matches!(err, Error::SaveChangedOnDisk));
    }
}
