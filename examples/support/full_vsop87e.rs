// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

// Shared by the desktop measurement harness and the full-vs-compact test.
use siderust_archive::vsop::{vsop87e_data::*, Vsop87};

fn axis(series: &[&[Vsop87]], t: f64) -> f64 {
    series.iter().rev().fold(0.0, |value, terms| {
        let sum: f64 = terms
            .iter()
            .map(|term| term.a.value() * libm::cos(term.b.value() + term.c * t))
            .sum();
        value * t + sum
    })
}

macro_rules! body {
    ($name:ident, $([$($terms:ident),+]),+) => {
        pub fn $name(jd: f64) -> [f64; 3] {
            let t = (jd - 2_451_545.0) / 365_250.0;
            [$(axis(&[$(&$terms),+], t)),+]
        }
    };
}

body!(
    earth,
    [EARTH_X0, EARTH_X1, EARTH_X2, EARTH_X3, EARTH_X4, EARTH_X5],
    [EARTH_Y0, EARTH_Y1, EARTH_Y2, EARTH_Y3, EARTH_Y4, EARTH_Y5],
    [EARTH_Z0, EARTH_Z1, EARTH_Z2, EARTH_Z3, EARTH_Z4, EARTH_Z5]
);

body!(
    jupiter,
    [JUPITER_X0, JUPITER_X1, JUPITER_X2, JUPITER_X3, JUPITER_X4, JUPITER_X5],
    [JUPITER_Y0, JUPITER_Y1, JUPITER_Y2, JUPITER_Y3, JUPITER_Y4, JUPITER_Y5],
    [JUPITER_Z0, JUPITER_Z1, JUPITER_Z2, JUPITER_Z3, JUPITER_Z4, JUPITER_Z5]
);

body!(
    mars,
    [MARS_X0, MARS_X1, MARS_X2, MARS_X3, MARS_X4, MARS_X5],
    [MARS_Y0, MARS_Y1, MARS_Y2, MARS_Y3, MARS_Y4, MARS_Y5],
    [MARS_Z0, MARS_Z1, MARS_Z2, MARS_Z3, MARS_Z4, MARS_Z5]
);

body!(
    mercury,
    [MERCURY_X0, MERCURY_X1, MERCURY_X2, MERCURY_X3, MERCURY_X4, MERCURY_X5],
    [MERCURY_Y0, MERCURY_Y1, MERCURY_Y2, MERCURY_Y3, MERCURY_Y4, MERCURY_Y5],
    [MERCURY_Z0, MERCURY_Z1, MERCURY_Z2, MERCURY_Z3, MERCURY_Z4, MERCURY_Z5]
);

body!(
    neptune,
    [NEPTUNE_X0, NEPTUNE_X1, NEPTUNE_X2, NEPTUNE_X3, NEPTUNE_X4],
    [NEPTUNE_Y0, NEPTUNE_Y1, NEPTUNE_Y2, NEPTUNE_Y3, NEPTUNE_Y4],
    [NEPTUNE_Z0, NEPTUNE_Z1, NEPTUNE_Z2, NEPTUNE_Z3]
);

body!(
    saturn,
    [SATURN_X0, SATURN_X1, SATURN_X2, SATURN_X3, SATURN_X4, SATURN_X5],
    [SATURN_Y0, SATURN_Y1, SATURN_Y2, SATURN_Y3, SATURN_Y4, SATURN_Y5],
    [SATURN_Z0, SATURN_Z1, SATURN_Z2, SATURN_Z3, SATURN_Z4, SATURN_Z5]
);

body!(
    sun,
    [SUN_X0, SUN_X1, SUN_X2, SUN_X3, SUN_X4, SUN_X5],
    [SUN_Y0, SUN_Y1, SUN_Y2, SUN_Y3, SUN_Y4, SUN_Y5],
    [SUN_Z0, SUN_Z1, SUN_Z2, SUN_Z3, SUN_Z4, SUN_Z5]
);

body!(
    uranus,
    [URANUS_X0, URANUS_X1, URANUS_X2, URANUS_X3, URANUS_X4],
    [URANUS_Y0, URANUS_Y1, URANUS_Y2, URANUS_Y3, URANUS_Y4],
    [URANUS_Z0, URANUS_Z1, URANUS_Z2, URANUS_Z3]
);

body!(
    venus,
    [VENUS_X0, VENUS_X1, VENUS_X2, VENUS_X3, VENUS_X4, VENUS_X5],
    [VENUS_Y0, VENUS_Y1, VENUS_Y2, VENUS_Y3, VENUS_Y4, VENUS_Y5],
    [VENUS_Z0, VENUS_Z1, VENUS_Z2, VENUS_Z3, VENUS_Z4, VENUS_Z5]
);
