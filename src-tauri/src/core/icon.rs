use crate::core::paths;
use crate::plugins::manifest::Manifest;
use std::path::Path;

/// Nobody wants a 10 MB key art file inlined into a list of buttons, and a
/// runaway glob should not be able to blow up the frontend payload.
const MAX_ICON_BYTES: u64 = 512 * 1024;

/// Find a picture to show for a game, and return it as a `data:` URI.
///
/// Two sources, in order:
///
/// 1. `icon`: a file inside the plugin folder. For plugin authors who have
///    art they are allowed to redistribute.
/// 2. `icon_sources`, globs pointing at files **already on this computer**,
///    such as the picture Steam caches for a game you own.
///
/// The second exists so the app can show real game artwork without shipping
/// any: game art is copyrighted, and bundling it into an MIT-licensed
/// repository would not be ours to do. Reading a file the player already has
/// is a different thing entirely.
///
/// `None` simply means the GUI draws its generated tile instead, which is a
/// perfectly good outcome rather than an error.
pub fn resolve(manifest: &Manifest, plugin_dir: &Path) -> Option<String> {
    if let Some(rel) = &manifest.icon {
        // Confined to the plugin folder: a manifest must not be able to point
        // `icon` at an arbitrary file elsewhere on disk.
        let candidate = plugin_dir.join(rel);
        if is_inside(plugin_dir, &candidate) {
            if let Some(uri) = encode(&candidate) {
                return Some(uri);
            }
        }
    }

    let platform = paths::current_platform();
    for source in &manifest.icon_sources {
        if !source.platforms.is_empty() && !source.platforms.iter().any(|p| p == platform) {
            continue;
        }
        let Ok(pattern) = paths::expand(&source.path) else {
            continue;
        };
        let Ok(hits) = glob::glob(&pattern.to_string_lossy()) else {
            continue;
        };

        // Largest match wins: Steam keeps several sizes side by side and the
        // biggest is the one worth showing.
        let best = hits
            .flatten()
            .filter(|p| p.is_file())
            .filter_map(|p| std::fs::metadata(&p).ok().map(|m| (m.len(), p)))
            .filter(|(len, _)| *len <= MAX_ICON_BYTES)
            .max_by_key(|(len, _)| *len)
            .map(|(_, p)| p);

        if let Some(uri) = best.as_deref().and_then(encode) {
            return Some(uri);
        }
    }

    None
}

fn is_inside(root: &Path, candidate: &Path) -> bool {
    let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
    let candidate = candidate
        .canonicalize()
        .unwrap_or_else(|_| candidate.to_path_buf());
    candidate.starts_with(root)
}

fn encode(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() == 0 || meta.len() > MAX_ICON_BYTES {
        return None;
    }
    let mime = mime_for(path)?;
    let bytes = std::fs::read(path).ok()?;
    Some(format!("data:{mime};base64,{}", base64(&bytes)))
}

/// Only formats a browser will render inline, and only ones we can name
/// confidently from the extension.
fn mime_for(path: &Path) -> Option<&'static str> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Some(match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "ico" => "image/x-icon",
        _ => return None,
    })
}

/// Standard base64. Hand-rolled to avoid a dependency for thirty lines.
pub fn base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);

    for chunk in bytes.chunks(3) {
        let b = [
            chunk[0],
            *chunk.get(1).unwrap_or(&0),
            *chunk.get(2).unwrap_or(&0),
        ];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;

        out.push(TABLE[(n >> 18 & 63) as usize] as char);
        out.push(TABLE[(n >> 12 & 63) as usize] as char);
        out.push(if chunk.len() > 1 {
            TABLE[(n >> 6 & 63) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            TABLE[(n & 63) as usize] as char
        } else {
            '='
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(extra: &str) -> Manifest {
        serde_json::from_str(&format!(
            r#"{{"id":"t","name":"T","version":"1","format":"json",
               "save_locations":[{{"root":"{{HOME}}/t","pattern":"*.json"}}],
               "groups":[{{"id":"g","label":"G","fields":[
                 {{"id":"a","label":"A","pointer":"/a","type":"integer"}}]}}]
               {extra}}}"#
        ))
        .unwrap()
    }

    /// A 1x1 PNG, so the bytes are a real image rather than arbitrary data.
    const PNG: &[u8] = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F,
        0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];

    #[test]
    fn base64_matches_known_vectors() {
        assert_eq!(base64(b""), "");
        assert_eq!(base64(b"f"), "Zg==");
        assert_eq!(base64(b"fo"), "Zm8=");
        assert_eq!(base64(b"foo"), "Zm9v");
        assert_eq!(base64(b"foob"), "Zm9vYg==");
        assert_eq!(base64(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn base64_handles_bytes_above_127() {
        assert_eq!(base64(&[0xFF, 0xFE, 0xFD]), "//79");
    }

    #[test]
    fn reads_a_bundled_icon() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("icon.png"), PNG).unwrap();

        let uri = resolve(&manifest(r#","icon":"icon.png""#), tmp.path()).unwrap();
        assert!(uri.starts_with("data:image/png;base64,"));
        assert!(uri.len() > 40);
    }

    #[test]
    fn a_missing_icon_is_not_an_error() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(resolve(&manifest(r#","icon":"nope.png""#), tmp.path()).is_none());
    }

    #[test]
    fn no_icon_declared_means_no_icon() {
        let tmp = tempfile::tempdir().unwrap();
        assert!(resolve(&manifest(""), tmp.path()).is_none());
    }

    #[test]
    fn a_bundled_icon_cannot_escape_the_plugin_folder() {
        let tmp = tempfile::tempdir().unwrap();
        let plugin = tmp.path().join("plugin");
        std::fs::create_dir(&plugin).unwrap();
        std::fs::write(tmp.path().join("secret.png"), PNG).unwrap();

        let m = manifest(r#","icon":"../secret.png""#);
        assert!(resolve(&m, &plugin).is_none(), "escaped the plugin folder");
    }

    #[test]
    fn oversized_files_are_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("big.png"),
            vec![0u8; (MAX_ICON_BYTES + 1) as usize],
        )
        .unwrap();
        assert!(resolve(&manifest(r#","icon":"big.png""#), tmp.path()).is_none());
    }

    #[test]
    fn unrenderable_formats_are_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("icon.exe"), PNG).unwrap();
        assert!(resolve(&manifest(r#","icon":"icon.exe""#), tmp.path()).is_none());
    }

    #[test]
    fn an_empty_file_is_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("icon.png"), b"").unwrap();
        assert!(resolve(&manifest(r#","icon":"icon.png""#), tmp.path()).is_none());
    }

    /// `STEAM_PATH` is process-wide, and `cargo test` runs tests in threads, so
    /// two tests setting it at once will clobber each other. This lock is what
    /// keeps that from happening, it is not a detail of the code under test.
    static STEAM_ENV: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn with_steam_path<T>(path: &std::path::Path, f: impl FnOnce() -> T) -> T {
        // A panic in an earlier test poisons the lock but leaves it usable.
        let _guard = STEAM_ENV.lock().unwrap_or_else(|e| e.into_inner());
        std::env::set_var("STEAM_PATH", path);
        let out = f();
        std::env::remove_var("STEAM_PATH");
        out
    }

    #[test]
    fn icon_sources_pick_the_largest_match() {
        let tmp = tempfile::tempdir().unwrap();
        let art = tmp.path().join("art");
        std::fs::create_dir(&art).unwrap();
        std::fs::write(art.join("small.png"), PNG).unwrap();

        let mut bigger = PNG.to_vec();
        bigger.extend_from_slice(&[0u8; 200]);
        std::fs::write(art.join("large.png"), &bigger).unwrap();

        let m = manifest(r#","icon_sources":[{"path":"{STEAM}/art/*.png"}]"#);
        let uri = with_steam_path(tmp.path(), || resolve(&m, tmp.path())).unwrap();

        assert!(uri.starts_with("data:image/png;base64,"));
        assert!(
            uri.contains(&base64(&bigger)),
            "did not pick the larger file"
        );
    }

    #[test]
    fn icon_sources_for_another_platform_are_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("icon.png"), PNG).unwrap();

        let other = if cfg!(target_os = "windows") {
            "linux"
        } else {
            "windows"
        };
        let m = manifest(&format!(
            r#","icon_sources":[{{"platforms":["{other}"],"path":"{{STEAM}}/*.png"}}]"#
        ));
        let got = with_steam_path(tmp.path(), || resolve(&m, tmp.path()));

        assert!(got.is_none());
    }
}
