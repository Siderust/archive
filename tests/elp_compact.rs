// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

#![cfg(feature = "elp-compact")]

use siderust_archive::elp_compact::{
    moon_geocentric, try_moon_geocentric, CompactLunarError, END_JD_TDB, FIRST_JD_TDB,
};

#[cfg(feature = "elp")]
#[path = "../examples/support/full_elp.rs"]
mod full;

fn difference(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    core::array::from_fn(|i| a[i] - b[i])
}

fn norm(vector: [f64; 3]) -> f64 {
    vector.iter().map(|x| x * x).sum::<f64>().sqrt()
}

fn angle(a: [f64; 3], b: [f64; 3]) -> f64 {
    let cross =
        core::array::from_fn(|i| a[(i + 1) % 3] * b[(i + 2) % 3] - a[(i + 2) % 3] * b[(i + 1) % 3]);
    let dot = a.iter().zip(b).map(|(x, y)| x * y).sum();
    norm(cross).atan2(dot).to_degrees() * 3600.0
}

#[test]
fn checks_tdb_interval() {
    for jd in [FIRST_JD_TDB, END_JD_TDB.next_down()] {
        assert_eq!(try_moon_geocentric(jd), Ok(moon_geocentric(jd)));
    }
    for jd in [FIRST_JD_TDB.next_down(), END_JD_TDB, END_JD_TDB.next_up()] {
        assert_eq!(
            try_moon_geocentric(jd),
            Err(CompactLunarError::OutsideValidityInterval)
        );
    }
    for jd in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        assert_eq!(
            try_moon_geocentric(jd),
            Err(CompactLunarError::NonFiniteEpoch)
        );
    }
}

#[test]
fn agrees_with_de440s_at_geocentric_and_topocentric_stress_epochs() {
    let mut previous = None;
    let mut count = 0;
    for line in include_str!("fixtures/de440s_lunar.csv")
        .lines()
        .filter(|line| !line.starts_with('#'))
    {
        let mut columns = line.split(',').map(|field| field.parse::<f64>().unwrap());
        let jd = columns.next().unwrap();
        if let Some(last) = previous {
            assert!(jd > last);
        } else {
            assert_eq!(jd, FIRST_JD_TDB);
        }
        previous = Some(jd);
        count += 1;
        let expected = core::array::from_fn(|_| columns.next().unwrap());
        let actual = try_moon_geocentric(jd)
            .unwrap()
            .map(|coordinate| coordinate.value());
        assert!(actual.iter().all(|value| value.is_finite()));
        assert!(
            norm(difference(actual, expected)) < 5.0,
            "Cartesian error at {jd}"
        );
        assert!(
            (norm(actual) - norm(expected)).abs() < 2.0,
            "radial error at {jd}"
        );
        assert!(angle(actual, expected) < 4.0, "geocentric error at {jd}");

        let longitude = |[x, y, _]: [f64; 3]| y.atan2(x);
        let latitude = |[x, y, z]: [f64; 3]| z.atan2(x.hypot(y));
        let delta_lon = longitude(actual) - longitude(expected);
        assert!(delta_lon.sin().atan2(delta_lon.cos()).to_degrees().abs() * 3600.0 < 4.0);
        assert!((latitude(actual) - latitude(expected)).to_degrees().abs() * 3600.0 < 4.0);
        for _ in 0..4 {
            let observer = core::array::from_fn(|_| columns.next().unwrap());
            assert!(
                angle(difference(actual, observer), difference(expected, observer)) < 4.0,
                "topocentric error at {jd}"
            );
        }
        assert!(columns.next().is_none());
    }
    assert_eq!(previous, Some(END_JD_TDB.next_down()));
    assert!(count >= 24);
}

#[cfg(feature = "elp")]
#[test]
fn full_baseline_matches_imcce_notice_table_h() {
    for (jd, expected) in [
        (
            2_469_000.5,
            [-361_602.985_36, 44_996.995_10, -30_696.653_16],
        ),
        (
            2_449_000.5,
            [-363_132.342_48, 35_863.653_78, -33_196.004_09],
        ),
    ] {
        assert!(norm(difference(full::moon(jd), expected)) < 0.000_02);
    }
}

#[cfg(feature = "vsop-compact")]
#[test]
fn combined_compact_features_evaluate_at_the_same_epoch() {
    assert!(try_moon_geocentric(FIRST_JD_TDB).is_ok());
    assert!(siderust_archive::vsop_compact::try_earth(FIRST_JD_TDB).is_ok());
}

mod allocations {
    use super::*;
    use std::{
        alloc::{GlobalAlloc, Layout, System},
        cell::Cell,
    };

    std::thread_local! {
        static COUNT: Cell<usize> = const { Cell::new(0) };
    }

    struct CountingAllocator;

    // SAFETY: Forward every operation to System with the original pointer
    // and layout. The counter is thread-local and const-initialized.
    unsafe impl GlobalAlloc for CountingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let _ = COUNT.try_with(|count| count.set(count.get().saturating_add(1)));
            unsafe { System.alloc(layout) }
        }

        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            unsafe { System.dealloc(ptr, layout) }
        }
    }

    #[global_allocator]
    static ALLOCATOR: CountingAllocator = CountingAllocator;

    #[test]
    fn checked_evaluation_allocates_nothing() {
        use std::hint::black_box;
        let before = COUNT.get();
        let result = black_box(try_moon_geocentric(black_box(FIRST_JD_TDB)));
        let after = COUNT.get();
        assert!(result.is_ok());
        assert_eq!(before, after);
    }
}
