// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! JPL DE-series ephemeris download and cache manager.
//!
//! [`DatasetManager`] resolves a local cache directory and provides methods to
//! download, verify, and access JPL BSP kernel files (DE440, DE441).
//!
//! # Example
//!
//! ```rust,ignore
//! use siderust_archive::jpl::{DatasetManager, refs::JplDatasetId};
//!
//! let dm = DatasetManager::new()?;
//! let path = dm.ensure(JplDatasetId::De441)?;
//! println!("DE441 ready at: {}", path.display());
//! ```

pub mod constants;
pub mod refs;

#[cfg(feature = "fetch")]
mod cache;
#[cfg(feature = "fetch")]
mod download;

#[cfg(feature = "fetch")]
pub use download::ProgressCallback;
pub use refs::JplDatasetId;

#[cfg(feature = "fetch")]
use crate::error::ArchiveError;
#[cfg(feature = "fetch")]
use refs::JplDatasetMeta;
#[cfg(feature = "fetch")]
use std::path::{Path, PathBuf};

/// Manages a local data cache for JPL BSP kernel downloads.
///
/// Calling [`Self::ensure`] or [`Self::download`] will download the kernel
/// from the JPL NAIF server if it is not already cached locally.
///
/// The default cache directory is `~/.siderust/data/`; override with
/// `SIDERUST_DATA_DIR`.
///
/// # Example
///
/// ```rust,ignore
/// use siderust_archive::jpl::{DatasetManager, refs::JplDatasetId};
///
/// let dm = DatasetManager::new()?;
/// let bsp_path = dm.ensure(JplDatasetId::De440)?;
/// ```
#[cfg(feature = "fetch")]
pub struct DatasetManager {
    data_dir: PathBuf,
}

#[cfg(feature = "fetch")]
impl DatasetManager {
    /// Create a new `DatasetManager` using the default data directory.
    ///
    /// Default: `~/.siderust/data/` (override with `SIDERUST_DATA_DIR`).
    pub fn new() -> Result<Self, ArchiveError> {
        let data_dir = cache::resolve_data_dir()?;
        cache::ensure_data_dir(&data_dir)?;
        Ok(Self { data_dir })
    }

    /// Create a `DatasetManager` pointing to a specific directory.
    pub fn with_dir(dir: impl Into<PathBuf>) -> Result<Self, ArchiveError> {
        let data_dir = dir.into();
        cache::ensure_data_dir(&data_dir)?;
        Ok(Self { data_dir })
    }

    /// Returns the path to the data directory.
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Check whether a dataset is cached and meets the minimum size threshold.
    pub fn is_available(&self, id: JplDatasetId) -> bool {
        cache::is_cached(&self.data_dir, id.meta())
    }

    /// Return the cached file path if the dataset is available.
    pub fn load_path(&self, id: JplDatasetId) -> Option<PathBuf> {
        if cache::is_cached(&self.data_dir, id.meta()) {
            Some(cache::dataset_path(&self.data_dir, id.meta()))
        } else {
            None
        }
    }

    /// Ensure a dataset is available: verify cache or download if missing.
    ///
    /// Returns the path to the cached BSP file.
    pub fn ensure(&self, id: JplDatasetId) -> Result<PathBuf, ArchiveError> {
        let meta = id.meta();
        let path = cache::dataset_path(&self.data_dir, meta);

        if cache::is_cached(&self.data_dir, meta) {
            cache::verify(meta.name, &path, meta)?;
            return Ok(path);
        }

        self.download_inner(meta, &path, None)?;
        Ok(path)
    }

    /// Download a dataset, with an optional progress callback.
    pub fn download(
        &self,
        id: JplDatasetId,
        progress: Option<ProgressCallback>,
    ) -> Result<PathBuf, ArchiveError> {
        let meta = id.meta();
        let path = cache::dataset_path(&self.data_dir, meta);
        self.download_inner(meta, &path, progress)?;
        Ok(path)
    }

    /// List all known datasets and their cache availability.
    pub fn list(&self) -> Vec<(JplDatasetId, bool)> {
        refs::ALL_DATASETS
            .iter()
            .map(|&id| (id, self.is_available(id)))
            .collect()
    }

    fn download_inner(
        &self,
        meta: &'static JplDatasetMeta,
        path: &Path,
        progress: Option<ProgressCallback>,
    ) -> Result<(), ArchiveError> {
        download::download(meta.name, meta, path, progress)?;
        cache::verify(meta.name, path, meta)?;
        Ok(())
    }
}

#[cfg(all(feature = "fetch", test))]
mod tests {
    use super::*;

    fn temp_dir(suffix: &str) -> PathBuf {
        std::env::temp_dir().join(format!("siderust_archive_jpl_{suffix}"))
    }

    #[test]
    fn with_dir_creates_manager() {
        let dir = temp_dir("with_dir");
        let _ = std::fs::remove_dir_all(&dir);
        let dm = DatasetManager::with_dir(&dir).unwrap();
        assert_eq!(dm.data_dir(), dir.as_path());
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn is_available_returns_false_when_not_cached() {
        let dir = temp_dir("not_cached");
        let _ = std::fs::remove_dir_all(&dir);
        let dm = DatasetManager::with_dir(&dir).unwrap();
        assert!(!dm.is_available(JplDatasetId::De440));
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn list_returns_de440_and_de441() {
        let dir = temp_dir("list_test");
        let dm = DatasetManager::with_dir(&dir).unwrap();
        let items = dm.list();
        assert_eq!(items.len(), 2);
        assert!(items.iter().any(|(id, _)| *id == JplDatasetId::De440));
        assert!(items.iter().any(|(id, _)| *id == JplDatasetId::De441));
        std::fs::remove_dir_all(&dir).ok();
    }
}
