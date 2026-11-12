// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

#![cfg(feature = "vsop-compact")]

use siderust_archive::vsop_compact::{self, END_JD_TDB, FIRST_JD_TDB};

#[cfg(feature = "vsop")]
#[path = "../examples/support/full_vsop87e.rs"]
mod full;

fn compact(jd: f64) -> [[f64; 3]; 9] {
    [
        vsop_compact::sun(jd),
        vsop_compact::mercury(jd),
        vsop_compact::venus(jd),
        vsop_compact::earth(jd),
        vsop_compact::mars(jd),
        vsop_compact::jupiter(jd),
        vsop_compact::saturn(jd),
        vsop_compact::uranus(jd),
        vsop_compact::neptune(jd),
    ]
    .map(|position| position.map(|coordinate| coordinate.value()))
}

fn difference(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    core::array::from_fn(|i| a[i] - b[i])
}

fn norm(a: [f64; 3]) -> f64 {
    a.iter().map(|v| v * v).sum::<f64>().sqrt()
}

fn arcseconds(a: [f64; 3], b: [f64; 3]) -> f64 {
    let cross =
        core::array::from_fn(|i| a[(i + 1) % 3] * b[(i + 2) % 3] - a[(i + 2) % 3] * b[(i + 1) % 3]);
    let dot = a.iter().zip(b).map(|(a, b)| a * b).sum();
    norm(cross).atan2(dot).to_degrees() * 3600.0
}

#[test]
fn positions_match_independent_de440s_fixture() {
    const EARTH: usize = 3;
    let mut first = None;
    let mut last = 0.0;
    for row in include_str!("fixtures/de440s_vsop87e.csv")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let mut fields = row.split(',').map(|field| field.parse::<f64>().unwrap());
        let jd = fields.next().unwrap();
        first.get_or_insert(jd);
        assert!(jd > last);
        last = jd;
        let jpl: [[f64; 3]; 9] =
            core::array::from_fn(|_| core::array::from_fn(|_| fields.next().unwrap()));
        assert!(fields.next().is_none());
        let actual = compact(jd);
        for (body, (actual_body, jpl_body)) in actual.into_iter().zip(jpl).enumerate() {
            assert!(actual_body.iter().all(|value| value.is_finite()));
            if body == 0 {
                // The solar vector is short near the SSB. Test its absolute
                // position, and test the pointing angle from Earth below.
                assert!(norm(difference(actual_body, jpl_body)) < 9e-6);
            } else {
                assert!(
                    arcseconds(actual_body, jpl_body) < 8.0,
                    "body {body}, JD {jd}"
                );
            }
            if body != EARTH {
                let angle = arcseconds(
                    difference(actual_body, actual[EARTH]),
                    difference(jpl_body, jpl[EARTH]),
                );
                assert!(angle < 6.0, "body {body}, JD {jd}, error {angle} arcsec");
            }
        }
    }
    assert_eq!(first, Some(FIRST_JD_TDB));
    assert_eq!(last, f64::from_bits(END_JD_TDB.to_bits() - 1));
}

#[cfg(feature = "vsop")]
#[test]
fn compact_respects_each_cartesian_omission_budget() {
    let budgets = [2e-7, 1e-5, 1.5e-5, 8e-6, 2e-5, 2e-4, 4e-4, 8e-4, 1e-3];
    for index in 0..=256 {
        let jd = FIRST_JD_TDB + (END_JD_TDB - FIRST_JD_TDB) * f64::from(index) / 256.0;
        let full = [
            full::sun(jd),
            full::mercury(jd),
            full::venus(jd),
            full::earth(jd),
            full::mars(jd),
            full::jupiter(jd),
            full::saturn(jd),
            full::uranus(jd),
            full::neptune(jd),
        ];
        for (body, (compact, full)) in compact(jd).into_iter().zip(full).enumerate() {
            for delta in difference(compact, full) {
                assert!(
                    delta.abs() <= budgets[body] + 1e-12,
                    "body {body}, JD {jd}, delta {delta} AU"
                );
            }
        }
    }
}
