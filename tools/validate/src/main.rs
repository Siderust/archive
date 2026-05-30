// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Structural validator for the siderust-archive MANIFEST.toml and all
//! referenced family manifests.
//!
//! # Usage
//!
//! ```text
//! cargo run -p archive-validate -- MANIFEST.toml
//! cargo run -p archive-validate -- path/to/archive/MANIFEST.toml
//! ```
//!
//! # Exit codes
//!
//! * 0 — all manifests pass validation.
//! * 1 — one or more manifests failed validation.
//! * 2 — usage error (bad arguments, manifest file not found).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Deserialize;
use sha2::{Digest, Sha256};

const ALLOWED_KINDS: &[&str] = &[
    "planetary-theory",
    "lunar-theory",
    "nutation",
    "planetary-series",
    "lagrange-chebyshev",
    "time-scale",
    "reference-frame",
    "body-constants",
    "spice-kernel",
    "planetary-ephemeris",
    "atmosphere-model",
    "geopotential",
    "atmosphere",
];

const ALLOWED_STATUSES: &[&str] = &["active", "partial", "pending-migration", "skeleton"];

#[derive(Debug, Deserialize)]
struct RootManifest {
    #[serde(default)]
    schema_version: Option<u32>,
    #[serde(default)]
    archive_name: String,
    #[serde(default)]
    archive_version: String,
    #[serde(rename = "family", default)]
    families: Vec<FamilyEntry>,
}

#[derive(Debug, Deserialize)]
struct FamilyEntry {
    id: String,
    manifest: String,
    kind: String,
    status: String,
}

#[derive(Debug, Deserialize)]
struct FamilyManifest {
    schema_version: u32,
    dataset_id: String,
    dataset_kind: String,
    source: String,
    generator: String,
    generator_version: String,
    generated_at: String,
    time_scale: String,
    frame: String,
    center: String,
    units: String,
    valid_from_jd: f64,
    valid_to_jd: f64,
    dynamical_model: String,
    #[serde(default)]
    files: Vec<FileEntry>,
    #[serde(default, rename = "remote_files")]
    remote_files: Vec<RemoteFileEntry>,
    #[serde(default, rename = "references")]
    references: Vec<ReferenceEntry>,
    #[serde(default)]
    error_metrics: Option<toml::Value>,
    #[serde(default, rename = "points")]
    points: Vec<toml::Value>,
}

#[derive(Debug, Deserialize)]
struct FileEntry {
    path: String,
    sha256: String,
    bytes: u64,
    #[serde(default)]
    format: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RemoteFileEntry {
    path: String,
    url: String,
    sha256: String,
    #[serde(default)]
    bytes: Option<u64>,
    #[serde(default)]
    min_size: Option<u64>,
    #[serde(default)]
    format: Option<String>,
    #[serde(default)]
    size_hint: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReferenceEntry {
    #[serde(default)]
    citation: Option<String>,
    #[serde(default)]
    doi: Option<String>,
    #[serde(default)]
    url: Option<String>,
}

#[derive(Default)]
struct Report {
    errors: Vec<String>,
    warnings: Vec<String>,
}

impl Report {
    fn error(&mut self, msg: impl Into<String>) {
        self.errors.push(msg.into());
    }

    fn warn(&mut self, msg: impl Into<String>) {
        self.warnings.push(msg.into());
    }

    fn ok(&self) -> bool {
        self.errors.is_empty()
    }
}

fn is_valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|c| c.is_ascii_hexdigit())
}

fn validate_non_empty(path: &Path, name: &str, value: &str, report: &mut Report) {
    if value.is_empty() {
        report.error(format!("{}: {name} is empty", path.display()));
    }
}

fn validate_relative_path(path: &Path, label: &str, rel_path: &str, report: &mut Report) {
    if rel_path.is_empty() {
        report.error(format!("{}: {label} has empty path", path.display()));
    }
    if rel_path.contains("..") || rel_path.starts_with('/') {
        report.error(format!(
            "{}: {label} '{}' path traversal or absolute path",
            path.display(),
            rel_path
        ));
    }
}

fn validate_root(root: &RootManifest, archive_dir: &Path, report: &mut Report) {
    match root.schema_version {
        Some(1) => {}
        Some(v) => report.error(format!("MANIFEST.toml: schema_version must be 1, got {v}")),
        None => report.error("MANIFEST.toml: schema_version is missing"),
    }
    if root.archive_name.is_empty() {
        report.error("MANIFEST.toml: archive_name is missing or empty");
    }
    if root.archive_version.is_empty() {
        report.error("MANIFEST.toml: archive_version is missing or empty");
    }
    if root.families.is_empty() {
        report.warn("MANIFEST.toml: no [[family]] entries found");
    }
    for entry in &root.families {
        if entry.id.is_empty() {
            report.error("MANIFEST.toml: [[family]] entry has empty id");
        }
        if entry.manifest.is_empty() {
            report.error(format!("family {}: manifest path is empty", entry.id));
        } else if entry.manifest.contains("..") || entry.manifest.starts_with('/') {
            report.error(format!(
                "family {}: manifest path '{}' contains path traversal or is absolute",
                entry.id, entry.manifest
            ));
        }
        if entry.kind.is_empty() {
            report.error(format!("family {}: kind is empty", entry.id));
        } else if !ALLOWED_KINDS.contains(&entry.kind.as_str()) {
            report.error(format!(
                "family {}: kind '{}' is not in the allowed set",
                entry.id, entry.kind
            ));
        }
        if entry.status.is_empty() {
            report.error(format!("family {}: status is empty", entry.id));
        } else if !ALLOWED_STATUSES.contains(&entry.status.as_str()) {
            report.error(format!(
                "family {}: status '{}' is not in the allowed set {:?}",
                entry.id, entry.status, ALLOWED_STATUSES
            ));
        }
        let manifest_path = archive_dir.join(&entry.manifest);
        if !manifest_path.exists() {
            report.error(format!(
                "family {}: manifest file not found at {}",
                entry.id,
                manifest_path.display()
            ));
        }
    }
}

fn validate_family(path: &Path, expected_id: Option<&str>, report: &mut Report) {
    let manifest_dir = path.parent().unwrap_or(Path::new("."));
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) => {
            report.error(format!("cannot read {}: {e}", path.display()));
            return;
        }
    };
    let manifest: FamilyManifest = match toml::from_str(&text) {
        Ok(m) => m,
        Err(e) => {
            report.error(format!("{}: TOML parse error: {e}", path.display()));
            return;
        }
    };

    if manifest.schema_version != 1 {
        report.error(format!(
            "{}: schema_version must be 1, got {}",
            path.display(),
            manifest.schema_version
        ));
    }
    validate_non_empty(path, "dataset_id", &manifest.dataset_id, report);
    validate_non_empty(path, "dataset_kind", &manifest.dataset_kind, report);
    validate_non_empty(path, "source", &manifest.source, report);
    validate_non_empty(path, "generator", &manifest.generator, report);
    validate_non_empty(
        path,
        "generator_version",
        &manifest.generator_version,
        report,
    );
    validate_non_empty(path, "generated_at", &manifest.generated_at, report);
    validate_non_empty(path, "time_scale", &manifest.time_scale, report);
    validate_non_empty(path, "frame", &manifest.frame, report);
    validate_non_empty(path, "center", &manifest.center, report);
    validate_non_empty(path, "units", &manifest.units, report);
    validate_non_empty(path, "dynamical_model", &manifest.dynamical_model, report);

    if !manifest.dataset_kind.is_empty() && !ALLOWED_KINDS.contains(&manifest.dataset_kind.as_str())
    {
        report.error(format!(
            "{}: dataset_kind '{}' is not in the allowed set",
            path.display(),
            manifest.dataset_kind
        ));
    }

    if let Some(expected) = expected_id {
        if manifest.dataset_id != expected {
            report.warn(format!(
                "{}: dataset_id '{}' does not match registry id '{}'",
                path.display(),
                manifest.dataset_id,
                expected
            ));
        }
    }

    if (manifest.valid_from_jd != 0.0 || manifest.valid_to_jd != 0.0)
        && manifest.valid_to_jd <= manifest.valid_from_jd
    {
        report.error(format!(
            "{}: valid_to_jd ({}) must be greater than valid_from_jd ({})",
            path.display(),
            manifest.valid_to_jd,
            manifest.valid_from_jd
        ));
    }

    // Lagrange-Chebyshev: require error metrics or per-point validation data.
    if manifest.dataset_kind == "lagrange-chebyshev"
        && manifest.error_metrics.is_none()
        && manifest.points.is_empty()
    {
        report.error(format!(
            "{}: dataset_kind 'lagrange-chebyshev' requires either [error_metrics] or [[points]] \
             entries with validation metrics",
            path.display()
        ));
    }

    for (i, file) in manifest.files.iter().enumerate() {
        validate_relative_path(path, "[[files]] entry", &file.path, report);
        match &file.format {
            Some(fmt) if fmt.is_empty() => report.error(format!(
                "{}: file '{}' (index {i}) has empty format",
                path.display(),
                file.path
            )),
            None => report.error(format!(
                "{}: file '{}' (index {i}) is missing required format field",
                path.display(),
                file.path
            )),
            _ => {}
        }
        if file.sha256.is_empty() {
            report.error(format!(
                "{}: file '{}' has empty sha256",
                path.display(),
                file.path
            ));
        } else if !is_valid_sha256(&file.sha256) {
            report.error(format!(
                "{}: file '{}' sha256 is not a valid 64-char hex string",
                path.display(),
                file.path
            ));
        }
        if file.bytes == 0 {
            report.error(format!(
                "{}: file '{}' has bytes = 0",
                path.display(),
                file.path
            ));
        }

        let file_path = manifest_dir.join(&file.path);
        if !file_path.exists() {
            report.error(format!(
                "{}: file '{}' not found on disk",
                path.display(),
                file.path
            ));
            continue;
        }

        let data = match std::fs::read(&file_path) {
            Ok(data) => data,
            Err(err) => {
                report.error(format!(
                    "{}: cannot read file '{}': {err}",
                    path.display(),
                    file.path
                ));
                continue;
            }
        };
        if data.len() as u64 != file.bytes {
            report.error(format!(
                "{}: file '{}' byte count mismatch: expected {}, got {}",
                path.display(),
                file.path,
                file.bytes,
                data.len()
            ));
        }
        let hash = hex::encode(Sha256::digest(&data));
        if hash != file.sha256 {
            report.error(format!(
                "{}: file '{}' SHA-256 mismatch:\n  expected {}\n  got      {}",
                path.display(),
                file.path,
                file.sha256,
                hash
            ));
        }
    }

    for (i, file) in manifest.remote_files.iter().enumerate() {
        validate_relative_path(path, "[[remote_files]] entry", &file.path, report);
        if file.url.is_empty() {
            report.error(format!(
                "{}: remote file '{}' has empty url",
                path.display(),
                file.path
            ));
        }
        match &file.format {
            Some(fmt) if fmt.is_empty() => report.error(format!(
                "{}: remote file '{}' (index {i}) has empty format",
                path.display(),
                file.path
            )),
            None => report.error(format!(
                "{}: remote file '{}' (index {i}) is missing required format field",
                path.display(),
                file.path
            )),
            _ => {}
        }
        if file.sha256.is_empty() {
            report.error(format!(
                "{}: remote file '{}' has empty sha256",
                path.display(),
                file.path
            ));
        } else if !is_valid_sha256(&file.sha256) {
            report.error(format!(
                "{}: remote file '{}' sha256 is not a valid 64-char hex string",
                path.display(),
                file.path
            ));
        }
        if file.bytes.is_none() && file.min_size.is_none() {
            report.error(format!(
                "{}: remote file '{}' must have at least one of 'bytes' or 'min_size'",
                path.display(),
                file.path
            ));
        }
        if matches!(file.bytes, Some(0)) {
            report.error(format!(
                "{}: remote file '{}' has bytes = 0",
                path.display(),
                file.path
            ));
        }
        if matches!(file.min_size, Some(0)) {
            report.error(format!(
                "{}: remote file '{}' has min_size = 0",
                path.display(),
                file.path
            ));
        }
        if let (Some(bytes), Some(min_size)) = (file.bytes, file.min_size) {
            if min_size > bytes {
                report.error(format!(
                    "{}: remote file '{}' min_size ({min_size}) exceeds bytes ({bytes})",
                    path.display(),
                    file.path
                ));
            }
        }
        let _ = (&file.size_hint, &file.notes);
    }

    for (i, reference) in manifest.references.iter().enumerate() {
        if reference.citation.is_none() && reference.doi.is_none() && reference.url.is_none() {
            report.warn(format!(
                "{}: [[references]] entry {i} has none of 'citation', 'doi', or 'url'",
                path.display()
            ));
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("Usage: archive-validate <path/to/MANIFEST.toml>");
        eprintln!();
        eprintln!("Validates MANIFEST.toml, family manifests, and committed file integrity.");
        eprintln!("Exit 0 = OK, exit 1 = validation errors, exit 2 = usage error.");
        return ExitCode::from(2);
    }

    let manifest_path = PathBuf::from(&args[1]);
    if !manifest_path.exists() {
        eprintln!(
            "error: manifest file not found: {}",
            manifest_path.display()
        );
        return ExitCode::from(2);
    }

    let archive_dir = match manifest_path.parent() {
        Some(d) => d.to_path_buf(),
        None => PathBuf::from("."),
    };

    let text = match std::fs::read_to_string(&manifest_path) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("error: cannot read {}: {e}", manifest_path.display());
            return ExitCode::from(2);
        }
    };

    let root: RootManifest = match toml::from_str(&text) {
        Ok(m) => m,
        Err(e) => {
            eprintln!(
                "error: TOML parse error in {}: {e}",
                manifest_path.display()
            );
            return ExitCode::FAILURE;
        }
    };

    let mut all_ok = true;
    let mut root_report = Report::default();
    validate_root(&root, &archive_dir, &mut root_report);

    for w in &root_report.warnings {
        eprintln!("warning: {w}");
    }
    for e in &root_report.errors {
        eprintln!("error: {e}");
    }
    if !root_report.ok() {
        all_ok = false;
    }

    for entry in &root.families {
        let family_path = archive_dir.join(&entry.manifest);
        if !family_path.exists() {
            continue;
        }
        let mut family_report = Report::default();
        validate_family(&family_path, Some(&entry.id), &mut family_report);
        for w in &family_report.warnings {
            eprintln!("warning [{}]: {w}", entry.id);
        }
        for e in &family_report.errors {
            eprintln!("error [{}]: {e}", entry.id);
        }
        if !family_report.ok() {
            all_ok = false;
        }
    }

    if all_ok {
        println!(
            "OK: {} families validated",
            root.families
                .iter()
                .filter(|f| archive_dir.join(&f.manifest).exists())
                .count()
        );
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
