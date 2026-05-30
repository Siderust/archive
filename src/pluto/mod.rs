// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Pluto abbreviated series (Meeus 1998, ch. 37).
//!
//! ## Scientific scope
//!
//! Low-precision heliocentric position of Pluto from the Meeus (1998)
//! abbreviated series (Astronomical Algorithms, 2nd ed., ch. 37).
//! Accurate to ≈ 0.1 arcsec over 1885–2099.
//!
//! The [`pluto_data`] sub-module exposes:
//!
//! * [`pluto_data::PLUTO_ARGUMENTS`] — 43 Delaunay-like argument sets (j, s, p).
//! * [`pluto_data::PLUTO_LONGITUDE_TERMS`] — sine/cosine amplitudes for longitude.
//! * [`pluto_data::PLUTO_LATITUDE_TERMS`] — sine/cosine amplitudes for latitude.
//! * [`pluto_data::PLUTO_RADIUS_TERMS`] — sine/cosine amplitudes for radius.
//!
//! ## References
//!
//! * Meeus, J. (1998). Astronomical Algorithms (2nd ed.). Willmann-Bell. Ch. 37.

pub mod refs;

/// Argument multipliers for Jupiter (j), Saturn (s), and Pluto (p) mean
/// longitudes in the Meeus (1998) abbreviated Pluto series.
pub struct PlutoArgument {
    /// Jupiter mean longitude multiplier.
    pub j: f64,
    /// Saturn mean longitude multiplier.
    pub s: f64,
    /// Pluto mean longitude multiplier.
    pub p: f64,
}

/// Sine (`a`) and cosine (`b`) amplitude pair for a single Pluto series term.
///
/// Longitude and latitude amplitudes are in units of 0.1 µas; radius
/// amplitudes are in units of 1e-7 AU.
pub struct PlutoTerm {
    pub a: f64,
    pub b: f64,
}

pub mod pluto_data {
    include!(concat!(env!("OUT_DIR"), "/pluto_data.rs"));
}

/// Provenance for the Pluto abbreviated series.
pub fn provenance() -> crate::provenance::DatasetProvenance {
    crate::provenance::DatasetProvenance {
        source: "Meeus, J. (1998). Astronomical Algorithms, 2nd ed., ch. 37. Willmann-Bell."
            .to_string(),
        generator: "siderust-archive/src/pluto/pipeline.rs".to_string(),
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: None,
        generated_at: "2026-05-28T00:00:00Z".to_string(),
    }
}
