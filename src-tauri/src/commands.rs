use crate::backup::BackupManager;
use crate::core::error::{Error, Result};
use crate::core::model::*;
use crate::plugins::registry::{PluginProblem, Registry};
use crate::save;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use tauri::State;

/// Everything the commands need. Built once at startup.
pub struct AppState {
    pub registry: RwLock<Registry>,
    pub plugin_dirs: Vec<PathBuf>,
    pub backups: BackupManager,
    pub backup_root: PathBuf,
    /// Item artwork per game, read once and kept.
    ///
    /// Pulling 118 pictures out of a 1.4 GB archive takes about 150 ms, fine
    /// once, wasteful on every trip back to the editor.
    icon_cache: RwLock<std::collections::HashMap<String, std::sync::Arc<ItemIcons>>>,
}

/// Option value -> `data:` URI.
pub type ItemIcons = std::collections::HashMap<String, String>;

impl AppState {
    pub fn new(plugin_dirs: Vec<PathBuf>, backup_root: PathBuf) -> Self {
        Self {
            registry: RwLock::new(Registry::load(&plugin_dirs)),
            plugin_dirs,
            backups: BackupManager::new(backup_root.clone()),
            backup_root,
            icon_cache: RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppInfo {
    pub version: String,
    pub backup_folder: String,
    pub plugin_folders: Vec<String>,
    pub plugin_problems: Vec<PluginProblem>,
}

#[tauri::command]
pub fn app_info(state: State<AppState>) -> AppInfo {
    let registry = state.registry.read().expect("registry lock");
    AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        backup_folder: state.backup_root.display().to_string(),
        plugin_folders: state
            .plugin_dirs
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        plugin_problems: registry.problems().to_vec(),
    }
}

/// Games shown on the first screen.
#[tauri::command]
pub fn list_games(state: State<AppState>) -> Vec<GameSummary> {
    let registry = state.registry.read().expect("registry lock");
    registry
        .all()
        .map(|p| GameSummary {
            id: p.manifest.id.clone(),
            name: p.manifest.name.clone(),
            version: p.manifest.version.clone(),
            description: p.manifest.description.clone(),
            author: p.manifest.author.clone(),
            icon: crate::core::icon::resolve(&p.manifest, &p.dir),
        })
        .collect()
}

/// Re-scan the plugin folders, so a newly dropped-in plugin appears without a
/// restart.
#[tauri::command]
pub fn reload_plugins(state: State<AppState>) -> Vec<GameSummary> {
    {
        let mut registry = state.registry.write().expect("registry lock");
        *registry = Registry::load(&state.plugin_dirs);
    }
    list_games(state)
}

#[tauri::command]
pub fn list_saves(
    state: State<AppState>,
    game_id: String,
    locale: String,
) -> Result<Vec<SaveSummary>> {
    let registry = state.registry.read().expect("registry lock");
    let plugin = registry.get(&game_id)?;
    save::detect::find_saves(&plugin.manifest, &locale)
}

#[tauri::command]
pub fn open_save(
    state: State<AppState>,
    game_id: String,
    path: String,
    locale: String,
) -> Result<EditorDocument> {
    let registry = state.registry.read().expect("registry lock");
    let plugin = registry.get(&game_id)?;
    let path = checked_path(&plugin.manifest, &path)?;

    let doc = save::detect::load_document(&plugin.manifest, &path)?;

    // Collected here rather than inside the editor so the field layer stays
    // ignorant of processes and cloud folders.
    let context = save::editor::Context {
        stamp: save::detect::stamp_of(&path),
        game_running: crate::core::process::running_among(&plugin.manifest.process_names),
        cloud_synced: save::detect::is_cloud_synced(&path),
    };

    Ok(save::editor::build(
        &plugin.manifest,
        &path.to_string_lossy(),
        &doc,
        &locale,
        context,
    ))
}

/// Apply edits. Always takes a backup first, see [`save::apply_and_write`].
#[tauri::command]
pub fn write_save(
    state: State<AppState>,
    game_id: String,
    path: String,
    edits: Vec<Edit>,
    expected: Option<SaveStamp>,
    // `confirm` is false on the first attempt. If the backend answers
    // `error.needsConfirmation`, the GUI shows what is risky and calls again
    // with true.
    confirm: bool,
) -> Result<WriteReport> {
    let registry = state.registry.read().expect("registry lock");
    let plugin = registry.get(&game_id)?;
    let path = checked_path(&plugin.manifest, &path)?;

    save::apply_and_write(
        &plugin.manifest,
        &state.backups,
        &path,
        &edits,
        expected.as_ref(),
        confirm,
    )
}

/// Run a preset.
///
/// Expanded into ordinary edits and pushed through the normal write pipeline,
/// so a preset that leaves the safe range asks for confirmation exactly like a
/// hand-typed value would.
#[tauri::command]
pub fn apply_preset(
    state: State<AppState>,
    game_id: String,
    path: String,
    preset_id: String,
    expected: Option<SaveStamp>,
    confirm: bool,
) -> Result<WriteReport> {
    let registry = state.registry.read().expect("registry lock");
    let plugin = registry.get(&game_id)?;
    let path = checked_path(&plugin.manifest, &path)?;

    let doc = save::detect::load_document(&plugin.manifest, &path)?;
    let preset = save::presets::find(&plugin.manifest, &doc, &preset_id)
        .ok_or_else(|| Error::UnknownField(preset_id.clone()))?;
    let edits = save::presets::expand(&plugin.manifest, &doc, preset);

    save::apply_and_write(
        &plugin.manifest,
        &state.backups,
        &path,
        &edits,
        expected.as_ref(),
        confirm,
    )
}

/// Add or remove one row of a list.
///
/// Separate from `write_save` because inserting or deleting renumbers every
/// row after it; mixing that with pointer-addressed field edits in one batch
/// would write values to the wrong items. This writes immediately, through the
/// same backup-and-verify pipeline, and the editor reloads afterwards.
#[tauri::command]
pub fn change_list_row(
    state: State<AppState>,
    game_id: String,
    path: String,
    list_id: String,
    change: save::structure::RowChange,
    index: usize,
    expected: Option<SaveStamp>,
) -> Result<WriteReport> {
    let registry = state.registry.read().expect("registry lock");
    let plugin = registry.get(&game_id)?;
    let path = checked_path(&plugin.manifest, &path)?;

    save::structure::change_row(
        &plugin.manifest,
        &state.backups,
        &path,
        &list_id,
        change,
        index,
        expected.as_ref(),
    )
}

/// Artwork for the items in this game's dropdowns, read from the player's own
/// installation. Empty when the game is not installed, which is not an error.
#[tauri::command]
pub fn item_icons(state: State<AppState>, game_id: String) -> std::sync::Arc<ItemIcons> {
    if let Some(cached) = state
        .icon_cache
        .read()
        .expect("icon cache")
        .get(&game_id)
        .cloned()
    {
        return cached;
    }

    let icons = {
        let registry = state.registry.read().expect("registry lock");
        match registry.get(&game_id) {
            Ok(plugin) => {
                // Only the option sets the plugin actually declares artwork for.
                let sets: Vec<(&str, Vec<String>)> = plugin
                    .manifest
                    .item_icons
                    .iter()
                    .filter_map(|spec| {
                        let values = plugin
                            .manifest
                            .option_sets
                            .get(&spec.options_ref)?
                            .iter()
                            .filter_map(|c| c.value.as_str().map(str::to_string))
                            .collect();
                        Some((spec.options_ref.as_str(), values))
                    })
                    .collect();
                crate::plugins::archive::item_icons(&plugin.manifest, &sets)
            }
            Err(_) => Default::default(),
        }
    };

    let shared = std::sync::Arc::new(icons);
    state
        .icon_cache
        .write()
        .expect("icon cache")
        .insert(game_id, shared.clone());
    shared
}

/// The game's own copies sitting beside a save: its `.bak` files and
/// anything it quarantined.
#[tauri::command]
pub fn list_recovery_files(
    state: State<AppState>,
    game_id: String,
    path: String,
) -> Result<Vec<RecoveryFile>> {
    let registry = state.registry.read().expect("registry lock");
    let plugin = registry.get(&game_id)?;
    let path = checked_path(&plugin.manifest, &path)?;
    Ok(save::recovery::find_for(&plugin.manifest, &path))
}

/// What a backup, or one of the game's own copies, would change if restored.
#[tauri::command]
pub fn preview_restore(
    state: State<AppState>,
    game_id: String,
    save_path: String,
    source_path: String,
    locale: String,
) -> Result<Vec<FieldChange>> {
    let registry = state.registry.read().expect("registry lock");
    let plugin = registry.get(&game_id)?;
    let save_path = checked_path(&plugin.manifest, &save_path)?;

    let source = Path::new(&source_path);
    if !save::recovery::within_plugin_reach(&plugin.manifest, source) {
        return Err(Error::PathNotAllowed);
    }

    let current = save::detect::load_document(&plugin.manifest, &save_path)?;
    let candidate = save::detect::load_document(&plugin.manifest, source)?;
    Ok(save::diff::compare(
        &plugin.manifest,
        &current,
        &candidate,
        &locale,
    ))
}

/// Copy one of the game's own files over the live save.
///
/// The file being replaced is backed up first, so this is as undoable as any
/// other write.
#[tauri::command]
pub fn restore_recovery_file(
    state: State<AppState>,
    game_id: String,
    save_path: String,
    source_path: String,
) -> Result<WriteReport> {
    let registry = state.registry.read().expect("registry lock");
    let plugin = registry.get(&game_id)?;
    let save_path = checked_path(&plugin.manifest, &save_path)?;

    let source = Path::new(&source_path);
    if !save::recovery::is_beside(&save_path, source) {
        return Err(Error::PathNotAllowed);
    }

    // Refuse a file the game would not accept either.
    save::detect::load_document(&plugin.manifest, source)?;
    let bytes = std::fs::read(source).map_err(|e| Error::SaveRead(e.to_string()))?;

    let backup_id = state.backups.create(&plugin.manifest.id, &save_path)?;
    save::io::write_atomically(&save_path, &bytes)?;

    Ok(WriteReport {
        backup_id,
        changed_fields: 0,
        save_path: save_path.to_string_lossy().into_owned(),
        stamp: save::detect::stamp_of(&save_path),
    })
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DraftPlugin {
    pub manifest: String,
    pub notes: Vec<String>,
}

/// Propose a starting `manifest.json` for a save file nothing supports yet.
///
/// The path is left unrestricted here, since the whole point is to look at a game
/// that has no plugin, so there are no declared folders to confine it to. The
/// file is only ever read, and only ever parsed as JSON.
#[tauri::command]
pub fn draft_plugin(game_name: String, path: String) -> Result<DraftPlugin> {
    let bytes = std::fs::read(&path).map_err(|e| Error::SaveRead(e.to_string()))?;
    let doc = crate::plugins::adapter::adapter_for("json")?.parse(&bytes)?;

    let draft = crate::plugins::scaffold::draft_from(&game_name, &path, &doc);
    Ok(DraftPlugin {
        manifest: draft.manifest,
        notes: draft.notes,
    })
}

/// How many of a save's values this plugin can edit.
#[tauri::command]
pub fn plugin_coverage(
    state: State<AppState>,
    game_id: String,
    path: String,
) -> Result<(usize, usize)> {
    let registry = state.registry.read().expect("registry lock");
    let plugin = registry.get(&game_id)?;
    let path = checked_path(&plugin.manifest, &path)?;
    let doc = save::detect::load_document(&plugin.manifest, &path)?;
    Ok(save::editor::coverage(&plugin.manifest, &doc))
}

#[tauri::command]
pub fn list_backups(state: State<AppState>, game_id: Option<String>) -> Vec<BackupEntry> {
    state.backups.list(game_id.as_deref())
}

#[tauri::command]
pub fn restore_backup(state: State<AppState>, backup_id: String) -> Result<String> {
    state.backups.restore(&backup_id)
}

#[tauri::command]
pub fn delete_backup(state: State<AppState>, backup_id: String) -> Result<()> {
    state.backups.delete(&backup_id)
}

/// Confine every read and write to the folders the plugin declares.
///
/// The GUI can only send back a path the backend handed it, but a command is a
/// public boundary: this makes a malformed or hostile request unable to reach
/// an arbitrary file on disk.
fn checked_path(manifest: &crate::plugins::manifest::Manifest, raw: &str) -> Result<PathBuf> {
    let path = Path::new(raw);
    if !save::detect::is_within_declared_locations(manifest, path) {
        return Err(Error::PathNotAllowed);
    }
    Ok(path.to_path_buf())
}
