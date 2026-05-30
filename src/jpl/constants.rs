// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! # JPL DE-series Physical Constants
//!
//! Physical constants taken from the JPL DE-series ephemeris headers.  Values
//! are consistent across DE440 and DE441.
//!
//! ## Source
//!
//! - Park, R. S. et al. (2021). "The JPL Planetary and Lunar Ephemerides
//!   DE440 and DE441". *Astronomical Journal* 161, 105.
//!   <https://doi.org/10.3847/1538-3881/abd414>

/// Earth/Moon mass ratio from the JPL DE4xx ephemeris headers.
///
/// Defined as `μ_Earth / μ_Moon`.  Consistent across DE440 and DE441.
pub const EARTH_MOON_RATIO: f64 = 81.300_569_074_190_62;
