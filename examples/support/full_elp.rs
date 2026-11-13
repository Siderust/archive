// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

// Scalar baseline over the unchanged archive tables, using the published
// IMCCE series equations and the same native frame as the compact profile.
use siderust_archive::elp::{elp_data::*, EarthPert, MainProblem, PlanetPert};

#[path = "../../src/elp/lunar_math.rs"]
mod lunar_math;
#[path = "../../src/elp/lunar_runtime.rs"]
mod lunar_runtime;

fn primary(table: &[MainProblem], axis: usize, args: [f64; 4]) -> f64 {
    table
        .iter()
        .map(|term| {
            let phase = term
                .ilu
                .into_iter()
                .zip(args)
                .map(|(m, a)| m as f64 * a)
                .sum();
            let value = if axis == 2 {
                libm::cos(phase)
            } else {
                libm::sin(phase)
            };
            lunar_math::main_amplitude(axis, term.a, term.b) * value
        })
        .sum()
}

fn perturbations<const N: usize>(
    terms: impl Iterator<Item = ([f64; N], f64, f64)>,
    series: usize,
    powers: [f64; 3],
) -> f64 {
    let mut args = [0.0; N];
    for (arg, &[offset, rate]) in args.iter_mut().zip(lunar_math::perturbation_basis(series)) {
        *arg = offset + rate * powers[1];
    }
    let sum: f64 = terms
        .map(|(multipliers, phase, amplitude)| {
            let angle = phase.to_radians()
                + multipliers
                    .into_iter()
                    .zip(args)
                    .map(|(m, a)| m * a)
                    .sum::<f64>();
            amplitude * libm::sin(angle)
        })
        .sum();
    sum * powers[lunar_math::time_power(series)]
}

pub fn moon(jd: f64) -> [f64; 3] {
    let t = (jd - 2_451_545.0) / 36_525.0;
    let args = lunar_runtime::delaunay(t);
    let powers = [1.0, t, t * t];
    let mut coords = [
        primary(ELP1, 0, args),
        primary(ELP2, 1, args),
        primary(ELP3, 2, args),
    ];
    let earth: [(usize, &[EarthPert]); 21] = [
        (4, ELP4),
        (5, ELP5),
        (6, ELP6),
        (7, ELP7),
        (8, ELP8),
        (9, ELP9),
        (22, ELP22),
        (23, ELP23),
        (24, ELP24),
        (25, ELP25),
        (26, ELP26),
        (27, ELP27),
        (28, ELP28),
        (29, ELP29),
        (30, ELP30),
        (31, ELP31),
        (32, ELP32),
        (33, ELP33),
        (34, ELP34),
        (35, ELP35),
        (36, ELP36),
    ];
    let planets: [&[PlanetPert]; 12] = [
        ELP10, ELP11, ELP12, ELP13, ELP14, ELP15, ELP16, ELP17, ELP18, ELP19, ELP20, ELP21,
    ];
    for (series, terms) in earth {
        let terms = terms.iter().map(|term| {
            let [d, m, mp, f] = term.ilu.map(|v| v as f64);
            ([term.iz, d, m, mp, f], term.o, term.a)
        });
        coords[(series - 1) % 3] += perturbations(terms, series, powers);
    }
    for (index, terms) in planets.into_iter().enumerate() {
        let terms = terms
            .iter()
            .map(|term| (term.ipla.map(|v| v as f64), term.theta, term.o));
        coords[index % 3] += perturbations(terms, index + 10, powers);
    }
    lunar_runtime::rectangular(coords, t)
}
