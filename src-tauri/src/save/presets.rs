//! One-click sets of edits.
//!
//! A preset is expanded into ordinary [`Edit`]s here and then handed to the
//! normal write pipeline. It gets no shortcuts: the same validation, the same
//! confirmation when a value leaves the safe range, the same backup. "Max out
//! this run" is exactly the edits a patient player would have typed.

use crate::core::model::{Edit, PresetView};
use crate::plugins::manifest::{ListSource, Manifest, Preset};
use serde_json::Value;

/// Presets that apply to this particular save, translated for display.
pub fn available(manifest: &Manifest, doc: &Value, locale: &str) -> Vec<PresetView> {
    use crate::core::i18n::pick;

    manifest
        .presets
        .iter()
        .filter(|p| applies(p, doc))
        .map(|p| PresetView {
            id: p.id.clone(),
            label: pick(&p.label, &p.label_i18n, locale).to_string(),
            description: pick(&p.description, &p.description_i18n, locale).to_string(),
        })
        .collect()
}

fn applies(preset: &Preset, doc: &Value) -> bool {
    match &preset.requires {
        Some(pointer) => doc.pointer(pointer).is_some(),
        None => true,
    }
}

/// Turn a preset into the edits it stands for.
///
/// Pointers that this particular save does not have are skipped rather than
/// failing: a preset written for a full run should still do what it can to a
/// profile, instead of refusing outright.
pub fn expand(manifest: &Manifest, doc: &Value, preset: &Preset) -> Vec<Edit> {
    let mut edits = Vec::new();

    for item in &preset.set {
        if doc.pointer(&item.pointer).is_some() {
            edits.push(Edit {
                pointer: item.pointer.clone(),
                value: item.value.clone(),
            });
        }
    }

    for spec in &preset.set_in_lists {
        let Some(list) = manifest
            .groups_for(doc)
            .flat_map(|g| g.lists.iter())
            .find(|l| l.id == spec.list)
        else {
            continue;
        };

        match (list.source, doc.pointer(&list.pointer)) {
            (ListSource::Array, Some(Value::Array(items))) => {
                let Some(field_id) = &spec.field else {
                    continue;
                };
                let Some(field) = list.fields.iter().find(|f| &f.id == field_id) else {
                    continue;
                };
                for (i, item) in items.iter().enumerate() {
                    // Hidden rows are not the preset's business either.
                    if let Some(filter) = &list.item_filter {
                        if !filter.matches(item) {
                            continue;
                        }
                    }
                    edits.push(Edit {
                        pointer: format!("{}/{}{}", list.pointer, i, field.pointer),
                        value: spec.value.clone(),
                    });
                }
            }
            (ListSource::Object, Some(Value::Object(map))) => {
                for key in map.keys() {
                    edits.push(Edit {
                        pointer: format!(
                            "{}/{}",
                            list.pointer,
                            key.replace('~', "~0").replace('/', "~1")
                        ),
                        value: spec.value.clone(),
                    });
                }
            }
            _ => {}
        }
    }

    edits
}

/// Find a preset by id among those that apply to this save.
pub fn find<'a>(manifest: &'a Manifest, doc: &Value, id: &str) -> Option<&'a Preset> {
    manifest
        .presets
        .iter()
        .find(|p| p.id == id && applies(p, doc))
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
                "fields":[{"id":"hp","label":"HP","pointer":"/player/hp","type":"integer"}],
                "lists":[
                  {"id":"gear","label":"Gear","pointer":"/loadout",
                   "item_filter":{"pointer":"/slot","starts_with":["E"]},
                   "item_label_pointer":"/slot",
                   "fields":[{"id":"rarity","label":"Rarity","pointer":"/rarity","type":"integer"}]},
                  {"id":"found","label":"Found","pointer":"/discoveries","source":"object",
                   "entry":{"id":"on","label":"On","pointer":"","type":"boolean"}}]}],
              "presets":[
                {"id":"maxed","label":"Max out","requires":"/player",
                 "set":[{"pointer":"/player/hp","value":999}],
                 "set_in_lists":[{"list":"gear","field":"rarity","value":3},
                                 {"list":"found","value":true}]},
                {"id":"runonly","label":"Run only","requires":"/nothing/here",
                 "set":[{"pointer":"/player/hp","value":1}]}]}"#,
        )
        .unwrap()
    }

    fn doc() -> Value {
        json!({
            "player": { "hp": 6 },
            "loadout": [
                {"slot":"ESlot1","rarity":0},
                {"slot":"ISlot1","rarity":0},
                {"slot":"ESlot5","rarity":1}
            ],
            "discoveries": { "a": false, "b/c": false }
        })
    }

    #[test]
    fn only_presets_that_apply_are_offered() {
        let views = available(&manifest(), &doc(), "en");
        let ids: Vec<&str> = views.iter().map(|v| v.id.as_str()).collect();
        assert_eq!(
            ids,
            vec!["maxed"],
            "a preset for another save kind was offered"
        );
    }

    #[test]
    fn expands_into_plain_edits() {
        let m = manifest();
        let d = doc();
        let preset = find(&m, &d, "maxed").unwrap();
        let edits = expand(&m, &d, preset);

        let pointers: Vec<&str> = edits.iter().map(|e| e.pointer.as_str()).collect();
        assert!(pointers.contains(&"/player/hp"));
        // Only the rows the filter shows: ESlot1 and ESlot5, not ISlot1.
        assert!(pointers.contains(&"/loadout/0/rarity"));
        assert!(pointers.contains(&"/loadout/2/rarity"));
        assert!(!pointers.contains(&"/loadout/1/rarity"));
        // Object keys are escaped.
        assert!(pointers.contains(&"/discoveries/b~1c"));
    }

    #[test]
    fn every_expanded_pointer_resolves() {
        let m = manifest();
        let d = doc();
        let preset = find(&m, &d, "maxed").unwrap();
        for edit in expand(&m, &d, preset) {
            assert!(
                d.pointer(&edit.pointer).is_some(),
                "{} does not resolve",
                edit.pointer
            );
        }
    }

    #[test]
    fn a_pointer_this_save_lacks_is_skipped_not_fatal() {
        let m = manifest();
        let d = json!({ "player": {}, "loadout": [], "discoveries": {} });
        let preset = find(&m, &d, "maxed").unwrap();
        let edits = expand(&m, &d, preset);
        assert!(edits.is_empty());
    }

    #[test]
    fn an_unknown_preset_is_not_found() {
        let m = manifest();
        assert!(find(&m, &doc(), "nope").is_none());
        // Nor one whose `requires` fails.
        assert!(find(&m, &doc(), "runonly").is_none());
    }
}
