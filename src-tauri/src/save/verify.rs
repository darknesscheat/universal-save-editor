//! Checking that the bytes about to be written still say what we meant.
//!
//! `apply_and_write` serialises the edited document, parses that back, and
//! compares. The comparison used to be `==`, which turned out to be stricter
//! than JSON itself can promise: a Sort Them Ducks save holds 36,000 floats,
//! and for one of them `parse -> write -> parse` returned a value one bit away
//! from where it started. Nothing was corrupted; a duck's rotation differed in
//! the seventeenth significant digit. The editor refused to save the file at
//! all.
//!
//! So the rule is now "the same document, allowing for how floating point is
//! written down". Everything that could actually break a save stays exact:
//!
//! - structure, keys and array lengths
//! - strings, booleans and nulls
//! - **whether a number is an integer or a decimal**, which is the one numeric
//!   property engines really do reject
//!
//! Only the last bit or two of a float may move.

use serde_json::Value;

/// How far two floats may drift and still count as the same number.
///
/// Two units in the last place: enough for a representation round trip,
/// far too little to hide a changed value.
const MAX_ULPS: i64 = 2;

/// Does the reparsed document still say what the edited one said?
pub fn matches(edited: &Value, rebuilt: &Value) -> bool {
    difference(edited, rebuilt, String::new()).is_none()
}

/// The first place the two documents diverge, as a JSON pointer and a
/// description. `None` when they agree.
///
/// Returning the location rather than a bare bool is what makes a failure
/// diagnosable: "the rebuilt save did not match" on a two megabyte file is not
/// something anyone can act on.
pub fn difference(edited: &Value, rebuilt: &Value, at: String) -> Option<String> {
    match (edited, rebuilt) {
        (Value::Object(a), Value::Object(b)) => {
            if a.len() != b.len() {
                return Some(format!("{at}: {} keys became {}", a.len(), b.len()));
            }
            for (key, value) in a {
                let escaped = key.replace('~', "~0").replace('/', "~1");
                match b.get(key) {
                    Some(other) => {
                        let found = difference(value, other, format!("{at}/{escaped}"));
                        if found.is_some() {
                            return found;
                        }
                    }
                    None => return Some(format!("{at}/{escaped}: went missing")),
                }
            }
            None
        }

        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                return Some(format!("{at}: {} items became {}", a.len(), b.len()));
            }
            for (i, (value, other)) in a.iter().zip(b).enumerate() {
                let found = difference(value, other, format!("{at}/{i}"));
                if found.is_some() {
                    return found;
                }
            }
            None
        }

        (Value::Number(a), Value::Number(b)) => {
            // An integer that came back a decimal, or the reverse, is exactly
            // the corruption this project exists to avoid. Never tolerated.
            if a.is_f64() != b.is_f64() {
                return Some(format!("{at}: {a} and {b} are not the same kind of number"));
            }
            if !a.is_f64() {
                return (a != b).then(|| format!("{at}: {a} became {b}"));
            }
            match (a.as_f64(), b.as_f64()) {
                (Some(x), Some(y)) if within_tolerance(x, y) => None,
                _ => Some(format!("{at}: {a} became {b}")),
            }
        }

        _ if edited == rebuilt => None,
        _ => Some(format!("{at}: {edited} became {rebuilt}")),
    }
}

/// Same number, or the same number written down slightly differently.
fn within_tolerance(a: f64, b: f64) -> bool {
    if a == b {
        return true;
    }
    // Infinities and NaN cannot survive JSON anyway, and opposite signs are
    // never a rounding artefact.
    if !a.is_finite() || !b.is_finite() || a.is_sign_negative() != b.is_sign_negative() {
        return false;
    }
    // For finite floats of one sign the bit patterns are ordered, so their
    // distance apart counts representable values between them.
    let steps = (a.to_bits() as i64).saturating_sub(b.to_bits() as i64);
    steps.saturating_abs() <= MAX_ULPS
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn an_identical_document_matches() {
        let doc = json!({ "money": 4.0, "ducks": [{ "rz": 1.5 }], "name": "x" });
        assert!(matches(&doc, &doc.clone()));
    }

    /// The case that started this: a float one bit away from where it began.
    #[test]
    fn a_float_that_moved_by_one_bit_still_matches() {
        let original: f64 = 1.1095581839981605e-6;
        let nudged = f64::from_bits(original.to_bits() - 1);
        assert_ne!(original, nudged);

        let a = json!({ "rz": original });
        let b = json!({ "rz": nudged });
        assert!(matches(&a, &b), "{:?}", difference(&a, &b, String::new()));
    }

    #[test]
    fn a_float_that_actually_changed_does_not_match() {
        let a = json!({ "money": 100.0 });
        let b = json!({ "money": 100.00001 });
        assert!(!matches(&a, &b));
    }

    /// The tolerance is relative to the size of the number, so a large value
    /// may not quietly gain whole units.
    #[test]
    fn a_large_float_cannot_drift_by_a_whole_unit() {
        let a = json!({ "money": 999999.0 });
        let b = json!({ "money": 1000000.0 });
        assert!(!matches(&a, &b));
    }

    #[test]
    fn an_integer_that_became_a_decimal_does_not_match() {
        // The whole reason the check exists: some engines reject `6.0` where
        // they wrote `6`.
        let a = json!({ "hp": 6 });
        let b = json!({ "hp": 6.0 });
        assert!(!matches(&a, &b));
        assert!(difference(&a, &b, String::new())
            .unwrap()
            .contains("not the same kind of number"));
    }

    #[test]
    fn integers_are_compared_exactly() {
        assert!(!matches(&json!({ "hp": 6 }), &json!({ "hp": 7 })));
        assert!(matches(&json!({ "hp": 6 }), &json!({ "hp": 6 })));
    }

    #[test]
    fn a_missing_key_is_reported_with_its_pointer() {
        let a = json!({ "player": { "hp": 1, "money": 2 } });
        let b = json!({ "player": { "hp": 1 } });
        let d = difference(&a, &b, String::new()).unwrap();
        assert!(d.starts_with("/player:"), "{d}");
    }

    #[test]
    fn a_changed_string_is_reported_with_its_pointer() {
        let a = json!({ "slots": [{ "id": "apple" }] });
        let b = json!({ "slots": [{ "id": "pizza" }] });
        let d = difference(&a, &b, String::new()).unwrap();
        assert!(d.starts_with("/slots/0/id:"), "{d}");
    }

    #[test]
    fn a_shorter_array_is_reported() {
        let a = json!({ "eggs": [1, 2, 3] });
        let b = json!({ "eggs": [1, 2] });
        assert!(difference(&a, &b, String::new())
            .unwrap()
            .contains("3 items became 2"));
    }

    #[test]
    fn a_key_containing_a_slash_is_escaped_in_the_pointer() {
        let a = json!({ "a/b": 1 });
        let b = json!({ "a/b": 2 });
        assert_eq!(
            difference(&a, &b, String::new()).unwrap(),
            "/a~1b: 1 became 2"
        );
    }

    #[test]
    fn opposite_signs_never_count_as_rounding() {
        let a = json!({ "x": 1e-300 });
        let b = json!({ "x": -1e-300 });
        assert!(!matches(&a, &b));
    }

    #[test]
    fn booleans_and_nulls_are_exact() {
        assert!(!matches(&json!({ "on": true }), &json!({ "on": false })));
        assert!(!matches(&json!({ "x": null }), &json!({ "x": 0 })));
    }
}
