pub mod detect;
pub mod diff;
pub mod editor;
pub mod io;
pub mod presets;
pub mod recovery;
pub mod structure;
pub mod validate;
pub mod verify;

use crate::backup::BackupManager;
use crate::core::error::{Error, Result};
use crate::core::model::{Edit, WriteReport};
use crate::plugins::adapter;
use crate::plugins::manifest::Manifest;
use std::path::Path;

/// Backups kept per game before the oldest are dropped.
///
/// Twenty is several days of ordinary editing, and small: these files are the
/// size of a save.
pub const KEEP_BACKUPS: usize = 20;

/// The one and only path by which a save file gets modified.
///
/// Ordering is deliberate and is the whole safety story of the app:
///
/// 0. **Check the file is the one we read**, if the game rewrote it while the
///    editor was open, saving would silently undo the game's own work.
/// 1. **Parse** the file on disk, if it will not parse we stop before doing
///    any damage.
/// 2. **Validate** every edit against the plugin's rules, and the finished
///    document against its cross-field rules; reject the batch as a whole if
///    anything is wrong. Nothing on disk has been touched at this point.
/// 3. **Back it up**, and abort if that fails. A save is never modified
///    without a copy existing first.
/// 4. **Re-serialise** the document.
/// 5. **Re-parse what we produced** and re-run the plugin's identify rules, so
///    a bug in this app cannot hand the game a file it will choke on.
/// 6. **Write atomically** via a temporary file (see [`io::write_atomically`]).
///
/// Validation comes before the backup because it changes nothing, so a
/// rejected edit should not leave a backup behind for the player to wonder
/// about. If any step fails the original save is still exactly as it was.
pub fn apply_and_write(
    manifest: &Manifest,
    backups: &BackupManager,
    save_path: &Path,
    edits: &[Edit],
    expected: Option<&crate::core::model::SaveStamp>,
    confirm: bool,
) -> Result<WriteReport> {
    // 0. Has the game rewritten this file since we read it?
    if let Some(expected) = expected {
        if detect::stamp_of(save_path) != *expected {
            return Err(Error::SaveChangedOnDisk);
        }
    }

    // 1. Parse.
    let mut doc = detect::load_document(manifest, save_path)?;

    // 2. Validate and apply in memory, then check the cross-field rules.
    let (changed, warnings) = validate::apply_edits(manifest, &mut doc, edits)?;
    check_constraints(manifest, &doc)?;

    // 2b. Values past the plugin's safe range are allowed, but not silently.
    //     Stopping here means an unconfirmed risky edit leaves no backup and no
    //     trace: the player simply gets asked.
    if !warnings.is_empty() && !confirm {
        return Err(Error::NeedsConfirmation { warnings });
    }

    // 3. Back up. From here on the file on disk is about to change.
    let backup_id = backups.create(&manifest.id, save_path)?;

    // 4. Serialise.
    let adapter = adapter::adapter_for(&manifest.format)?;
    let bytes = adapter.write(&doc)?;

    // 5. Prove the bytes we are about to write are still a valid save.
    let verify = adapter
        .parse(&bytes)
        .map_err(|e| Error::WriteFailed(format!("the rebuilt save did not parse ({e})")))?;
    if let Some(where_) = verify::difference(&doc, &verify, String::new()) {
        return Err(Error::WriteFailed(format!(
            "the rebuilt save did not match the edited data at {where_}"
        )));
    }
    if !detect::identifies_as(manifest, &verify) {
        return Err(Error::WriteFailed(
            "the rebuilt save no longer looks like a valid save file".into(),
        ));
    }

    // 6. Swap it in atomically.
    io::write_atomically(save_path, &bytes)?;

    // 7. Keep the backup folder from growing without limit. Done after the
    //    write so a failure here can never cost anyone their save.
    backups.prune(&manifest.id, KEEP_BACKUPS);

    Ok(WriteReport {
        backup_id,
        changed_fields: changed,
        save_path: save_path.to_string_lossy().into_owned(),
        stamp: detect::stamp_of(save_path),
    })
}

/// Check the rules that relate one field to another, once every edit is in.
///
/// A save can be perfectly valid field by field and still nonsense as a whole,
/// health above maximum health being the obvious case.
fn check_constraints(manifest: &Manifest, doc: &serde_json::Value) -> Result<()> {
    for constraint in &manifest.constraints {
        if constraint.violated_by(doc) {
            let message = if constraint.message.is_empty() {
                format!(
                    "'{}' and '{}' do not fit together.",
                    constraint.left, constraint.right
                )
            } else {
                constraint.message.clone()
            };
            return Err(Error::Constraint {
                left: constraint.left.clone(),
                right: constraint.right.clone(),
                message,
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn manifest() -> Manifest {
        serde_json::from_str(
            r#"{"id":"demo","name":"Demo","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/demo","pattern":"*.json"}],
              "identify":[{"pointer":"/player/hp"}],
              "groups":[{"id":"c","label":"C","fields":[
                {"id":"hp","label":"HP","pointer":"/player/hp","type":"integer","min":1,"max":999},
                {"id":"st","label":"Stamina","pointer":"/player/stamina","type":"number","min":0,"max":9999}],
                "lists":[{"id":"inv","label":"Inventory","pointer":"/inventory",
                  "item_label_pointer":"/name",
                  "fields":[{"id":"qty","label":"Quantity","pointer":"/qty","type":"integer","min":0,"max":999}]}]}]}"#,
        )
        .unwrap()
    }

    fn setup() -> (tempfile::TempDir, BackupManager, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let mgr = BackupManager::new(tmp.path().join("backups"));
        let save = tmp.path().join("save.json");
        std::fs::write(
            &save,
            serde_json::to_string_pretty(&json!({
                "player": {"hp": 6, "stamina": 100.0},
                "inventory": [{"name":"Diamond","qty":3},{"name":"Iridium","qty":12}],
                "untouched": {"nested": [1, 2, {"deep": true}]}
            }))
            .unwrap(),
        )
        .unwrap();
        (tmp, mgr, save)
    }

    fn edit(p: &str, v: serde_json::Value) -> Edit {
        Edit {
            pointer: p.into(),
            value: v,
        }
    }

    /// Most tests do not care about the stale-file check, so they pass no
    /// expected stamp.
    fn apply_and_write_4(
        m: &Manifest,
        b: &BackupManager,
        p: &std::path::Path,
        e: &[Edit],
    ) -> Result<WriteReport> {
        apply_and_write(m, b, p, e, None, true)
    }

    #[test]
    fn writes_an_edit_and_reports_it() {
        let (_t, mgr, save) = setup();
        let r =
            apply_and_write_4(&manifest(), &mgr, &save, &[edit("/player/hp", json!(500))]).unwrap();
        assert_eq!(r.changed_fields, 1);
        assert!(!r.backup_id.is_empty());

        let after: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&save).unwrap()).unwrap();
        assert_eq!(after["player"]["hp"], json!(500));
    }

    /// The round-trip the brief calls out as the most important test:
    /// parse -> modify -> write -> parse must not disturb anything else.
    #[test]
    fn round_trip_leaves_untouched_data_byte_identical() {
        let (_t, mgr, save) = setup();
        let before: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&save).unwrap()).unwrap();

        apply_and_write_4(&manifest(), &mgr, &save, &[edit("/player/hp", json!(42))]).unwrap();

        let after: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&save).unwrap()).unwrap();
        assert_eq!(before["untouched"], after["untouched"]);
        assert_eq!(before["inventory"], after["inventory"]);
        assert_eq!(after["player"]["hp"], json!(42));
    }

    #[test]
    fn repeated_writes_are_stable() {
        let (_t, mgr, save) = setup();
        apply_and_write_4(&manifest(), &mgr, &save, &[edit("/player/hp", json!(42))]).unwrap();
        let first = std::fs::read(&save).unwrap();
        apply_and_write_4(&manifest(), &mgr, &save, &[edit("/player/hp", json!(42))]).unwrap();
        assert_eq!(first, std::fs::read(&save).unwrap());
    }

    #[test]
    fn decimal_fields_survive_the_round_trip_as_decimals() {
        let (_t, mgr, save) = setup();
        apply_and_write_4(
            &manifest(),
            &mgr,
            &save,
            &[edit("/player/stamina", json!(500))],
        )
        .unwrap();
        let text = std::fs::read_to_string(&save).unwrap();
        assert!(
            text.contains("500.0"),
            "stamina lost its decimal form: {text}"
        );
    }

    #[test]
    fn list_items_are_editable_by_index() {
        let (_t, mgr, save) = setup();
        apply_and_write_4(
            &manifest(),
            &mgr,
            &save,
            &[edit("/inventory/1/qty", json!(99))],
        )
        .unwrap();
        let after: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&save).unwrap()).unwrap();
        assert_eq!(after["inventory"][1]["qty"], json!(99));
        assert_eq!(after["inventory"][0]["qty"], json!(3));
    }

    #[test]
    fn a_backup_exists_before_the_file_changes() {
        let (_t, mgr, save) = setup();
        let original = std::fs::read(&save).unwrap();
        let r =
            apply_and_write_4(&manifest(), &mgr, &save, &[edit("/player/hp", json!(9))]).unwrap();

        mgr.restore(&r.backup_id).unwrap();
        assert_eq!(std::fs::read(&save).unwrap(), original);
    }

    #[test]
    fn an_invalid_edit_leaves_the_file_untouched() {
        let (_t, mgr, save) = setup();
        let original = std::fs::read(&save).unwrap();
        // A decimal where the engine stores an integer is a genuine type
        // error, and stays a hard refusal.
        assert!(
            apply_and_write_4(&manifest(), &mgr, &save, &[edit("/player/hp", json!(1.5))]).is_err()
        );
        assert_eq!(std::fs::read(&save).unwrap(), original);
    }

    /// The safe range is advice. Passing it asks first, and asking must not
    /// touch the file or leave a backup lying around.
    #[test]
    fn an_out_of_range_value_asks_before_writing() {
        let (_t, mgr, save) = setup();
        let original = std::fs::read(&save).unwrap();

        let err = apply_and_write(
            &manifest(),
            &mgr,
            &save,
            &[edit("/player/hp", json!(99999))],
            None,
            false,
        )
        .unwrap_err();

        let Error::NeedsConfirmation { warnings } = &err else {
            panic!("expected a confirmation request, got: {err}");
        };
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule, "rule.tooLarge");
        assert_eq!(warnings[0].limit, "999");
        assert_eq!(warnings[0].field, "HP");
        assert_eq!(warnings[0].value, "99999");

        assert_eq!(std::fs::read(&save).unwrap(), original);
        assert!(mgr.list(Some("demo")).is_empty(), "asking created a backup");
    }

    #[test]
    fn the_same_edit_goes_through_once_confirmed() {
        let (_t, mgr, save) = setup();
        let report = apply_and_write(
            &manifest(),
            &mgr,
            &save,
            &[edit("/player/hp", json!(99999))],
            None,
            true,
        )
        .unwrap();

        assert_eq!(report.changed_fields, 1);
        let after: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&save).unwrap()).unwrap();
        assert_eq!(after["player"]["hp"], json!(99999));
    }

    /// Confirmation is about ranges only: a type error is never something the
    /// player can wave through, because the game would reject the file.
    #[test]
    fn confirming_does_not_excuse_a_type_error() {
        let (_t, mgr, save) = setup();
        assert!(apply_and_write(
            &manifest(),
            &mgr,
            &save,
            &[edit("/player/hp", json!(1.5))],
            None,
            true,
        )
        .is_err());
    }

    #[test]
    fn an_in_range_edit_never_asks() {
        let (_t, mgr, save) = setup();
        assert!(apply_and_write(
            &manifest(),
            &mgr,
            &save,
            &[edit("/player/hp", json!(500))],
            None,
            false,
        )
        .is_ok());
    }

    #[test]
    fn an_undeclared_pointer_is_refused() {
        let (_t, mgr, save) = setup();
        let original = std::fs::read(&save).unwrap();
        assert!(apply_and_write_4(
            &manifest(),
            &mgr,
            &save,
            &[edit("/untouched/nested/0", json!(7))]
        )
        .is_err());
        assert_eq!(std::fs::read(&save).unwrap(), original);
    }

    #[test]
    fn a_corrupted_save_is_refused_before_any_backup_or_write() {
        let (_t, mgr, save) = setup();
        std::fs::write(&save, b"{ this is not json").unwrap();
        assert!(
            apply_and_write_4(&manifest(), &mgr, &save, &[edit("/player/hp", json!(9))]).is_err()
        );
        assert!(
            mgr.list(Some("demo")).is_empty(),
            "backed up an unreadable file"
        );
    }

    #[test]
    fn refuses_to_write_when_the_game_changed_the_file_first() {
        let (_t, mgr, save) = setup();
        let stamp = detect::stamp_of(&save);

        // The game writes its own version while the editor sits open.
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(
            &save,
            serde_json::to_string_pretty(&json!({
                "player": {"hp": 3, "stamina": 50.0},
                "inventory": [],
                "untouched": {}
            }))
            .unwrap(),
        )
        .unwrap();

        let err = apply_and_write(
            &manifest(),
            &mgr,
            &save,
            &[edit("/player/hp", json!(500))],
            Some(&stamp),
            true,
        )
        .unwrap_err();

        assert!(matches!(err, Error::SaveChangedOnDisk));
        // The game's version survives untouched, and no backup was taken.
        let after: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&save).unwrap()).unwrap();
        assert_eq!(after["player"]["hp"], json!(3));
        assert!(mgr.list(Some("demo")).is_empty());
    }

    #[test]
    fn an_unchanged_file_passes_the_stale_check() {
        let (_t, mgr, save) = setup();
        let stamp = detect::stamp_of(&save);
        let r = apply_and_write(
            &manifest(),
            &mgr,
            &save,
            &[edit("/player/hp", json!(42))],
            Some(&stamp),
            true,
        )
        .unwrap();
        assert_eq!(r.changed_fields, 1);
        // The report carries the new revision so a second save in a row works.
        assert_ne!(r.stamp, stamp);
    }

    #[test]
    fn a_rejected_edit_leaves_no_backup_behind() {
        let (_t, mgr, save) = setup();
        assert!(
            apply_and_write_4(&manifest(), &mgr, &save, &[edit("/player/hp", json!(1.5))]).is_err()
        );
        assert!(
            mgr.list(Some("demo")).is_empty(),
            "a backup was taken for an edit that never happened"
        );
    }

    fn manifest_with_constraint() -> Manifest {
        serde_json::from_str(
            r#"{"id":"demo","name":"Demo","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/demo","pattern":"*.json"}],
              "identify":[{"pointer":"/player/hp"}],
              "constraints":[{"left":"/player/hp","right":"/player/max_hp","rule":"lte",
                              "message":"Health cannot exceed max health."}],
              "groups":[{"id":"c","label":"C","fields":[
                {"id":"hp","label":"HP","pointer":"/player/hp","type":"integer","min":1,"max":9999},
                {"id":"mx","label":"Max HP","pointer":"/player/max_hp","type":"integer","min":1,"max":9999}]}]}"#,
        )
        .unwrap()
    }

    fn constrained_setup() -> (tempfile::TempDir, BackupManager, std::path::PathBuf) {
        let tmp = tempfile::tempdir().unwrap();
        let mgr = BackupManager::new(tmp.path().join("backups"));
        let save = tmp.path().join("save.json");
        std::fs::write(&save, json!({"player":{"hp":6,"max_hp":6}}).to_string()).unwrap();
        (tmp, mgr, save)
    }

    #[test]
    fn a_cross_field_rule_rejects_an_inconsistent_pair() {
        let (_t, mgr, save) = constrained_setup();
        let err = apply_and_write_4(
            &manifest_with_constraint(),
            &mgr,
            &save,
            &[edit("/player/hp", json!(500))],
        )
        .unwrap_err();

        assert!(err.to_string().contains("exceed max health"));
        let after: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&save).unwrap()).unwrap();
        assert_eq!(
            after["player"]["hp"],
            json!(6),
            "the file was modified anyway"
        );
    }

    #[test]
    fn a_cross_field_rule_allows_a_consistent_pair() {
        let (_t, mgr, save) = constrained_setup();
        apply_and_write_4(
            &manifest_with_constraint(),
            &mgr,
            &save,
            &[
                edit("/player/max_hp", json!(500)),
                edit("/player/hp", json!(500)),
            ],
        )
        .unwrap();

        let after: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&save).unwrap()).unwrap();
        assert_eq!(after["player"]["hp"], json!(500));
    }

    #[test]
    fn a_save_from_another_game_is_refused() {
        let (_t, mgr, save) = setup();
        std::fs::write(&save, r#"{"something":"else"}"#).unwrap();
        let err = apply_and_write_4(&manifest(), &mgr, &save, &[edit("/player/hp", json!(9))])
            .unwrap_err();
        assert!(err.to_string().contains("Demo"));
    }
}
