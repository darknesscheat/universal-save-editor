//! Remembering the window's size and position between launches.
//!
//! `tauri.conf.json` only sets the size for a *first* run. Anyone who resizes
//! the window gets it thrown away on exit, which is a small annoyance every
//! single time the app opens.
//!
//! The restoring half is the part that can go wrong, so the geometry decisions
//! live here as plain functions with no Tauri types in them, and are tested
//! directly.
//!
//! **Everything here is in logical pixels**, the same unit `tauri.conf.json`
//! uses. Physical pixels were tried first and were wrong: on a display scaled
//! to 125%, restoring a size captured in physical pixels came back a quarter
//! smaller, so the window shrank on every launch until it hit its minimum size
//! and stuck there. Logical pixels also mean a window keeps its apparent size
//! when moved between monitors with different scaling.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Matches `minWidth`/`minHeight` in `tauri.conf.json`. A stored size below
/// this could only come from a corrupt file or an older build.
pub const MIN_WIDTH: u32 = 560;
pub const MIN_HEIGHT: u32 = 480;

/// A rectangle of desktop in logical pixels: x, y, width, height.
pub type Rect = (i32, i32, u32, u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowState {
    pub width: u32,
    pub height: u32,
    pub x: i32,
    pub y: i32,
    #[serde(default)]
    pub maximized: bool,
}

/// Can the user actually reach this window?
///
/// A saved position is only meaningful on the monitor layout that produced it.
/// Unplug the second screen, or dock a laptop, and the coordinates now point at
/// desktop that does not exist: the window opens somewhere invisible and the
/// app looks like it failed to start.
///
/// The bar is set low: enough of the title bar on screen to grab with
/// a mouse, not the whole window. Someone who likes a window half off the edge
/// gets to keep it there.
pub fn is_reachable(state: &WindowState, monitors: &[Rect]) -> bool {
    const GRABBABLE_WIDTH: i32 = 120;
    const TITLE_BAR: i32 = 30;

    monitors.iter().any(|&(mx, my, mw, mh)| {
        let (mr, mb) = (mx + mw as i32, my + mh as i32);
        let overlap_w = (state.x + state.width as i32).min(mr) - state.x.max(mx);
        // Only the top strip counts: a window whose title bar is above the
        // screen cannot be dragged back down, however much of its body shows.
        let overlap_h = (state.y + TITLE_BAR).min(mb) - state.y.max(my);
        overlap_w >= GRABBABLE_WIDTH && overlap_h >= TITLE_BAR
    })
}

/// Shrink a remembered size to fit the screen it will open on.
///
/// Monitors get smaller between launches, a 4K desktop today, a laptop panel
/// tomorrow, and a window larger than the screen has its controls off the
/// bottom edge.
pub fn clamp_size(state: WindowState, monitors: &[Rect]) -> WindowState {
    let (max_w, max_h) = monitors
        .iter()
        .fold((MIN_WIDTH, MIN_HEIGHT), |acc, &(_, _, w, h)| {
            (acc.0.max(w), acc.1.max(h))
        });
    WindowState {
        width: state.width.clamp(MIN_WIDTH, max_w),
        height: state.height.clamp(MIN_HEIGHT, max_h),
        ..state
    }
}

pub fn load(path: &Path) -> Option<WindowState> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// Failure stays silent here. Not remembering a window size is a nuisance,
/// and it must never be able to stop the app from closing.
pub fn save(path: &Path, state: &WindowState) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(text) = serde_json::to_string_pretty(state) {
        let _ = std::fs::write(path, text);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HD: Rect = (0, 0, 1920, 1080);

    fn at(x: i32, y: i32) -> WindowState {
        WindowState {
            width: 900,
            height: 720,
            x,
            y,
            maximized: false,
        }
    }

    #[test]
    fn a_window_in_the_middle_of_the_screen_is_reachable() {
        assert!(is_reachable(&at(300, 200), &[HD]));
    }

    #[test]
    fn a_window_hanging_off_the_edge_is_still_reachable() {
        // Half off the right edge, and off the bottom: the title bar is still
        // on screen, so this is a layout someone chose, not a lost window.
        assert!(is_reachable(&at(1500, 900), &[HD]));
    }

    #[test]
    fn a_window_on_a_monitor_that_is_gone_is_not_reachable() {
        // Saved on a second screen to the right, which is no longer connected.
        assert!(!is_reachable(&at(2400, 300), &[HD]));
    }

    #[test]
    fn a_window_above_the_top_edge_is_not_reachable() {
        // Its body is on screen but the title bar is not, so it cannot be
        // dragged back into view.
        assert!(!is_reachable(&at(300, -60), &[HD]));
    }

    #[test]
    fn a_second_monitor_makes_its_own_coordinates_reachable() {
        let second: Rect = (1920, 0, 1920, 1080);
        assert!(is_reachable(&at(2400, 300), &[HD, second]));
    }

    #[test]
    fn a_monitor_to_the_left_has_negative_coordinates() {
        // Windows places a left-hand second screen at negative x; this is the
        // ordinary case, not a lost window.
        let left: Rect = (-1920, 0, 1920, 1080);
        assert!(is_reachable(&at(-1500, 200), &[HD, left]));
    }

    #[test]
    fn a_window_larger_than_the_screen_is_shrunk_to_fit() {
        let huge = WindowState {
            width: 3800,
            height: 2000,
            x: 0,
            y: 0,
            maximized: false,
        };
        let fitted = clamp_size(huge, &[HD]);
        assert_eq!((fitted.width, fitted.height), (1920, 1080));
    }

    #[test]
    fn a_size_below_the_minimum_is_raised() {
        let tiny = WindowState {
            width: 10,
            height: 10,
            x: 0,
            y: 0,
            maximized: false,
        };
        let fitted = clamp_size(tiny, &[HD]);
        assert_eq!((fitted.width, fitted.height), (MIN_WIDTH, MIN_HEIGHT));
    }

    #[test]
    fn clamping_leaves_position_alone() {
        let s = at(300, 200);
        assert_eq!(clamp_size(s, &[HD]).x, 300);
        assert_eq!(clamp_size(s, &[HD]).y, 200);
    }

    #[test]
    fn a_size_that_already_fits_is_untouched() {
        let s = at(0, 0);
        assert_eq!(clamp_size(s, &[HD]), s);
    }

    #[test]
    fn a_missing_file_is_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(load(&tmp.path().join("nope.json")).is_none());
    }

    #[test]
    fn a_corrupt_file_is_ignored_rather_than_crashing() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("window.json");
        std::fs::write(&path, "{ not json").unwrap();
        assert!(load(&path).is_none());
    }

    #[test]
    fn what_is_saved_is_what_comes_back() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("sub/window.json");
        let state = WindowState {
            width: 574,
            height: 518,
            x: 12,
            y: 34,
            maximized: false,
        };
        save(&path, &state);
        assert_eq!(load(&path), Some(state));
    }
}
