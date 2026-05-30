// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Source references for the IAU 2000A / 2000B nutation series.
//!
//! The nutation tables are derived from the MHB2000 model.  No authoritative
//! machine-readable download URL exists; the normalized Rust form in
//! [`super::tables`] was produced from the published SOFA implementation.
//!
//! ## References
//!
//! * Mathews, P. M., Herring, T. A., & Buffett, B. A. (2002).
//!   Modeling of nutation and precession: New nutation series for nonrigid
//!   Earth and insights into the Earth's interior. JGR 107 (B4), 2068.
//!   <https://doi.org/10.1029/2001JB000390>
//! * IERS Conventions (2010), ch. 5.
//!   <https://www.iers.org/IERS/EN/Publications/TechnicalNotes/tn36.html>
//! * SOFA software: <https://www.iausofa.org>
