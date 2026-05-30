// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! EGM2008 geopotential coefficient tables.
//!
//! ## Scientific scope
//!
//! The Earth Gravitational Model 2008 (EGM2008) is a spherical harmonic
//! model of Earth's gravitational potential published by the National
//! Geospatial-Intelligence Agency (NGA).  This module ships the normalized
//! Stokes coefficients through degree/order 4, suitable for low-degree
//! perturbation force models in orbit propagation.
//!
//! The [`tables`] sub-module exposes:
//!
//! * [`tables::EGM2008_COEFFS`] — (n, m, C̄ₙₘ, S̄ₙₘ) tuples for n = 2..=4.
//!
//! ## Reference
//!
//! Pavlis, N. K., Holmes, S. A., Kenyon, S. C., & Factor, J. K. (2012).
//! The development and evaluation of the Earth Gravitational Model 2008
//! (EGM2008). JGR: Solid Earth, 117, B04406.
//! <https://doi.org/10.1029/2011JB008916>

pub mod refs;
pub mod tables;

/// Provenance for the EGM2008 geopotential coefficient table.
pub fn provenance() -> crate::provenance::DatasetProvenance {
    crate::provenance::DatasetProvenance {
        source: "Pavlis et al. (2012) EGM2008; NGA product. \
                 Full model: https://earth-info.nga.mil/index.php?dir=wgs84&action=wgs84"
            .to_string(),
        generator: "siderust-archive/src/gravity/pipeline.rs".to_string(),
        generator_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: None,
        generated_at: "2026-05-28T00:00:00Z".to_string(),
    }
}
