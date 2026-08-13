//! Integration tests that exercise the plugins actually shipped with the app
//! against a fixture save laid out exactly like the real thing.
//!
//! These are the tests that would catch a typo in `manifest.json`: a pointer
//! that does not match the game's field names, or an option whose value the
//! game would not recognise.

use std::path::PathBuf;
use universal_save_editor_lib::backup::BackupManager;
use universal_save_editor_lib::core::model::Edit;
use universal_save_editor_lib::plugins::manifest::Manifest;
use universal_save_editor_lib::plugins::registry::Registry;
use universal_save_editor_lib::save;

fn plugins_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("src-tauri has a parent")
        .join("plugins")
}

fn fixture(game: &str, name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(game)
        .join(name)
}

fn pathogenic() -> Manifest {
    let reg = Registry::load(&[plugins_dir()]);
    assert!(
        reg.problems().is_empty(),
        "bundled plugins failed to load: {:?}",
        reg.problems()
    );
    reg.get("pathogenic")
        .expect("the pathogenic plugin ships with the app")
        .manifest
        .clone()
}

/// Copy the fixture somewhere writable so a test can edit it.
fn scratch_save() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let dst = tmp.path().join("run_save.json");
    std::fs::copy(fixture("pathogenic", "run_save.json"), &dst).unwrap();
    (tmp, dst)
}

#[test]
fn every_bundled_plugin_loads_cleanly() {
    let reg = Registry::load(&[plugins_dir()]);
    assert!(
        reg.problems().is_empty(),
        "bundled plugins failed to load: {:?}",
        reg.problems()
    );
    assert!(reg.all().count() >= 1, "no plugins were found");
}

/// The example in the plugin-authoring guide must actually load. A broken
/// example is worse than none.
#[test]
fn the_documented_example_plugin_is_valid() {
    let docs = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs");
    let reg = Registry::load(&[docs]);
    assert!(
        reg.problems().is_empty(),
        "docs/example-plugin does not load: {:?}",
        reg.problems()
    );
    let example = reg.get("example-game").expect("example-game plugin");

    // Exercise it against a save shaped the way the guide describes.
    let doc = serde_json::json!({
        "version": "3",
        "chapter": 2,
        "player": {
            "name": "Alex", "health": 100, "stamina": 55.5,
            "gold": 250, "difficulty": "normal", "tutorial_done": true
        },
        "inventory": [{ "name": "Rope", "qty": 3 }]
    });
    let editor = save::editor::build(&example.manifest, "example", &doc, "", Default::default());
    let ids: Vec<&str> = editor.groups.iter().map(|g| g.id.as_str()).collect();
    assert_eq!(ids, vec!["character", "inventory"]);

    let writable = save::editor::writable_fields(&example.manifest, &doc);
    assert!(writable.contains_key("/player/gold"));
    assert!(writable.contains_key("/inventory/0/qty"));
    assert!(
        !writable.contains_key("/version"),
        "the read-only field is writable"
    );
}

/// The languages the app ships. A bundled plugin should cover all of them, or
/// players switching language get a half-translated editor.
const SHIPPED_LOCALES: [&str; 12] = [
    "tr", "de", "es", "fr", "it", "pt-BR", "ru", "pl", "uk", "ja", "ko", "zh-CN",
];

#[test]
fn pathogenic_labels_are_translated_into_every_shipped_language() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();

    let english = save::editor::build(&m, "fixture", &doc, "en", Default::default());
    let money_en = english.groups[0]
        .fields
        .iter()
        .find(|f| f.id == "money")
        .expect("money field");
    assert_eq!(money_en.label, "Money");

    for locale in SHIPPED_LOCALES {
        let view = save::editor::build(&m, "fixture", &doc, locale, Default::default());
        let group = &view.groups[0];
        assert_ne!(
            group.label, english.groups[0].label,
            "group label not translated for {locale}"
        );

        let money = group.fields.iter().find(|f| f.id == "money").unwrap();
        assert_ne!(money.label, "Money", "'Money' not translated for {locale}");
        assert!(!money.label.trim().is_empty());
    }
}

#[test]
fn turkish_gives_the_expected_wording() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();
    let view = save::editor::build(&m, "fixture", &doc, "tr", Default::default());

    let group = &view.groups[0];
    assert_eq!(group.label, "Karakter");
    assert_eq!(
        group.fields.iter().find(|f| f.id == "money").unwrap().label,
        "Para"
    );
    assert_eq!(
        group.fields.iter().find(|f| f.id == "hp").unwrap().label,
        "Can"
    );
}

/// A regional tag should fall back to the base language rather than to English.
#[test]
fn a_regional_locale_falls_back_to_its_base_language() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();

    let base = save::editor::build(&m, "fixture", &doc, "de", Default::default());
    let regional = save::editor::build(&m, "fixture", &doc, "de-AT", Default::default());
    assert_eq!(base.groups[0].label, regional.groups[0].label);
}

#[test]
fn an_unknown_locale_keeps_the_manifests_own_wording() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();
    let view = save::editor::build(&m, "fixture", &doc, "xx-YY", Default::default());
    assert_eq!(view.groups[0].label, "Character");
}

/// Body-part names are game content and must stay as the game writes them,
/// in every language.
#[test]
fn equipment_names_are_not_translated() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();

    for locale in ["en", "tr", "ja", "ru"] {
        let view = save::editor::build(&m, "fixture", &doc, locale, Default::default());
        let equipment = view.groups.iter().find(|g| g.id == "equipment").unwrap();
        let part = equipment.lists[0].items[0]
            .fields
            .iter()
            .find(|f| f.id == "bodypart")
            .unwrap();
        assert!(
            part.options.iter().any(|o| o.label == "Rocket Launcher"),
            "part names changed for {locale}"
        );
    }
}

/// Rarity is editor vocabulary, not an item name, so it *is* translated.
#[test]
fn rarity_values_are_translated() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();
    let view = save::editor::build(&m, "fixture", &doc, "tr", Default::default());
    let equipment = view.groups.iter().find(|g| g.id == "equipment").unwrap();
    let rarity = equipment.lists[0].items[0]
        .fields
        .iter()
        .find(|f| f.id == "rarity")
        .unwrap();
    assert!(rarity.options.iter().any(|o| o.label == "Efsanevi"));
}

/// Opens whatever saves this machine actually has and checks the editor can
/// both show and write them. Ignored by default because it depends on the game
/// being installed; run with `cargo test -- --ignored --nocapture`.
///
/// This is the check a screenshot was standing in for: real files, real
/// values, including any the game wrote past a declared limit.
#[test]
#[ignore]
fn real_saves_on_this_machine_open_and_stay_writable() {
    let m = pathogenic();
    let saves = save::detect::find_saves(&m, "en").unwrap();
    if saves.is_empty() {
        println!("no Pathogenic saves on this machine, nothing to check");
        return;
    }

    for s in &saves {
        let path = std::path::Path::new(&s.path);
        let doc = save::detect::load_document(&m, path)
            .unwrap_or_else(|e| panic!("{} failed to open: {e}", s.path));
        let view = save::editor::build(&m, &s.path, &doc, "tr", Default::default());

        // Report every value the game itself wrote outside the declared range.
        // These used to make the editor refuse to save anything at all.
        let mut beyond = Vec::new();
        for group in &view.groups {
            for f in &group.fields {
                let Some(n) = f.value.as_f64() else { continue };
                if f.min.map(|min| n < min).unwrap_or(false)
                    || f.max.map(|max| n > max).unwrap_or(false)
                {
                    beyond.push(format!(
                        "{} = {n} (range {:?}..{:?})",
                        f.label, f.min, f.max
                    ));
                }
            }
        }

        println!("\n{}, {}", s.title, s.path);
        for group in &view.groups {
            let rows: usize = group.lists.iter().map(|l| l.items.len()).sum();
            println!(
                "  [{}] {} field(s), {} list row(s)",
                group.label,
                group.fields.len(),
                rows
            );
        }
        let editable = save::editor::writable_fields(&m, &doc).len();
        println!("  editable values in total: {editable}");
        if beyond.is_empty() {
            println!("  every value sits inside its declared range");
        } else {
            println!(
                "  {} value(s) past a declared limit, and that is fine:",
                beyond.len()
            );
            for b in &beyond {
                println!("    {b}");
            }
        }

        // The editor must offer something to edit, whatever the values are.
        let writable = save::editor::writable_fields(&m, &doc);
        assert!(
            !writable.is_empty(),
            "{} opened with nothing editable",
            s.path
        );
    }
}

/// The profile save is the one a player always has, a run save only exists
/// mid-run. It must therefore be worth opening on its own.
#[test]
fn the_profile_save_offers_equipment_and_unlocks() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "save.json")).unwrap();
    let view = save::editor::build(&m, "p", &doc, "en", Default::default());

    let ids: Vec<&str> = view.groups.iter().map(|g| g.id.as_str()).collect();
    assert!(
        ids.contains(&"starting_equipment"),
        "the permanent starting loadout is not offered: {ids:?}"
    );
    assert!(
        ids.contains(&"discovery"),
        "unlock lists are not offered: {ids:?}"
    );

    // Equipment, without an active run.
    let equip = view
        .groups
        .iter()
        .find(|g| g.id == "starting_equipment")
        .unwrap();
    let rows: usize = equip.lists.iter().map(|l| l.items.len()).sum();
    assert!(rows > 0, "starting loadout came back empty");

    // All five unlock objects are surfaced, each expanding into rows. The
    // fixture is a trimmed save, so the count is small here; the real-save
    // check reports the true figure.
    let discovery = view.groups.iter().find(|g| g.id == "discovery").unwrap();
    assert_eq!(discovery.lists.len(), 5, "an unlock object is missing");
    for list in &discovery.lists {
        assert!(!list.items.is_empty(), "list '{}' came back empty", list.id);
    }

    // Keys are turned into something readable rather than shown raw.
    let enemies = discovery
        .lists
        .iter()
        .find(|l| l.id == "enemy_discoveries")
        .unwrap();
    assert!(
        enemies.items.iter().all(|i| !i.label.contains('_')),
        "raw object keys leaked into the list labels"
    );
    assert!(
        !enemies.bulk_actions.is_empty(),
        "no way to set all of them at once"
    );

    // And every one of those rows must actually be writable.
    let writable = save::editor::writable_fields(&m, &doc);
    for item in &enemies.items {
        let pointer = &item.fields[0].pointer;
        assert!(
            writable.contains_key(pointer),
            "{pointer} is shown but not writable"
        );
    }
}

/// Object keys can legally contain `/` and `~`, which are the two characters a
/// JSON pointer gives special meaning.
#[test]
fn awkward_object_keys_are_escaped_not_misread() {
    let manifest: Manifest = serde_json::from_str(
        r#"{"id":"t","name":"T","version":"1","format":"json",
          "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
          "identify":[{"pointer":"/flags"}],
          "groups":[{"id":"g","label":"G","lists":[
            {"id":"flags","label":"Flags","pointer":"/flags","source":"object",
             "entry":{"id":"on","label":"On","pointer":"","type":"boolean"}}]}]}"#,
    )
    .unwrap();

    let doc = serde_json::json!({ "flags": { "a/b": false, "c~d": false, "plain": true } });
    let view = save::editor::build(&manifest, "x", &doc, "en", Default::default());
    let list = &view.groups[0].lists[0];
    assert_eq!(list.items.len(), 3);

    // Every generated pointer must resolve back to the value it came from.
    for item in &list.items {
        assert!(
            doc.pointer(&item.fields[0].pointer).is_some(),
            "pointer {} does not resolve",
            item.fields[0].pointer
        );
    }
}

/// The player's own folder holds several files the game made and never
/// mentioned. They must be findable.
#[test]
fn pathogenic_finds_the_games_own_copies() {
    let m = pathogenic();
    assert!(
        !m.recovery_patterns.is_empty(),
        "no recovery patterns, so the game's own backups stay unreachable"
    );

    let tmp = tempfile::tempdir().unwrap();
    let save = tmp.path().join("save.json");
    for name in [
        "save.json",
        "save.json.bak",
        "save.json.bak2",
        "run_save.json.bak",
        "corrupted_1786483054_save.json",
        "steam_autocloud.vdf",
    ] {
        std::fs::write(tmp.path().join(name), "{}").unwrap();
    }

    let found = universal_save_editor_lib::save::recovery::find_for(&m, &save);
    let names: Vec<&str> = found.iter().map(|f| f.name.as_str()).collect();

    assert!(names.contains(&"save.json.bak"), "got {names:?}");
    assert!(names.contains(&"save.json.bak2"));
    assert!(names.contains(&"corrupted_1786483054_save.json"));
    assert!(!names.contains(&"save.json"), "offered the live save back");
    assert!(!names.contains(&"steam_autocloud.vdf"));

    // The quarantine time is read out of the file name.
    let quarantined = found
        .iter()
        .find(|f| f.name.starts_with("corrupted_"))
        .unwrap();
    assert!(
        quarantined.created.starts_with("2026-"),
        "{}",
        quarantined.created
    );
}

#[test]
fn pathogenic_offers_presets_for_the_save_at_hand() {
    let m = pathogenic();

    let profile = save::detect::load_document(&m, &fixture("pathogenic", "save.json")).unwrap();
    let for_profile = save::presets::available(&m, &profile, "tr");
    let ids: Vec<&str> = for_profile.iter().map(|p| p.id.as_str()).collect();
    assert!(ids.contains(&"legendary_start"), "got {ids:?}");
    assert!(ids.contains(&"discover_all"));
    assert!(
        !ids.contains(&"refill"),
        "a run-only preset was offered for a profile save"
    );
    assert!(for_profile.iter().all(|p| !p.label.is_empty()));

    let run = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();
    let for_run: Vec<&str> = save::presets::available(&m, &run, "en")
        .iter()
        .map(|p| p.id.as_str())
        .collect::<Vec<_>>()
        .iter()
        .map(|s| Box::leak(s.to_string().into_boxed_str()) as &str)
        .collect();
    assert!(for_run.contains(&"refill"));
    assert!(for_run.contains(&"legendary_run"));
}

/// Every preset must expand into pointers that exist, or it would fail the
/// moment someone pressed it.
#[test]
fn every_preset_expands_into_pointers_that_resolve() {
    let m = pathogenic();

    for file in ["run_save.json", "save.json"] {
        let doc = save::detect::load_document(&m, &fixture("pathogenic", file)).unwrap();
        for view in save::presets::available(&m, &doc, "en") {
            let preset = save::presets::find(&m, &doc, &view.id).unwrap();
            let edits = save::presets::expand(&m, &doc, preset);
            assert!(
                !edits.is_empty(),
                "{file}: preset '{}' does nothing",
                view.id
            );

            let writable = save::editor::writable_fields(&m, &doc);
            for edit in &edits {
                assert!(
                    doc.pointer(&edit.pointer).is_some(),
                    "{file}: preset '{}' points at {} which does not exist",
                    view.id,
                    edit.pointer
                );
                assert!(
                    writable.contains_key(&edit.pointer),
                    "{file}: preset '{}' would write {} which is not editable",
                    view.id,
                    edit.pointer
                );
            }
        }
    }
}

/// Sections that do not apply now explain themselves rather than vanishing.
#[test]
fn run_only_sections_say_why_they_are_empty() {
    let m = pathogenic();
    let profile = save::detect::load_document(&m, &fixture("pathogenic", "save.json")).unwrap();
    let view = save::editor::build(&m, "p", &profile, "tr", Default::default());

    let equipment = view
        .groups
        .iter()
        .find(|g| g.id == "equipment")
        .expect("the equipment section vanished instead of explaining itself");

    let reason = equipment
        .absent_reason
        .as_ref()
        .expect("no explanation given");
    assert!(!reason.is_empty());
    assert!(equipment.fields.is_empty() && equipment.lists.is_empty());
    // And it is translated, not left in English.
    assert!(reason.contains("run"), "{reason}");
}

/// No plugin may ship game artwork: this repository is MIT licensed and game
/// art is not ours to redistribute. Icons must come from the player's own
/// machine via `icon_sources`.
#[test]
fn no_bundled_plugin_ships_artwork() {
    let reg = Registry::load(&[plugins_dir()]);
    for plugin in reg.all() {
        assert!(
            plugin.manifest.icon.is_none(),
            "{} bundles an icon file; use icon_sources instead",
            plugin.manifest.id
        );
    }
}

#[test]
fn pathogenic_looks_for_its_icon_in_steams_cache() {
    let m = pathogenic();
    assert!(
        !m.icon_sources.is_empty(),
        "no icon source declared, so the game can never show its artwork"
    );
    for source in &m.icon_sources {
        assert!(
            source.path.contains("{STEAM}"),
            "icon source should use the {{STEAM}} placeholder, not a fixed path: {}",
            source.path
        );
    }
}

/// Steam keeps a 32x32 icon at the top level of its cache folder and much
/// larger art in the subfolders. The small one is the easiest to glob and
/// looks blurry on the card, so it must never be tried first.
#[test]
fn high_resolution_artwork_is_preferred_over_the_tiny_icon() {
    for plugin in Registry::load(&[plugins_dir()]).all() {
        let m = &plugin.manifest;
        if m.icon_sources.is_empty() {
            continue;
        }

        // The low-res source is the bare `*.jpg`; the good ones live in `*/`.
        let low_res = m
            .icon_sources
            .iter()
            .position(|s| !s.path.trim_end_matches(".jpg").contains('*'));

        let first_high_res = m
            .icon_sources
            .iter()
            .position(|s| s.path.contains("*/"))
            .unwrap_or_else(|| panic!("{}: no high-resolution source declared", m.id));

        if let Some(low) = low_res {
            assert!(
                first_high_res < low,
                "{}: the 32x32 icon is tried before the full-size artwork",
                m.id
            );
        }
    }
}

/// The game grid draws 2:3 covers, so a landscape header used in that slot gets
/// cropped down to a strip of the middle. Both shapes are worth declaring, not
/// every game has a capsule cached, but the portrait one has to come first.
#[test]
fn portrait_cover_art_is_preferred_over_the_landscape_header() {
    // Steam has used two names for the same portrait art over the years.
    const PORTRAIT: [&str; 2] = ["library_600x900", "library_capsule"];

    for plugin in Registry::load(&[plugins_dir()]).all() {
        let m = &plugin.manifest;
        let first = |needles: &[&str]| {
            m.icon_sources
                .iter()
                .position(|s| needles.iter().any(|n| s.path.contains(n)))
        };

        let Some(portrait) = first(&PORTRAIT) else {
            continue;
        };
        if let Some(landscape) = first(&["library_header", "logo."]) {
            assert!(
                portrait < landscape,
                "{}: landscape artwork would win over the portrait cover",
                m.id
            );
        }
    }
}

/// Resolving an icon must never fail loudly, a player without the game
/// installed still has to be able to open the app.
#[test]
fn a_missing_game_just_means_no_icon() {
    let reg = Registry::load(&[plugins_dir()]);
    let plugin = reg.get("pathogenic").unwrap();

    std::env::set_var("STEAM_PATH", "/definitely/not/here");
    let icon = universal_save_editor_lib::core::icon::resolve(&plugin.manifest, &plugin.dir);
    std::env::remove_var("STEAM_PATH");

    assert!(icon.is_none());
}

/// Regression for a real defect: the manifest listed `ESlot1`..`ESlot4`, but
/// the game uses `ESlot5` and `ESlot6` in late runs, those weapons were
/// invisible and uneditable. Slot filters must match on the prefix.
#[test]
fn late_game_weapon_slots_are_not_hidden() {
    let m = pathogenic();

    let doc = serde_json::json!({
        "player": {
            "hp": 6, "money": 10,
            "loadout": [
                {"bodypart": "gun",            "rarity": 0, "slot": "ESlot1"},
                {"bodypart": "rocket_launcher","rarity": 3, "slot": "ESlot5"},
                {"bodypart": "sniper",         "rarity": 2, "slot": "ESlot6"},
                {"bodypart": "lash",           "rarity": 1, "slot": "EBackSlot2"},
                {"bodypart": "homing",         "rarity": 3, "slot": "ISlot5"}
            ]
        }
    });

    let view = save::editor::build(&m, "fixture", &doc, "en", Default::default());
    let equipment = view.groups.iter().find(|g| g.id == "equipment").unwrap();

    let weapons = equipment.lists.iter().find(|l| l.id == "weapons").unwrap();
    assert_eq!(weapons.items.len(), 4, "a weapon slot was filtered out");

    let organs = equipment.lists.iter().find(|l| l.id == "organs").unwrap();
    assert_eq!(organs.items.len(), 1, "ISlot5 was filtered out");

    // And they must be writable, not merely visible.
    let writable = save::editor::writable_fields(&m, &doc);
    assert!(writable.contains_key("/player/loadout/1/rarity"));
    assert!(writable.contains_key("/player/loadout/2/bodypart"));
    assert!(writable.contains_key("/player/loadout/4/rarity"));
}

/// Every slot name that has ever appeared in a real save must survive the
/// filter. `past_runs.json` is the game's own record of what it produces.
#[test]
fn every_slot_name_seen_in_real_runs_is_covered() {
    let m = pathogenic();
    let equipment = m.groups.iter().find(|g| g.id == "equipment").unwrap();

    let observed = [
        "ESlot1",
        "ESlot2",
        "ESlot3",
        "ESlot4",
        "ESlot5",
        "ESlot6",
        "EBackSlot1",
        "EBackSlot2",
        "ISlot1",
        "ISlot2",
        "ISlot3",
        "ISlot4",
        "ISlot5",
    ];

    for slot in observed {
        let item = serde_json::json!({ "slot": slot });
        let covered = equipment.lists.iter().any(|l| {
            l.item_filter
                .as_ref()
                .map(|f| f.matches(&item))
                .unwrap_or(true)
        });
        assert!(covered, "{slot} is not covered by any equipment list");
    }
}

#[test]
fn pathogenic_warns_about_its_own_process() {
    let m = pathogenic();
    assert!(
        m.process_names.iter().any(|p| p == "pathogenic"),
        "no process name declared, so the 'game is running' warning can never fire"
    );
}

#[test]
fn pathogenic_declares_health_and_stamina_constraints() {
    let m = pathogenic();
    assert_eq!(m.constraints.len(), 2);

    // Health above max health must be caught.
    let bad = serde_json::json!({"player":{"hp":500,"max_hp":6,"stamina":1.0,"max_stamina":100.0}});
    assert!(m.constraints.iter().any(|c| c.violated_by(&bad)));

    let good = serde_json::json!({"player":{"hp":6,"max_hp":6,"stamina":1.0,"max_stamina":100.0}});
    assert!(!m.constraints.iter().any(|c| c.violated_by(&good)));
}

#[test]
fn pathogenic_recognises_a_real_save_layout() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();
    assert_eq!(doc["player"]["money"], serde_json::json!(250));
}

#[test]
fn pathogenic_rejects_a_file_from_another_game() {
    let m = pathogenic();
    let tmp = tempfile::tempdir().unwrap();
    let f = tmp.path().join("other.json");
    std::fs::write(&f, r#"{"farm":{"gold":100}}"#).unwrap();
    assert!(save::detect::load_document(&m, &f).is_err());
}

/// Every pointer a group declares must resolve in a save that group applies to.
/// This is the check that catches a mistyped field name.
#[test]
fn every_declared_pointer_exists_in_the_fixture() {
    let m = pathogenic();

    for file in ["run_save.json", "save.json"] {
        let doc = save::detect::load_document(&m, &fixture("pathogenic", file)).unwrap();

        let mut missing = Vec::new();
        let mut groups_shown = 0;
        for g in m.groups_for(&doc) {
            groups_shown += 1;
            for f in &g.fields {
                if doc.pointer(&f.pointer).is_none() {
                    missing.push(format!("{}: {}", g.id, f.pointer));
                }
            }
            for l in &g.lists {
                use universal_save_editor_lib::plugins::manifest::ListSource;
                let ok = match l.source {
                    ListSource::Array => doc
                        .pointer(&l.pointer)
                        .map(|v| v.is_array())
                        .unwrap_or(false),
                    ListSource::Object => doc
                        .pointer(&l.pointer)
                        .map(|v| v.is_object())
                        .unwrap_or(false),
                };
                if !ok {
                    missing.push(format!("{}: {} ({:?} expected)", g.id, l.pointer, l.source));
                }
            }
        }

        assert!(groups_shown > 0, "{file}: no groups applied to this save");
        assert!(
            missing.is_empty(),
            "{file}: pointers not found: {missing:?}"
        );
    }
}

/// The two kinds of Pathogenic save must each show only what belongs to them.
#[test]
fn run_and_profile_saves_show_different_sections() {
    let m = pathogenic();

    let run = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();
    let run_groups: Vec<&str> = m.groups_for(&run).map(|g| g.id.as_str()).collect();
    assert!(run_groups.contains(&"character"));
    assert!(run_groups.contains(&"equipment"));
    assert!(
        !run_groups.contains(&"progression"),
        "profile-only section leaked into a run save"
    );

    let profile = save::detect::load_document(&m, &fixture("pathogenic", "save.json")).unwrap();
    let profile_groups: Vec<&str> = m.groups_for(&profile).map(|g| g.id.as_str()).collect();
    assert!(profile_groups.contains(&"progression"));
    assert!(
        !profile_groups.contains(&"character"),
        "run-only section leaked into a profile save"
    );
}

/// A field hidden because its group does not apply must not be writable either.
#[test]
fn hidden_sections_cannot_be_written() {
    let m = pathogenic();
    let tmp = tempfile::tempdir().unwrap();
    let profile = tmp.path().join("save.json");
    std::fs::copy(fixture("pathogenic", "save.json"), &profile).unwrap();
    let backups = BackupManager::new(tmp.path().join("backups"));

    let err = save::apply_and_write(
        &m,
        &backups,
        &profile,
        &[Edit {
            pointer: "/player/money".into(),
            value: serde_json::json!(1),
        }],
        None,
        true,
    )
    .unwrap_err();
    assert!(
        err.to_string().contains("not an editable field"),
        "got: {err}"
    );
}

#[test]
fn a_profile_save_can_be_edited() {
    let m = pathogenic();
    let tmp = tempfile::tempdir().unwrap();
    let profile = tmp.path().join("save.json");
    std::fs::copy(fixture("pathogenic", "save.json"), &profile).unwrap();
    let backups = BackupManager::new(tmp.path().join("backups"));

    let report = save::apply_and_write(
        &m,
        &backups,
        &profile,
        &[Edit {
            pointer: "/plasmids/fragment_num".into(),
            value: serde_json::json!(500),
        }],
        None,
        true,
    )
    .unwrap();
    assert_eq!(report.changed_fields, 1);

    let after: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&profile).unwrap()).unwrap();
    // The game stores this as a float; it must stay one.
    assert_eq!(
        serde_json::to_string(&after["plasmids"]["fragment_num"]).unwrap(),
        "500.0"
    );
}

/// The GUI must be able to show every part the game actually has, and must not
/// offer one it does not.
#[test]
fn equipment_options_cover_the_parts_in_a_real_save() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();
    let editor = save::editor::build(&m, "fixture", &doc, "", Default::default());

    let equipment = editor
        .groups
        .iter()
        .find(|g| g.id == "equipment")
        .expect("equipment group");

    // Six slots in the fixture, split across the two filtered lists.
    let shown: usize = equipment.lists.iter().map(|l| l.items.len()).sum();
    assert_eq!(shown, 6, "filters dropped or duplicated equipment slots");

    for list in &equipment.lists {
        for item in &list.items {
            let part = item.fields.iter().find(|f| f.id == "bodypart").unwrap();
            assert!(
                part.options.iter().any(|o| o.value == part.value),
                "{:?} is equipped but not offered as an option in '{}'",
                part.value,
                list.label
            );
        }
    }
}

#[test]
fn weapon_and_organ_slots_are_kept_apart() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();
    let editor = save::editor::build(&m, "fixture", &doc, "", Default::default());
    let equipment = editor.groups.iter().find(|g| g.id == "equipment").unwrap();

    let weapons = equipment.lists.iter().find(|l| l.id == "weapons").unwrap();
    let organs = equipment.lists.iter().find(|l| l.id == "organs").unwrap();
    assert_eq!(weapons.items.len(), 4);
    assert_eq!(organs.items.len(), 2);

    // An organ must never appear as a choice in a weapon slot.
    let weapon_choices: Vec<_> = weapons.items[0]
        .fields
        .iter()
        .find(|f| f.id == "bodypart")
        .unwrap()
        .options
        .iter()
        .map(|o| o.value.clone())
        .collect();
    assert!(weapon_choices.contains(&serde_json::json!("rocket_launcher")));
    assert!(
        !weapon_choices.contains(&serde_json::json!("damage_mult")),
        "an internal organ was offered for an external slot"
    );
}

#[test]
fn the_mutation_a_save_already_has_is_a_valid_option() {
    let m = pathogenic();
    let doc = save::detect::load_document(&m, &fixture("pathogenic", "run_save.json")).unwrap();
    let editor = save::editor::build(&m, "fixture", &doc, "", Default::default());
    let group = editor.groups.iter().find(|g| g.id == "mutations").unwrap();
    let item = &group.lists[0].items[0];
    let field = &item.fields[0];
    assert!(
        field.options.iter().any(|o| o.value == field.value),
        "the equipped mutation is missing from the option list"
    );
}

/// The end-to-end flow from the brief: open a save, change values, write, and
/// confirm the game-facing file is still coherent.
#[test]
fn full_edit_flow_over_a_real_save_layout() {
    let m = pathogenic();
    let (tmp, save_path) = scratch_save();
    let backups = BackupManager::new(tmp.path().join("backups"));
    let before = std::fs::read(&save_path).unwrap();

    let report = save::apply_and_write(
        &m,
        &backups,
        &save_path,
        &[
            Edit {
                pointer: "/player/money".into(),
                value: serde_json::json!(50000),
            },
            Edit {
                pointer: "/player/hp".into(),
                value: serde_json::json!(99),
            },
            Edit {
                pointer: "/player/max_hp".into(),
                value: serde_json::json!(99),
            },
            // Raised together: the plugin declares stamina <= max stamina, and
            // a save that breaks that is one the game would not have written.
            Edit {
                pointer: "/player/max_stamina".into(),
                value: serde_json::json!(500),
            },
            Edit {
                pointer: "/player/stamina".into(),
                value: serde_json::json!(500),
            },
            Edit {
                pointer: "/player/loadout/0/rarity".into(),
                value: serde_json::json!(3),
            },
            Edit {
                pointer: "/player/loadout/0/bodypart".into(),
                value: serde_json::json!("rocket_launcher"),
            },
        ],
        None,
        true,
    )
    .unwrap();
    assert_eq!(report.changed_fields, 7);

    let after: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&save_path).unwrap()).unwrap();
    assert_eq!(after["player"]["money"], serde_json::json!(50000));
    assert_eq!(
        after["player"]["loadout"][0]["bodypart"],
        serde_json::json!("rocket_launcher")
    );
    assert_eq!(
        after["player"]["loadout"][0]["rarity"],
        serde_json::json!(3)
    );

    // The engine stores stamina as a float; it must stay one.
    let text = std::fs::read_to_string(&save_path).unwrap();
    assert!(text.contains("500.0"), "stamina was written as an integer");

    // Nothing outside the edited fields moved.
    let original: serde_json::Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(original["secret_code"], after["secret_code"]);
    assert_eq!(original["rooms"], after["rooms"]);
    assert_eq!(original["seed"], after["seed"]);
    assert_eq!(original["run_time"], after["run_time"]);

    // And the backup restores the file exactly.
    backups.restore(&report.backup_id).unwrap();
    assert_eq!(std::fs::read(&save_path).unwrap(), before);
}

#[test]
fn a_weapon_cannot_be_written_into_an_organ_slot() {
    let m = pathogenic();
    let (tmp, save_path) = scratch_save();
    let backups = BackupManager::new(tmp.path().join("backups"));

    // Index 4 is ISlot1 in the fixture.
    let err = save::apply_and_write(
        &m,
        &backups,
        &save_path,
        &[Edit {
            pointer: "/player/loadout/4/bodypart".into(),
            value: serde_json::json!("rocket_launcher"),
        }],
        None,
        true,
    )
    .unwrap_err();

    assert!(
        err.to_string().contains("available options"),
        "unexpected error: {err}"
    );
}

/// Checks save detection against whatever is really installed on this machine.
///
/// Ignored by default because it depends on the game being present, which is
/// not true on CI. Run it with:
/// `cargo test -- --ignored --nocapture detects_saves_on_this_machine`
#[test]
#[ignore]
fn detects_saves_on_this_machine() {
    let m = pathogenic();
    for loc in &m.save_locations {
        let root = universal_save_editor_lib::core::paths::expand(&loc.root);
        println!("location {:?} -> {:?}", loc.root, root);
        if let Ok(r) = &root {
            println!("  exists: {}", r.is_dir());
            let base = r.to_string_lossy().to_string();
            let mixed = format!("{base}\\{}", loc.pattern);
            let back = mixed.replace('/', "\\");
            let fwd = mixed.replace('\\', "/");
            for (name, pat) in [("mixed", &mixed), ("backslash", &back), ("forward", &fwd)] {
                match glob::glob(pat) {
                    Ok(it) => {
                        let hits: Vec<_> = it.flatten().collect();
                        println!("  {name:<10} {pat}  -> {} hit(s)", hits.len());
                    }
                    Err(e) => println!("  {name:<10} {pat}  -> error {e}"),
                }
            }
        }
    }
    let saves = save::detect::find_saves(&m, "").unwrap();
    println!("found {} save(s) for {}", saves.len(), m.name);
    for s in &saves {
        println!("  {}, {}, {}", s.title, s.subtitle, s.path);
        // Anything listed must also open cleanly.
        save::detect::load_document(&m, std::path::Path::new(&s.path))
            .unwrap_or_else(|e| panic!("listed save {} failed to open: {e}", s.path));
    }
}

/// A value past the plugin's safe range stops and asks, without touching the
/// file, and the very same edit goes through once the player says yes.
#[test]
fn out_of_range_values_ask_first_then_apply() {
    let m = pathogenic();
    let (tmp, save_path) = scratch_save();
    let backups = BackupManager::new(tmp.path().join("backups"));
    let before = std::fs::read(&save_path).unwrap();

    let huge = Edit {
        pointer: "/player/money".into(),
        value: serde_json::json!(99_999_999_999i64),
    };

    let err = save::apply_and_write(
        &m,
        &backups,
        &save_path,
        std::slice::from_ref(&huge),
        None,
        false,
    )
    .unwrap_err();
    assert_eq!(err.code(), "error.needsConfirmation");
    assert_eq!(
        std::fs::read(&save_path).unwrap(),
        before,
        "the file changed while merely asking"
    );
    assert!(backups.list(Some("pathogenic")).is_empty());

    save::apply_and_write(&m, &backups, &save_path, &[huge], None, true).unwrap();
    let after: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&save_path).unwrap()).unwrap();
    assert_eq!(
        after["player"]["money"],
        serde_json::json!(99_999_999_999i64)
    );
}

/// The defect that prompted all of this: a real save held `stats0/max_hp` =
/// 1009 against a declared ceiling of 999, and the editor refused to save
/// anything at all until the value was "fixed". Opening and saving such a file
/// has to work.
#[test]
fn a_value_the_game_itself_wrote_past_the_limit_does_not_block_saving() {
    let m = pathogenic();
    let tmp = tempfile::tempdir().unwrap();
    let profile = tmp.path().join("save.json");

    let mut doc: serde_json::Value =
        serde_json::from_slice(&std::fs::read(fixture("pathogenic", "save.json")).unwrap())
            .unwrap();
    doc["stats0"]["max_hp"] = serde_json::json!(1009);
    std::fs::write(&profile, serde_json::to_vec_pretty(&doc).unwrap()).unwrap();

    let backups = BackupManager::new(tmp.path().join("backups"));

    // The editor opens it.
    let loaded = save::detect::load_document(&m, &profile).unwrap();
    let view = save::editor::build(&m, "p", &loaded, "en", Default::default());
    assert!(!view.groups.is_empty());

    // And an unrelated edit saves without complaint, leaving 1009 alone.
    save::apply_and_write(
        &m,
        &backups,
        &profile,
        &[Edit {
            pointer: "/plasmids/fragment_num".into(),
            value: serde_json::json!(200),
        }],
        None,
        false,
    )
    .unwrap();

    let after: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&profile).unwrap()).unwrap();
    assert_eq!(after["stats0"]["max_hp"], serde_json::json!(1009));
}

// ---------------------------------------------------------------------------
// Feed The Pit
//
// Godot again, but shaped very differently from Pathogenic: three save slots in
// one file, an unstarted slot written as `{}`, and tool inventories stored as
// JSON *objects* keyed "0".."5" whose values are records. That last shape is
// why `list.source = "object"` grew support for `fields`.

fn feed_the_pit() -> Manifest {
    let reg = Registry::load(&[plugins_dir()]);
    assert!(
        reg.problems().is_empty(),
        "bundled plugins failed to load: {:?}",
        reg.problems()
    );
    reg.get("feed-the-pit")
        .expect("the feed-the-pit plugin ships with the app")
        .manifest
        .clone()
}

fn ftp_progress() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().unwrap();
    let dst = tmp.path().join("progress.save");
    std::fs::copy(fixture("feed-the-pit", "progress.save"), &dst).unwrap();
    (tmp, dst)
}

#[test]
fn feed_the_pit_reads_a_real_progress_save() {
    let m = feed_the_pit();
    let doc = save::detect::load_document(&m, &fixture("feed-the-pit", "progress.save")).unwrap();
    let view = save::editor::build(&m, "p", &doc, "en", Default::default());

    let slot1 = view
        .groups
        .iter()
        .find(|g| g.id == "slot1")
        .expect("slot 1");
    assert!(slot1.absent_reason.is_none());

    let money = slot1.fields.iter().find(|f| f.id == "currency").unwrap();
    assert_eq!(money.value, serde_json::json!(275.0));

    let tools = slot1.lists.iter().find(|l| l.id == "slot1_tools").unwrap();
    assert_eq!(tools.items.len(), 6, "the game gives you six carried slots");

    // Each row is a record, so it exposes both the tool and its durability.
    let first = &tools.items[0];
    let ids: Vec<_> = first.fields.iter().map(|f| f.id.as_str()).collect();
    assert_eq!(ids, vec!["id", "durability"]);
    assert_eq!(first.fields[0].value, serde_json::json!("apple"));
    assert_eq!(first.fields[1].value, serde_json::json!(4));
}

#[test]
fn an_unstarted_slot_explains_itself_instead_of_vanishing() {
    let m = feed_the_pit();
    let doc = save::detect::load_document(&m, &fixture("feed-the-pit", "progress.save")).unwrap();
    let view = save::editor::build(&m, "p", &doc, "tr", Default::default());

    // Slots 2 and 3 exist in the file as `{}`. Hiding them would read as a
    // missing feature; saying why reads as a fact about the save.
    for id in ["slot2", "slot3"] {
        let g = view.groups.iter().find(|g| g.id == id).expect(id);
        let reason = g.absent_reason.as_deref().unwrap_or_default();
        assert!(
            reason.contains("Bu yuva bos") || reason.contains("Bu yuva boş"),
            "{id}: {reason}"
        );
        assert!(g.fields.is_empty() && g.lists.is_empty());
    }
}

#[test]
fn every_tool_the_game_defines_is_offered() {
    let m = feed_the_pit();
    let tools = m.option_sets.get("tools").expect("a 'tools' option set");

    // 77 tools plus the empty choice. If a game update adds tools, re-run
    // tools/extract-tools.py, this number is the reminder.
    assert_eq!(tools.len(), 78, "77 tools and one empty slot");
    assert!(tools.iter().any(|c| c.value == "apple"));
    assert!(tools.iter().any(|c| c.value == "card_angle"));

    // An empty slot has to stay reachable, or a slot could be filled but never
    // cleared.
    assert_eq!(tools[0].value, "");

    let cards = tools
        .iter()
        .filter(|c| c.value.as_str().is_some_and(|v| v.starts_with("card_")))
        .count();
    assert_eq!(cards, 51, "the deck the extractor found");
}

#[test]
fn a_tool_slot_can_be_filled_and_cleared() {
    let m = feed_the_pit();
    let (tmp, path) = ftp_progress();
    let backups = BackupManager::new(tmp.path().join("backups"));

    let slot = "/tracked_progress/save_slots/1/tools/2";
    save::apply_and_write(
        &m,
        &backups,
        &path,
        &[
            Edit {
                pointer: format!("{slot}/id"),
                value: serde_json::json!("golden_apple"),
            },
            Edit {
                pointer: format!("{slot}/durability"),
                value: serde_json::json!(9),
            },
        ],
        None,
        false,
    )
    .unwrap();

    let after: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        after.pointer(&format!("{slot}/id")).unwrap(),
        "golden_apple"
    );
    assert_eq!(after.pointer(&format!("{slot}/durability")).unwrap(), 9);

    // And back to empty.
    save::apply_and_write(
        &m,
        &backups,
        &path,
        &[Edit {
            pointer: format!("{slot}/id"),
            value: serde_json::json!(""),
        }],
        None,
        false,
    )
    .unwrap();
    let after: serde_json::Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(after.pointer(&format!("{slot}/id")).unwrap(), "");
}

#[test]
fn money_keeps_its_fractional_form() {
    // Godot wrote `275.0`. Handing the game back `275` is exactly the mistake
    // that corrupts saves, so a `number` field must stay a float.
    let m = feed_the_pit();
    let (tmp, path) = ftp_progress();
    let backups = BackupManager::new(tmp.path().join("backups"));

    save::apply_and_write(
        &m,
        &backups,
        &path,
        &[Edit {
            pointer: "/tracked_progress/save_slots/1/currency".into(),
            value: serde_json::json!(9999),
        }],
        None,
        false,
    )
    .unwrap();

    let text = std::fs::read_to_string(&path).unwrap();
    assert!(
        text.contains("9999.0"),
        "money lost its decimal point:\n{text}"
    );
}

#[test]
fn an_empty_slot_cannot_be_written_to() {
    // Slot 2 is `{}`. Its pointers are not in the writable set, because that
    // set is built from the document actually on disk.
    let m = feed_the_pit();
    let (tmp, path) = ftp_progress();
    let backups = BackupManager::new(tmp.path().join("backups"));

    let err = save::apply_and_write(
        &m,
        &backups,
        &path,
        &[Edit {
            pointer: "/tracked_progress/save_slots/2/currency".into(),
            value: serde_json::json!(500.0),
        }],
        None,
        false,
    )
    .unwrap_err();
    assert!(format!("{err}").to_lowercase().contains("not"), "{err}");
}

#[test]
fn the_save_version_is_shown_but_not_writable() {
    let m = feed_the_pit();
    let doc = save::detect::load_document(&m, &fixture("feed-the-pit", "progress.save")).unwrap();
    let view = save::editor::build(&m, "p", &doc, "en", Default::default());

    let tracking = view.groups.iter().find(|g| g.id == "tracking").unwrap();
    let version = tracking.fields.iter().find(|f| f.id == "version").unwrap();
    assert_eq!(version.value, serde_json::json!(77.1));
    assert!(version.read_only);

    let writable = save::editor::writable_fields(&m, &doc);
    assert!(!writable.contains_key("/version"));
}

#[test]
fn character_memories_are_a_second_save_of_the_same_game() {
    let m = feed_the_pit();
    let doc = save::detect::load_document(&m, &fixture("feed-the-pit", "character_memories.save"))
        .unwrap();
    let view = save::editor::build(&m, "p", &doc, "en", Default::default());

    // The progress groups do not apply here, and the memories group does.
    let memories = view.groups.iter().find(|g| g.id == "memories").unwrap();
    assert!(memories.absent_reason.is_none());
    assert!(memories.fields.iter().any(|f| f.id == "deaths"));

    let cardmaster = memories
        .lists
        .iter()
        .find(|l| l.id == "cardmaster")
        .unwrap();
    assert_eq!(cardmaster.items.len(), 4, "m1..m4");
}

#[test]
fn feed_the_pit_labels_are_translated_into_every_shipped_language() {
    let m = feed_the_pit();
    let doc = save::detect::load_document(&m, &fixture("feed-the-pit", "progress.save")).unwrap();

    for locale in [
        "tr", "de", "es", "fr", "it", "pt-BR", "ru", "pl", "uk", "ja", "ko", "zh-CN",
    ] {
        let view = save::editor::build(&m, "p", &doc, locale, Default::default());
        let slot1 = view.groups.iter().find(|g| g.id == "slot1").unwrap();
        assert_ne!(
            slot1.label, "Save slot 1",
            "{locale} left the group label in English"
        );

        let money = slot1.fields.iter().find(|f| f.id == "currency").unwrap();
        assert_ne!(money.label, "Money", "{locale} left 'Money' in English");
    }
}

#[test]
fn tool_names_are_never_translated() {
    // "Golden Apple" is what a player types into a wiki search. Translating it
    // would make the editor harder to use, not easier.
    let m = feed_the_pit();
    let apple = m
        .option_sets
        .get("tools")
        .unwrap()
        .iter()
        .find(|c| c.value == "golden_apple")
        .unwrap();
    assert_eq!(apple.label, "Golden Apple");
    assert!(apple.label_i18n.is_empty());
}
