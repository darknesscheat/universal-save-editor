use crate::core::i18n::Translations;
use serde::{Deserialize, Serialize};

/// A game plugin, described entirely by data.
///
/// Adding support for a new game whose save is JSON needs **no Rust code**,
/// only a `manifest.json` in the `plugins/` folder. Games with an exotic
/// container format additionally need a [`crate::plugins::adapter::FormatAdapter`],
/// but the field/validation/GUI layer below is shared by all of them.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: String,
    #[serde(default)]
    pub description: String,

    /// Which container parser to use. Only `"json"` ships in the MVP.
    pub format: String,

    /// A picture for the game-selection screen, relative to the plugin folder.
    ///
    /// Only for art a plugin author is allowed to redistribute. To show a
    /// game's real artwork, point `icon_sources` at a copy the player already
    /// has instead of bundling one.
    #[serde(default)]
    pub icon: Option<String>,

    /// Globs for pictures already present on the player's computer, tried in
    /// order when `icon` is absent or missing. See [`crate::core::icon`].
    #[serde(default)]
    pub icon_sources: Vec<IconSource>,

    /// Where to find artwork for the *items* in a dropdown, inside the game's
    /// own installed files. See [`crate::plugins::archive`].
    #[serde(default)]
    pub item_icons: Vec<ItemIcons>,

    /// Where this game keeps its saves, per platform.
    pub save_locations: Vec<SaveLocation>,

    /// Executable names (without extension) this game runs under.
    ///
    /// Used only to warn the player that the game is open and may write over
    /// their edits when it exits, which is exactly how a set of edits was
    /// lost while this app was being built.
    #[serde(default)]
    pub process_names: Vec<String>,

    /// Rules that relate one field to another, checked on every write.
    #[serde(default)]
    pub constraints: Vec<Constraint>,

    /// Globs, relative to a save's own folder, matching copies the *game*
    /// keeps, its rolling `.bak` files and anything it quarantined.
    ///
    /// Only ever read. Offering them turns "my save is broken" into a list of
    /// working copies instead of a dead end.
    #[serde(default)]
    pub recovery_patterns: Vec<String>,

    /// Cheap sanity checks run after parsing: if any fails we refuse the file
    /// rather than showing the user a screen full of empty fields.
    #[serde(default)]
    pub identify: Vec<IdentifyRule>,

    /// How to title a save in the save-picker.
    #[serde(default)]
    pub label: SaveLabel,

    /// Reusable dropdown option lists, referenced by `options_ref`.
    #[serde(default)]
    pub option_sets: std::collections::HashMap<String, Vec<Choice>>,

    /// The editor screen, section by section.
    pub groups: Vec<Group>,

    /// One-click sets of edits, "refill health", "make everything legendary".
    ///
    /// They are ordinary edits underneath, so a preset goes through the same
    /// validation, the same confirmation when it leaves the safe range, and
    /// the same backup as anything typed by hand.
    #[serde(default)]
    pub presets: Vec<Preset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Preset {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub label_i18n: Translations,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub description_i18n: Translations,
    /// Offer it only when this pointer resolves, the same way a group works.
    #[serde(default)]
    pub requires: Option<String>,
    /// Literal pointer/value pairs.
    #[serde(default)]
    pub set: Vec<PresetEdit>,
    /// Set one field across every row of a list.
    #[serde(default)]
    pub set_in_lists: Vec<PresetListEdit>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetEdit {
    pub pointer: String,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetListEdit {
    /// The list's id, as declared in a group.
    pub list: String,
    /// Which field of each row; omitted for object-backed lists.
    #[serde(default)]
    pub field: Option<String>,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveLocation {
    /// Empty means "every platform".
    #[serde(default)]
    pub platforms: Vec<String>,
    /// Folder template, e.g. `{APPDATA}/Godot/app_userdata/Pathogenic`.
    pub root: String,
    /// Glob relative to `root`, e.g. `profile_*/run_save.json`.
    pub pattern: String,
    /// What to call saves found here, e.g. `"Current run"`. Games often keep
    /// more than one kind of save file side by side.
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub label_i18n: Translations,
    /// Markers specific to this pattern. Falls back to the manifest-level
    /// `identify` when absent.
    #[serde(default)]
    pub identify: Option<Vec<IdentifyRule>>,
}

/// A relationship between two fields that a valid save has to satisfy.
///
/// Range checks catch a value that is silly on its own; this catches a pair
/// that is silly together, health above maximum health, say. Each side is a
/// JSON pointer, so no game-specific code is involved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Constraint {
    pub left: String,
    pub right: String,
    pub rule: ConstraintRule,
    /// Shown when the rule is broken. Falls back to a generated sentence.
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub message_i18n: Translations,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConstraintRule {
    /// left <= right
    Lte,
    /// left >= right
    Gte,
}

impl Constraint {
    /// `None` when the rule holds or either side is missing or not numeric,
    /// a constraint has nothing to say about a field this save does not have.
    pub fn violated_by(&self, doc: &serde_json::Value) -> bool {
        let (Some(l), Some(r)) = (
            doc.pointer(&self.left).and_then(|v| v.as_f64()),
            doc.pointer(&self.right).and_then(|v| v.as_f64()),
        ) else {
            return false;
        };
        match self.rule {
            ConstraintRule::Lte => l > r,
            ConstraintRule::Gte => l < r,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IconSource {
    /// Empty means "every platform".
    #[serde(default)]
    pub platforms: Vec<String>,
    /// Glob template, e.g. `{STEAM}/appcache/librarycache/3808690/*.jpg`.
    /// When several files match, the largest is used.
    pub path: String,
}

/// Item artwork read out of the game's own archive.
///
/// Names are not a reliable index into game art: Pathogenic's `assault_rifle`
/// draws a file called `Player weapon - 3_shot_burst.png`. So a plugin points
/// at the *resource* for each option and the reader follows it to whatever
/// texture it actually uses.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemIcons {
    /// Which `option_sets` entry these pictures belong to.
    pub options_ref: String,
    /// The engine archive format. `"godot_pck"` is implemented.
    pub format: String,
    /// Glob for the archive in the player's installation.
    pub archive: String,
    /// Path of the resource for one option, with `{value}` substituted.
    pub resource_pattern: String,
    #[serde(default)]
    pub platforms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifyRule {
    /// RFC-6901 JSON pointer that must resolve for the file to be accepted.
    pub pointer: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveLabel {
    /// Pointer whose value becomes the save's headline, if present.
    #[serde(default)]
    pub title_pointer: Option<String>,
    /// Pointer shown underneath, e.g. a seed or play time.
    #[serde(default)]
    pub subtitle_pointer: Option<String>,
    /// Prefix for the subtitle, e.g. `"Seed: "`.
    #[serde(default)]
    pub subtitle_prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Group {
    pub id: String,
    pub label: String,
    /// Locale tag -> translated label. See [`crate::core::i18n`].
    #[serde(default)]
    pub label_i18n: Translations,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub description_i18n: Translations,
    /// Show this group only when the pointer resolves.
    ///
    /// Lets one plugin cover several kinds of save file, a run-in-progress and
    /// a profile, say, without showing the player a screen of greyed-out rows
    /// that do not apply to the file they opened.
    #[serde(default)]
    pub requires: Option<String>,
    /// Shown in place of the section when `requires` is not satisfied.
    ///
    /// Hiding a section silently is how a player concludes a feature does not
    /// exist. Pathogenic's equipment lives in a file the game deletes when a
    /// run ends, and the section simply vanished, saying so costs one line.
    #[serde(default)]
    pub when_absent: String,
    #[serde(default)]
    pub when_absent_i18n: Translations,
    #[serde(default)]
    pub fields: Vec<Field>,
    #[serde(default)]
    pub lists: Vec<ListField>,
}

/// One editable value.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub label_i18n: Translations,
    #[serde(default)]
    pub help: String,
    #[serde(default)]
    pub help_i18n: Translations,
    /// RFC-6901 JSON pointer into the save document. For list item fields this
    /// is relative to the item.
    pub pointer: String,
    #[serde(rename = "type")]
    pub kind: FieldKind,
    #[serde(default)]
    pub min: Option<f64>,
    #[serde(default)]
    pub max: Option<f64>,
    #[serde(default)]
    pub max_length: Option<usize>,
    /// Inline dropdown options.
    #[serde(default)]
    pub options: Vec<Choice>,
    /// Dropdown options pulled from `option_sets`.
    #[serde(default)]
    pub options_ref: Option<String>,
    /// Shown in the GUI but rejected on write. Useful for context like a seed.
    #[serde(default)]
    pub read_only: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FieldKind {
    /// Whole number. Written back as a JSON integer, never `1.0`, which some
    /// engines reject.
    Integer,
    /// Decimal number. Always written with a fractional part preserved.
    Number,
    Text,
    Boolean,
    /// One of a fixed set of values.
    Choice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Choice {
    pub value: serde_json::Value,
    pub label: String,
    #[serde(default)]
    pub label_i18n: Translations,
}

/// Keeps a list to the items its test accepts.
///
/// Prefer `starts_with` over `equals` when a game numbers its slots. Listing
/// `ESlot1`…`ESlot4` looks safe but silently hides anything the game adds
/// later: Pathogenic turned out to use `ESlot5` and `ESlot6` in late runs, and
/// those weapons were invisible in the editor until the filter matched on the
/// prefix instead.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemFilter {
    /// Pointer relative to the item, e.g. `/slot`.
    pub pointer: String,
    /// The value must be exactly one of these.
    #[serde(default)]
    pub equals: Vec<serde_json::Value>,
    /// …or the value is a string beginning with one of these.
    #[serde(default)]
    pub starts_with: Vec<String>,
    /// …and never one of these, which wins over the two above. Lets a broad
    /// prefix carve out an exception without enumerating everything else.
    #[serde(default)]
    pub except: Vec<serde_json::Value>,
}

impl ItemFilter {
    pub fn matches(&self, item: &serde_json::Value) -> bool {
        let Some(value) = item.pointer(&self.pointer) else {
            return false;
        };
        if self.except.iter().any(|e| e == value) {
            return false;
        }

        let by_value = self.equals.iter().any(|e| e == value);
        let by_prefix = value
            .as_str()
            .map(|s| self.starts_with.iter().any(|p| s.starts_with(p.as_str())))
            .unwrap_or(false);

        // A filter that declares no test at all keeps everything, which is the
        // least surprising reading of "no condition".
        if self.equals.is_empty() && self.starts_with.is_empty() {
            return true;
        }
        by_value || by_prefix
    }
}

/// Where a list's rows come from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ListSource {
    /// A JSON array; each element is a row. The usual case.
    #[default]
    Array,
    /// A JSON object; each key is a row and its value the thing being edited.
    ///
    /// Games keep a surprising amount this way, Pathogenic stores 161 unlock
    /// flags and kill counters as five plain objects, and none of it was
    /// reachable while lists could only be arrays.
    Object,
}

/// One button that writes the same value into every row of a list.
///
/// Forty-nine "discovered" flags is not something anyone should tick one at a
/// time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkAction {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub label_i18n: Translations,
    /// Which field of each row to write. Omitted for an object source, where
    /// there is only ever the one value per row.
    #[serde(default)]
    pub field: Option<String>,
    pub value: serde_json::Value,
}

/// A repeated structure: an inventory, an equipment loadout, a quest log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListField {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub label_i18n: Translations,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub description_i18n: Translations,
    /// Pointer to the JSON array, or object, see [`ListSource`].
    pub pointer: String,
    /// Whether `pointer` holds an array of items or an object of key/value
    /// pairs. Defaults to an array.
    #[serde(default)]
    pub source: ListSource,
    /// For an object source: the field describing each value. The item label
    /// is the key itself, prettified.
    #[serde(default)]
    pub entry: Option<Field>,
    /// Buttons that set the same field on every visible row at once.
    #[serde(default)]
    pub bulk_actions: Vec<BulkAction>,

    /// May the player add a row? Requires `new_item` to say what a new one
    /// looks like.
    #[serde(default)]
    pub allow_add: bool,
    /// May the player delete a row?
    #[serde(default)]
    pub allow_remove: bool,
    /// Refuse to delete below this many rows.
    #[serde(default)]
    pub min_items: Option<usize>,
    /// Refuse to add beyond this many rows.
    #[serde(default)]
    pub max_items: Option<usize>,
    /// The value inserted by "add". For an array source this is the whole new
    /// element; games are particular about the shape, so the plugin spells it
    /// out rather than the app guessing.
    #[serde(default)]
    pub new_item: Option<serde_json::Value>,
    /// Show only the items matching this test.
    ///
    /// Lets one array be surfaced as several lists with different rules, e.g.
    /// an equipment array split into weapon slots and organ slots, each
    /// offering only the items that legitimately fit there.
    #[serde(default)]
    pub item_filter: Option<ItemFilter>,
    /// Pointer *within an item* used as its display name.
    #[serde(default)]
    pub item_label_pointer: Option<String>,
    /// Option set used to prettify the item label.
    #[serde(default)]
    pub item_label_options_ref: Option<String>,
    /// The editable columns of each item, with pointers relative to the row.
    ///
    /// An object source may use these too, when each key holds a record rather
    /// than a single value; it uses [`ListField::entry`] instead when it does
    /// not. Exactly one of the two applies.
    #[serde(default)]
    pub fields: Vec<Field>,
}

impl Manifest {
    /// Resolve a field's dropdown options, following `options_ref`.
    pub fn choices<'a>(&'a self, field: &'a Field) -> &'a [Choice] {
        if !field.options.is_empty() {
            return &field.options;
        }
        field
            .options_ref
            .as_ref()
            .and_then(|k| self.option_sets.get(k))
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Every field in the manifest.
    pub fn scalar_fields(&self) -> impl Iterator<Item = &Field> {
        self.groups.iter().flat_map(|g| g.fields.iter())
    }

    pub fn lists(&self) -> impl Iterator<Item = &ListField> {
        self.groups.iter().flat_map(|g| g.lists.iter())
    }

    /// The groups that apply to a particular save document.
    pub fn groups_for<'a>(&'a self, doc: &'a serde_json::Value) -> impl Iterator<Item = &'a Group> {
        self.groups.iter().filter(move |g| match &g.requires {
            Some(p) => doc.pointer(p).is_some(),
            None => true,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn item(slot: &str) -> serde_json::Value {
        json!({ "slot": slot })
    }

    fn filter(body: &str) -> ItemFilter {
        serde_json::from_str(body).unwrap()
    }

    #[test]
    fn equals_matches_exact_values() {
        let f = filter(r#"{"pointer":"/slot","equals":["ESlot1","ESlot2"]}"#);
        assert!(f.matches(&item("ESlot1")));
        assert!(!f.matches(&item("ESlot3")));
    }

    /// The bug this feature exists for: the game grew `ESlot5` and `ESlot6`,
    /// and an enumerated filter hid those weapons completely.
    #[test]
    fn starts_with_covers_slots_the_manifest_never_listed() {
        let f = filter(r#"{"pointer":"/slot","starts_with":["ESlot","EBackSlot"]}"#);
        for slot in [
            "ESlot1",
            "ESlot4",
            "ESlot5",
            "ESlot6",
            "ESlot99",
            "EBackSlot2",
        ] {
            assert!(f.matches(&item(slot)), "{slot} was filtered out");
        }
        assert!(!f.matches(&item("ISlot1")));
    }

    #[test]
    fn except_overrides_a_broad_prefix() {
        let f = filter(r#"{"pointer":"/slot","starts_with":["ESlot"],"except":["ESlot0"]}"#);
        assert!(f.matches(&item("ESlot1")));
        assert!(!f.matches(&item("ESlot0")));
    }

    #[test]
    fn a_filter_with_no_test_keeps_everything() {
        let f = filter(r#"{"pointer":"/slot"}"#);
        assert!(f.matches(&item("anything")));
    }

    #[test]
    fn a_missing_pointer_never_matches() {
        let f = filter(r#"{"pointer":"/slot","starts_with":["E"]}"#);
        assert!(!f.matches(&json!({ "other": "ESlot1" })));
    }

    fn constraint(rule: &str) -> Constraint {
        serde_json::from_str(&format!(
            r#"{{"left":"/hp","right":"/max_hp","rule":"{rule}"}}"#
        ))
        .unwrap()
    }

    #[test]
    fn lte_is_violated_only_when_left_exceeds_right() {
        let c = constraint("lte");
        assert!(c.violated_by(&json!({"hp": 10, "max_hp": 6})));
        assert!(!c.violated_by(&json!({"hp": 6, "max_hp": 6})));
        assert!(!c.violated_by(&json!({"hp": 1, "max_hp": 6})));
    }

    #[test]
    fn gte_is_the_mirror_of_lte() {
        let c = constraint("gte");
        assert!(c.violated_by(&json!({"hp": 1, "max_hp": 6})));
        assert!(!c.violated_by(&json!({"hp": 10, "max_hp": 6})));
    }

    /// A rule has nothing to say about a save that lacks either side, a
    /// profile save has no `/player/hp` and must not be rejected for it.
    #[test]
    fn a_constraint_ignores_fields_the_save_does_not_have() {
        let c = constraint("lte");
        assert!(!c.violated_by(&json!({"hp": 10})));
        assert!(!c.violated_by(&json!({})));
        assert!(!c.violated_by(&json!({"hp": "text", "max_hp": 6})));
    }
}
