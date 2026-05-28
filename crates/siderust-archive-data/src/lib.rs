// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! # siderust-archive-data
//!
//! Reusable Rust bindings for the [Siderust Archive](https://github.com/Siderust/archive).
//!
//! The archive is the canonical, repo-agnostic store for scientific datasets
//! (kernels, IERS time data, planetary theories, …). It is consumed as a git
//! submodule, so any repository can depend on this crate through a path
//! dependency into the submodule and reuse the same data-access layer:
//!
//! ```toml
//! # In a consuming crate's Cargo.toml, with the archive checked out at ./archive
//! siderust-archive-data = { path = "archive/crates/siderust-archive-data", features = ["fetch"] }
//! ```
//!
//! Remember to initialise the submodule:
//!
//! ```text
//! git submodule update --init --recursive
//! ```
//!
//! ## Modules
//!
//! * [`manifest`] — TOML manifest model (schema v1): the top-level registry and
//!   per-family dataset manifests with provenance, units, validity, checksums.
//! * [`checksum`] — SHA-256 integrity helpers.
//! * [`time`] — IERS time-scale data (leap seconds, ΔT, Earth Orientation
//!   Parameters): source URLs, parsers, typed bundle, and (behind the `fetch`
//!   feature) a runtime download manager with provenance and integrity checks.
//!
//! ## Features
//!
//! * `default` — pure parsing and manifest access, no network or crypto deps.
//! * `fetch` — enables runtime downloading (`ureq`), SHA-256 verification
//!   (`sha2`), and provenance (de)serialisation (`serde_json`).

pub mod checksum;
pub mod manifest;
pub mod time;

pub use manifest::{ArchiveManifest, FamilyManifest, SCHEMA_VERSION};
