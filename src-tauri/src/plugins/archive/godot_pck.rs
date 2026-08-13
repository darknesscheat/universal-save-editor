//! Reading a Godot `.pck` archive, and pulling item artwork out of it.
//!
//! Layout, worked out by inspection because the obvious guess is wrong:
//!
//! ```text
//! 0    "GDPC"
//! 4    pack format version
//! 8    Godot major / minor / patch
//! 20   flags        (bit 1 = offsets are relative to file_base)
//! 24   file_base    u64
//! 32   dir_offset   u32   <- the index lives near the END, after the data
//! 36   reserved, zeroed
//! …    file data
//! dir  u32 count, then per entry:
//!        u32 path length, path, u64 offset, u64 size, 16-byte MD5, u32 flags
//! ```
//!
//! Everything here only ever **reads** the player's own installed game. No
//! artwork is copied into this repository: the same rule the store-artwork
//! feature follows.

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

/// An opened archive: the index in memory, the data still on disk.
pub struct Pck {
    file: File,
    entries: HashMap<String, (u64, u64)>,
}

impl Pck {
    pub fn open(path: &Path) -> Option<Self> {
        let mut file = File::open(path).ok()?;

        let mut head = [0u8; 36];
        file.read_exact(&mut head).ok()?;
        if &head[0..4] != b"GDPC" {
            return None;
        }

        let pack_ver = u32::from_le_bytes(head[4..8].try_into().ok()?);
        let flags = u32::from_le_bytes(head[20..24].try_into().ok()?);
        let file_base = u64::from_le_bytes(head[24..32].try_into().ok()?);
        let dir_offset = u32::from_le_bytes(head[32..36].try_into().ok()?) as u64;
        let relative = flags & 2 != 0;

        let size = file.metadata().ok()?.len();
        if dir_offset == 0 || dir_offset >= size {
            return None;
        }

        // The whole index at once: 12k entries is under a megabyte, and
        // seeking per entry across a 1.4 GB file would be far slower.
        file.seek(SeekFrom::Start(dir_offset)).ok()?;
        let mut index = Vec::new();
        file.read_to_end(&mut index).ok()?;

        let mut cursor = 0usize;
        let count = read_u32(&index, &mut cursor)? as usize;
        // A corrupt header could claim billions of entries.
        if count > 1_000_000 {
            return None;
        }

        let mut entries = HashMap::with_capacity(count);
        for _ in 0..count {
            let len = read_u32(&index, &mut cursor)? as usize;
            if len == 0 || len > 4096 || cursor + len > index.len() {
                return None;
            }
            let raw = &index[cursor..cursor + len];
            cursor += len;

            let path = String::from_utf8_lossy(raw)
                .trim_end_matches('\0')
                .to_string();
            let mut offset = read_u64(&index, &mut cursor)?;
            let size = read_u64(&index, &mut cursor)?;
            cursor += 16; // md5
            if pack_ver >= 2 {
                cursor += 4; // per-file flags
            }
            if cursor > index.len() {
                return None;
            }
            if relative {
                offset += file_base;
            }
            entries.insert(path, (offset, size));
        }

        Some(Pck { file, entries })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn has(&self, path: &str) -> bool {
        self.entries.contains_key(path)
    }

    pub fn read(&mut self, path: &str) -> Option<Vec<u8>> {
        let &(offset, size) = self.entries.get(path)?;
        // A single asset that large is not something we want in memory.
        if size > 32 * 1024 * 1024 {
            return None;
        }
        self.file.seek(SeekFrom::Start(offset)).ok()?;
        let mut buf = vec![0u8; size as usize];
        self.file.read_exact(&mut buf).ok()?;
        Some(buf)
    }

    /// Every path in the archive, for building lookup tables.
    pub fn paths(&self) -> impl Iterator<Item = &str> {
        self.entries.keys().map(|s| s.as_str())
    }
}

fn read_u32(buf: &[u8], at: &mut usize) -> Option<u32> {
    let v = u32::from_le_bytes(buf.get(*at..*at + 4)?.try_into().ok()?);
    *at += 4;
    Some(v)
}

fn read_u64(buf: &[u8], at: &mut usize) -> Option<u64> {
    let v = u64::from_le_bytes(buf.get(*at..*at + 8)?.try_into().ok()?);
    *at += 8;
    Some(v)
}

/// Follow a resource to the picture it uses.
///
/// Names are no help here: Pathogenic's `assault_rifle` draws
/// `Player weapon - 3_shot_burst.png` and `cannon` draws `Player weapon 2.png`.
/// Matching on filenames covered barely a third of the parts; reading what the
/// resource actually points at covers all of them.
///
/// The chain is `foo.tres` → `foo.tres.remap` → an exported `.res` → the
/// texture path inside it.
pub fn texture_for_resource(pck: &mut Pck, resource_path: &str) -> Option<String> {
    let source = if pck.has(&format!("{resource_path}.remap")) {
        let remap = pck.read(&format!("{resource_path}.remap"))?;
        let text = String::from_utf8_lossy(&remap);
        let quoted = text.split("path=\"").nth(1)?;
        let path = quoted.split('"').next()?;
        path.trim_start_matches("res://").to_string()
    } else if pck.has(resource_path) {
        resource_path.to_string()
    } else {
        return None;
    };

    let blob = pck.read(&source)?;
    first_image_reference(&blob)
}

/// The first `res://…` string in a binary resource that names an image.
fn first_image_reference(blob: &[u8]) -> Option<String> {
    const NEEDLE: &[u8] = b"res://";
    let mut i = 0;

    while i + NEEDLE.len() < blob.len() {
        if &blob[i..i + NEEDLE.len()] != NEEDLE {
            i += 1;
            continue;
        }
        let start = i;
        let mut end = start;
        while end < blob.len() && (0x20..0x7f).contains(&blob[end]) {
            end += 1;
        }
        let text = String::from_utf8_lossy(&blob[start..end]).to_string();
        let lower = text.to_ascii_lowercase();
        if lower.ends_with(".png")
            || lower.ends_with(".webp")
            || lower.ends_with(".jpg")
            || lower.ends_with(".jpeg")
        {
            return Some(text.trim_start_matches("res://").to_string());
        }
        i = end.max(start + 1);
    }
    None
}

/// Godot imports every image into a `.ctex`; this maps original name -> that file.
///
/// The imported name is `<original>-<md5>.ctex`, sometimes with a compression
/// tag before the extension.
pub fn imported_texture_index(pck: &Pck) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for path in pck.paths() {
        if !path.ends_with(".ctex") {
            continue;
        }
        let Some(file_name) = path.rsplit('/').next() else {
            continue;
        };
        if let Some(original) = strip_import_suffix(file_name) {
            map.entry(original).or_insert_with(|| path.to_string());
        }
    }
    map
}

/// `Player weapon - gun.png-<32 hex>.bptc.ctex` -> `Player weapon - gun.png`
fn strip_import_suffix(file_name: &str) -> Option<String> {
    let stem = file_name.strip_suffix(".ctex")?;
    // Drop an optional compression tag such as `.bptc` or `.s3tc`.
    let stem = match stem.rfind('.') {
        Some(dot) if stem[dot + 1..].chars().all(|c| c.is_ascii_alphanumeric()) => &stem[..dot],
        _ => stem,
    };
    // Then the `-<md5>` the importer appends.
    let dash = stem.rfind('-')?;
    let hash = &stem[dash + 1..];
    if hash.len() == 32 && hash.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(stem[..dash].to_string())
    } else {
        None
    }
}

/// The image inside a `CompressedTexture2D`, ready to hand to a browser.
///
/// A `.ctex` is a `GST2` header wrapped around the imported image. For a
/// lossless or lossy import that payload is a WebP or a PNG, which the webview
/// renders directly. Block-compressed formats (BC7 and friends) are raw GPU
/// data and would need a decoder, so those are declined rather than guessed at.
pub fn image_in_ctex(blob: &[u8]) -> Option<(&'static str, Vec<u8>)> {
    if blob.len() < 32 || &blob[0..4] != b"GST2" {
        return None;
    }

    if let Some(i) = find(blob, b"RIFF") {
        if blob.get(i + 8..i + 12) == Some(b"WEBP") {
            let size = u32::from_le_bytes(blob.get(i + 4..i + 8)?.try_into().ok()?) as usize + 8;
            let end = (i + size).min(blob.len());
            return Some(("image/webp", blob[i..end].to_vec()));
        }
    }
    if let Some(i) = find(blob, b"\x89PNG\r\n\x1a\n") {
        return Some(("image/png", blob[i..].to_vec()));
    }
    None
}

fn find(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn import_suffixes_are_stripped() {
        assert_eq!(
            strip_import_suffix("gun.png-e43191b0084bad51fa5a9caea1b0fe7a.ctex").as_deref(),
            Some("gun.png")
        );
        assert_eq!(
            strip_import_suffix("1-1.png-fc8183987e81ddcf034eaaf18b3a7c71.bptc.ctex").as_deref(),
            Some("1-1.png")
        );
        assert_eq!(
            strip_import_suffix("Player weapon - gun.png-fc8183987e81ddcf034eaaf18b3a7c71.ctex")
                .as_deref(),
            Some("Player weapon - gun.png")
        );
    }

    #[test]
    fn a_name_without_a_hash_is_not_an_import() {
        assert_eq!(strip_import_suffix("plain.ctex"), None);
        assert_eq!(strip_import_suffix("thing-notahash.ctex"), None);
        assert_eq!(strip_import_suffix("gun.png"), None);
    }

    #[test]
    fn the_first_image_reference_wins_and_scripts_are_ignored() {
        let blob = b"RSRC\x00\x00res://scn/thing.gd\x00res://gfx/parts/Player weapon - gun.png\x00";
        assert_eq!(
            first_image_reference(blob).as_deref(),
            Some("gfx/parts/Player weapon - gun.png")
        );
    }

    #[test]
    fn a_resource_with_no_picture_returns_nothing() {
        let blob = b"RSRC\x00res://scn/thing.gd\x00res://scn/other.tres\x00";
        assert_eq!(first_image_reference(blob), None);
    }

    #[test]
    fn a_ctex_wrapping_a_png_gives_the_png_back() {
        let png = b"\x89PNG\r\n\x1a\n_body_here";
        let mut blob = b"GST2".to_vec();
        blob.extend_from_slice(&[0u8; 28]);
        blob.extend_from_slice(png);

        let (mime, data) = image_in_ctex(&blob).unwrap();
        assert_eq!(mime, "image/png");
        assert_eq!(&data, png);
    }

    #[test]
    fn a_ctex_wrapping_a_webp_gives_the_webp_back() {
        let mut webp = b"RIFF".to_vec();
        webp.extend_from_slice(&8u32.to_le_bytes()); // payload length
        webp.extend_from_slice(b"WEBP1234");

        let mut blob = b"GST2".to_vec();
        blob.extend_from_slice(&[0u8; 28]);
        blob.extend_from_slice(&webp);

        let (mime, data) = image_in_ctex(&blob).unwrap();
        assert_eq!(mime, "image/webp");
        assert_eq!(data.len(), 16);
    }

    /// Block-compressed textures are raw GPU data; decoding them is a separate
    /// project, so they are declined rather than mangled.
    #[test]
    fn a_ctex_with_no_recognised_payload_is_declined() {
        let mut blob = b"GST2".to_vec();
        blob.extend_from_slice(&[0u8; 64]);
        assert!(image_in_ctex(&blob).is_none());
    }

    #[test]
    fn something_that_is_not_a_ctex_is_declined() {
        assert!(image_in_ctex(b"not a texture at all, really not").is_none());
        assert!(image_in_ctex(b"GST2").is_none());
    }

    #[test]
    fn a_missing_archive_is_not_a_crash() {
        assert!(Pck::open(Path::new("/definitely/not/here.pck")).is_none());
    }

    #[test]
    fn a_file_that_is_not_a_pck_is_declined() {
        let tmp = tempfile::tempdir().unwrap();
        let fake = tmp.path().join("fake.pck");
        std::fs::write(&fake, b"this is not a Godot archive").unwrap();
        assert!(Pck::open(&fake).is_none());
    }
}
