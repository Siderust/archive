// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

// IMCCE ELP82B notice and DE200/LE200 constants (MCJCGF.9601).
pub const ARCSEC: f64 = core::f64::consts::PI / 648_000.0;
const DEG: f64 = core::f64::consts::PI / 180.0;

const fn longitude(deg: f64, min: f64, sec: f64) -> f64 {
    (deg + min / 60.0 + sec / 3600.0) * DEG
}

const fn coefficients(degrees: f64, rates: [f64; 4]) -> [f64; 5] {
    [
        degrees,
        rates[0] * ARCSEC,
        rates[1] * ARCSEC,
        rates[2] * ARCSEC,
        rates[3] * ARCSEC,
    ]
}

pub const W: [f64; 5] = coefficients(
    longitude(218., 18., 59.95571),
    [1_732_559_343.736_04, -5.8883, 0.006604, -0.00003169],
);
const PERIGEE: [f64; 5] = coefficients(
    longitude(83., 21., 11.67475),
    [14_643_420.263_2, -38.2776, -0.045047, 0.00021301],
);
const NODE: [f64; 5] = coefficients(
    longitude(125., 2., 40.39816),
    [-6_967_919.362_2, 6.3622, 0.007625, -0.00003586],
);
const EARTH: [f64; 5] = coefficients(
    longitude(100., 27., 59.22059),
    [129_597_742.275_8, -0.0202, 0.000009, 0.00000015],
);
const SOLAR_PERIGEE: [f64; 5] = coefficients(
    longitude(102., 56., 14.42753),
    [1161.2283, 0.5327, -0.000138, 0.0],
);

const fn subtract(a: [f64; 5], b: [f64; 5]) -> [f64; 5] {
    [
        a[0] - b[0],
        a[1] - b[1],
        a[2] - b[2],
        a[3] - b[3],
        a[4] - b[4],
    ]
}

const fn mean_elongation() -> [f64; 5] {
    let mut d = subtract(W, EARTH);
    // Earth's heliocentric longitude is opposite the geocentric Sun.
    d[0] += core::f64::consts::PI;
    d
}

pub const DELAUNAY: [[f64; 5]; 4] = [
    mean_elongation(),
    subtract(EARTH, SOLAR_PERIGEE),
    subtract(W, PERIGEE),
    subtract(W, NODE),
];

const PLANETS: [[f64; 2]; 8] = [
    [longitude(252., 15., 3.25986), 538_101_628.688_98 * ARCSEC],
    [longitude(181., 58., 47.28305), 210_664_136.433_55 * ARCSEC],
    [EARTH[0], EARTH[1]],
    [longitude(355., 25., 59.78866), 68_905_077.592_84 * ARCSEC],
    [longitude(34., 21., 5.34212), 10_925_660.428_61 * ARCSEC],
    [longitude(50., 4., 38.89694), 4_399_609.659_32 * ARCSEC],
    [longitude(314., 3., 18.01841), 1_542_481.193_93 * ARCSEC],
    [longitude(304., 20., 55.19575), 786_550.320_74 * ARCSEC],
];

pub fn main_amplitude(axis: usize, a: f64, b: [f64; 6]) -> f64 {
    let delta_n = 0.55604 * ARCSEC / W[1];
    let delta_n_sun = -0.06424 * ARCSEC / W[1];
    let ratio = 0.074801329518;
    let derivative = b[0] + (2.0 * 0.002571881335 / (3.0 * ratio)) * b[4];
    let base = if axis == 2 {
        a * (1.0 - 2.0 * delta_n / 3.0)
    } else {
        a
    };
    base + derivative * (delta_n_sun - ratio * delta_n)
        + ARCSEC * (-0.08066 * b[1] + 0.01789 * b[2] - 0.12879 * b[3])
}

pub fn time_power(series: usize) -> usize {
    match series {
        7..=9 | 13..=15 | 19..=21 | 25..=27 => 1,
        34..=36 => 2,
        _ => 0,
    }
}

// Perturbations use only the constant and linear argument coefficients.
pub fn perturbation_basis(series: usize) -> &'static [[f64; 2]] {
    const fn linear(a: [f64; 5]) -> [f64; 2] {
        [a[0], a[1]]
    }
    const D: [f64; 2] = linear(DELAUNAY[0]);
    const M: [f64; 2] = linear(DELAUNAY[1]);
    const MP: [f64; 2] = linear(DELAUNAY[2]);
    const F: [f64; 2] = linear(DELAUNAY[3]);
    const EARTH_TERMS: [[f64; 2]; 5] = [[W[0], W[1] + 5029.0966 * ARCSEC], D, M, MP, F];
    const TABLE_1: [[f64; 2]; 11] = [
        PLANETS[0], PLANETS[1], PLANETS[2], PLANETS[3], PLANETS[4], PLANETS[5], PLANETS[6],
        PLANETS[7], D, MP, F,
    ];
    const TABLE_2: [[f64; 2]; 11] = [
        PLANETS[0], PLANETS[1], PLANETS[2], PLANETS[3], PLANETS[4], PLANETS[5], PLANETS[6], D, M,
        MP, F,
    ];
    match series {
        10..=15 => &TABLE_1,
        16..=21 => &TABLE_2,
        _ => &EARTH_TERMS,
    }
}
