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

#[cfg(feature = "vsop")]
fn full_positions(jd: f64) -> [[f64; 3]; 9] {
    [
        full::sun(jd),
        full::mercury(jd),
        full::venus(jd),
        full::earth(jd),
        full::mars(jd),
        full::jupiter(jd),
        full::saturn(jd),
        full::uranus(jd),
        full::neptune(jd),
    ]
}

#[test]
fn every_checked_body_enforces_the_tdb_interval() {
    use vsop_compact::CompactEphemerisError::{NonFiniteEpoch, OutsideValidityInterval};

    macro_rules! check {
        ($($checked:ident => $unchecked:ident),+ $(,)?) => {
            $(
                for jd in [FIRST_JD_TDB, FIRST_JD_TDB + 4321.25, END_JD_TDB.next_down()] {
                    assert_eq!(vsop_compact::$checked(jd), Ok(vsop_compact::$unchecked(jd)));
                }
                for jd in [FIRST_JD_TDB.next_down(), END_JD_TDB, END_JD_TDB.next_up()] {
                    assert_eq!(vsop_compact::$checked(jd), Err(OutsideValidityInterval));
                }
                for jd in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
                    assert_eq!(vsop_compact::$checked(jd), Err(NonFiniteEpoch));
                }
            )+
        };
    }
    check!(
        try_sun => sun,
        try_mercury => mercury,
        try_venus => venus,
        try_earth => earth,
        try_mars => mars,
        try_jupiter => jupiter,
        try_saturn => saturn,
        try_uranus => uranus,
        try_neptune => neptune,
    );
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

type Positions = [[f64; 3]; 9];

fn de440s_fixture() -> impl Iterator<Item = (f64, Positions)> {
    include_str!("fixtures/de440s_vsop87e.csv")
        .lines()
        .filter(|line| !line.starts_with('#'))
        .map(|row| {
            let mut fields = row.split(',').map(|field| field.parse::<f64>().unwrap());
            let jd = fields.next().unwrap();
            let jpl = core::array::from_fn(|_| core::array::from_fn(|_| fields.next().unwrap()));
            assert!(fields.next().is_none());
            (jd, jpl)
        })
}

fn assert_de440s_agreement(jd: f64, actual: Positions, jpl: Positions) {
    const EARTH: usize = 3;
    for (body, (actual_body, jpl_body)) in actual.into_iter().zip(jpl).enumerate() {
        assert!(actual_body.iter().all(|value| value.is_finite()));
        if body == 0 {
            // The solar vector is short near the SSB. Test its absolute
            // position, and test the pointing angle from Earth below.
            let distance = norm(difference(actual_body, jpl_body));
            assert!(distance < 9e-6, "Sun, JD {jd}, SSB error {distance} AU");
        } else {
            let angle = arcseconds(actual_body, jpl_body);
            assert!(
                angle < 8.0,
                "body {body}, JD {jd}, SSB error {angle} arcsec"
            );
        }
        if body != EARTH {
            let angle = arcseconds(
                difference(actual_body, actual[EARTH]),
                difference(jpl_body, jpl[EARTH]),
            );
            assert!(
                angle < 6.0,
                "body {body}, JD {jd}, geocentric error {angle} arcsec"
            );
        }
    }
}

#[test]
fn compact_positions_match_independent_de440s_fixture() {
    let mut last = None;
    let mut count = 0;
    for (jd, jpl) in de440s_fixture() {
        if let Some(previous) = last {
            assert!(jd > previous);
        } else {
            assert_eq!(jd, FIRST_JD_TDB);
        }
        last = Some(jd);
        count += 1;
        assert_de440s_agreement(jd, compact(jd), jpl);
    }
    assert_eq!(last, Some(END_JD_TDB.next_down()));
    assert!(
        count >= 50,
        "stress case coverage dropped to {count} epochs"
    );
}

#[cfg(feature = "vsop")]
#[test]
fn full_positions_match_independent_de440s_fixture() {
    for (jd, jpl) in de440s_fixture() {
        assert_de440s_agreement(jd, full_positions(jd), jpl);
    }
}

#[cfg(feature = "vsop")]
#[test]
fn compact_respects_each_cartesian_omission_budget() {
    let budgets = [2e-7, 1e-5, 1.5e-5, 8e-6, 2e-5, 2e-4, 4e-4, 8e-4, 1e-3];
    for index in 0..=256 {
        let jd = FIRST_JD_TDB + (END_JD_TDB - FIRST_JD_TDB) * f64::from(index) / 256.0;
        let full = full_positions(jd);
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
