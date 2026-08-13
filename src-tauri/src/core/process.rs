//! Is the game running right now?
//!
//! Editing a save while its game is open is usually pointless and sometimes
//! destructive: many games hold their state in memory and write the whole file
//! out when they exit, silently discarding whatever was changed underneath
//! them. That happened during this app's own development, which is why the
//! warning exists.
//!
//! This only ever *reads* the process list, and only to compare names a plugin
//! declared. Nothing is started, stopped or inspected further.

/// Which of `names` are currently running.
///
/// Comparison ignores case and any `.exe` suffix, so a manifest can say
/// `"pathogenic"` and match `Pathogenic.exe`.
///
/// A platform we cannot query returns an empty list: the warning is a courtesy,
/// and failing to show it must never block editing.
pub fn running_among(names: &[String]) -> Vec<String> {
    if names.is_empty() {
        return Vec::new();
    }
    let running = list_processes();
    names
        .iter()
        .filter(|wanted| {
            let wanted = normalise(wanted);
            running.contains(&wanted)
        })
        .cloned()
        .collect()
}

fn normalise(name: &str) -> String {
    name.trim()
        .trim_end_matches(".exe")
        .trim_end_matches(".EXE")
        .to_ascii_lowercase()
}

#[cfg(target_os = "windows")]
fn list_processes() -> Vec<String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    // `tasklist` ships with Windows, so this needs no crate and no unsafe FFI.
    // CSV output because the default table truncates long names.
    let out = std::process::Command::new("tasklist")
        .args(["/FO", "CSV", "/NH"])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    let Ok(out) = out else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| line.split('"').nth(1))
        .map(normalise)
        .collect()
}

#[cfg(not(target_os = "windows"))]
fn list_processes() -> Vec<String> {
    // /proc/<pid>/comm holds the executable name on Linux. macOS has no /proc,
    // so fall back to `ps`.
    if let Ok(entries) = std::fs::read_dir("/proc") {
        let names: Vec<String> = entries
            .flatten()
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or(false)
            })
            .filter_map(|e| std::fs::read_to_string(e.path().join("comm")).ok())
            .map(|n| normalise(n.trim()))
            .collect();
        if !names.is_empty() {
            return names;
        }
    }

    let Ok(out) = std::process::Command::new("ps")
        .args(["-A", "-o", "comm="])
        .output()
    else {
        return Vec::new();
    };
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(|l| {
            // `ps` prints full paths on macOS; we only want the file name.
            let base = l.rsplit('/').next().unwrap_or(l);
            normalise(base)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_names_means_no_work() {
        assert!(running_among(&[]).is_empty());
    }

    #[test]
    fn a_game_that_is_not_running_is_not_reported() {
        let names = vec!["definitely-not-a-real-process-x9f2".to_string()];
        assert!(running_among(&names).is_empty());
    }

    #[test]
    fn normalise_ignores_case_and_extension() {
        assert_eq!(normalise("Pathogenic.exe"), "pathogenic");
        assert_eq!(normalise("  OAKENTOWER.EXE "), "oakentower");
        assert_eq!(normalise("ULTRAKILL"), "ultrakill");
    }

    /// The current test binary is by definition running, so it makes a
    /// dependable positive case on any platform.
    ///
    /// Queried once and reasoned about, rather than asking the operating
    /// system twice and assuming both answers agree, process lists move.
    #[test]
    fn a_running_process_is_found() {
        let running = list_processes();
        if running.is_empty() {
            // A platform we cannot query. Reporting nothing is the documented
            // outcome, so there is nothing to assert.
            return;
        }

        let me = std::env::current_exe().unwrap();
        let name = me.file_name().unwrap().to_string_lossy().into_owned();
        assert!(
            running.contains(&normalise(&name)),
            "the test binary {name} was not in the process list"
        );
    }
}
