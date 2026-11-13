// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Embedded geometric lunar position relative to Earth's center.
//!
//! Output is Cartesian XYZ in kilometers in the native ELP2000-82B mean
//! dynamical ecliptic and inertial equinox of J2000.0. Pass a TDB Julian
//! date. The evaluator requires no allocation or I/O. For topocentric
//! geometry, subtract the observer's position expressed in the same frame.

use qtty::Kilometer;

#[path = "elp/lunar_math.rs"]
#[allow(dead_code)]
mod lunar_math;
#[path = "elp/lunar_runtime.rs"]
mod lunar_runtime;

use lunar_runtime::{delaunay, rectangular};

/// Invalid epoch for the compact lunar ephemeris.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactLunarError {
    /// The Julian date was NaN or infinite.
    NonFiniteEpoch,
    /// The finite Julian date falls outside the supported interval.
    OutsideValidityInterval,
}

impl core::fmt::Display for CompactLunarError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonFiniteEpoch => f.write_str("compact ELP requires a finite TDB Julian date"),
            Self::OutsideValidityInterval => write!(
                f,
                "compact ELP requires a TDB Julian date in [{FIRST_JD_TDB}, {END_JD_TDB})"
            ),
        }
    }
}

impl core::error::Error for CompactLunarError {}

#[repr(C)]
struct MainTerm {
    a: f64,
    m: [i8; 4],
}

#[allow(clippy::approx_constant)]
mod data {
    use super::MainTerm;
    include!(concat!(env!("OUT_DIR"), "/elp_compact.rs"));
}

pub use data::{END_JD_TDB, FIRST_JD_TDB, SOURCE_SHA256};

/// Evaluate the Earth-centered lunar vector at a supported TDB Julian date.
///
/// # Errors
///
/// Returns a [`CompactLunarError`] for non-finite or out-of-range dates.
#[inline]
pub fn try_moon_geocentric(jd_tdb: f64) -> Result<[Kilometer; 3], CompactLunarError> {
    if !jd_tdb.is_finite() {
        return Err(CompactLunarError::NonFiniteEpoch);
    }
    if !(FIRST_JD_TDB..END_JD_TDB).contains(&jd_tdb) {
        return Err(CompactLunarError::OutsideValidityInterval);
    }
    Ok(moon_geocentric(jd_tdb))
}

/// Evaluate the lunar vector without checking the input epoch.
///
/// Finite dates outside the supported interval extrapolate the truncated
/// series. Prefer [`try_moon_geocentric`] for externally supplied dates.
pub fn moon_geocentric(jd_tdb: f64) -> [Kilometer; 3] {
    let t = (jd_tdb - 2_451_545.0) / 36_525.0;
    let arguments = delaunay(t);
    let longitude = sum(
        data::LONGITUDE_MAIN,
        data::LONGITUDE_PERT,
        arguments,
        t,
        libm::sin,
    );
    let latitude = sum(
        data::LATITUDE_MAIN,
        data::LATITUDE_PERT,
        arguments,
        t,
        libm::sin,
    );
    let radius = sum(
        data::DISTANCE_MAIN,
        data::DISTANCE_PERT,
        arguments,
        t,
        libm::cos,
    );
    rectangular([longitude, latitude, radius], t).map(Kilometer::new)
}

fn sum(
    main: &[MainTerm],
    perturbations: [&[[f64; 3]]; 3],
    arguments: [f64; 4],
    t: f64,
    trig: fn(f64) -> f64,
) -> f64 {
    let primary: f64 = main
        .iter()
        .map(|term| {
            let phase: f64 = term
                .m
                .into_iter()
                .zip(arguments)
                .map(|(m, x)| f64::from(m) * x)
                .sum();
            term.a * trig(phase)
        })
        .sum();
    let perturbation = perturbations.into_iter().rev().fold(0.0, |acc, terms| {
        let sum: f64 = terms
            .iter()
            .map(|&[a, phase, rate]| a * libm::sin(phase + rate * t))
            .sum();
        acc * t + sum
    });
    primary + perturbation
}
