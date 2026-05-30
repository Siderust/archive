// SPDX-License-Identifier: BSD-3-Clause
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

    /// Validate archive-level invariants. Returns list of errors.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        if self.archive_name.is_empty() {
            errors.push("archive_name is empty".into());
        }
        if self.archive_version.is_empty() {
            errors.push("archive_version is empty".into());
        }
        if self.families.is_empty() {
            errors.push("no [[family]] entries".into());
        }
        for f in &self.families {
            if f.id.is_empty() {
                errors.push("[[family]] entry has empty id".into());
            }
            if f.manifest.is_empty() {
                errors.push(format!("family {}: manifest path is empty", f.id));
            }
        }
        errors
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
    #[serde(default, rename = "remote_files")]
    pub remote_files: Vec<RemoteFileEntry>,
    #[serde(default, rename = "references")]
    pub references: Vec<Reference>,
    #[serde(default)]
    pub error_metrics: Option<ErrorMetrics>,
}

/// A single committed file shipped with a dataset family.
#[derive(Debug, Clone, Deserialize)]
pub struct FileEntry {
    /// Path relative to the manifest's directory.
    pub path: String,
    #[serde(default)]
    pub format: Option<String>,
    pub sha256: String,
    pub bytes: u64,
}

/// A remote-only file that is not committed but can be downloaded at runtime.
#[derive(Debug, Clone, Deserialize)]
pub struct RemoteFileEntry {
    pub path: String,
    pub url: String,
    pub sha256: String,
    #[serde(default)]
    pub bytes: Option<u64>,
    #[serde(default)]
    pub min_size: Option<u64>,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default)]
    pub size_hint: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
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

    /// Validate family-level invariants. Returns list of errors.
    pub fn validate(&self) -> Vec<String> {
        let mut errors = Vec::new();
        macro_rules! require_non_empty {
            ($field:expr, $name:literal) => {
                if $field.is_empty() {
                    errors.push(format!("{} is empty", $name));
                }
            };
        }
        require_non_empty!(self.dataset_id, "dataset_id");
        require_non_empty!(self.dataset_kind, "dataset_kind");
        require_non_empty!(self.source, "source");
        require_non_empty!(self.generator, "generator");
        require_non_empty!(self.generator_version, "generator_version");
        require_non_empty!(self.generated_at, "generated_at");
        require_non_empty!(self.time_scale, "time_scale");
        require_non_empty!(self.frame, "frame");
        require_non_empty!(self.center, "center");
        require_non_empty!(self.units, "units");
        require_non_empty!(self.dynamical_model, "dynamical_model");
        if (self.valid_from_jd != 0.0 || self.valid_to_jd != 0.0)
            && self.valid_to_jd <= self.valid_from_jd
        {
            errors.push(format!(
                "valid_to_jd ({}) must be > valid_from_jd ({})",
                self.valid_to_jd, self.valid_from_jd
            ));
        }
        for file in &self.files {
            if file.path.is_empty() {
                errors.push("[[files]] entry has empty path".into());
            }
            if file.sha256.is_empty() {
                errors.push(format!("file '{}': sha256 is empty", file.path));
            }
            if file.bytes == 0 {
                errors.push(format!("file '{}': bytes is 0", file.path));
            }
            if file.path.contains("..") || file.path.starts_with('/') {
                errors.push(format!("file '{}': path traversal or absolute", file.path));
            }
        }
        errors
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
bytes  = 10000

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
        assert!(m.validate().is_empty());
    }

    #[test]
    fn parses_family_manifest() {
        let m = FamilyManifest::parse(FAMILY).unwrap();
        assert_eq!(m.dataset_id, "time-iers-eop");
        assert_eq!(m.files.len(), 1);
        assert_eq!(m.files[0].format.as_deref(), Some("iers-finals2000A"));
        assert_eq!(m.files[0].bytes, 10000);
        assert_eq!(m.references.len(), 1);
        assert!(m.valid_from_jd < m.valid_to_jd);
        assert!(m.validate().is_empty());
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
