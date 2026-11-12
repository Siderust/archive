// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Compact embedded VSOP87E positions for the Sun and eight planets.
//!
//! The body functions return geometric barycentric `[X, Y, Z]` in AU, in the
//! native VSOP87 dynamical ecliptic and equinox of J2000.0. Pass a Julian date
//! in TDB.
//! Subtract the Earth or Sun vector at the same epoch for geometric geocentric
//! or heliocentric positions. The functions perform no light-time, aberration,
//! or observer corrections.
//!
//! Use the checked `try_*` functions for input that needs validation. They
//! accept finite TDB Julian dates in [`FIRST_JD_TDB`, `END_JD_TDB`). The direct
//! body functions are unchecked and extrapolate the truncated series outside
//! the validated interval. This profile provides positions only.
//!
//! Each body has its own static coefficients so the linker can discard unused
//! bodies. Evaluation uses `f64` throughout and does not allocate.
//!
//! ```
//! use siderust_archive::vsop_compact::{try_earth, try_sun, FIRST_JD_TDB};
//!
//! let earth = try_earth(FIRST_JD_TDB)?;
//! let sun = try_sun(FIRST_JD_TDB)?;
//! let geocentric_sun = core::array::from_fn::<_, 3, _>(|i| sun[i] - earth[i]);
//! assert!(geocentric_sun[0].value().is_finite());
//! # Ok::<(), siderust_archive::vsop_compact::CompactEphemerisError>(())
//! ```

use qtty::AstronomicalUnit;

/// An epoch rejected by a checked compact VSOP87E position function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompactEphemerisError {
    /// The Julian date was NaN or infinite.
    NonFiniteEpoch,
    /// The finite Julian date was outside the supported interval.
    OutsideValidityInterval,
}

impl core::fmt::Display for CompactEphemerisError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::NonFiniteEpoch => {
                f.write_str("compact VSOP87E requires a finite TDB Julian date")
            }
            Self::OutsideValidityInterval => write!(
                f,
                "compact VSOP87E requires a TDB Julian date in [{FIRST_JD_TDB}, {END_JD_TDB})"
            ),
        }
    }
}

impl core::error::Error for CompactEphemerisError {}

#[inline]
fn validate_epoch(jd_tdb: f64) -> Result<(), CompactEphemerisError> {
    if !jd_tdb.is_finite() {
        return Err(CompactEphemerisError::NonFiniteEpoch);
    }
    if !(FIRST_JD_TDB..END_JD_TDB).contains(&jd_tdb) {
        return Err(CompactEphemerisError::OutsideValidityInterval);
    }
    Ok(())
}

fn axis(terms: &[[f64; 3]], ends: [u16; 6], t: f64) -> AstronomicalUnit {
    // Horner evaluation of the six polynomial orders. Terms within each
    // order retain their original source order.
    let mut value = 0.0;
    for order in (0..ends.len()).rev() {
        let start = if order == 0 { 0 } else { ends[order - 1] };
        let sum: f64 = terms[usize::from(start)..usize::from(ends[order])]
            .iter()
            .map(|&[a, b, c]| a * libm::cos(b + c * t))
            .sum();
        value = value * t + sum;
    }
    AstronomicalUnit::new(value)
}

#[allow(clippy::excessive_precision, clippy::approx_constant)]
mod data {
    use super::{axis, validate_epoch, AstronomicalUnit, CompactEphemerisError};
    include!(concat!(env!("OUT_DIR"), "/vsop87e_compact.rs"));
}

pub use data::*;

#[cfg(test)]
mod tests {
    use super::axis;

    #[test]
    fn untruncated_layout_matches_direct_polynomial_sum() {
        let groups: [&[[f64; 3]]; 6] = [
            &[[0.25, 0.1, 23.0], [0.5, 2.4, 0.0]],
            &[],
            &[[0.8, 3.1, 150.0]],
            &[],
            &[[0.1, 0.2, 4.0]],
            &[],
        ];
        let mut ends = [0; 6];
        let mut terms = alloc::vec::Vec::new();
        for (group, end) in groups.iter().zip(&mut ends) {
            terms.extend_from_slice(group);
            *end = terms.len() as u16;
        }
        for t in [0.0, 0.02, 0.031, 0.051] {
            let mut direct = 0.0;
            let mut power = 1.0;
            for group in groups {
                for &[a, b, c] in group {
                    direct += power * a * libm::cos(b + c * t);
                }
                power *= t;
            }
            assert!((axis(&terms, ends, t).value() - direct).abs() < 1e-14);
        }
    }
}
