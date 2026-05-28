// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! TOML manifest model for the Siderust Archive (schema v1).
//!
//! Two layers are represented:
//! * [`ArchiveManifest`] — the top-level `MANIFEST.toml` registry that lists
//!   each dataset [`Family`].
//! * [`FamilyManifest`] — a per-family `manifest.toml` describing the dataset's
//!   provenance, units, validity interval, files, and checksums.
//!
//! All archive metadata is TOML; JSON is not used anywhere in the archive.
//! See `schema/archive-manifest-v1.md` for the authoritative contract.

use serde::Deserialize;

/// Error raised while parsing a manifest.
#[derive(Debug)]
pub enum ManifestError {
    /// The TOML text was malformed or did not match the schema.
    Toml(toml::de::Error),
    /// The declared `schema_version` is not supported by this crate.
    UnsupportedSchema(u32),
}

impl std::fmt::Display for ManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Toml(err) => write!(f, "manifest parse error: {err}"),
            Self::UnsupportedSchema(v) => {
                write!(f, "unsupported manifest schema_version {v} (expected 1)")
            }
        }
    }
}

impl std::error::Error for ManifestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Toml(err) => Some(err),
            _ => None,
        }
    }
}

impl From<toml::de::Error> for ManifestError {
    fn from(value: toml::de::Error) -> Self {
        Self::Toml(value)
    }
}

/// Current supported manifest schema version.
pub const SCHEMA_VERSION: u32 = 1;

/// Top-level `MANIFEST.toml` registry.
#[derive(Debug, Clone, Deserialize)]
pub struct ArchiveManifest {
    pub schema_version: u32,
    pub archive_name: String,
    pub archive_version: String,
    #[serde(default, rename = "family")]
    pub families: Vec<Family>,
}

/// One dataset family entry in the top-level registry.
#[derive(Debug, Clone, Deserialize)]
pub struct Family {
    pub id: String,
    /// Relative path to the family's `manifest.toml`.
    pub manifest: String,
    pub kind: String,
    #[serde(default)]
    pub status: Option<String>,
}

impl ArchiveManifest {
    /// Parse the top-level registry, validating the schema version.
    pub fn parse(text: &str) -> Result<Self, ManifestError> {
        let manifest: ArchiveManifest = toml::from_str(text)?;
        if manifest.schema_version != SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchema(manifest.schema_version));
        }
        Ok(manifest)
    }

    /// Find a family entry by id.
    pub fn family(&self, id: &str) -> Option<&Family> {
        self.families.iter().find(|f| f.id == id)
    }
}

/// Per-family `manifest.toml`.
#[derive(Debug, Clone, Deserialize)]
pub struct FamilyManifest {
    pub schema_version: u32,
    pub dataset_id: String,
    pub dataset_kind: String,
    pub source: String,
    pub generator: String,
    pub generator_version: String,
    pub generated_at: String,
    pub time_scale: String,
    pub frame: String,
    pub center: String,
    pub units: String,
    pub valid_from_jd: f64,
    pub valid_to_jd: f64,
    pub dynamical_model: String,
    #[serde(default)]
    pub git_commit: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default, rename = "files")]
    pub files: Vec<FileEntry>,
    #[serde(default, rename = "references")]
    pub references: Vec<Reference>,
    #[serde(default)]
    pub error_metrics: Option<ErrorMetrics>,
}

/// A single file shipped with a dataset family.
#[derive(Debug, Clone, Deserialize)]
pub struct FileEntry {
    /// Path relative to the manifest's directory.
    pub path: String,
    pub format: String,
    pub sha256: String,
    #[serde(default)]
    pub bytes: Option<u64>,
}

/// A bibliographic reference for a dataset.
#[derive(Debug, Clone, Deserialize)]
pub struct Reference {
    #[serde(default)]
    pub citation: Option<String>,
    #[serde(default)]
    pub doi: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

/// Fitting/error metrics for derived datasets.
#[derive(Debug, Clone, Deserialize)]
pub struct ErrorMetrics {
    #[serde(default)]
    pub max_abs_error_m: Option<f64>,
    #[serde(default)]
    pub rms_error_m: Option<f64>,
    #[serde(default)]
    pub validation_step_seconds: Option<f64>,
}

impl FamilyManifest {
    /// Parse a per-family manifest, validating the schema version.
    pub fn parse(text: &str) -> Result<Self, ManifestError> {
        let manifest: FamilyManifest = toml::from_str(text)?;
        if manifest.schema_version != SCHEMA_VERSION {
            return Err(ManifestError::UnsupportedSchema(manifest.schema_version));
        }
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TOP_LEVEL: &str = r#"
schema_version  = 1
archive_name    = "siderust-archive"
archive_version = "0.1.0"

[[family]]
id       = "time"
manifest = "time/manifest.toml"
kind     = "time-scale"
status   = "pending-migration"
"#;

    const FAMILY: &str = r#"
schema_version    = 1
dataset_id        = "time-iers-eop"
dataset_kind      = "time-scale"
source            = "IERS Bulletin A + C04 (finals2000A.all)"
generator         = "upstream"
generator_version = "iers-finals2000A"
generated_at      = "2026-05-28T00:00:00Z"
time_scale        = "UTC/TAI/UT1"
frame             = "ITRF/celestial-pole-offset"
center            = "Earth"
units             = "arcsec, s, ms, mas"
valid_from_jd     = 2437665.5
valid_to_jd       = 2461000.5
dynamical_model   = "Observed/predicted Earth orientation"

[[files]]
path   = "raw/finals2000A.all"
format = "iers-finals2000A"
sha256 = "f18123bd6cb801f308be476de7b17f8193084fecf70baebc1b944ab1fd1e6d19"

[[references]]
citation = "IERS Earth Orientation Parameters."
url      = "https://datacenter.iers.org/data/9/finals2000A.all"
"#;

    #[test]
    fn parses_top_level_registry() {
        let m = ArchiveManifest::parse(TOP_LEVEL).unwrap();
        assert_eq!(m.archive_name, "siderust-archive");
        assert_eq!(m.families.len(), 1);
        let time = m.family("time").unwrap();
        assert_eq!(time.manifest, "time/manifest.toml");
        assert_eq!(time.kind, "time-scale");
    }

    #[test]
    fn parses_family_manifest() {
        let m = FamilyManifest::parse(FAMILY).unwrap();
        assert_eq!(m.dataset_id, "time-iers-eop");
        assert_eq!(m.files.len(), 1);
        assert_eq!(m.files[0].format, "iers-finals2000A");
        assert_eq!(m.references.len(), 1);
        assert!(m.valid_from_jd < m.valid_to_jd);
    }

    #[test]
    fn rejects_unsupported_schema() {
        let bad = TOP_LEVEL.replace("schema_version  = 1", "schema_version  = 2");
        assert!(matches!(
            ArchiveManifest::parse(&bad),
            Err(ManifestError::UnsupportedSchema(2))
        ));
    }
}
