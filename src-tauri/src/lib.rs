//! Universal Save Editor, edit offline game saves without touching raw files.
//!
//! The core knows nothing about any specific game. Everything game-shaped lives
//! in a plugin: a folder with a `manifest.json` describing where saves live,
//! which fields are editable, and what values are allowed.

pub mod backup;
pub mod commands;
pub mod core;
pub mod plugins;
pub mod save;

use std::path::PathBuf;
use tauri::Manager;

/// Drop Windows' `\\?\` extended-length prefix.
///
/// It is meaningful to the filesystem API and meaningless to a person reading
/// a folder path in Settings, where it only makes the same directory look like
/// two different ones.
fn tidy_path(path: PathBuf) -> PathBuf {
    let text = path.to_string_lossy();
    match text.strip_prefix(r"\\?\") {
        Some(stripped) => PathBuf::from(stripped),
        None => path,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data = app
                .path()
                .app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));

            let mut plugin_dirs = Vec::new();

            // Plugins shipped with the app.
            if let Ok(resources) = app.path().resource_dir() {
                plugin_dirs.push(resources.join("plugins"));
            }
            // Next to the executable, how a portable, unpacked copy is laid out.
            if let Ok(exe) = std::env::current_exe() {
                if let Some(dir) = exe.parent() {
                    plugin_dirs.push(dir.join("plugins"));
                }
            }
            // Running from a checkout: `src-tauri/../plugins`.
            #[cfg(debug_assertions)]
            plugin_dirs.push(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .parent()
                    .map(|p| p.join("plugins"))
                    .unwrap_or_default(),
            );
            // The user's own plugin folder, which wins on id collision.
            plugin_dirs.push(app_data.join("plugins"));

            // The same folder can arrive twice, `resource_dir()` hands back a
            // `\\?\` extended-length path on Windows while the checkout
            // fallback does not, and Settings would then list it twice.
            plugin_dirs = plugin_dirs
                .into_iter()
                .map(tidy_path)
                .fold(Vec::new(), |mut acc, dir| {
                    let key = dir.canonicalize().unwrap_or_else(|_| dir.clone());
                    if !acc.iter().any(|(k, _): &(PathBuf, PathBuf)| *k == key) {
                        acc.push((key, dir));
                    }
                    acc
                })
                .into_iter()
                .map(|(_, dir)| dir)
                .collect();

            let backup_root = app_data.join("backups");
            // Created eagerly so the Settings screen can always show the path
            // and the folder can be opened before the first edit.
            let _ = std::fs::create_dir_all(&backup_root);

            app.manage(commands::AppState::new(plugin_dirs, backup_root));
            remember_window(app, app_data.join("window.json"));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::app_info,
            commands::list_games,
            commands::reload_plugins,
            commands::list_saves,
            commands::open_save,
            commands::write_save,
            commands::change_list_row,
            commands::apply_preset,
            commands::list_recovery_files,
            commands::preview_restore,
            commands::restore_recovery_file,
            commands::draft_plugin,
            commands::plugin_coverage,
            commands::item_icons,
            commands::list_backups,
            commands::restore_backup,
            commands::delete_backup,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Universal Save Editor");
}

/// Restore the window to the size and place it was left in, and put it back
/// when the app closes.
///
/// The size in `tauri.conf.json` only ever applies to a first run. Everything
/// that decides *whether* a remembered position is usable lives in
/// [`core::window`], where it is tested; this function is the wiring.
fn remember_window(app: &tauri::App, path: PathBuf) {
    use crate::core::window::{self, WindowState};
    use tauri::{LogicalPosition, LogicalSize};

    let Some(win) = app.get_webview_window("main") else {
        return;
    };

    // Logical pixels throughout, see the note in `core::window`. Each monitor
    // is converted using *its own* scale factor, which is what keeps a
    // mixed-DPI desktop consistent.
    let monitors: Vec<window::Rect> = win
        .available_monitors()
        .unwrap_or_default()
        .iter()
        .map(|m| {
            let scale = m.scale_factor();
            let p = m.position().to_logical::<i32>(scale);
            let s = m.size().to_logical::<u32>(scale);
            (p.x, p.y, s.width, s.height)
        })
        .collect();

    if let Some(saved) = window::load(&path) {
        let sized = window::clamp_size(saved, &monitors);
        let _ = win.set_size(LogicalSize::new(sized.width, sized.height));
        // Size is always safe to restore; the position might not be, so a
        // window saved on a monitor that is no longer here keeps the size and
        // lets the window manager choose where to put it.
        if window::is_reachable(&sized, &monitors) {
            let _ = win.set_position(LogicalPosition::new(sized.x, sized.y));
        }
        if saved.maximized {
            let _ = win.maximize();
        }
    }

    let handle = win.clone();
    win.on_window_event(move |event| {
        if !matches!(event, tauri::WindowEvent::CloseRequested { .. }) {
            return;
        }
        let maximized = handle.is_maximized().unwrap_or(false);
        // A maximized window reports the size of the screen. Storing that
        // would lose the size to go back to when it is un-maximized, so the
        // previous entry is kept and only the flag is updated.
        let restored = window::load(&path);
        // `inner_size`, not `outer_size`, because `set_size` on the way back in
        // sets the *inner* size. Storing the outer one made the window grow by
        // the width of its own border and title bar on every single launch.
        let scale = handle.scale_factor().unwrap_or(1.0);
        let (Ok(size), Ok(pos)) = (handle.inner_size(), handle.outer_position()) else {
            return;
        };
        let size = size.to_logical::<u32>(scale);
        let pos = pos.to_logical::<i32>(scale);
        let state = match (maximized, restored) {
            (true, Some(previous)) => WindowState {
                maximized: true,
                ..previous
            },
            _ => WindowState {
                width: size.width,
                height: size.height,
                x: pos.x,
                y: pos.y,
                maximized,
            },
        };
        window::save(&path, &state);
    });
}
