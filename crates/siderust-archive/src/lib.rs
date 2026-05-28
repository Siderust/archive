// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! # siderust-archive
//!
//! Reusable Rust bindings for the [Siderust Archive](https://github.com/Siderust/archive),
//! the canonical, repo-agnostic store for scientific datasets (IERS time data,
//! SPICE-style kernels, planetary theories, …).
//!
//! ## Features
//!
//! * `default` — manifest + checksum APIs only. Zero network dependencies, tiny
//!   compile-time footprint. Suitable for libraries that just need to load a
//!   pre-distributed archive directory.
//! * `time` — adds the [`time`] module: IERS UTC-TAI / ΔT / Earth Orientation
//!   Parameter types, parsers, provenance records, and bundle representation.
//! * `fetch` — implies `time` and additionally enables runtime download of the
//!   IERS bundle via [`time::TimeDataManager`]. Brings in `ureq`, `sha2`, and
//!   `serde_json`.
//!
//! ## Typical usage
//!
//! ```toml
//! # Library that only needs to parse the manifest schema:
//! siderust-archive = "0.1"
//!
//! # Application that wants the IERS time data bundled at runtime:
//! siderust-archive = { version = "0.1", features = ["fetch"] }
//!
//! # Time data only (e.g. tempoch consumes just the parsers/types):
//! siderust-archive = { version = "0.1", features = ["time"] }
//! ```
//!
//! ## Modules
//!
//! * [`manifest`] — TOML manifest model (schema v1): top-level registry and
//!   per-family dataset manifests with provenance, units, validity, checksums.
//! * [`checksum`] — SHA-256 integrity helpers.
//! * [`time`] (feature `time`) — IERS time-scale data: source URLs, parsers,
//!   typed bundle, provenance, and (with `fetch`) a runtime download manager.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod checksum;
pub mod manifest;

#[cfg(feature = "time")]
pub mod time;

pub use manifest::{ArchiveManifest, FamilyManifest, SCHEMA_VERSION};
