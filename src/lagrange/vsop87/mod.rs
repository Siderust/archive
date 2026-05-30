// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Embedded Sun-Earth Lagrange Chebyshev kernel payloads.
//!
//! Each file is a Siderust Chebyshev Kernel (SCK v1) binary (~464 KB) containing
//! 2283 Chebyshev records that span JD 2415020.5–2488070.5 (1900–2100).
//! Consumers parse the raw bytes with a SCK reader (e.g. `siderust::formats::sck`).
//!
//! See `manifest.toml` in this directory for dataset provenance and per-file
//! SHA-256 checksums.

/// Raw SCK bytes for Sun-Earth L1.
pub static L1_BYTES: &[u8] = include_bytes!("l1.sck");
/// Raw SCK bytes for Sun-Earth L2.
pub static L2_BYTES: &[u8] = include_bytes!("l2.sck");
/// Raw SCK bytes for Sun-Earth L3.
pub static L3_BYTES: &[u8] = include_bytes!("l3.sck");
/// Raw SCK bytes for Sun-Earth L4.
pub static L4_BYTES: &[u8] = include_bytes!("l4.sck");
/// Raw SCK bytes for Sun-Earth L5.
pub static L5_BYTES: &[u8] = include_bytes!("l5.sck");
