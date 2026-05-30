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
//! | Feature        | Effect |
//! |----------------|--------|
//! | `default`      | Manifest + checksum + provenance APIs only. Zero network/crypto dependencies. |
//! | `vsop`         | VSOP87A/E planetary theory tables (build-time generated via `build.rs`). |
//! | `elp`          | ELP2000-82B lunar theory tables (build-time generated via `build.rs`). |
//! | `time`         | IERS time-scale data: UTC-TAI, ΔT, EOP — types, parsers, bundled snapshot. |
//! | `bundled-time` | Compiled UTC-TAI / ΔT fallback snapshot (offline, no network). Implied by `time`. |
//! | `jpl`          | JPL DE440/DE441 ephemeris download/cache manager. |
//! | `fetch`        | Runtime network download for IERS and JPL datasets. Implies `time` and `jpl`. |
//! | `nutation`     | IAU 2000A/2000B nutation coefficient tables (MHB2000). |
//! | `gravity`      | EGM2008 geopotential coefficients (low-degree subset). |
//! | `atmosphere`   | NRLMSISE-00 atmosphere model table. |
//! | `frames`       | SPICE-style frame definitions (stub). |
//! | `constants`    | SPICE-style body constants (stub). |
//! | `lagrange`     | Sun-Earth Lagrange Chebyshev kernels (stub). |
//! | `pluto`        | Pluto abbreviated series — Meeus (1998) (stub). |
//!
//! ## Typical usage
//!
//! ```toml
//! # Manifest + checksum only:
//! siderust-archive = "0.1"
//!
//! # IERS time data with offline fallback:
//! siderust-archive = { version = "0.1", features = ["time", "bundled-time"] }
//!
//! # Full runtime IERS download + JPL ephemeris manager:
//! siderust-archive = { version = "0.1", features = ["fetch"] }
//!
//! # VSOP87 planetary theory tables:
//! siderust-archive = { version = "0.1", features = ["vsop"] }
//! ```
//!
//! ## Modules
//!
//! | Module        | Feature      | Purpose |
//! |---------------|--------------|---------|
//! | [`manifest`]  | always on    | TOML manifest schema v1: archive and family manifests. |
//! | [`checksum`]  | always on    | SHA-256 hex helpers and mismatch error. |
//! | [`provenance`]| always on    | Generic dataset provenance record. |
//! | [`error`]     | always on    | Shared `ArchiveError` type. |
//! | [`vsop`]      | `vsop`       | VSOP87A/E coefficient tables and accessor types. |
//! | [`elp`]       | `elp`        | ELP2000-82B lunar theory coefficient tables. |
//! | [`time`]      | `time`       | IERS UTC-TAI / ΔT / EOP types, parsers, and (with `fetch`) download manager. |
//! | [`jpl`]       | `jpl`        | JPL DE440/DE441 dataset manager. |
//! | [`nutation`]  | `nutation`   | IAU 2000A/2000B nutation coefficient tables (MHB2000). |
//! | [`gravity`]   | `gravity`    | EGM2008 geopotential coefficients (low-degree subset). |
//! | [`atmosphere`]| `atmosphere` | NRLMSISE-00 atmosphere model table. |
//! | [`frames`]    | `frames`     | SPICE-style frame definition references (stub). |
//! | [`constants`] | `constants`  | SPICE-style body-constant references (stub). |
//! | [`lagrange`]  | `lagrange`   | Sun-Earth Lagrange Chebyshev kernel references (stub). |
//! | [`pluto`]     | `pluto`      | Pluto abbreviated series — Meeus (1998) (stub). |

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod checksum;
pub mod error;
pub mod manifest;
pub mod provenance;

#[cfg(feature = "vsop")]
pub mod vsop;

#[cfg(feature = "elp")]
pub mod elp;

#[cfg(feature = "time")]
pub mod time;

#[cfg(feature = "jpl")]
pub mod jpl;

#[cfg(feature = "nutation")]
pub mod nutation;

#[cfg(feature = "gravity")]
pub mod gravity;

#[cfg(feature = "atmosphere")]
pub mod atmosphere;

#[cfg(feature = "frames")]
pub mod frames;

#[cfg(feature = "constants")]
pub mod constants;

#[cfg(feature = "lagrange")]
pub mod lagrange;

#[cfg(feature = "pluto")]
pub mod pluto;

pub use error::ArchiveError;
pub use manifest::{ArchiveManifest, FamilyManifest, SCHEMA_VERSION};
