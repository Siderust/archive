// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

#[cfg(all(feature = "vsop", not(feature = "vsop-compact")))]
mod full_vsop87e;

#[cfg(all(feature = "vsop", not(feature = "vsop-compact")))]
use full_vsop87e as model;

#[cfg(feature = "vsop-compact")]
use siderust_archive::vsop_compact as model;

pub fn positions(jd: f64) -> [[f64; 3]; 9] {
    let positions = [
        model::sun(jd),
        model::mercury(jd),
        model::venus(jd),
        model::earth(jd),
        model::mars(jd),
        model::jupiter(jd),
        model::saturn(jd),
        model::uranus(jd),
        model::neptune(jd),
    ];
    #[cfg(feature = "vsop-compact")]
    let positions = positions.map(|position| position.map(|coordinate| coordinate.value()));
    positions
}
