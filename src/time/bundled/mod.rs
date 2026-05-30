// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! Compiled UTC-TAI and ΔT fallback tables checked into the archive crate.

pub mod snapshot;

pub use snapshot::{
    DELTA_T_OBSERVED_URL, DELTA_T_PREDICTIONS_URL, MODERN_DELTA_T_END_MJD,
    MODERN_DELTA_T_OBSERVED_END_MJD, MODERN_DELTA_T_POINTS, MODERN_DELTA_T_START_MJD,
    PRE_1961_TAI_MINUS_UTC_APPROX, UTC_TAI_HISTORY_START_MJD, UTC_TAI_HISTORY_URL,
    UTC_TAI_SEGMENTS,
};

use super::{TimeDataBundle, TimeDataProvenance, UtcTaiSegment};

/// Build the compiled-in bundled time-data snapshot.
///
/// The bundle contains UTC-TAI history and ΔT tables compiled into the crate.
/// **EOP data is not included**; callers needing Earth orientation parameters
/// must load a cached bundle via [`super::TimeDataManager`] (feature `fetch`).
pub fn bundled_time_data() -> TimeDataBundle {
    TimeDataBundle::new(
        snapshot::UTC_TAI_SEGMENTS
            .iter()
            .map(|s| UtcTaiSegment {
                start_mjd: s.start_mjd,
                end_mjd: s.end_mjd,
                base_seconds: s.base_seconds,
                reference_mjd: s.reference_mjd,
                slope_seconds_per_day: s.slope_seconds_per_day,
            })
            .collect(),
        snapshot::MODERN_DELTA_T_POINTS.to_vec(),
        snapshot::MODERN_DELTA_T_OBSERVED_END_MJD,
        Vec::new(),
        TimeDataProvenance::new("compiled", "compiled", "compiled", "compiled", "compiled"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_time_data_has_utc_tai_segments() {
        let bundle = bundled_time_data();
        assert!(!bundle.utc_tai_segments().is_empty());
    }

    #[test]
    fn bundled_time_data_has_no_eop() {
        let bundle = bundled_time_data();
        assert!(bundle.eop_points().is_empty());
    }
}
