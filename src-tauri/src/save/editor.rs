use crate::core::i18n::pick;
use crate::core::model::*;
use crate::plugins::manifest::{Field, FieldKind, ListField, ListSource, Manifest};
use serde_json::Value;
use std::collections::HashMap;

/// Build the screen the GUI renders, by pairing every declared field with the
/// value currently in the save.
///
/// `locale` selects the plugin's translations for labels and help text; an
/// unknown or empty locale leaves everything in the manifest's own wording.
pub fn build(
    manifest: &Manifest,
    save_path: &str,
    doc: &Value,
    locale: &str,
    context: Context,
) -> EditorDocument {
    // Sections that do not apply are still listed when the plugin has
    // something to say about why. A vanished section reads as a missing
    // feature; an explained one reads as a fact about the save file.
    let groups = manifest
        .groups
        .iter()
        .filter_map(|g| {
            let applies = match &g.requires {
                Some(pointer) => doc.pointer(pointer).is_some(),
                None => true,
            };

            if !applies {
                let reason = pick(&g.when_absent, &g.when_absent_i18n, locale);
                if reason.is_empty() {
                    return None;
                }
                return Some(GroupView {
                    id: g.id.clone(),
                    label: pick(&g.label, &g.label_i18n, locale).to_string(),
                    description: String::new(),
                    absent_reason: Some(reason.to_string()),
                    fields: Vec::new(),
                    lists: Vec::new(),
                });
            }

            Some(GroupView {
                id: g.id.clone(),
                label: pick(&g.label, &g.label_i18n, locale).to_string(),
                description: pick(&g.description, &g.description_i18n, locale).to_string(),
                absent_reason: None,
                fields: g
                    .fields
                    .iter()
                    .map(|f| field_view(manifest, f, &f.pointer, doc, locale))
                    .collect(),
                lists: g
                    .lists
                    .iter()
                    .map(|l| list_view(manifest, l, doc, locale))
                    .collect(),
            })
        })
        .collect();

    EditorDocument {
        game_id: manifest.id.clone(),
        game_name: manifest.name.clone(),
        save_path: save_path.to_string(),
        groups,
        presets: crate::save::presets::available(manifest, doc, locale),
        stamp: context.stamp,
        game_running: context.game_running,
        cloud_synced: context.cloud_synced,
    }
}

/// Facts about the file and the machine that the editor screen shows but the
/// field layer knows nothing about.
#[derive(Debug, Clone, Default)]
pub struct Context {
    pub stamp: crate::core::model::SaveStamp,
    pub game_running: Vec<String>,
    pub cloud_synced: bool,
}

fn field_view(
    manifest: &Manifest,
    field: &Field,
    pointer: &str,
    doc: &Value,
    locale: &str,
) -> FieldView {
    let found = doc.pointer(pointer);
    FieldView {
        id: field.id.clone(),
        label: pick(&field.label, &field.label_i18n, locale).to_string(),
        help: pick(&field.help, &field.help_i18n, locale).to_string(),
        pointer: pointer.to_string(),
        kind: kind_name(field.kind).to_string(),
        value: found.cloned().unwrap_or(Value::Null),
        min: field.min,
        max: field.max,
        max_length: field.max_length,
        options: manifest
            .choices(field)
            .iter()
            .map(|c| ChoiceView {
                value: c.value.clone(),
                label: pick(&c.label, &c.label_i18n, locale).to_string(),
            })
            .collect(),
        read_only: field.read_only,
        missing: found.is_none(),
    }
}

fn list_view(manifest: &Manifest, list: &ListField, doc: &Value, locale: &str) -> ListView {
    let items = match (list.source, doc.pointer(&list.pointer)) {
        (ListSource::Array, Some(Value::Array(arr))) => arr
            .iter()
            .enumerate()
            .filter(|(_, item)| match &list.item_filter {
                Some(f) => f.matches(item),
                None => true,
            })
            .map(|(i, item)| ListItemView {
                label: item_label(manifest, list, item, i, locale),
                index: i,
                fields: list
                    .fields
                    .iter()
                    .map(|f| {
                        let abs = format!("{}/{}{}", list.pointer, i, f.pointer);
                        field_view(manifest, f, &abs, doc, locale)
                    })
                    .collect(),
            })
            .collect(),

        // Each key of the object is a row, the key itself being the name.
        //
        // The value is either a single thing, `entry` describes it, or a
        // small record, in which case `fields` describes its parts exactly as
        // it would for an array row. Feed The Pit stores its tool slots the
        // second way: keys "0".."5", each holding an id and a durability.
        (ListSource::Object, Some(Value::Object(map))) => {
            if list.entry.is_none() && list.fields.is_empty() {
                return empty_list(list, locale);
            }
            map.keys()
                .map(|key| {
                    let base = format!("{}/{}", list.pointer, escape_pointer_token(key));
                    let fields = if let Some(entry) = &list.entry {
                        vec![field_view(manifest, entry, &base, doc, locale)]
                    } else {
                        list.fields
                            .iter()
                            .map(|f| {
                                let abs = format!("{base}{}", f.pointer);
                                field_view(manifest, f, &abs, doc, locale)
                            })
                            .collect()
                    };
                    ListItemView {
                        label: prettify(key),
                        // Object rows have no array position; nothing can be
                        // added or removed here, so zero is never used.
                        index: 0,
                        fields,
                    }
                })
                .collect()
        }

        _ => Vec::new(),
    };

    ListView {
        id: list.id.clone(),
        label: pick(&list.label, &list.label_i18n, locale).to_string(),
        description: pick(&list.description, &list.description_i18n, locale).to_string(),
        items,
        allow_add: list.allow_add && list.new_item.is_some(),
        allow_remove: list.allow_remove,
        bulk_actions: bulk_action_views(list, locale),
    }
}

fn empty_list(list: &ListField, locale: &str) -> ListView {
    ListView {
        id: list.id.clone(),
        label: pick(&list.label, &list.label_i18n, locale).to_string(),
        description: pick(&list.description, &list.description_i18n, locale).to_string(),
        items: Vec::new(),
        allow_add: list.allow_add && list.new_item.is_some(),
        allow_remove: list.allow_remove,
        bulk_actions: bulk_action_views(list, locale),
    }
}

fn bulk_action_views(list: &ListField, locale: &str) -> Vec<BulkActionView> {
    list.bulk_actions
        .iter()
        .map(|a| BulkActionView {
            id: a.id.clone(),
            label: pick(&a.label, &a.label_i18n, locale).to_string(),
            field: a.field.clone(),
            value: a.value.clone(),
        })
        .collect()
}

/// RFC-6901 escaping for a single path token.
///
/// A key containing `/` or `~` would otherwise be read as extra levels of
/// nesting. Game data is full of keys nobody thought about.
fn escape_pointer_token(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

/// `armor_maker` -> `Armor Maker`.
///
/// Object keys are identifiers the game never meant a player to read, so they
/// get the same treatment the plugin generator gives option values.
fn prettify(key: &str) -> String {
    key.split(['_', '-'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Prefer a human name from the item itself, prettified through the plugin's
/// option set when one is declared, so the GUI shows `Rocket Launcher` rather
/// than `rocket_launcher`.
fn item_label(
    manifest: &Manifest,
    list: &ListField,
    item: &Value,
    index: usize,
    locale: &str,
) -> String {
    let raw = list
        .item_label_pointer
        .as_ref()
        .and_then(|p| item.pointer(p));

    let Some(raw) = raw else {
        return format!("#{}", index + 1);
    };

    if let Some(set) = list
        .item_label_options_ref
        .as_ref()
        .and_then(|k| manifest.option_sets.get(k))
    {
        if let Some(choice) = set.iter().find(|c| &c.value == raw) {
            return pick(&choice.label, &choice.label_i18n, locale).to_string();
        }
    }

    match raw {
        Value::String(s) if !s.is_empty() => s.clone(),
        Value::Null => format!("#{}", index + 1),
        other => other.to_string(),
    }
}

pub fn kind_name(k: FieldKind) -> &'static str {
    match k {
        FieldKind::Integer => "integer",
        FieldKind::Number => "number",
        FieldKind::Text => "text",
        FieldKind::Boolean => "boolean",
        FieldKind::Choice => "choice",
    }
}

/// Every pointer the user is allowed to write, mapped to the rules governing it.
///
/// This is the security boundary of the whole editor: an edit whose pointer is
/// absent from this map is refused. It is rebuilt from the manifest *and the
/// document just read from disk*, so list indices reflect reality and an edit
/// cannot invent `/player/loadout/999`.
pub fn writable_fields(manifest: &Manifest, doc: &Value) -> HashMap<String, Field> {
    let mut map = HashMap::new();

    // Only groups that apply to this document, a hidden group must not be
    // writable, or the GUI and the backend would disagree about what exists.
    for group in manifest.groups_for(doc) {
        for f in &group.fields {
            if f.read_only {
                continue;
            }
            map.insert(f.pointer.clone(), f.clone());
        }
        for list in &group.lists {
            match (list.source, doc.pointer(&list.pointer)) {
                (ListSource::Array, Some(Value::Array(arr))) => {
                    for (i, item) in arr.iter().enumerate() {
                        // A filtered-out item is not shown, so it is not
                        // writable either: the two must agree or the GUI could
                        // offer an edit the backend would reject.
                        if let Some(filter) = &list.item_filter {
                            if !filter.matches(item) {
                                continue;
                            }
                        }
                        for f in &list.fields {
                            if f.read_only {
                                continue;
                            }
                            map.insert(format!("{}/{}{}", list.pointer, i, f.pointer), f.clone());
                        }
                    }
                }
                (ListSource::Object, Some(Value::Object(obj))) => {
                    for key in obj.keys() {
                        let base = format!("{}/{}", list.pointer, escape_pointer_token(key));
                        match &list.entry {
                            Some(entry) if !entry.read_only => {
                                map.insert(base, entry.clone());
                            }
                            Some(_) => {}
                            // A record per key: the same fields an array row
                            // would declare, hung off the key instead.
                            None => {
                                for f in &list.fields {
                                    if f.read_only {
                                        continue;
                                    }
                                    map.insert(format!("{base}{}", f.pointer), f.clone());
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
    map
}

/// How much of a save the plugin actually reaches.
///
/// A plugin author needs to know what they have not covered yet, and a player
/// deserves an honest answer to "is this everything?". Counts leaf values only:
/// containers are structure, not settings.
pub fn coverage(manifest: &Manifest, doc: &Value) -> (usize, usize) {
    let editable = writable_fields(manifest, doc).len();
    let mut total = 0;
    count_leaves(doc, &mut total);
    (editable, total)
}

fn count_leaves(value: &Value, total: &mut usize) {
    match value {
        Value::Object(map) => {
            for child in map.values() {
                count_leaves(child, total);
            }
        }
        Value::Array(items) => {
            for child in items {
                count_leaves(child, total);
            }
        }
        Value::Null => {}
        _ => *total += 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        serde_json::from_str(
            r#"{"id":"t","name":"T","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
              "option_sets":{"parts":[{"value":"gun","label":"Gun"}]},
              "groups":[{"id":"c","label":"Character",
                "fields":[
                  {"id":"money","label":"Money","pointer":"/player/money","type":"integer"},
                  {"id":"seed","label":"Seed","pointer":"/seed","type":"text","read_only":true}],
                "lists":[{"id":"loadout","label":"Equipment","pointer":"/player/loadout",
                  "item_label_pointer":"/bodypart","item_label_options_ref":"parts",
                  "fields":[{"id":"rarity","label":"Rarity","pointer":"/rarity","type":"integer"}]}]}]}"#,
        )
        .unwrap()
    }

    fn doc() -> Value {
        serde_json::json!({
            "seed": "ABC",
            "player": { "money": 100, "loadout": [ {"bodypart":"gun","rarity":0},
                                                   {"bodypart":"lash","rarity":1} ] }
        })
    }

    #[test]
    fn builds_fields_with_current_values() {
        let d = build(&manifest(), "/tmp/s.json", &doc(), "", Context::default());
        let money = &d.groups[0].fields[0];
        assert_eq!(money.value, serde_json::json!(100));
        assert!(!money.missing);
    }

    #[test]
    fn marks_absent_pointers_as_missing_rather_than_zero() {
        let d = build(
            &manifest(),
            "/tmp/s.json",
            &serde_json::json!({"seed":"A"}),
            "",
            Context::default(),
        );
        assert!(d.groups[0].fields[0].missing);
        assert_eq!(d.groups[0].fields[0].value, Value::Null);
    }

    #[test]
    fn expands_list_item_pointers_to_absolute_form() {
        let d = build(&manifest(), "/tmp/s.json", &doc(), "", Context::default());
        let list = &d.groups[0].lists[0];
        assert_eq!(list.items.len(), 2);
        assert_eq!(list.items[1].fields[0].pointer, "/player/loadout/1/rarity");
    }

    #[test]
    fn prettifies_item_labels_through_option_sets() {
        let d = build(&manifest(), "/tmp/s.json", &doc(), "", Context::default());
        let list = &d.groups[0].lists[0];
        assert_eq!(list.items[0].label, "Gun");
        // No option entry for "lash", fall back to the raw value.
        assert_eq!(list.items[1].label, "lash");
    }

    #[test]
    fn writable_map_excludes_read_only_fields() {
        let w = writable_fields(&manifest(), &doc());
        assert!(w.contains_key("/player/money"));
        assert!(!w.contains_key("/seed"));
    }

    fn filtered_manifest() -> Manifest {
        serde_json::from_str(
            r#"{"id":"t","name":"T","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
              "groups":[{"id":"c","label":"C","lists":[
                {"id":"internal","label":"Organs","pointer":"/player/loadout",
                 "item_filter":{"pointer":"/slot","equals":["ISlot1"]},
                 "fields":[{"id":"rarity","label":"Rarity","pointer":"/rarity","type":"integer"}]}]}]}"#,
        )
        .unwrap()
    }

    fn slotted_doc() -> Value {
        serde_json::json!({"player":{"loadout":[
            {"bodypart":"gun","rarity":0,"slot":"ESlot1"},
            {"bodypart":"poison","rarity":1,"slot":"ISlot1"}]}})
    }

    #[test]
    fn item_filter_shows_only_matching_items() {
        let d = build(
            &filtered_manifest(),
            "/tmp/s.json",
            &slotted_doc(),
            "",
            Context::default(),
        );
        let list = &d.groups[0].lists[0];
        assert_eq!(list.items.len(), 1);
        // Index 1 in the underlying array, not renumbered to 0.
        assert_eq!(list.items[0].fields[0].pointer, "/player/loadout/1/rarity");
    }

    #[test]
    fn item_filter_also_narrows_what_may_be_written() {
        let w = writable_fields(&filtered_manifest(), &slotted_doc());
        assert!(w.contains_key("/player/loadout/1/rarity"));
        assert!(
            !w.contains_key("/player/loadout/0/rarity"),
            "a hidden item was left writable"
        );
    }

    #[test]
    fn writable_map_covers_exactly_the_existing_list_indices() {
        let w = writable_fields(&manifest(), &doc());
        assert!(w.contains_key("/player/loadout/0/rarity"));
        assert!(w.contains_key("/player/loadout/1/rarity"));
        assert!(!w.contains_key("/player/loadout/2/rarity"));
    }
}
