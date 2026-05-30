// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! EGM2008 normalized Stokes coefficients through degree/order 4.
//!
//! ## Coefficient layout
//!
//! Each entry is `(n, m, C̄ₙₘ, S̄ₙₘ)` where `n` is degree, `m` is order.
//! The C̄ and S̄ values are fully normalized per the geodetic convention.
//!
//! ## Source
//!
//! Coefficients are from the official EGM2008 product published by the NGA.
//! Provenance: Pavlis et al. (2012), JGR 117 B04406.  Raw data:
//! `raw/egm2008_n2_4.csv`.

include!(concat!(env!("OUT_DIR"), "/gravity_data.rs"));
