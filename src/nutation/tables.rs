// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! IAU 2000A and IAU 2000B nutation coefficient tables.
//!
//! ## Scientific scope
//!
//! The IAU 2000A series (MHB2000) is the definitive luni-solar and planetary
//! nutation model adopted by the International Astronomical Union.  It describes
//! the periodic wobble of Earth's rotation axis (nutation in longitude Δψ and
//! obliquity Δε) as a sum of 678 luni-solar trigonometric terms and 687
//! planetary trigonometric terms.  The truncated IAU 2000B variant (77
//! luni-solar terms + a fixed planetary correction) reproduces IAU 2000A to
//! better than ≈ 1 mas.
//!
//! ## Sources
//!
//! * MHB2000: Mathews, Herring & Buffett (2002), JGR 107 (B4), §2263.
//! * SOFA routines `iauNut00a`, `iauNut00b`.
//! * IERS Conventions (2010), ch. 5, Tables 5.3a/5.3b.
//! * McCarthy & Luzum (2003) for IAU 2000B planetary corrections.
//!
//! ## Coefficient units
//!
//! All terms are in 0.1 µas (microarcseconds); t-dependent terms are in
//! 0.1 µas/century.  Raw data: `raw/nut00a_ls.csv`, `raw/nut00a_pl.csv`,
//! `raw/nut00b_ls.csv`.

include!(concat!(env!("OUT_DIR"), "/nutation_data.rs"));
