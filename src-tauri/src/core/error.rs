use serde::{Serialize, Serializer};
use serde_json::json;

/// Every fallible operation in the app funnels through this type.
///
/// Each variant carries a stable `code` and its parameters as well as an
/// English message. The frontend translates by code and falls back to the
/// message when it has no translation, so adding a language never risks
/// leaving the user with a blank error.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Plugin '{0}' was not found.")]
    PluginNotFound(String),

    #[error("The plugin folder could not be read: {0}")]
    PluginLoad(String),

    #[error("This save file no longer exists on disk.")]
    SaveMissing,

    #[error("This save file could not be read: {0}")]
    SaveRead(String),

    #[error("This does not look like a {game} save file.")]
    SaveFormat { game: String },

    #[error("'{field}' {reason}")]
    Validation { field: String, reason: String },

    /// A validation failure the frontend can phrase itself.
    #[error("'{field}' {}", reason.message())]
    FieldRule { field: String, reason: Rule },

    #[error("'{0}' is not an editable field in this save.")]
    UnknownField(String),

    #[error("That file is outside the folders this plugin manages.")]
    PathNotAllowed,

    #[error("'{list}' cannot have rows added or removed.")]
    ListNotEditable { list: String },

    #[error("'{list}' cannot hold more than {max} entries.")]
    ListFull { list: String, max: usize },

    #[error("'{list}' must keep at least {min} entries.")]
    ListAtMinimum { list: String, min: usize },

    #[error("This save was changed by the game after you opened it. Reload before saving.")]
    SaveChangedOnDisk,

    #[error("{message}")]
    Constraint {
        left: String,
        right: String,
        message: String,
    },

    /// Not a failure: the edit is legal but some values sit outside the range
    /// the plugin calls safe. The frontend shows them and asks; a second call
    /// with `confirm` goes through.
    #[error("{} value(s) are outside the safe range.", warnings.len())]
    NeedsConfirmation { warnings: Vec<Warning> },

    #[error("The backup could not be created, so the save was left untouched: {0}")]
    BackupFailed(String),

    #[error("Backup '{0}' was not found.")]
    BackupNotFound(String),

    #[error("The save could not be written: {0}. Your original save is unchanged.")]
    WriteFailed(String),

    #[error("{0}")]
    Io(String),
}

/// A value the plugin considers risky but not illegal.
///
/// Range limits used to be hard failures, which turned out to be wrong: the
/// games themselves write values past those limits (Pathogenic recorded a max
/// health of 1009 against a declared ceiling of 999), so a save could refuse to
/// open for being *too real*. Now the range describes what is known-safe, and
/// going beyond it is the player's call to make with their eyes open.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Warning {
    /// Where the value lives, so the GUI can highlight the exact row.
    pub pointer: String,
    /// The field's translated label, ready to show.
    pub field: String,
    /// `rule.tooSmall` or `rule.tooLarge`.
    pub rule: String,
    /// The safe limit that was passed, rendered for display.
    pub limit: String,
    /// What the player asked for.
    pub value: String,
}

/// The specific reason a value was rejected, kept structured so it can be
/// translated with the numbers substituted in the right places.
#[derive(Debug, Clone)]
pub enum Rule {
    NotWholeNumber,
    HasDecimalPoint,
    NotANumber,
    NotText,
    NotABoolean,
    NotAnOption,
    TooSmall(f64),
    TooLarge(f64),
    TooLong(usize),
    TooLargeForGame,
    NotPresent,
}

impl Rule {
    pub fn code(&self) -> &'static str {
        match self {
            Rule::NotWholeNumber => "rule.notWholeNumber",
            Rule::HasDecimalPoint => "rule.hasDecimalPoint",
            Rule::NotANumber => "rule.notANumber",
            Rule::NotText => "rule.notText",
            Rule::NotABoolean => "rule.notABoolean",
            Rule::NotAnOption => "rule.notAnOption",
            Rule::TooSmall(_) => "rule.tooSmall",
            Rule::TooLarge(_) => "rule.tooLarge",
            Rule::TooLong(_) => "rule.tooLong",
            Rule::TooLargeForGame => "rule.tooLargeForGame",
            Rule::NotPresent => "rule.notPresent",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Rule::NotWholeNumber => "must be a whole number.".into(),
            Rule::HasDecimalPoint => "must be a whole number, without a decimal point.".into(),
            Rule::NotANumber => "must be a number.".into(),
            Rule::NotText => "must be text.".into(),
            Rule::NotABoolean => "must be on or off.".into(),
            Rule::NotAnOption => "is not one of the available options.".into(),
            Rule::TooSmall(n) => format!("cannot be lower than {}.", trim(*n)),
            Rule::TooLarge(n) => format!("cannot be higher than {}.", trim(*n)),
            Rule::TooLong(n) => format!("must be at most {n} characters."),
            Rule::TooLargeForGame => "is too large for this game.".into(),
            Rule::NotPresent => "is not present in this save file.".into(),
        }
    }

    /// The limit this rule is about, rendered for display. Empty when the rule
    /// has no number in it.
    pub fn limit_text(&self) -> String {
        match self {
            Rule::TooSmall(n) | Rule::TooLarge(n) => trim(*n),
            Rule::TooLong(n) => n.to_string(),
            _ => String::new(),
        }
    }

    fn params(&self) -> serde_json::Value {
        match self {
            Rule::TooSmall(n) | Rule::TooLarge(n) => json!({ "limit": trim(*n) }),
            Rule::TooLong(n) => json!({ "limit": n }),
            _ => json!({}),
        }
    }
}

fn trim(n: f64) -> String {
    if n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        format!("{n}")
    }
}

impl Error {
    /// Stable identifier the frontend looks up in its translation tables.
    pub fn code(&self) -> &'static str {
        match self {
            Error::PluginNotFound(_) => "error.pluginNotFound",
            Error::PluginLoad(_) => "error.pluginLoad",
            Error::SaveMissing => "error.saveMissing",
            Error::SaveRead(_) => "error.saveRead",
            Error::SaveFormat { .. } => "error.saveFormat",
            Error::Validation { .. } => "error.validation",
            Error::FieldRule { .. } => "error.fieldRule",
            Error::UnknownField(_) => "error.unknownField",
            Error::PathNotAllowed => "error.pathNotAllowed",
            Error::ListNotEditable { .. } => "error.listNotEditable",
            Error::ListFull { .. } => "error.listFull",
            Error::ListAtMinimum { .. } => "error.listAtMinimum",
            Error::SaveChangedOnDisk => "error.saveChangedOnDisk",
            Error::Constraint { .. } => "error.constraint",
            Error::NeedsConfirmation { .. } => "error.needsConfirmation",
            Error::BackupFailed(_) => "error.backupFailed",
            Error::BackupNotFound(_) => "error.backupNotFound",
            Error::WriteFailed(_) => "error.writeFailed",
            Error::Io(_) => "error.io",
        }
    }

    /// Values to substitute into the translated message.
    pub fn params(&self) -> serde_json::Value {
        match self {
            Error::PluginNotFound(id) | Error::BackupNotFound(id) => json!({ "id": id }),
            Error::PluginLoad(detail)
            | Error::SaveRead(detail)
            | Error::BackupFailed(detail)
            | Error::WriteFailed(detail)
            | Error::Io(detail) => json!({ "detail": detail }),
            Error::SaveFormat { game } => json!({ "game": game }),
            Error::Validation { field, reason } => json!({ "field": field, "reason": reason }),
            Error::UnknownField(field) => json!({ "field": field }),
            Error::FieldRule { field, reason } => {
                let mut params = reason.params();
                params["field"] = json!(field);
                params["rule"] = json!(reason.code());
                params
            }
            Error::Constraint {
                left,
                right,
                message,
            } => {
                json!({ "left": left, "right": right, "message": message })
            }
            Error::NeedsConfirmation { warnings } => json!({ "warnings": warnings }),
            Error::ListNotEditable { list } => json!({ "list": list }),
            Error::ListFull { list, max } => json!({ "list": list, "max": max }),
            Error::ListAtMinimum { list, min } => json!({ "list": list, "min": min }),
            Error::SaveMissing | Error::PathNotAllowed | Error::SaveChangedOnDisk => json!({}),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

/// Sent to the frontend as `{ code, message, params }`.
impl Serialize for Error {
    fn serialize<S: Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        let mut st = s.serialize_struct("AppError", 3)?;
        st.serialize_field("code", self.code())?;
        st.serialize_field("message", &self.to_string())?;
        st.serialize_field("params", &self.params())?;
        st.end()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_with_a_code_and_params() {
        let e = Error::SaveFormat {
            game: "Pathogenic".into(),
        };
        let v: serde_json::Value = serde_json::to_value(&e).unwrap();
        assert_eq!(v["code"], "error.saveFormat");
        assert_eq!(v["params"]["game"], "Pathogenic");
        assert!(v["message"].as_str().unwrap().contains("Pathogenic"));
    }

    #[test]
    fn field_rules_carry_their_limit() {
        let e = Error::FieldRule {
            field: "Money".into(),
            reason: Rule::TooSmall(0.0),
        };
        let v: serde_json::Value = serde_json::to_value(&e).unwrap();
        assert_eq!(v["code"], "error.fieldRule");
        assert_eq!(v["params"]["rule"], "rule.tooSmall");
        assert_eq!(v["params"]["limit"], "0");
        assert_eq!(v["params"]["field"], "Money");
    }

    #[test]
    fn the_english_message_is_still_readable_on_its_own() {
        let e = Error::FieldRule {
            field: "Name".into(),
            reason: Rule::TooLong(8),
        };
        assert_eq!(e.to_string(), "'Name' must be at most 8 characters.");
    }

    #[test]
    fn every_variant_has_a_distinct_code() {
        let codes = [
            Error::PluginNotFound("x".into()).code(),
            Error::PluginLoad("x".into()).code(),
            Error::SaveMissing.code(),
            Error::PathNotAllowed.code(),
            Error::SaveChangedOnDisk.code(),
            Error::Constraint {
                left: "a".into(),
                right: "b".into(),
                message: "m".into(),
            }
            .code(),
            Error::NeedsConfirmation { warnings: vec![] }.code(),
            Error::ListNotEditable { list: "l".into() }.code(),
            Error::ListFull {
                list: "l".into(),
                max: 1,
            }
            .code(),
            Error::ListAtMinimum {
                list: "l".into(),
                min: 1,
            }
            .code(),
            Error::SaveRead("x".into()).code(),
            Error::SaveFormat { game: "x".into() }.code(),
            Error::Validation {
                field: "x".into(),
                reason: "y".into(),
            }
            .code(),
            Error::FieldRule {
                field: "x".into(),
                reason: Rule::NotPresent,
            }
            .code(),
            Error::UnknownField("x".into()).code(),
            Error::BackupFailed("x".into()).code(),
            Error::BackupNotFound("x".into()).code(),
            Error::WriteFailed("x".into()).code(),
            Error::Io("x".into()).code(),
        ];
        let unique: std::collections::HashSet<_> = codes.iter().collect();
        assert_eq!(unique.len(), codes.len());
    }
}
