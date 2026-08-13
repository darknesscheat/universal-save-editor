use crate::core::error::{Error, Result};
use std::path::{Path, PathBuf};

/// Platform-independent placeholders a plugin manifest may use in a save path.
///
/// Manifests never contain a literal `C:\Users\...`; they say
/// `{APPDATA}/Godot/app_userdata/Pathogenic` and this module resolves it on
/// whatever machine the app happens to be running on.
///
/// | Placeholder  | Windows                      | Linux                     | macOS                              |
/// |--------------|------------------------------|---------------------------|------------------------------------|
/// | `{HOME}`     | `C:\Users\me`                | `/home/me`                | `/Users/me`                        |
/// | `{APPDATA}`  | `…\AppData\Roaming`          | `~/.local/share`          | `~/Library/Application Support`    |
/// | `{LOCALAPPDATA}` | `…\AppData\Local`        | `~/.local/share`          | `~/Library/Application Support`    |
/// | `{DOCUMENTS}`| `…\Documents`                | `~/Documents`             | `~/Documents`                      |
/// | `{CONFIG}`   | `…\AppData\Roaming`          | `~/.config`               | `~/Library/Application Support`    |
pub fn expand(template: &str) -> Result<PathBuf> {
    let home =
        dirs::home_dir().ok_or_else(|| Error::Io("Home folder could not be located.".into()))?;

    let appdata = dirs::data_dir().unwrap_or_else(|| home.join(".local/share"));
    let local = dirs::data_local_dir().unwrap_or_else(|| appdata.clone());
    let documents = dirs::document_dir().unwrap_or_else(|| home.join("Documents"));
    let config = dirs::config_dir().unwrap_or_else(|| appdata.clone());

    let mut out = template.to_string();
    for (key, value) in [
        ("{HOME}", home.clone()),
        ("{APPDATA}", appdata),
        ("{LOCALAPPDATA}", local),
        ("{DOCUMENTS}", documents),
        ("{CONFIG}", config),
        ("{PROGRAMFILES}", program_files(false)),
        ("{PROGRAMFILES_X86}", program_files(true)),
        ("{STEAM}", steam_root(&home)),
    ] {
        if out.contains(key) {
            out = out.replace(key, &value.to_string_lossy());
        }
    }

    if let Some(start) = out.find('{') {
        let end = out[start..]
            .find('}')
            .map(|e| start + e + 1)
            .unwrap_or(out.len());
        return Err(Error::PluginLoad(format!(
            "unknown path placeholder '{}'",
            &out[start..end]
        )));
    }

    // Normalise separators so a manifest can always use '/'.
    Ok(PathBuf::from(
        out.replace('/', std::path::MAIN_SEPARATOR_STR),
    ))
}

fn program_files(x86: bool) -> PathBuf {
    let var = if x86 {
        "ProgramFiles(x86)"
    } else {
        "ProgramFiles"
    };
    std::env::var_os(var).map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(if x86 {
            "C:\\Program Files (x86)"
        } else {
            "C:\\Program Files"
        })
    })
}

/// Where Steam is installed.
///
/// `STEAM_PATH` wins if it is set; otherwise we probe the usual places for the
/// platform. If none of them exist we still return the most likely one, a
/// caller only ever uses this to look for a file, and a miss simply means the
/// feature that wanted it (a game icon, say) falls back gracefully.
fn steam_root(home: &Path) -> PathBuf {
    if let Some(explicit) = std::env::var_os("STEAM_PATH") {
        return PathBuf::from(explicit);
    }

    let candidates: Vec<PathBuf> = if cfg!(target_os = "windows") {
        vec![
            program_files(true).join("Steam"),
            program_files(false).join("Steam"),
            PathBuf::from("C:\\Steam"),
        ]
    } else if cfg!(target_os = "macos") {
        vec![home.join("Library/Application Support/Steam")]
    } else {
        vec![
            home.join(".local/share/Steam"),
            home.join(".steam/steam"),
            home.join(".var/app/com.valvesoftware.Steam/.local/share/Steam"),
        ]
    };

    candidates
        .iter()
        .find(|p| p.is_dir())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone())
}

/// The current platform as manifests spell it.
pub fn current_platform() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expands_home_placeholder() {
        let p = expand("{HOME}/games").unwrap();
        assert!(p.ends_with("games"));
        assert!(!p.to_string_lossy().contains('{'));
    }

    #[test]
    fn rejects_unknown_placeholder() {
        let err = expand("{NOPE}/x").unwrap_err();
        assert!(err.to_string().contains("{NOPE}"));
    }

    #[test]
    fn platform_is_one_of_the_known_three() {
        assert!(["windows", "macos", "linux"].contains(&current_platform()));
    }
}
