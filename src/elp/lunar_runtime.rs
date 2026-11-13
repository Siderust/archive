// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

use super::lunar_math::{ARCSEC, DELAUNAY, W};

fn polynomial(coefficients: [f64; 5], t: f64) -> f64 {
    coefficients
        .into_iter()
        .rev()
        .fold(0.0, |sum, c| sum * t + c)
}

pub fn delaunay(t: f64) -> [f64; 4] {
    DELAUNAY.map(|coefficients| polynomial(coefficients, t))
}

pub fn rectangular([longitude, latitude, radius]: [f64; 3], t: f64) -> [f64; 3] {
    let (sin_l, cos_l) = libm::sincos(polynomial(W, t) + longitude * ARCSEC);
    let (sin_b, cos_b) = libm::sincos(latitude * ARCSEC);
    let r = radius * (384_747.980_644_895_4 / 384_747.980_674_316_5);
    let [x, y, z] = [r * cos_b * cos_l, r * cos_b * sin_l, r * sin_b];

    // Laskar ecliptic rotation into native J2000.
    let p = t * polynomial(
        [
            1.0180391e-5,
            4.7020439e-7,
            -5.417367e-10,
            -2.507948e-12,
            4.63486e-15,
        ],
        t,
    );
    let q = t * polynomial(
        [
            -1.13469002e-4,
            1.2372674e-7,
            1.265417e-9,
            -1.371808e-12,
            -3.20334e-15,
        ],
        t,
    );
    let scale = 2.0 * libm::sqrt(1.0 - p * p - q * q);
    [
        x + p * (-2.0 * p * x + 2.0 * q * y + scale * z),
        y + q * (2.0 * p * x - 2.0 * q * y - scale * z),
        (1.0 - 2.0 * (p * p + q * q)) * z + scale * (q * y - p * x),
    ]
}
