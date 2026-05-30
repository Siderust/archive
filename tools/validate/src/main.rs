// SPDX-License-Identifier: AGPL-3.0-only
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
//! * 0 — all manifests pass structural validation.
//! * 1 — one or more manifests failed validation.
//! * 2 — usage error (bad arguments, manifest file not found).

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Deserialize;

// ---------------------------------------------------------------------------
// Top-level MANIFEST.toml schema
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct RootManifest {
    #[serde(default)]
    schema_version: Option<u32>,
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

// ---------------------------------------------------------------------------
// Family manifest schema
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
struct FamilyManifest {
    schema_version: u32,
    dataset_id: String,
    dataset_kind: String,
    #[allow(dead_code)]
    description: Option<String>,
    valid_from_jd: Option<f64>,
    valid_to_jd: Option<f64>,
    #[serde(default)]
    files: Vec<FileEntry>,
    #[serde(flatten)]
    #[allow(dead_code)]
    _extra: toml::Value,
}

#[derive(Debug, Deserialize)]
struct FileEntry {
    path: String,
    sha256: String,
    bytes: u64,
    #[serde(default)]
    #[allow(dead_code)]
    format: Option<String>,
}

// ---------------------------------------------------------------------------
// Validation logic
// ---------------------------------------------------------------------------

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

fn validate_root(root: &RootManifest, archive_dir: &Path, report: &mut Report) {
    if root.schema_version.is_some_and(|v| v != 1) {
        report.error(format!(
            "MANIFEST.toml: schema_version must be 1, got {:?}",
            root.schema_version
        ));
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
        }
        if entry.kind.is_empty() {
            report.error(format!("family {}: kind is empty", entry.id));
        }
        if entry.status.is_empty() {
            report.error(format!("family {}: status is empty", entry.id));
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
    if manifest.dataset_id.is_empty() {
        report.error(format!("{}: dataset_id is empty", path.display()));
    }
    if manifest.dataset_kind.is_empty() {
        report.error(format!("{}: dataset_kind is empty", path.display()));
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
    if let (Some(from), Some(to)) = (manifest.valid_from_jd, manifest.valid_to_jd) {
        // A zero interval is used as a placeholder in skeleton manifests; skip.
        if (from != 0.0 || to != 0.0) && to <= from {
            report.error(format!(
                "{}: valid_to_jd ({to}) must be greater than valid_from_jd ({from})",
                path.display()
            ));
        }
    }
    for file in &manifest.files {
        if file.path.is_empty() {
            report.error(format!(
                "{}: [[files]] entry has empty path",
                path.display()
            ));
        }
        if file.sha256.is_empty() {
            report.error(format!(
                "{}: file '{}' has empty sha256",
                path.display(),
                file.path
            ));
        } else if file.sha256.len() != 64 || !file.sha256.chars().all(|c| c.is_ascii_hexdigit()) {
            report.error(format!(
                "{}: file '{}' sha256 is not a valid 64-char hex string",
                path.display(),
                file.path
            ));
        }
        if file.bytes == 0 {
            report.warn(format!(
                "{}: file '{}' has bytes = 0",
                path.display(),
                file.path
            ));
        }
        if file.path.contains("..") || file.path.starts_with('/') {
            report.error(format!(
                "{}: file '{}' path traversal or absolute path",
                path.display(),
                file.path
            ));
        }
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 || args.iter().any(|a| a == "-h" || a == "--help") {
        eprintln!("Usage: archive-validate <path/to/MANIFEST.toml>");
        eprintln!();
        eprintln!("Structurally validates MANIFEST.toml and all referenced family manifests.");
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
            // Already reported above.
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
