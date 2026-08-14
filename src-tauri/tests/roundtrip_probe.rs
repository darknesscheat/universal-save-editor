//! A scratch probe, run by hand against a save on this machine:
//!
//!     cargo test --test roundtrip_probe -- --ignored --nocapture
//!
//! It answers one question: does `parse -> write -> parse` come back equal?
//! That is the check `save::apply_and_write` makes before it touches the file,
//! and when it fails the editor refuses to save.

use std::path::PathBuf;
use universal_save_editor_lib::plugins::adapter;
use universal_save_editor_lib::save::verify;

fn ducks_save() -> PathBuf {
    PathBuf::from(std::env::var("USERPROFILE").unwrap())
        .join("AppData/LocalLow/Mr_Duck/Sort Them Ducks/duckgame_save.json")
}

#[test]
#[ignore = "needs Sort Them Ducks installed and played once"]
fn a_real_unity_save_survives_the_round_trip() {
    let path = ducks_save();
    if !path.is_file() {
        println!("no save at {}, nothing to probe", path.display());
        return;
    }

    let json = adapter::adapter_for("json").unwrap();
    let bytes = std::fs::read(&path).unwrap();

    let doc = json.parse(&bytes).expect("the save parses");
    let rebuilt = json.write(&doc).expect("it serialises");
    let again = json.parse(&rebuilt).expect("the rebuilt save parses");

    println!(
        "original {} bytes -> rebuilt {} bytes",
        bytes.len(),
        rebuilt.len()
    );

    match verify::difference(&doc, &again, String::new()) {
        None => println!("round trip is exact"),
        Some(d) => panic!("round trip changed something:\n  {d}"),
    }
}
