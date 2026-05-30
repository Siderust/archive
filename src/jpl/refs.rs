// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! JPL planetary ephemeris dataset references.
//!
//! This module contains the authoritative download metadata for the JPL
//! DE440 and DE441 BSP kernel files.

/// Identifies a JPL DE-series planetary ephemeris kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum JplDatasetId {
    /// JPL DE440 planetary ephemeris (~120 MB BSP, 1550–2650 CE).
    De440,
    /// JPL DE441 planetary ephemeris part-2 (~1.65 GB BSP, long-range).
    De441,
}

impl JplDatasetId {
    /// Short string identifier used in filenames and log messages.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::De440 => "de440",
            Self::De441 => "de441",
        }
    }

    /// Return the download metadata for this dataset.
    pub const fn meta(self) -> &'static JplDatasetMeta {
        match self {
            Self::De440 => &DE440,
            Self::De441 => &DE441,
        }
    }
}

impl std::fmt::Display for JplDatasetId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Download metadata for a JPL BSP kernel.
#[derive(Debug, Clone)]
pub struct JplDatasetMeta {
    /// Human-readable dataset name.
    pub name: &'static str,
    /// Primary download URL.
    pub url: &'static str,
    /// Filename stored in the local cache directory.
    pub filename: &'static str,
    /// Expected SHA-256 hex digest (empty = skip verification).
    pub sha256: &'static str,
    /// Minimum plausible file size in bytes.
    pub min_size: u64,
    /// Human-readable size hint for progress messages.
    pub size_hint: &'static str,
}

/// DE440 download metadata.
pub static DE440: JplDatasetMeta = JplDatasetMeta {
    name: "JPL DE440",
    url: "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440.bsp",
    filename: "de440.bsp",
    sha256: "a4ce9bf9b3282becc9f4b2ac3cebe03a2ae7599981aabd7265fd8482fff7c4b5",
    min_size: 100_000_000,
    size_hint: "~120 MB",
};

/// DE441 (part 2) download metadata.
pub static DE441: JplDatasetMeta = JplDatasetMeta {
    name: "JPL DE441 (part 2)",
    url: "https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de441_part-2.bsp",
    filename: "de441_part-2.bsp",
    sha256: "3abb17dae2d78dd34880377544aacb54892104a0d4462b322cb9f4454d4887f6",
    min_size: 1_500_000_000,
    size_hint: "~1.65 GB",
};

/// All known JPL dataset identifiers.
pub const ALL_DATASETS: &[JplDatasetId] = &[JplDatasetId::De440, JplDatasetId::De441];
