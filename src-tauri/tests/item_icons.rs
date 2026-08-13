//! Measure the icon reader against the real installation.
use std::path::PathBuf;
use universal_save_editor_lib::plugins::{archive, registry::Registry};

#[test]
#[ignore]
fn item_icons_from_the_real_game() {
    let plugins = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("plugins");
    let reg = Registry::load(&[plugins]);
    let m = &reg.get("pathogenic").unwrap().manifest;

    let mut sets: Vec<(&str, Vec<String>)> = Vec::new();
    for key in ["weapons", "organs"] {
        let vals = m
            .option_sets
            .get(key)
            .map(|v| {
                v.iter()
                    .filter_map(|c| c.value.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default();
        sets.push((key, vals));
    }
    let wanted: usize = sets.iter().map(|(_, v)| v.len()).sum();

    let started = std::time::Instant::now();
    let icons = archive::item_icons(m, &sets);
    let took = started.elapsed();

    println!("istenen : {wanted}");
    println!(
        "bulunan : {} ({}%)",
        icons.len(),
        100 * icons.len() / wanted.max(1)
    );
    println!("sure    : {:?}", took);
    let bytes: usize = icons.values().map(|v| v.len()).sum();
    println!("toplam  : {} KB (data URI)", bytes / 1024);

    for name in ["rocket_launcher", "assault_rifle", "cannon", "homing"] {
        match icons.get(name) {
            Some(uri) => println!(
                "  {name:18} {} ... {} byte",
                &uri[..30.min(uri.len())],
                uri.len()
            ),
            None => println!("  {name:18} YOK"),
        }
    }
}
