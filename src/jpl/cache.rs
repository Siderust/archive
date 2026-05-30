// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! Filesystem cache for JPL datasets.

use crate::error::ArchiveError;
use crate::jpl::refs::JplDatasetMeta;
use std::path::{Path, PathBuf};

/// Environment variable to override the data directory.
pub const DATA_DIR_ENV: &str = "SIDERUST_DATA_DIR";

const DEFAULT_SUBDIR: &str = ".siderust/data";

pub(super) fn resolve_data_dir() -> Result<PathBuf, ArchiveError> {
    if let Ok(dir) = std::env::var(DATA_DIR_ENV) {
        let dir = dir.trim();
        if !dir.is_empty() {
            return Ok(PathBuf::from(dir));
        }
    }

    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| {
            ArchiveError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Cannot determine home directory. Set SIDERUST_DATA_DIR explicitly.",
            ))
        })?;

    Ok(PathBuf::from(home).join(DEFAULT_SUBDIR))
}

pub(super) fn ensure_data_dir(dir: &Path) -> Result<(), ArchiveError> {
    std::fs::create_dir_all(dir)?;
    Ok(())
}

pub(super) fn dataset_path(data_dir: &Path, meta: &JplDatasetMeta) -> PathBuf {
    data_dir.join(meta.filename)
}

pub(super) fn is_cached(data_dir: &Path, meta: &JplDatasetMeta) -> bool {
    let path = dataset_path(data_dir, meta);
    match std::fs::metadata(&path) {
        Ok(m) => m.len() >= meta.min_size,
        Err(_) => false,
    }
}

pub(super) fn verify(name: &str, path: &Path, meta: &JplDatasetMeta) -> Result<(), ArchiveError> {
    let file_meta = std::fs::metadata(path)?;
    if file_meta.len() < meta.min_size {
        return Err(ArchiveError::Integrity(format!(
            "{}: file too small ({} bytes, expected >= {})",
            name,
            file_meta.len(),
            meta.min_size,
        )));
    }

    if !meta.sha256.is_empty() {
        let actual = sha256_file(path)?;
        if actual != meta.sha256 {
            return Err(ArchiveError::Integrity(format!(
                "{}: SHA-256 mismatch (expected {}, got {})",
                name, meta.sha256, actual,
            )));
        }
    }

    Ok(())
}

fn sha256_file(path: &Path) -> Result<String, ArchiveError> {
    use sha2::{Digest, Sha256};
    use std::io::Read;

    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20];

    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }

    let digest = hasher.finalize();
    Ok(digest.iter().map(|b| format!("{:02x}", b)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jpl::refs::{JplDatasetMeta, DE440};

    fn dummy_meta(filename: &'static str, min_size: u64) -> JplDatasetMeta {
        JplDatasetMeta {
            name: "test",
            url: "https://example.com/file",
            filename,
            sha256: "",
            min_size,
            size_hint: "1 B",
        }
    }

    #[test]
    fn resolve_data_dir_uses_env_var() {
        let tmp = std::env::temp_dir().join("siderust_archive_test_jpl_data_dir");
        std::env::set_var(DATA_DIR_ENV, tmp.to_str().unwrap());
        let result = resolve_data_dir().unwrap();
        assert_eq!(result, tmp);
        std::env::remove_var(DATA_DIR_ENV);
    }

    #[test]
    fn is_cached_returns_true_when_file_meets_min_size() {
        let tmp = std::env::temp_dir().join("siderust_archive_test_jpl_cache_ok");
        ensure_data_dir(&tmp).unwrap();
        let path = tmp.join("ok_file.bsp");
        std::fs::write(&path, &vec![0u8; 100]).unwrap();
        let rdm = dummy_meta("ok_file.bsp", 50);
        assert!(is_cached(&tmp, &rdm));
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&tmp).ok();
    }

    #[test]
    fn verify_passes_when_no_sha256_and_size_ok() {
        let tmp = std::env::temp_dir().join("siderust_archive_test_jpl_verify_ok");
        ensure_data_dir(&tmp).unwrap();
        let path = tmp.join("ok.bsp");
        std::fs::write(&path, &vec![0u8; 200]).unwrap();
        let rdm = dummy_meta("ok.bsp", 100);
        verify("test", &path, &rdm).unwrap();
        std::fs::remove_file(&path).ok();
        std::fs::remove_dir(&tmp).ok();
    }

    #[test]
    fn de440_meta_fields_non_empty() {
        assert!(!DE440.url.is_empty());
        assert!(!DE440.filename.is_empty());
        assert!(DE440.min_size > 0);
    }
}
