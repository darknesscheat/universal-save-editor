//! Reading artwork out of a game's own installed files.
//!
//! Same principle as the store artwork on the game-selection screen: nothing is
//! bundled, everything is read from the copy the player already owns. Game art
//! is copyrighted and this repository is MIT; redistributing it would not be
//! ours to do, and reading a file already on the machine is a different act.
//!
//! One engine is supported so far. The seam is `ItemIcons::format`, so adding
//! another means adding a module here rather than touching the editor.

pub mod godot_pck;

use crate::core::paths;
use crate::plugins::manifest::{ItemIcons, Manifest};
use std::collections::HashMap;

/// Rendering one icon larger than this in a dropdown row is pointless, and a
/// hundred of them would make the payload to the GUI silly.
const MAX_ICON_BYTES: usize = 256 * 1024;

/// Total across every icon for one game, so a pathological archive cannot make
/// the editor take a second to open.
const MAX_TOTAL_BYTES: usize = 12 * 1024 * 1024;

/// Option value -> `data:` URI, for every option the plugin can find a picture
/// for.
///
/// Missing pictures are not errors. A player who owns the game on another
/// store, or has it uninstalled, simply gets the text they had before.
pub fn item_icons(
    manifest: &Manifest,
    locale_free_values: &[(&str, Vec<String>)],
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    let mut budget = MAX_TOTAL_BYTES;

    for spec in &manifest.item_icons {
        if spec.format != "godot_pck" {
            continue;
        }
        let platform = paths::current_platform();
        if !spec.platforms.is_empty() && !spec.platforms.iter().any(|p| p == platform) {
            continue;
        }

        let Some(values) = locale_free_values
            .iter()
            .find(|(set, _)| *set == spec.options_ref)
            .map(|(_, v)| v)
        else {
            continue;
        };

        collect_from_godot(spec, values, &mut out, &mut budget);
    }

    out
}

fn collect_from_godot(
    spec: &ItemIcons,
    values: &[String],
    out: &mut HashMap<String, String>,
    budget: &mut usize,
) {
    let Ok(archive) = paths::expand(&spec.archive) else {
        return;
    };
    // A glob keeps the manifest honest about installs in unusual places.
    let Some(archive) = glob::glob(&archive.to_string_lossy())
        .ok()
        .and_then(|mut hits| hits.find_map(|h| h.ok()))
    else {
        return;
    };

    let Some(mut pck) = godot_pck::Pck::open(&archive) else {
        return;
    };
    let imported = godot_pck::imported_texture_index(&pck);

    for value in values {
        if *budget == 0 {
            return;
        }
        if out.contains_key(value) {
            continue;
        }

        let resource = spec.resource_pattern.replace("{value}", value);
        let Some(texture) = godot_pck::texture_for_resource(&mut pck, &resource) else {
            continue;
        };
        let Some(file_name) = texture.rsplit('/').next() else {
            continue;
        };
        let Some(ctex_path) = imported.get(file_name) else {
            continue;
        };
        let Some(blob) = pck.read(ctex_path) else {
            continue;
        };
        let Some((mime, image)) = godot_pck::image_in_ctex(&blob) else {
            continue;
        };
        if image.len() > MAX_ICON_BYTES || image.len() > *budget {
            continue;
        }

        *budget -= image.len();
        out.insert(
            value.clone(),
            format!("data:{mime};base64,{}", crate::core::icon::base64(&image)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(body: &str) -> Manifest {
        serde_json::from_str(body).unwrap()
    }

    #[test]
    fn an_unknown_engine_is_skipped_quietly() {
        let m = manifest(
            r#"{"id":"t","name":"T","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
              "item_icons":[{"options_ref":"weapons","format":"unity_bundle",
                             "archive":"{HOME}/nope","resource_pattern":"{value}"}],
              "groups":[{"id":"g","label":"G","fields":[
                {"id":"a","label":"A","pointer":"/a","type":"integer"}]}]}"#,
        );
        let values = vec![("weapons", vec!["gun".to_string()])];
        assert!(item_icons(&m, &values).is_empty());
    }

    #[test]
    fn a_missing_archive_yields_no_icons_rather_than_failing() {
        let m = manifest(
            r#"{"id":"t","name":"T","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
              "item_icons":[{"options_ref":"weapons","format":"godot_pck",
                             "archive":"{HOME}/definitely-not-here.pck",
                             "resource_pattern":"scn/{value}.tres"}],
              "groups":[{"id":"g","label":"G","fields":[
                {"id":"a","label":"A","pointer":"/a","type":"integer"}]}]}"#,
        );
        let values = vec![("weapons", vec!["gun".to_string()])];
        assert!(item_icons(&m, &values).is_empty());
    }

    #[test]
    fn a_plugin_with_no_item_icons_does_no_work() {
        let m = manifest(
            r#"{"id":"t","name":"T","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
              "groups":[{"id":"g","label":"G","fields":[
                {"id":"a","label":"A","pointer":"/a","type":"integer"}]}]}"#,
        );
        assert!(item_icons(&m, &[]).is_empty());
    }
}
