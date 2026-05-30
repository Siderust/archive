// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! NRLMSISE-00 atmosphere density tables.
//!
//! ## Scientific scope
//!
//! The Naval Research Laboratory Mass Spectrometer and Incoherent Scatter
//! Radar Exosphere model (NRLMSISE-00) is an empirical atmosphere model
//! providing air density as a function of altitude.  This module ships the
//! lookup table used by the lite-approximation density provider in `siderust`.
//!
//! The [`tables`] sub-module exposes:
//!
//! * [`tables::NRLMSISE_TABLE`] — (altitude_km, density_kg_m³) pairs.
//!
//! ## Reference
//!
//! Picone, J. M., Hedin, A. E., Drob, D. P., & Aikin, A. C. (2002).
//! NRLMSISE-00 empirical model of the atmosphere: Statistical comparisons and
//! scientific issues. JGR: Space Physics, 107(A12), 1468.
//! <https://doi.org/10.1029/2002JA009430>

pub mod refs;
pub mod tables;

/// Provenance for the NRLMSISE-00 atmosphere density table.
pub fn provenance() -> crate::provenance::DatasetProvenance {
    crate::provenance::DatasetProvenance {
        source: "Picone et al. (2002) NRLMSISE-00; Vallado Table 8-4 approximation. \
                 Full model: https://ccmc.gsfc.nasa.gov/models/NRLMSISE00/"
            .to_string(),
        generator: "siderust-archive/src/atmosphere/pipeline.rs".to_string(),
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: None,
        generated_at: "2026-05-28T00:00:00Z".to_string(),
    }
}
