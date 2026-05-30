// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! VSOP87 dataset source references.
//!
//! This module collects the authoritative download location and expected file
//! inventory for the VSOP87 coefficient files as distributed by the IMCCE.

/// Base URL of the IMCCE VSOP87 public mirror.
///
/// The directory listing at this URL contains files named `VSOP87X.yyy` where
/// `X` is the version letter (A, B, C, D, E, …) and `yyy` is the planet
/// extension (e.g. `ear` = Earth, `mar` = Mars, …).
pub const BASE_URL: &str = "https://ftp.imcce.fr/pub/ephem/planets/vsop87/";

/// VSOP87 file-name regex pattern.
///
/// Used by the build pipeline to recognise and filter VSOP87 coefficient files
/// in a local directory or a remote directory listing.
pub const FILE_PATTERN: &str = r"VSOP87[A-Z]\.[A-Za-z]{3}";

/// Validity interval for the VSOP87 series.
///
/// Accuracy degrades outside `[START_YEAR, END_YEAR]` but the series remain
/// formally defined beyond this window.
pub const START_YEAR: i32 = -2000;
/// See [`START_YEAR`].
pub const END_YEAR: i32 = 6000;

/// The two VSOP87 versions compiled into siderust-archive.
///
/// - `A`: heliocentric ecliptic rectangular coordinates, equinox J2000.0.
/// - `E`: barycentric ecliptic rectangular coordinates, equinox J2000.0.
pub const COMPILED_VERSIONS: &[char] = &['A', 'E'];
