//! What changed between two versions of a save.
//!
//! Backups were listed by timestamp alone, which makes choosing one an act of
//! memory. Comparing through the manifest turns a restore into a decision you
//! can actually make: *"Money 250 → 999999, Health 6 → 50"*.
//!
//! Only fields the plugin declares are compared. Anything else is the game's
//! own bookkeeping and would be noise.

use crate::core::i18n::pick;
use crate::core::model::FieldChange;
use crate::plugins::manifest::{ListSource, Manifest};
use serde_json::Value;

/// Field-by-field differences between `before` and `after`.
///
/// Ordering follows the manifest, so the list reads in the same order as the
/// editor screen rather than in whatever order the JSON happened to be in.
pub fn compare(
    manifest: &Manifest,
    before: &Value,
    after: &Value,
    locale: &str,
) -> Vec<FieldChange> {
    let mut changes = Vec::new();

    for group in &manifest.groups {
        for field in &group.fields {
            push_if_different(
                &mut changes,
                &field.pointer,
                pick(&field.label, &field.label_i18n, locale),
                before,
                after,
            );
        }

        for list in &group.lists {
            match list.source {
                ListSource::Array => {
                    // Compare by position. A list that grew or shrank shows up
                    // as the rows that moved, which is honest if not pretty.
                    let len = [before, after]
                        .iter()
                        .filter_map(|d| d.pointer(&list.pointer))
                        .filter_map(|v| v.as_array())
                        .map(|a| a.len())
                        .max()
                        .unwrap_or(0);

                    for i in 0..len {
                        for field in &list.fields {
                            let pointer = format!("{}/{}{}", list.pointer, i, field.pointer);
                            let label = format!(
                                "{} #{} · {}",
                                pick(&list.label, &list.label_i18n, locale),
                                i + 1,
                                pick(&field.label, &field.label_i18n, locale)
                            );
                            push_if_different(&mut changes, &pointer, &label, before, after);
                        }
                    }
                }
                ListSource::Object => {
                    let Some(entry) = &list.entry else { continue };
                    let mut keys: Vec<&String> = Vec::new();
                    for doc in [before, after] {
                        if let Some(Value::Object(map)) = doc.pointer(&list.pointer) {
                            for k in map.keys() {
                                if !keys.contains(&k) {
                                    keys.push(k);
                                }
                            }
                        }
                    }
                    for key in keys {
                        let pointer = format!(
                            "{}/{}",
                            list.pointer,
                            key.replace('~', "~0").replace('/', "~1")
                        );
                        let label =
                            format!("{} · {}", pick(&list.label, &list.label_i18n, locale), key);
                        let _ = entry;
                        push_if_different(&mut changes, &pointer, &label, before, after);
                    }
                }
            }
        }
    }

    changes
}

fn push_if_different(
    out: &mut Vec<FieldChange>,
    pointer: &str,
    label: &str,
    before: &Value,
    after: &Value,
) {
    let old = before.pointer(pointer);
    let new = after.pointer(pointer);
    if old == new {
        return;
    }
    out.push(FieldChange {
        pointer: pointer.to_string(),
        label: label.to_string(),
        before: old.cloned(),
        after: new.cloned(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn manifest() -> Manifest {
        serde_json::from_str(
            r#"{"id":"t","name":"T","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
              "groups":[{"id":"g","label":"G",
                "fields":[
                  {"id":"money","label":"Money","pointer":"/money","type":"integer"},
                  {"id":"hp","label":"Health","pointer":"/hp","type":"integer"}],
                "lists":[
                  {"id":"gear","label":"Gear","pointer":"/gear",
                   "item_label_pointer":"/name",
                   "fields":[{"id":"rarity","label":"Rarity","pointer":"/rarity","type":"integer"}]},
                  {"id":"found","label":"Found","pointer":"/found","source":"object",
                   "entry":{"id":"on","label":"On","pointer":"","type":"boolean"}}]}]}"#,
        )
        .unwrap()
    }

    #[test]
    fn reports_only_what_actually_changed() {
        let m = manifest();
        let before = json!({"money": 250, "hp": 6, "gear": [], "found": {}});
        let after = json!({"money": 999999, "hp": 6, "gear": [], "found": {}});

        let changes = compare(&m, &before, &after, "en");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].label, "Money");
        assert_eq!(changes[0].before, Some(json!(250)));
        assert_eq!(changes[0].after, Some(json!(999999)));
    }

    #[test]
    fn identical_documents_produce_nothing() {
        let m = manifest();
        let d = json!({"money": 1, "hp": 1, "gear": [{"rarity":0}], "found": {"a":true}});
        assert!(compare(&m, &d, &d, "en").is_empty());
    }

    #[test]
    fn covers_list_rows_and_object_entries() {
        let m = manifest();
        let before = json!({"money":1,"hp":1,"gear":[{"rarity":0}],"found":{"a":false}});
        let after = json!({"money":1,"hp":1,"gear":[{"rarity":3}],"found":{"a":true}});

        let changes = compare(&m, &before, &after, "en");
        let labels: Vec<&str> = changes.iter().map(|c| c.label.as_str()).collect();
        assert!(labels.iter().any(|l| l.contains("Gear #1")));
        assert!(labels.iter().any(|l| l.contains("Found · a")));
    }

    /// Undeclared data is the game's own bookkeeping; listing it would bury
    /// the changes that matter.
    #[test]
    fn ignores_fields_the_plugin_does_not_declare() {
        let m = manifest();
        let before = json!({"money":1,"hp":1,"gear":[],"found":{},"internal":{"seed":"a"}});
        let after = json!({"money":1,"hp":1,"gear":[],"found":{},"internal":{"seed":"b"}});
        assert!(compare(&m, &before, &after, "en").is_empty());
    }

    #[test]
    fn a_row_that_appeared_is_reported_as_a_change() {
        let m = manifest();
        let before = json!({"money":1,"hp":1,"gear":[],"found":{}});
        let after = json!({"money":1,"hp":1,"gear":[{"rarity":2}],"found":{}});

        let changes = compare(&m, &before, &after, "en");
        assert_eq!(changes.len(), 1);
        assert_eq!(changes[0].before, None);
        assert_eq!(changes[0].after, Some(json!(2)));
    }
}
