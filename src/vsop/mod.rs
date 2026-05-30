// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! # VSOP87 Planetary Theory
//!
//! ## Scientific scope
//!
//! VSOP87 (*Variations Séculaires des Orbites Planétaires* 1987) is a
//! semi-analytical planetary theory by Bretagnon and Francou (Bureau des
//! Longitudes, Paris).  It models the heliocentric and barycentric positions
//! of the eight major planets as trigonometric power series in time:
//!
//! ```text
//!   X = Σ_k  T^k · Σ_j  A_{kj} · cos(B_{kj} + C_{kj} · T)
//! ```
//!
//! where `T` is Julian millennia from J2000.0 (TDB),
//! coefficients `(A, B, C)` are tabulated in the generated data arrays,
//! and `X ∈ {X, Y, Z}` in astronomical units.
//!
//! Two variants are compiled here:
//!
//! - **VSOP87A** — heliocentric ecliptic rectangular coordinates, equinox J2000.0.
//! - **VSOP87E** — barycentric ecliptic rectangular coordinates, equinox J2000.0.
//!
//! Accuracy: better than 1″ over ±2000 years; better than 1 AU·mas over the
//! 1900–2100 interval for inner planets.
//!
//! ## References
//!
//! - Bretagnon, P., & Francou, G. (1988). "Planetary theories in rectangular
//!   and spherical variables: VSOP87 solutions".
//!   *Astronomy and Astrophysics* 202, 309–315.
//! - IMCCE VSOP87 coefficient archive:
//!   <https://www.imcce.fr/inpop/ephemerides/vsop87/>

pub mod refs;

use qtty::{AstronomicalUnit, Radian};

/// One VSOP87 coefficient term: `a · cos(b + c·T)`.
///
/// - `a` — amplitude (AU).
/// - `b` — phase offset (rad).
/// - `c` — frequency (rad / Julian millennium); kept as `f64` because no
///   qtty unit exists for rad·millennium⁻¹.
#[derive(Debug, Clone, Copy)]
pub struct Vsop87 {
    pub a: AstronomicalUnit,
    pub b: Radian,
    pub c: f64,
}

/// Generated VSOP87A coefficient tables.
///
/// The arrays within this module are produced by the build pipeline
/// ([`pipeline`]) from the raw IMCCE data files.  Each static is named
/// `{PLANET}_{AXIS}{T_POWER}`, e.g. `EARTH_X0`, `MARS_Y3`.
#[allow(clippy::all, unreachable_pub, dead_code)]
pub mod vsop87a_data {
    include!(concat!(env!("OUT_DIR"), "/vsop87a.rs"));
}

/// Generated VSOP87E coefficient tables (barycentric).
#[allow(clippy::all, unreachable_pub, dead_code)]
pub mod vsop87e_data {
    include!(concat!(env!("OUT_DIR"), "/vsop87e.rs"));
}

/// Provenance for the VSOP87 planetary theory coefficient tables.
pub fn provenance() -> crate::provenance::DatasetProvenance {
    crate::provenance::DatasetProvenance {
        source: "IMCCE VSOP87 (Bretagnon & Francou 1988). \
                 Mirror: https://ftp.imcce.fr/pub/ephem/planets/vsop87/"
            .to_string(),
        generator: "siderust-archive/src/vsop/pipeline.rs".to_string(),
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: None,
        generated_at: "2026-05-28T00:00:00Z".to_string(),
    }
}
