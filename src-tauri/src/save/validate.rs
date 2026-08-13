use crate::core::error::{Error, Result, Rule, Warning};
use crate::core::model::Edit;
use crate::plugins::manifest::{Field, FieldKind, Manifest};
use crate::save::editor;
use serde_json::{Number, Value};

/// Apply a batch of edits to a parsed save, validating every one first.
///
/// All-or-nothing: if any edit is rejected the document is returned untouched,
/// so a typo in one field can never leave the save half-modified.
///
/// Returns the number of values that actually changed, plus any values that
/// went outside the plugin''s safe range. Warnings do not stop the write,
/// deciding what to do with them is the caller''s job.
pub fn apply_edits(
    manifest: &Manifest,
    doc: &mut Value,
    edits: &[Edit],
) -> Result<(usize, Vec<Warning>)> {
    let writable = editor::writable_fields(manifest, doc);

    // Validate and coerce everything before touching the document.
    let mut prepared = Vec::with_capacity(edits.len());
    let mut warnings = Vec::new();
    for edit in edits {
        let field = writable
            .get(&edit.pointer)
            .ok_or_else(|| Error::UnknownField(edit.pointer.clone()))?;
        let (value, warning) = coerce(manifest, field, &edit.value)?;
        if let Some(rule) = warning {
            warnings.push(Warning {
                pointer: edit.pointer.clone(),
                field: field.label.clone(),
                rule: rule.code().to_string(),
                limit: rule.limit_text(),
                value: render(&value),
            });
        }

        if doc.pointer(&edit.pointer).is_none() {
            return Err(Error::FieldRule {
                field: field.label.clone(),
                reason: Rule::NotPresent,
            });
        }
        prepared.push((edit.pointer.clone(), value));
    }

    let mut changed = 0;
    for (pointer, value) in prepared {
        // Safe: presence was checked above and nothing has mutated since.
        let slot = doc
            .pointer_mut(&pointer)
            .expect("pointer verified before mutation");
        if *slot != value {
            *slot = value;
            changed += 1;
        }
    }
    Ok((changed, warnings))
}

/// A value as the player would read it back.
fn render(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    }
}

/// Check a single incoming value against its field rules and convert it to the
/// exact JSON shape the game expects.
///
/// Type problems are errors, a decimal where the engine wants an integer will
/// break the save. Range problems are only warnings: `min`/`max` describe what
/// is known to be safe, not what is possible.
pub fn coerce(manifest: &Manifest, field: &Field, input: &Value) -> Result<(Value, Option<Rule>)> {
    let reject = |reason: Rule| Error::FieldRule {
        field: field.label.clone(),
        reason,
    };

    match field.kind {
        FieldKind::Integer => {
            let n = as_f64(input).ok_or_else(|| reject(Rule::NotWholeNumber))?;
            if !n.is_finite() {
                return Err(reject(Rule::NotWholeNumber));
            }
            if n.fract() != 0.0 {
                return Err(reject(Rule::HasDecimalPoint));
            }
            // Outside i64 there is no integer left to write, so this one is a
            // hard limit rather than a matter of taste.
            if n < i64::MIN as f64 || n > i64::MAX as f64 {
                return Err(reject(Rule::TooLargeForGame));
            }
            // Written as a JSON integer, engines that expect `6` often reject `6.0`.
            Ok((
                Value::Number(Number::from(n as i64)),
                range_warning(field, n),
            ))
        }

        FieldKind::Number => {
            let n = as_f64(input).ok_or_else(|| reject(Rule::NotANumber))?;
            if !n.is_finite() {
                return Err(reject(Rule::NotANumber));
            }
            // `from_f64` keeps the value a JSON float, so `9999` submitted for a
            // decimal field is written back as `9999.0` and the game still sees
            // the type it stored.
            let value = Number::from_f64(n)
                .map(Value::Number)
                .ok_or_else(|| reject(Rule::NotANumber))?;
            Ok((value, range_warning(field, n)))
        }

        FieldKind::Text => {
            let s = input.as_str().ok_or_else(|| reject(Rule::NotText))?;
            if let Some(max) = field.max_length {
                if s.chars().count() > max {
                    return Err(reject(Rule::TooLong(max)));
                }
            }
            Ok((Value::String(s.to_string()), None))
        }

        FieldKind::Boolean => input
            .as_bool()
            .map(|b| (Value::Bool(b), None))
            .ok_or_else(|| reject(Rule::NotABoolean)),

        FieldKind::Choice => {
            let allowed = manifest.choices(field);
            if allowed.iter().any(|c| &c.value == input) {
                Ok((input.clone(), None))
            } else {
                Err(reject(Rule::NotAnOption))
            }
        }
    }
}

/// `min`/`max` mark the range the plugin author knows to be safe. Beyond it the
/// value still gets written, the player is told and asked first.
fn range_warning(field: &Field, n: f64) -> Option<Rule> {
    if let Some(min) = field.min {
        if n < min {
            return Some(Rule::TooSmall(min));
        }
    }
    if let Some(max) = field.max {
        if n > max {
            return Some(Rule::TooLarge(max));
        }
    }
    None
}

/// Accept a number, or a numeric string, text inputs in the GUI hand us
/// strings and making the user's browser locale matter would be unkind.
fn as_f64(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse::<f64>().ok(),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        serde_json::from_str(
            r#"{"id":"t","name":"T","version":"1","format":"json",
              "save_locations":[{"root":"{HOME}/t","pattern":"*.json"}],
              "option_sets":{"rarity":[{"value":0,"label":"Common"},{"value":3,"label":"Legendary"}]},
              "groups":[{"id":"c","label":"C","fields":[
                {"id":"money","label":"Money","pointer":"/money","type":"integer","min":0,"max":999999999},
                {"id":"stamina","label":"Stamina","pointer":"/stamina","type":"number","min":0,"max":9999},
                {"id":"name","label":"Name","pointer":"/name","type":"text","max_length":8},
                {"id":"flag","label":"Flag","pointer":"/flag","type":"boolean"},
                {"id":"rank","label":"Rank","pointer":"/rank","type":"choice","options_ref":"rarity"},
                {"id":"seed","label":"Seed","pointer":"/seed","type":"text","read_only":true}]}]}"#,
        )
        .unwrap()
    }

    fn doc() -> Value {
        serde_json::json!({"money":100,"stamina":100.0,"name":"Alex","flag":false,"rank":0,"seed":"XYZ"})
    }

    fn edit(p: &str, v: Value) -> Edit {
        Edit {
            pointer: p.into(),
            value: v,
        }
    }

    #[test]
    fn applies_a_valid_edit() {
        let m = manifest();
        let mut d = doc();
        let (n, warnings) =
            apply_edits(&m, &mut d, &[edit("/money", serde_json::json!(50000))]).unwrap();
        assert_eq!(n, 1);
        assert!(warnings.is_empty(), "an in-range value produced a warning");
        assert_eq!(d["money"], serde_json::json!(50000));
    }

    #[test]
    fn integer_field_stays_an_integer() {
        let m = manifest();
        let mut d = doc();
        apply_edits(&m, &mut d, &[edit("/money", serde_json::json!(42.0))]).unwrap();
        assert!(d["money"].is_i64(), "money became {:?}", d["money"]);
        assert_eq!(serde_json::to_string(&d["money"]).unwrap(), "42");
    }

    #[test]
    fn decimal_field_keeps_its_fractional_form() {
        // The exact mistake that corrupts hand-edited saves: writing 9999
        // where the engine stored 100.0.
        let m = manifest();
        let mut d = doc();
        apply_edits(&m, &mut d, &[edit("/stamina", serde_json::json!(9999))]).unwrap();
        assert_eq!(serde_json::to_string(&d["stamina"]).unwrap(), "9999.0");
    }

    #[test]
    fn integer_field_rejects_a_decimal() {
        let m = manifest();
        let mut d = doc();
        let err = apply_edits(&m, &mut d, &[edit("/money", serde_json::json!(1.5))]).unwrap_err();
        assert!(err.to_string().contains("whole number"));
    }

    /// Ranges are advice, not law. The games themselves write values past the
    /// limits a plugin declares, so going beyond one has to be possible, the
    /// caller is simply told which values were involved.
    #[test]
    fn a_value_below_the_safe_range_warns_and_is_still_applied() {
        let m = manifest();
        let mut d = doc();
        let (n, warnings) =
            apply_edits(&m, &mut d, &[edit("/money", serde_json::json!(-5000))]).unwrap();

        assert_eq!(n, 1);
        assert_eq!(d["money"], serde_json::json!(-5000));
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule, "rule.tooSmall");
        assert_eq!(warnings[0].limit, "0");
        assert_eq!(warnings[0].field, "Money");
        assert_eq!(warnings[0].pointer, "/money");
    }

    #[test]
    fn a_value_above_the_safe_range_warns_and_is_still_applied() {
        let m = manifest();
        let mut d = doc();
        let (_, warnings) =
            apply_edits(&m, &mut d, &[edit("/money", serde_json::json!(1e12))]).unwrap();

        assert_eq!(d["money"], serde_json::json!(1_000_000_000_000i64));
        assert_eq!(warnings.len(), 1);
        assert_eq!(warnings[0].rule, "rule.tooLarge");
        assert_eq!(warnings[0].limit, "999999999");
    }

    #[test]
    fn several_out_of_range_values_are_all_reported() {
        let m = manifest();
        let mut d = doc();
        let (_, warnings) = apply_edits(
            &m,
            &mut d,
            &[
                edit("/money", serde_json::json!(1e12)),
                edit("/stamina", serde_json::json!(50000)),
            ],
        )
        .unwrap();
        assert_eq!(warnings.len(), 2);
    }

    /// A number too big to be an integer at all is still a hard error: there is
    /// no value left to write.
    #[test]
    fn a_number_beyond_integer_range_is_refused_outright() {
        let m = manifest();
        let mut d = doc();
        assert!(apply_edits(&m, &mut d, &[edit("/money", serde_json::json!(1e30))]).is_err());
        assert_eq!(
            d["money"],
            serde_json::json!(100),
            "the value was applied anyway"
        );
    }

    #[test]
    fn rejects_overlong_text() {
        let m = manifest();
        let mut d = doc();
        let err = apply_edits(
            &m,
            &mut d,
            &[edit("/name", serde_json::json!("Bartholomew"))],
        )
        .unwrap_err();
        assert!(err.to_string().contains("at most 8 characters"));
    }

    #[test]
    fn rejects_a_value_outside_the_choice_list() {
        let m = manifest();
        let mut d = doc();
        let err = apply_edits(&m, &mut d, &[edit("/rank", serde_json::json!(7))]).unwrap_err();
        assert!(err.to_string().contains("available options"));
    }

    #[test]
    fn rejects_writes_to_read_only_fields() {
        let m = manifest();
        let mut d = doc();
        let err = apply_edits(&m, &mut d, &[edit("/seed", serde_json::json!("HACK"))]).unwrap_err();
        assert!(matches!(err, Error::UnknownField(_)));
        assert_eq!(d["seed"], serde_json::json!("XYZ"));
    }

    #[test]
    fn rejects_pointers_the_manifest_never_declared() {
        let m = manifest();
        let mut d = doc();
        let err = apply_edits(
            &m,
            &mut d,
            &[edit("/../../etc/passwd", serde_json::json!(1))],
        )
        .unwrap_err();
        assert!(matches!(err, Error::UnknownField(_)));
    }

    #[test]
    fn a_rejected_edit_leaves_the_whole_batch_unapplied() {
        let m = manifest();
        let mut d = doc();
        let err = apply_edits(
            &m,
            &mut d,
            &[
                edit("/money", serde_json::json!(50000)),
                edit("/name", serde_json::json!("WayTooLongName")),
            ],
        )
        .unwrap_err();
        assert!(err.to_string().contains("at most 8"));
        assert_eq!(
            d["money"],
            serde_json::json!(100),
            "first edit leaked through"
        );
    }

    #[test]
    fn numeric_strings_from_the_gui_are_accepted() {
        let m = manifest();
        let mut d = doc();
        apply_edits(&m, &mut d, &[edit("/money", serde_json::json!("50000"))]).unwrap();
        assert_eq!(d["money"], serde_json::json!(50000));
    }

    #[test]
    fn unchanged_values_are_not_counted_as_edits() {
        let m = manifest();
        let mut d = doc();
        let (n, _) = apply_edits(&m, &mut d, &[edit("/money", serde_json::json!(100))]).unwrap();
        assert_eq!(n, 0);
    }

    #[test]
    fn rejects_nan_and_infinity() {
        let m = manifest();
        let mut d = doc();
        assert!(apply_edits(&m, &mut d, &[edit("/stamina", serde_json::json!("NaN"))]).is_err());
        assert!(apply_edits(&m, &mut d, &[edit("/stamina", serde_json::json!("inf"))]).is_err());
    }
}
