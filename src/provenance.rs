// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Generic dataset provenance record.
//!
//! A [`DatasetProvenance`] captures the minimum lineage information that every
//! dataset in the archive carries: where the data came from, which tool
//! produced the derived artefact, and when it was generated.
//!
//! Domain-specific provenance types (e.g. [`crate::time::TimeDataProvenance`])
//! extend or complement this with additional per-format fields.

/// Lineage record for a single derived dataset or downloaded resource.
///
/// # Example
///
/// ```rust
/// use siderust_archive::provenance::DatasetProvenance;
///
/// let prov = DatasetProvenance {
///     source: "IMCCE VSOP87 ftp mirror".to_string(),
///     generator: "siderust/import-vsop87".to_string(),
///     generator_version: "0.8.0".to_string(),
///     git_commit: Some("deadbeef".to_string()),
///     generated_at: "2026-05-29T00:00:00Z".to_string(),
/// };
/// assert_eq!(prov.source, "IMCCE VSOP87 ftp mirror");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetProvenance {
    /// Human-readable description of where the upstream data was obtained.
    pub source: String,
    /// Crate or script that produced the derived artefact.
    pub generator: String,
    /// Version of the generator at the time of generation.
    pub generator_version: String,
    /// Git commit of the generator repository at generation time, if available.
    pub git_commit: Option<String>,
    /// ISO 8601 UTC timestamp of when the artefact was generated.
    pub generated_at: String,
}
