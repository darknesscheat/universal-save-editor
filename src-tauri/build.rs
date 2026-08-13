use std::path::{Path, PathBuf};

fn main() {
    mirror_plugins();
    tauri_build::build()
}

/// Copy `plugins/` next to the executable being built.
///
/// `tauri.conf.json` already declares `"resources": ["../plugins"]`, but that
/// only takes effect when the *bundler* runs. Anyone who runs the binary
/// straight out of `target/<profile>/`, during development, or after a bundle
/// step failed part way, gets whatever happens to be sitting in that folder
/// already, which may be several versions old.
///
/// That failure is silent and convincing: the app starts, lists some games, and
/// simply omits the plugin you just wrote. Doing the copy here means the
/// binary and its plugins are always built together.
fn mirror_plugins() {
    let Some(source) = manifest_dir().parent().map(|p| p.join("plugins")) else {
        return;
    };
    if !source.is_dir() {
        return;
    }
    println!("cargo:rerun-if-changed={}", source.display());

    let Some(target) = profile_dir() else { return };
    let dest = target.join("plugins");

    // Remove first: a plugin deleted from the repo must not linger here, or it
    // keeps loading and the developer keeps wondering why.
    let _ = std::fs::remove_dir_all(&dest);
    if let Err(e) = copy_dir(&source, &dest) {
        // A build should not fail over this. Warn loudly instead: the binary
        // still works, it just falls back to the folders it finds at runtime.
        println!(
            "cargo:warning=could not copy plugins to {}: {e}",
            dest.display()
        );
    }
}

fn manifest_dir() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets this"))
}

/// `target/<profile>/`, derived from `OUT_DIR`.
///
/// Cargo exposes no variable for it, and `target/` may be relocated by
/// `CARGO_TARGET_DIR`, so the supported route is to climb out of `OUT_DIR`:
/// `target/<profile>/build/<pkg>-<hash>/out` → three levels up.
fn profile_dir() -> Option<PathBuf> {
    let out = PathBuf::from(std::env::var("OUT_DIR").ok()?);
    let dir = out.ancestors().nth(3)?.to_path_buf();
    // Only trust it if it looks the way we expect; better no copy than a
    // recursive one into somewhere unrelated.
    dir.join("build").is_dir().then_some(dir)
}

fn copy_dir(from: &Path, to: &Path) -> std::io::Result<()> {
    std::fs::create_dir_all(to)?;
    for entry in std::fs::read_dir(from)? {
        let entry = entry?;
        let target = to.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            std::fs::copy(entry.path(), &target)?;
        }
    }
    Ok(())
}
