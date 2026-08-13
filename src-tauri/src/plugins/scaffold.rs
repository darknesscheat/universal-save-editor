//! Turning an unknown save file into a plugin someone can finish.
//!
//! The project's whole claim is "a new game is one file", which is only true if
//! writing that file is not an afternoon of counting JSON pointers by hand.
//! This walks a save, proposes a field for every scalar it finds, and hands
//! back a manifest that already loads.
//!
//! What it produces is a starting point, not an answer: it cannot know which
//! numbers are health and which are a random seed. That judgement is the part
//! a person has to bring.

use serde_json::Value;

/// A draft `manifest.json` for `doc`, plus what had to be guessed.
pub struct Draft {
    pub manifest: String,
    /// Human-readable notes about the guesses, shown beside the draft.
    pub notes: Vec<String>,
}

/// Deeper than this and a save is describing structure rather than settings;
/// the fields stop being things a player would recognise.
const MAX_DEPTH: usize = 4;

/// Beyond this many fields the draft is unreadable and the plugin author is
/// better served by a sample than a wall.
const MAX_FIELDS: usize = 120;

pub fn draft_from(game_name: &str, save_path: &str, doc: &Value) -> Draft {
    let mut fields = Vec::new();
    let mut notes = Vec::new();
    let mut truncated = false;

    collect(doc, "", 0, &mut fields, &mut truncated);

    if truncated {
        notes.push(format!(
            "Stopped after {MAX_FIELDS} fields; the save has more. Keep the ones that mean something and delete the rest."
        ));
    }

    let arrays = count_arrays(doc, "", 0);
    if arrays > 0 {
        notes.push(format!(
            "Found {arrays} list(s). Lists need a `lists` entry rather than a field, see docs/plugin-development.md."
        ));
    }

    notes.push(
        "Ranges are guesses based on the current value. Set them to what the game is known to accept, or remove them.".into(),
    );
    notes.push(
        "Pick `identify` pointers that another game would not have, or this plugin will try to open other files.".into(),
    );

    let id = slug(game_name);
    let folder = save_path
        .rsplit(['/', '\\'])
        .nth(1)
        .unwrap_or("MyGame")
        .to_string();
    let file = save_path.rsplit(['/', '\\']).next().unwrap_or("save.json");

    let identify: Vec<&Field> = fields.iter().take(2).collect();
    let identify_json = identify
        .iter()
        .map(|f| format!("    {{ \"pointer\": \"{}\" }}", f.pointer))
        .collect::<Vec<_>>()
        .join(",\n");

    let fields_json = fields
        .iter()
        .map(|f| f.to_json())
        .collect::<Vec<_>>()
        .join(",\n");

    let manifest = format!(
        r#"{{
  "id": "{id}",
  "name": "{game_name}",
  "version": "0.1.0",
  "description": "",
  "format": "json",

  "save_locations": [
    {{
      "root": "{{APPDATA}}/{folder}",
      "pattern": "{file}"
    }}
  ],

  "identify": [
{identify_json}
  ],

  "groups": [
    {{
      "id": "main",
      "label": "{game_name}",
      "fields": [
{fields_json}
      ]
    }}
  ]
}}
"#
    );

    Draft { manifest, notes }
}

struct Field {
    pointer: String,
    id: String,
    label: String,
    kind: &'static str,
    min: Option<i64>,
    max: Option<i64>,
}

impl Field {
    fn to_json(&self) -> String {
        let mut parts = vec![
            format!("\"id\": \"{}\"", self.id),
            format!("\"label\": \"{}\"", self.label),
            format!("\"pointer\": \"{}\"", self.pointer),
            format!("\"type\": \"{}\"", self.kind),
        ];
        if let (Some(min), Some(max)) = (self.min, self.max) {
            parts.push(format!("\"min\": {min}"));
            parts.push(format!("\"max\": {max}"));
        }
        format!("        {{ {} }}", parts.join(", "))
    }
}

fn collect(value: &Value, prefix: &str, depth: usize, out: &mut Vec<Field>, truncated: &mut bool) {
    if depth > MAX_DEPTH || out.len() >= MAX_FIELDS {
        if out.len() >= MAX_FIELDS {
            *truncated = true;
        }
        return;
    }

    let Value::Object(map) = value else { return };

    for (key, child) in map {
        let pointer = format!("{prefix}/{}", key.replace('~', "~0").replace('/', "~1"));

        match child {
            Value::Object(_) => collect(child, &pointer, depth + 1, out, truncated),
            // Arrays need a `lists` entry, which is a judgement call about what
            // the rows mean; the notes point the author at it instead.
            Value::Array(_) => {}
            Value::Null => {}
            _ => {
                if out.len() >= MAX_FIELDS {
                    *truncated = true;
                    return;
                }
                out.push(describe(&pointer, key, child));
            }
        }
    }
}

fn describe(pointer: &str, key: &str, value: &Value) -> Field {
    let (kind, min, max) = match value {
        Value::Bool(_) => ("boolean", None, None),
        Value::String(_) => ("text", None, None),
        Value::Number(n) if n.is_f64() && n.as_f64().map(|f| f.fract() != 0.0).unwrap_or(false) => {
            ("number", None, None)
        }
        Value::Number(n) => {
            // A range an order of magnitude around what is already there: wide
            // enough to be useful, obviously a guess.
            let current = n.as_f64().unwrap_or(0.0).abs().max(1.0);
            let ceiling = (current * 10.0).min(999_999_999.0) as i64;
            let is_float = n.is_f64();
            (
                if is_float { "number" } else { "integer" },
                Some(0),
                Some(ceiling.max(100)),
            )
        }
        _ => ("text", None, None),
    };

    Field {
        pointer: pointer.to_string(),
        id: slug(key),
        label: prettify(key),
        kind,
        min,
        max,
    }
}

fn count_arrays(value: &Value, prefix: &str, depth: usize) -> usize {
    if depth > MAX_DEPTH {
        return 0;
    }
    let Value::Object(map) = value else { return 0 };

    map.iter()
        .map(|(key, child)| match child {
            Value::Array(a) if !a.is_empty() => 1,
            Value::Object(_) => count_arrays(child, &format!("{prefix}/{key}"), depth + 1),
            _ => 0,
        })
        .sum()
}

fn slug(text: &str) -> String {
    let s: String = text
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let trimmed = s.trim_matches('-').to_string();
    // Collapse runs of separators so "My  Game!" does not become "my--game-".
    let mut out = String::with_capacity(trimmed.len());
    let mut last_dash = false;
    for c in trimmed.chars() {
        if c == '-' {
            if !last_dash {
                out.push(c);
            }
            last_dash = true;
        } else {
            out.push(c);
            last_dash = false;
        }
    }
    if out.is_empty() {
        "my-game".into()
    } else {
        out
    }
}

fn prettify(key: &str) -> String {
    key.split(['_', '-'])
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut c = p.chars();
            match c.next() {
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> Value {
        json!({
            "player": { "name": "Alex", "gold": 250, "health": 6, "speed": 1.5, "alive": true },
            "inventory": [{ "name": "Rope" }],
            "nothing": null
        })
    }

    #[test]
    fn the_draft_is_a_manifest_that_actually_loads() {
        let draft = draft_from("My Game", "C:/x/MyGame/save.json", &sample());
        let parsed: crate::plugins::manifest::Manifest =
            serde_json::from_str(&draft.manifest).expect("draft did not parse");

        assert_eq!(parsed.id, "my-game");
        assert_eq!(parsed.format, "json");
        assert!(!parsed.identify.is_empty());
        assert!(!parsed.groups[0].fields.is_empty());
    }

    #[test]
    fn types_follow_the_values_found() {
        let draft = draft_from("G", "save.json", &sample());
        let parsed: crate::plugins::manifest::Manifest =
            serde_json::from_str(&draft.manifest).unwrap();

        let by_id = |id: &str| {
            parsed.groups[0]
                .fields
                .iter()
                .find(|f| f.id == id)
                .unwrap_or_else(|| panic!("{id} missing"))
                .kind
        };
        use crate::plugins::manifest::FieldKind::*;
        assert_eq!(by_id("gold"), Integer);
        assert_eq!(by_id("speed"), Number);
        assert_eq!(by_id("name"), Text);
        assert_eq!(by_id("alive"), Boolean);
    }

    #[test]
    fn pointers_resolve_against_the_save_they_came_from() {
        let doc = sample();
        let draft = draft_from("G", "save.json", &doc);
        let parsed: crate::plugins::manifest::Manifest =
            serde_json::from_str(&draft.manifest).unwrap();

        for field in &parsed.groups[0].fields {
            assert!(
                doc.pointer(&field.pointer).is_some(),
                "{} does not resolve",
                field.pointer
            );
        }
    }

    #[test]
    fn nulls_and_arrays_are_left_to_the_author() {
        let draft = draft_from("G", "save.json", &sample());
        assert!(!draft.manifest.contains("/nothing"));
        assert!(!draft.manifest.contains("/inventory"));
        assert!(
            draft.notes.iter().any(|n| n.contains("list")),
            "the author was not told about the array"
        );
    }

    #[test]
    fn the_notes_admit_what_was_guessed() {
        let draft = draft_from("G", "save.json", &sample());
        assert!(draft.notes.iter().any(|n| n.contains("Ranges are guesses")));
        assert!(draft.notes.iter().any(|n| n.contains("identify")));
    }

    #[test]
    fn a_name_with_punctuation_still_makes_a_usable_id() {
        assert_eq!(slug("My  Game!"), "my-game");
        assert_eq!(slug("Pathogenic"), "pathogenic");
        assert_eq!(slug("!!!"), "my-game");
    }

    #[test]
    fn a_huge_save_is_truncated_rather_than_unreadable() {
        let mut map = serde_json::Map::new();
        for i in 0..500 {
            map.insert(format!("stat_{i}"), json!(i));
        }
        let draft = draft_from("G", "save.json", &Value::Object(map));
        let parsed: crate::plugins::manifest::Manifest =
            serde_json::from_str(&draft.manifest).unwrap();

        assert!(parsed.groups[0].fields.len() <= MAX_FIELDS);
        assert!(draft.notes.iter().any(|n| n.contains("Stopped after")));
    }
}
