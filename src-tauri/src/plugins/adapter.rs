use crate::core::error::{Error, Result};
use serde_json::Value;

/// Turns the bytes of a save file into a JSON document and back.
///
/// The editor, validator and GUI all work on `serde_json::Value`, so a game
/// with an unusual container (base64-wrapped INI, a binary blob, XML) only has
/// to implement this one trait to gain the whole feature set.
pub trait FormatAdapter: Send + Sync {
    /// Identifier used by `"format"` in a manifest.
    fn id(&self) -> &'static str;

    fn parse(&self, bytes: &[u8]) -> Result<Value>;

    /// Must round-trip: `write(parse(x))` has to stay loadable by the game.
    fn write(&self, value: &Value) -> Result<Vec<u8>>;
}

/// Plain UTF-8 JSON, the most common modern save format.
///
/// Key order is preserved (see the `preserve_order` feature on `serde_json`)
/// so a rewritten save stays as close to the original as possible, that keeps
/// diffs small and avoids upsetting engines that are picky about layout.
pub struct JsonAdapter;

impl FormatAdapter for JsonAdapter {
    fn id(&self) -> &'static str {
        "json"
    }

    fn parse(&self, bytes: &[u8]) -> Result<Value> {
        // Tolerate a UTF-8 BOM; several engines emit one.
        let text = std::str::from_utf8(bytes)
            .map_err(|_| Error::SaveRead("the file is not valid UTF-8 text".into()))?
            .trim_start_matches('\u{feff}');
        serde_json::from_str(text).map_err(|e| Error::SaveRead(e.to_string()))
    }

    fn write(&self, value: &Value) -> Result<Vec<u8>> {
        // Tab indentation matches what Godot's `JSON.stringify(v, "\t")`
        // produces, which is what the reference save used.
        let mut buf = Vec::new();
        let indent = b"\t";
        let formatter = serde_json::ser::PrettyFormatter::with_indent(indent);
        let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
        serde::Serialize::serialize(value, &mut ser)
            .map_err(|e| Error::WriteFailed(e.to_string()))?;
        Ok(buf)
    }
}

/// Look up an adapter by the `"format"` string in a manifest.
pub fn adapter_for(format: &str) -> Result<&'static dyn FormatAdapter> {
    match format {
        "json" => Ok(&JsonAdapter),
        other => Err(Error::PluginLoad(format!(
            "unsupported save format '{other}' (this build knows: json)"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_round_trips_without_losing_data() {
        let src = br#"{"b":1,"a":{"n":2.5,"t":"x","flag":true,"list":[1,2]}}"#;
        let parsed = JsonAdapter.parse(src).unwrap();
        let written = JsonAdapter.write(&parsed).unwrap();
        let reparsed = JsonAdapter.parse(&written).unwrap();
        assert_eq!(parsed, reparsed);
    }

    #[test]
    fn json_preserves_key_order() {
        let parsed = JsonAdapter.parse(br#"{"z":1,"a":2,"m":3}"#).unwrap();
        let text = String::from_utf8(JsonAdapter.write(&parsed).unwrap()).unwrap();
        let z = text.find("\"z\"").unwrap();
        let a = text.find("\"a\"").unwrap();
        let m = text.find("\"m\"").unwrap();
        assert!(z < a && a < m, "key order changed: {text}");
    }

    #[test]
    fn json_keeps_integers_integral() {
        let parsed = JsonAdapter.parse(br#"{"hp":6}"#).unwrap();
        let text = String::from_utf8(JsonAdapter.write(&parsed).unwrap()).unwrap();
        assert!(text.contains("\"hp\": 6"), "got {text}");
        assert!(!text.contains("6.0"));
    }

    #[test]
    fn json_accepts_a_bom() {
        let mut bytes = vec![0xEF, 0xBB, 0xBF];
        bytes.extend_from_slice(br#"{"a":1}"#);
        assert!(JsonAdapter.parse(&bytes).is_ok());
    }

    #[test]
    fn unknown_format_is_rejected() {
        assert!(adapter_for("sqlite").is_err());
    }
}
