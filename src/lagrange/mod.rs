// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Sun-Earth Lagrange Chebyshev kernel dataset family.
//!
//! Exposes the five SCK binary payloads (`l1.sck`–`l5.sck`) for Sun-Earth
//! Lagrange points L1–L5.  Each kernel covers JD 2415020.5–2488070.5
//! (approximately 1900–2100) at 32-day block resolution with 8 Chebyshev
//! coefficients per coordinate.
//!
//! Dataset provenance, per-file SHA-256 checksums, and generation metadata are
//! in `vsop87/manifest.toml`.
//!
//! The raw bytes are suitable for any SCK v1 reader; the reference consumer is
//! `siderust::formats::sck::parse_sck`.

pub mod refs;
pub mod vsop87;
