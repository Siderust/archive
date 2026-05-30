// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Source URLs and environment variables for IERS time-scale data.

/// Environment variable used by `tempoch` to locate IERS data files.
pub const TEMPOCH_DATA_DIR_ENV: &str = "TEMPOCH_DATA_DIR";

/// Authoritative UTC-TAI history from the IERS Paris observatory.
pub const UTC_TAI_HISTORY_URL: &str = "https://hpiers.obspm.fr/eoppc/bul/bulc/UTC-TAI.history";

/// Observed ΔT (TT − UT1) time series from USNO.
pub const DELTA_T_OBSERVED_URL: &str = "https://maia.usno.navy.mil/ser7/deltat.data";

/// ΔT predictions (short-range extrapolation) from USNO.
pub const DELTA_T_PREDICTIONS_URL: &str = "https://maia.usno.navy.mil/ser7/deltat.preds";

/// IERS Earth Orientation Parameters (finals2000A.all).
pub const EOP_FINALS_URL: &str = "https://datacenter.iers.org/data/9/finals2000A.all";

/// Approximate TAI − UTC offset used for pre-1961 epochs (seconds).
pub const PRE_1961_TAI_MINUS_UTC_APPROX: f64 = 10.0;
