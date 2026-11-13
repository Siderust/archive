// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

#[cfg(all(feature = "elp", not(feature = "elp-compact")))]
mod full_elp;

#[cfg(all(feature = "elp", not(feature = "elp-compact")))]
pub use full_elp::moon;

#[cfg(feature = "elp-compact")]
pub fn moon(jd: f64) -> [f64; 3] {
    siderust_archive::elp_compact::moon_geocentric(jd).map(|value| value.value())
}
