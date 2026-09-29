// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! # siderust-archive
//!
//! Reusable Rust bindings for the [Siderust Archive](https://github.com/Siderust/archive),
//! the canonical, repo-agnostic store for scientific datasets (IERS time data,
//! SPICE-style kernels, planetary theories, …).
//!
//! ## `std` / `no_std`
//!
//! By default the crate enables the `std` feature. For `no_std` + `alloc`:
//!
//! ```toml
//! siderust-archive = { version = "0.1", default-features = false }
//! ```
//!
//! Filesystem, network, environment-variable, and OS I/O APIs require `std`.
//! The `fetch` feature implies `std`.
//!
//! ## Features
//!
//! | Feature        | Effect |
//! |----------------|--------|
//! | `default`      | Enables `std`. |
//! | `std`          | Standard library (I/O, env, OS errors). Absent ⇒ `no_std` + `alloc`. |
//! | `vsop`         | VSOP87A/E planetary theory tables (build-time generated via `build.rs`). |
//! | `elp`          | ELP2000-82B lunar theory tables (build-time generated via `build.rs`). |
//! | `time`         | IERS time-scale data: UTC-TAI, ΔT, EOP — types, parsers, bundled snapshot. |
//! | `bundled-time` | Compiled UTC-TAI / ΔT fallback snapshot (offline, no network). Implied by `time`. |
//! | `jpl`          | JPL DE440/DE441 ephemeris metadata. Download/cache requires `fetch`. |
//! | `fetch`        | Runtime network download for IERS and JPL datasets. Requires `std`; implies `time` and `jpl`. |
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
//! # Manifest + checksum only (with std):
//! siderust-archive = "0.1"
//!
//! # no_std + alloc (manifest, checksum, provenance, scientific tables):
//! siderust-archive = { version = "0.1", default-features = false }
//!
//! # IERS time data with offline fallback:
//! siderust-archive = { version = "0.1", features = ["time"] }
//!
//! # Full runtime IERS download + JPL ephemeris manager (requires std):
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
//! | [`jpl`]       | `jpl`        | JPL DE440/DE441 dataset metadata (download/cache with `fetch`). |
//! | [`nutation`]  | `nutation`   | IAU 2000A/2000B nutation coefficient tables (MHB2000). |
//! | [`gravity`]   | `gravity`    | EGM2008 geopotential coefficients (low-degree subset). |
//! | [`atmosphere`]| `atmosphere` | NRLMSISE-00 atmosphere model table. |
//! | [`frames`]    | `frames`     | SPICE-style frame definition references (stub). |
//! | [`constants`] | `constants`  | SPICE-style body-constant references (stub). |
//! | [`lagrange`]  | `lagrange`   | Sun-Earth Lagrange Chebyshev kernel references (stub). |
//! | [`pluto`]     | `pluto`      | Pluto abbreviated series — Meeus (1998) (stub). |

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

extern crate alloc;

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
