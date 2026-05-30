// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! ELP2000-82B dataset source references.

/// Base URL of the CDS VizieR FTP mirror for ELP2000-82B.
pub const BASE_URL: &str = "https://cdsarc.cds.unistra.fr/ftp/VI/79/";

/// Number of sub-series files (ELP1 through ELP36).
pub const FILE_COUNT: u8 = 36;

/// Validity interval for the ELP2000-82B series (formal definition).
pub const START_YEAR: i32 = -2000;
/// See [`START_YEAR`].
pub const END_YEAR: i32 = 4000;
