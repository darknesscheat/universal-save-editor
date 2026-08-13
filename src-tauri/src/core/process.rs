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

/// Pull the executable name out of the bytes of `/proc/<pid>/cmdline`.
///
/// The file is the argument vector with NUL separators, so the first entry is
/// the command as it was invoked, sometimes as a path. Kernel threads have an
/// empty cmdline, which is what `None` means here.
#[cfg(not(target_os = "windows"))]
fn name_from_cmdline(raw: &[u8]) -> Option<String> {
    let argv0 = raw.split(|b| *b == 0).next()?;
    let text = std::str::from_utf8(argv0).ok()?.trim();
    let base = text.rsplit('/').next().unwrap_or(text);
    (!base.is_empty()).then(|| base.to_string())
}

#[cfg(not(target_os = "windows"))]
fn list_processes() -> Vec<String> {
    // Linux keeps this under /proc. macOS has no /proc, so it falls through to
    // `ps` below.
    //
    // `cmdline` rather than the more obvious `comm`, because the kernel
    // truncates `comm` to 15 characters. That is long enough to look correct in
    // testing and short enough to silently break any game whose executable has
    // a longer name.
    if let Ok(entries) = std::fs::read_dir("/proc") {
        let names: Vec<String> = entries
            .flatten()
            .filter(|e| {
                e.file_name()
                    .to_str()
                    .map(|n| n.chars().all(|c| c.is_ascii_digit()))
                    .unwrap_or(false)
            })
            .filter_map(|e| {
                let dir = e.path();
                let from_cmdline = std::fs::read(dir.join("cmdline"))
                    .ok()
                    .and_then(|raw| name_from_cmdline(&raw));
                match from_cmdline {
                    Some(name) => Some(name),
                    None => std::fs::read_to_string(dir.join("comm"))
                        .ok()
                        .map(|n| n.trim().to_string()),
                }
            })
            .map(|n| normalise(&n))
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

    /// Regression for a CI failure that only appeared on Linux: names came from
    /// `/proc/<pid>/comm`, which the kernel truncates to 15 characters, so a
    /// long executable name never matched what a plugin declared.
    #[cfg(not(target_os = "windows"))]
    #[test]
    fn a_long_name_survives_being_read_from_cmdline() {
        let long = "universal-save-editor-with-a-very-long-name";
        assert!(long.len() > 15);

        let raw = format!("/usr/bin/{long}\0--flag\0");
        assert_eq!(name_from_cmdline(raw.as_bytes()).as_deref(), Some(long));
    }

    #[cfg(not(target_os = "windows"))]
    #[test]
    fn an_empty_cmdline_has_no_name() {
        // Kernel threads look like this; the caller falls back to `comm`.
        assert_eq!(name_from_cmdline(b""), None);
        assert_eq!(name_from_cmdline(b"\0\0"), None);
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
