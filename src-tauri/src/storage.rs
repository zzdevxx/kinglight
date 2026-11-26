use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::engine::{validate_shape_kind, validate_shape_params};
use crate::model::ShapeParams;

pub const PROFILE_DIGEST: &str = "kinglight-generic-mh8-v1";
pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_SHOW_BYTES: u64 = 10 * 1024 * 1024;
const SHOW_EXTENSION: &str = "kinglight";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredFixture {
    pub slot: u8,
    pub id: String,
    pub label: String,
    pub dimmer: u16,
    pub pan: u16,
    pub tilt: u16,
    pub color: [u16; 3],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredCueFixture {
    pub values: [u16; 6],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredSlot {
    pub dimmer: u16,
    pub pan: u16,
    pub tilt: u16,
    pub color: [u16; 3],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixtures: Option<Vec<StoredCueFixture>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredShape {
    pub kind: String,
    pub params: ShapeParams,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StoredProgrammer {
    pub values: [Option<u16>; 6],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShowDocument {
    pub schema_version: u32,
    pub show_id: String,
    pub name: String,
    pub profile_digest: String,
    pub master: u16,
    pub fixtures: Vec<StoredFixture>,
    pub pages: Vec<Vec<StoredSlot>>,
    pub shape: StoredShape,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub programmer: Option<Vec<StoredProgrammer>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recorded_slots: Option<Vec<Vec<bool>>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub playback_levels: Option<Vec<Vec<u16>>>,
}

pub fn validate_show_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("INVALID_SHOW_NAME: nama show kosong".to_string());
    }
    if trimmed.chars().count() > 80 {
        return Err("INVALID_SHOW_NAME: nama show lebih dari 80 karakter".to_string());
    }
    if trimmed.contains('/')
        || trimmed.contains('\\')
        || trimmed.contains("..")
        || trimmed.starts_with('.')
        || trimmed.contains(':')
        || Path::new(trimmed).is_absolute()
    {
        return Err("PATH_TRAVERSAL: nama show tidak boleh memuat path".to_string());
    }
    if trimmed.chars().any(|character| character.is_control()) {
        return Err("INVALID_SHOW_NAME: karakter kontrol tidak diizinkan".to_string());
    }
    let uppercase = trimmed.to_ascii_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL"]
        .iter()
        .any(|stem| uppercase == *stem || uppercase.starts_with(&format!("{stem}.")))
        || ["COM", "LPT"].iter().any(|prefix| {
            uppercase.starts_with(prefix)
                && uppercase[prefix.len()..]
                    .chars()
                    .all(|character| ('1'..='9').contains(&character))
                && uppercase.len() == prefix.len() + 1
        });
    if reserved {
        return Err("INVALID_SHOW_NAME: nama perangkat Windows tidak diizinkan".to_string());
    }
    if trimmed.to_ascii_lowercase().ends_with(&format!(".{SHOW_EXTENSION}")) {
        Ok(trimmed.to_string())
    } else {
        Ok(format!("{trimmed}.{SHOW_EXTENSION}"))
    }
}

fn show_path(root: &Path, name: &str) -> Result<PathBuf, String> {
    let file_name = validate_show_name(name)?;
    fs::create_dir_all(root)
        .map_err(|error| format!("SHOW_IO_ERROR: direktori data tidak dapat dibuat: {error}"))?;
    let candidate = root.join(&file_name);
    let canonical_root = fs::canonicalize(root)
        .map_err(|error| format!("SHOW_IO_ERROR: direktori data tidak dapat dibaca: {error}"))?;
    let parent = candidate
        .parent()
        .ok_or_else(|| "PATH_TRAVERSAL: show tidak memiliki direktori induk".to_string())?;
    let canonical_parent = fs::canonicalize(parent)
        .map_err(|error| format!("SHOW_IO_ERROR: direktori tujuan tidak dapat dibaca: {error}"))?;
    if canonical_parent != canonical_root {
        return Err("PATH_TRAVERSAL: show harus berada di direktori data aplikasi".to_string());
    }
    Ok(candidate)
}

#[cfg(windows)]
fn replace_file(from: &Path, to: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;

    use windows_sys::Win32::Storage::FileSystem::{MOVEFILE_REPLACE_EXISTING, MoveFileExW};

    let from_wide: Vec<u16> = from
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let to_wide: Vec<u16> = to
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let result = unsafe {
        MoveFileExW(
            from_wide.as_ptr(),
            to_wide.as_ptr(),
            MOVEFILE_REPLACE_EXISTING,
        )
    };
    if result == 0 {
        Err(io::Error::last_os_error())
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn replace_file(from: &Path, to: &Path) -> io::Result<()> {
    fs::rename(from, to)
}

pub fn validate_document(document: &ShowDocument) -> Result<(), String> {
    if document.schema_version != SCHEMA_VERSION {
        return Err("UNSUPPORTED_SCHEMA: schemaVersion bukan 1".to_string());
    }
    if document.profile_digest != PROFILE_DIGEST {
        return Err("PROFILE_DIGEST_MISMATCH: digest profil tidak cocok".to_string());
    }
    if document.show_id.trim().is_empty() || document.name.trim().is_empty() {
        return Err("INVALID_SHOW: identitas show kosong".to_string());
    }
    if document.fixtures.len() != 16 || document.pages.len() != 6 {
        return Err("INVALID_SHOW: struktur show tidak sesuai profil".to_string());
    }
    for (index, fixture) in document.fixtures.iter().enumerate() {
        if fixture.slot as usize != index + 1 {
            return Err("INVALID_SHOW: slot fixture tidak berurutan".to_string());
        }
    }
    for page in &document.pages {
        if page.len() != 16 {
            return Err("INVALID_SHOW: page harus memiliki 16 slot".to_string());
        }
        for slot in page {
            if let Some(fixtures) = &slot.fixtures {
                if fixtures.len() != 16 {
                    return Err("INVALID_SHOW: cue harus memiliki 16 fixture".to_string());
                }
            }
        }
    }
    validate_shape_kind(&document.shape.kind)?;
    validate_shape_params(&document.shape.params)?;
    if let Some(programmer) = &document.programmer {
        if programmer.len() != 16 {
            return Err("INVALID_SHOW: jumlah programmer fixture bukan 16".to_string());
        }
    }
    if let Some(recorded) = &document.recorded_slots {
        if recorded.len() != 6 || recorded.iter().any(|page| page.len() != 16) {
            return Err("INVALID_SHOW: struktur recordedSlots tidak valid".to_string());
        }
    }
    if let Some(levels) = &document.playback_levels {
        if levels.len() != 6 || levels.iter().any(|page| page.len() != 16) {
            return Err("INVALID_SHOW: struktur playbackLevels tidak valid".to_string());
        }
    }
    Ok(())
}

pub fn save_document(root: &Path, name: &str, document: &ShowDocument) -> Result<PathBuf, String> {
    validate_document(document)?;
    let path = show_path(root, name)?;
    let temporary = path.with_extension(format!("{SHOW_EXTENSION}.tmp"));
    let json = serde_json::to_vec_pretty(document)
        .map_err(|error| format!("SHOW_SERIALIZE_ERROR: {error}"))?;

    let write_result = (|| -> Result<(), String> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temporary)
            .map_err(|error| format!("SHOW_IO_ERROR: file sementara gagal dibuka: {error}"))?;
        file.write_all(&json)
            .map_err(|error| format!("SHOW_IO_ERROR: tulis show gagal: {error}"))?;
        file.flush()
            .map_err(|error| format!("SHOW_IO_ERROR: flush show gagal: {error}"))?;
        file.sync_all()
            .map_err(|error| format!("SHOW_IO_ERROR: sync show gagal: {error}"))?;
        replace_file(&temporary, &path)
            .map_err(|error| format!("SHOW_IO_ERROR: replace show gagal: {error}"))
    })();

    if write_result.is_err() && temporary.exists() {
        if let Err(error) = fs::remove_file(&temporary) {
            eprintln!("[kinglight] error=show_temp_cleanup detail={error}");
        }
    }
    write_result?;
    Ok(path)
}

pub fn load_document(root: &Path, name: &str) -> Result<ShowDocument, String> {
    let path = show_path(root, name)?;
    let metadata = fs::metadata(&path)
        .map_err(|error| format!("SHOW_IO_ERROR: show tidak dapat dibaca: {error}"))?;
    if metadata.len() > MAX_SHOW_BYTES {
        return Err("FILE_TOO_LARGE: show melebihi 10 MiB".to_string());
    }
    let bytes = fs::read(&path)
        .map_err(|error| format!("SHOW_IO_ERROR: show tidak dapat dibaca: {error}"))?;
    let document: ShowDocument =
        serde_json::from_slice(&bytes).map_err(|error| format!("INVALID_JSON: {error}"))?;
    validate_document(&document)?;
    Ok(document)
}

pub fn list_documents(root: &Path) -> Result<Vec<String>, String> {
    fs::create_dir_all(root)
        .map_err(|error| format!("SHOW_IO_ERROR: direktori data tidak dapat dibuat: {error}"))?;
    let mut names = Vec::new();
    for entry in fs::read_dir(root)
        .map_err(|error| format!("SHOW_IO_ERROR: direktori data tidak dapat dibaca: {error}"))?
    {
        let entry = entry.map_err(|error| format!("SHOW_IO_ERROR: enturi gagal dibaca: {error}"))?;
        let path = entry.path();
        if !path.is_file() || path.extension().and_then(|value| value.to_str()) != Some(SHOW_EXTENSION)
        {
            continue;
        }
        if let Some(stem) = path.file_stem().and_then(|value| value.to_str()) {
            names.push(stem.to_string());
        }
    }
    names.sort();
    Ok(names)
}

pub fn ensure_data_dir(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root)
        .map_err(|error| format!("SHOW_IO_ERROR: direktori data tidak dapat dibuat: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::EngineState;

    struct TempDir(PathBuf);

    impl TempDir {
        fn new(label: &str) -> Self {
            let path = std::env::temp_dir().join(format!(
                "kinglight-{label}-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn show_document_round_trip_preserves_fixtures_pages_and_shape() {
        let dir = TempDir::new("roundtrip");
        let document = EngineState::new().to_document();
        let path = save_document(&dir.0, "latihan", &document).unwrap();
        assert!(path.is_absolute());
        assert_eq!(path.file_name().unwrap(), "latihan.kinglight");
        let loaded = load_document(&dir.0, "latihan").unwrap();
        assert_eq!(loaded.schema_version, 1);
        assert_eq!(loaded.fixtures.len(), 16);
        assert_eq!(loaded.pages.len(), 6);
        assert_eq!(loaded.fixtures[0].pan, document.fixtures[0].pan);
        assert_eq!(loaded.shape.kind, "Circle");
        assert_eq!(list_documents(&dir.0).unwrap(), vec!["latihan".to_string()]);
    }

    #[test]
    fn path_traversal_names_are_rejected_before_touching_disk() {
        for name in ["../escape", "..", "a/b", "a\\b", "C:\\temp\\show", "CON", ""] {
            assert!(validate_show_name(name).is_err(), "name should fail: {name}");
        }
    }

    #[test]
    fn unsupported_schema_and_bad_json_do_not_replace_show() {
        let dir = TempDir::new("reject");
        let document = EngineState::new().to_document();
        save_document(&dir.0, "aman", &document).unwrap();
        let before = fs::read(dir.0.join("aman.kinglight")).unwrap();

        let mut unsupported = document.clone();
        unsupported.schema_version = 2;
        assert!(save_document(&dir.0, "aman", &unsupported).is_err());
        assert_eq!(fs::read(dir.0.join("aman.kinglight")).unwrap(), before);
        assert!(load_document(&dir.0, "tidak-ada").is_err());
    }

    #[test]
    fn profile_digest_mismatch_is_rejected() {
        let mut document = EngineState::new().to_document();
        document.profile_digest = "lain".to_string();
        assert!(validate_document(&document).is_err());
    }
}
