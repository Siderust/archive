// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! # ELP2000-82B Lunar Theory
//!
//! ## Scientific scope
//!
//! ELP2000-82B is a semi-analytical theory of the Moon's orbital motion by
//! Chapront-Touzé and Chapront (Bureau des Longitudes, Paris).  It expresses
//! the Moon's geocentric ecliptic longitude, latitude, and distance as Poisson
//! series in `T` (Julian millennia from J2000.0).
//!
//! - **36 sub-series** split into three families: Main Problem, Earth
//!   Perturbations, Planetary Perturbations.
//! - Accuracy: ±10″ over ±10 centuries from J2000.0.
//!
//! ## References
//!
//! - Chapront-Touzé & Chapront (1983). "The lunar ephemeris ELP 2000".
//!   *A&A* 124, 50–62.
//! - Chapront-Touzé & Chapront (1988). "ELP 2000-82B".
//!   *A&A* 190, 342–352.

pub mod constants;
pub mod refs;

/// One term of the ELP2000 Main Problem series.
#[derive(Debug, Clone, Copy)]
pub struct MainProblem {
    pub ilu: [i64; 4],
    pub a: f64,
    pub b: [f64; 6],
}

/// One term of the ELP2000 Earth Perturbation series.
#[derive(Debug, Clone, Copy)]
pub struct EarthPert {
    pub iz: f64,
    pub ilu: [i64; 4],
    pub o: f64,
    pub a: f64,
    pub p: f64,
}

/// One term of the ELP2000 Planetary Perturbation series.
#[derive(Debug, Clone, Copy)]
pub struct PlanetPert {
    pub ipla: [i64; 11],
    pub theta: f64,
    pub o: f64,
    pub p: f64,
}

/// Generated ELP2000-82B coefficient tables.
///
/// Arrays within this module are produced by the build pipeline
/// ([`pipeline`]) from the raw CDS data files.  Each static is named
/// `ELP{N}` where `N` is 1–36.
#[allow(clippy::all, unreachable_pub, dead_code)]
pub mod elp_data {
    include!(concat!(env!("OUT_DIR"), "/elp_data.rs"));
}

/// Provenance for the ELP2000-82B lunar theory coefficient tables.
pub fn provenance() -> crate::provenance::DatasetProvenance {
    crate::provenance::DatasetProvenance {
        source: "ELP2000-82B (Chapront-Touzé & Chapront 1983/1988). \
                 CDS catalog: https://cdsarc.cds.unistra.fr/viz-bin/cat/VI/79"
            .to_string(),
        generator: "siderust-archive/src/elp/pipeline.rs".to_string(),
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: None,
        generated_at: "2026-05-28T00:00:00Z".to_string(),
    }
}
