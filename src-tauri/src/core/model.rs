use serde::{Deserialize, Serialize};
use serde_json::Value;

/// A game the app can edit, as shown on the game-selection screen.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameSummary {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    /// A `data:` URI, or `None` when the GUI should draw its own tile.
    pub icon: Option<String>,
}

/// One save file found on disk.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSummary {
    /// Absolute path, also the identity used by later commands.
    pub path: String,
    /// Headline, e.g. the character or farm name.
    pub title: String,
    /// Secondary line, e.g. `Seed: UDWTYHDA`.
    pub subtitle: String,
    /// Last-modified time, ISO-8601 local.
    pub modified: String,
    pub size_bytes: u64,
}

/// Identifies the exact revision of a file we read.
///
/// Carried out to the GUI and handed back on write, so an edit can be refused
/// if the game rewrote the save in the meantime. Cheap fields only, no
/// hashing, because this runs on every open and every save.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveStamp {
    /// Milliseconds since the Unix epoch. `0` when the platform cannot say.
    pub modified_ms: i64,
    pub size_bytes: u64,
}

/// The editor screen: descriptors and current values together, so the GUI can
/// render without knowing anything about the game.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorDocument {
    pub game_id: String,
    pub game_name: String,
    pub save_path: String,
    pub groups: Vec<GroupView>,
    /// One-click sets of edits that apply to this save.
    pub presets: Vec<PresetView>,
    /// The revision this screen was built from.
    pub stamp: SaveStamp,
    /// This game's processes found running when the save was opened. Empty is
    /// the normal case; anything here means the game may overwrite the edit.
    pub game_running: Vec<String>,
    /// The save folder looks like it is synchronised by Steam Cloud, which can
    /// undo a local edit.
    pub cloud_synced: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PresetView {
    pub id: String,
    pub label: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupView {
    pub id: String,
    pub label: String,
    pub description: String,
    /// Set when the section does not apply to this save. The GUI shows the
    /// explanation instead of the fields, rather than hiding the section and
    /// leaving the player to guess.
    pub absent_reason: Option<String>,
    pub fields: Vec<FieldView>,
    pub lists: Vec<ListView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldView {
    pub id: String,
    pub label: String,
    pub help: String,
    /// Absolute pointer into the document. The GUI echoes this back when
    /// submitting an edit.
    pub pointer: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub value: Value,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub max_length: Option<usize>,
    pub options: Vec<ChoiceView>,
    pub read_only: bool,
    /// Set when the pointer does not resolve in this particular save; the GUI
    /// greys the row out instead of pretending the value is zero.
    pub missing: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChoiceView {
    pub value: Value,
    pub label: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListView {
    pub id: String,
    pub label: String,
    pub description: String,
    pub items: Vec<ListItemView>,
    /// May the player add or delete rows here? Games define some lists
    /// themselves, equipment slots, for instance, and inventing an entry
    /// would break the save.
    pub allow_add: bool,
    pub allow_remove: bool,
    /// Buttons that set the same field on every row at once. The GUI expands
    /// one into ordinary edits, so they go through the same validation,
    /// confirmation and backup as anything typed by hand.
    pub bulk_actions: Vec<BulkActionView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkActionView {
    pub id: String,
    pub label: String,
    /// Which field of each row to write; `None` for object-backed lists, where
    /// each row has only one value.
    pub field: Option<String>,
    pub value: Value,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListItemView {
    pub label: String,
    /// Position in the underlying array. Not the same as the position on
    /// screen once a filter hides some rows, and it is this one that a delete
    /// has to name.
    pub index: usize,
    /// Field views with pointers already expanded to absolute form,
    /// e.g. `/player/loadout/0/rarity`.
    pub fields: Vec<FieldView>,
}

/// A single change submitted by the GUI.
#[derive(Debug, Clone, Deserialize)]
pub struct Edit {
    pub pointer: String,
    pub value: Value,
}

/// Result of a successful write, shown as confirmation.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteReport {
    pub backup_id: String,
    pub changed_fields: usize,
    pub save_path: String,
    /// The revision the file now has, so the editor can keep editing without
    /// tripping its own stale-file check on the next save.
    pub stamp: SaveStamp,
}

/// One field that differs between two versions of a save.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldChange {
    pub pointer: String,
    pub label: String,
    /// `None` when the field was absent in that version.
    pub before: Option<Value>,
    pub after: Option<Value>,
}

/// A copy of a save that the game itself made.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryFile {
    pub path: String,
    pub name: String,
    pub created: String,
    pub size_bytes: u64,
}

/// One entry in the backup history.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BackupEntry {
    pub id: String,
    pub game_id: String,
    /// Where this backup will be restored to.
    pub original_path: String,
    pub created: String,
    pub size_bytes: u64,
}
