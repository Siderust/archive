// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! IAU 2000A and IAU 2000B nutation coefficient tables.
//!
//! ## Scientific scope
//!
//! Tabulated nutation series from the MHB2000 model, as adopted by the IAU in
//! 2000 and documented in IERS Conventions (2010), ch. 5.  The [`tables`]
//! sub-module exposes:
//!
//! * [`tables::NUT00A_LS`] — 678 luni-solar terms (IAU 2000A).
//! * [`tables::NUT00A_PL`] — 687 planetary terms (IAU 2000A).
//! * [`tables::NUT00B_LS`] — 77 luni-solar terms (IAU 2000B).
//!
//! These are the canonical, normalized Rust representation of the series
//! derived from SOFA `iauNut00a` / `iauNut00b`.  The raw provenance is
//! recorded in `nutation/manifest.toml`.

pub mod refs;
pub mod tables;

/// Provenance for the IAU 2000A/2000B nutation coefficient tables.
pub fn provenance() -> crate::provenance::DatasetProvenance {
    crate::provenance::DatasetProvenance {
        source: "MHB2000 (Mathews, Herring & Buffett 2002); SOFA iauNut00a/iauNut00b; \
                 IERS Conventions (2010) Tables 5.3a/5.3b"
            .to_string(),
        generator: "siderust-archive/src/nutation/pipeline.rs".to_string(),
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: None,
        generated_at: "2026-05-28T00:00:00Z".to_string(),
    }
}
