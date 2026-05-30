// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! NRLMSISE-00 tabulated density profile.
//!
//! ## Table layout
//!
//! Each entry is `(altitude: Kilometers, density: KilogramsPerCubicMeter)`.
//! The table covers 50–1000 km altitude at representative solar activity
//! (F10.7 = 140, Ap = 15).  Accuracy is ±50 % under mean conditions —
//! suitable for teaching and low-fidelity propagation.
//!
//! ## Source
//!
//! Approximate NRLMSISE-00 output derived from Picone et al. (2002) and
//! Vallado Table 8-4.  Raw data: `raw/nrlmsise_table.csv`.

include!(concat!(env!("OUT_DIR"), "/atmosphere_data.rs"));
