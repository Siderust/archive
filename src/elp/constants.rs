// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! # ELP2000-82B Algorithm Constants
//!
//! Physical and algorithmic constants used by the ELP2000-82B lunar theory
//! implementation.  All angular quantities use [`qtty::Radian`]; distances
//! use [`qtty::Kilometer`].
//!
//! ## Sources
//!
//! - Chapront-Touzé, M., & Chapront, J. (1988). "ELP 2000-82B".
//!   *Astronomy and Astrophysics* 190, 342–352.
//! - Laskar, J. (1986). "Secular terms of classical planetary theories using
//!   the results of general theory". *A&A* 157, 59–70.
//! - Lieske, J. H. et al. (1977). "Expressions for the precession quantities
//!   based upon the IAU (1976) system of astronomical constants".
//!   *A&A* 58, 1–16.

use qtty::{Kilometer, Radian};

// ── Physical lengths ──────────────────────────────────────────────────────

/// Mean distance Earth–Moon used as semi-major axis reference.
///
/// ELP2000-82B, Chapront-Touzé & Chapront (1988), Eq. (1).
pub const A0: Kilometer = Kilometer::new(384_747.980_644_895_4);

/// Mean distance Earth–Moon used as distance unit in the theory.
///
/// Differs from [`A0`] by a sub-metre factor arising from the choice of
/// reference frame; ELP2000-82B, Table 1.
pub const ATH: Kilometer = Kilometer::new(384_747.980_674_316_5);

// ── Dimensionless ratios ──────────────────────────────────────────────────

/// Ratio of the Moon's mean motion to the Sun's mean motion.
///
/// `AM = n_Moon / n_Sun` (ELP2000-82B, Table 1).
pub const AM: f64 = 0.074_801_329_518;

/// Ratio of the Earth's equatorial radius to the mean Earth–Moon distance.
///
/// `ALPHA = a_E / A0` (ELP2000-82B, Table 1).
pub const ALPHA: f64 = 0.002_571_881_335;

/// Derived coefficient `2·ALPHA / (3·AM)`, appears in the ELP series scaling.
pub const DTASM: f64 = 0.022_921_886_117_733_68;

// ── Precession constant ───────────────────────────────────────────────────

/// IAU 1976 lunisolar precession rate (rad / Julian century).
///
/// Source value: 5 029.096 6″/Julian-century (Lieske et al. 1977).
pub const PRECES: Radian = Radian::new(0.024_381_748_353_014_515);

// ── Precession matrix coefficients (Laskar 1986) ──────────────────────────
//
// These are the off-diagonal elements of the rotation matrix that converts
// coordinates from the mean ecliptic/equinox of date to the J2000.0 frame.
// Dimensionless — no qtty wrapper.

/// Laskar (1986) precession matrix coefficient P₁.
pub const P1: f64 = 0.10180391e-4;
/// Laskar (1986) precession matrix coefficient P₂.
pub const P2: f64 = 0.47020439e-6;
/// Laskar (1986) precession matrix coefficient P₃.
pub const P3: f64 = -0.5417367e-9;
/// Laskar (1986) precession matrix coefficient P₄.
pub const P4: f64 = -0.2507948e-11;
/// Laskar (1986) precession matrix coefficient P₅.
pub const P5: f64 = 0.463486e-14;

/// Laskar (1986) precession matrix coefficient Q₁.
pub const Q1: f64 = -0.113469002e-3;
/// Laskar (1986) precession matrix coefficient Q₂.
pub const Q2: f64 = 0.12372674e-6;
/// Laskar (1986) precession matrix coefficient Q₃.
pub const Q3: f64 = 0.1265417e-8;
/// Laskar (1986) precession matrix coefficient Q₄.
pub const Q4: f64 = -0.1371808e-11;
/// Laskar (1986) precession matrix coefficient Q₅.
pub const Q5: f64 = -0.320334e-14;

// ── ELP2000-82B numerical corrections ────────────────────────────────────
//
// Corrections to the ELP fundamental arguments derived from LLR data
// (ELP2000-82B, Section 2.3).

/// Correction to the Moon's mean motion Δν.
pub const DELNU: Radian = Radian::new(3.209_356_158_623_529e-10);
/// Correction to the Moon's eccentricity Δe.
pub const DELE: Radian = Radian::new(8.673_316_755_049_599e-8);
/// Correction to the Earth's eccentricity γ Δγ.
pub const DELG: Radian = Radian::new(-3.910_507_151_829_518e-7);
/// Correction to the precession in longitude Δn'.
pub const DELNP: Radian = Radian::new(-3.707_809_503_452_549e-11);
/// Correction to the obliquity Δε.
pub const DELEP: Radian = Radian::new(-6.243_915_399_009_714e-7);

// ── Delaunay arguments (polynomial coefficients in Julian millennia) ──────
//
// `DEL[i][k]` is the coefficient of `T^k` (T in Julian millennia from J2000)
// for Delaunay argument `i`:  0 = D, 1 = M (Sun), 2 = M′ (Moon), 3 = F.

#[allow(clippy::all)]
pub const DEL: [[Radian; 5]; 4] = [
    [
        Radian::new(5.198_466_741_027_443),
        Radian::new(7_771.377_146_811_758_394),
        Radian::new(-2.844_935_162_1e-5),
        Radian::new(3.197_346_2e-8),
        Radian::new(-1.543_65e-10),
    ],
    [
        Radian::new(-0.043_125_180_208_125),
        Radian::new(628.301_955_168_488_007),
        Radian::new(-2.680_534_843_0e-6),
        Radian::new(7.126_76e-10),
        Radian::new(7.270_0e-13),
    ],
    [
        Radian::new(2.355_555_898_265_799),
        Radian::new(8_328.691_426_955_554_562),
        Radian::new(1.570_277_576_16e-4),
        Radian::new(2.504_111_14e-7),
        Radian::new(-1.186_339e-9),
    ],
    [
        Radian::new(1.627_905_233_371_468),
        Radian::new(8_433.466_158_130_539_043),
        Radian::new(-5.939_210_000_4e-5),
        Radian::new(-4.949_948e-9),
        Radian::new(2.021_7e-11),
    ],
];

// ── Fundamental lunar arguments: longitude offset and rate ────────────────
//
// `ZETA[0]` is the epoch offset; `ZETA[1]` is the rate (rad / Julian
// millennium), including the IAU 1976 precession correction.

pub const ZETA: [Radian; 2] = [
    Radian::new(3.810_344_430_588_308), // (218° 18′ 59.95571″) in rad
    Radian::new(8_399.709_113_522_269), // 1 732 559 343.736 04″ + PRECES, in rad/millennium
];

// ── Planetary argument constants ──────────────────────────────────────────
//
// `P_ARGS[p] = [offset, rate]` for planets 0–7:
//   0 = Mercury, 1 = Venus, 2 = Earth-Moon Barycenter, 3 = Mars,
//   4 = Jupiter, 5 = Saturn, 6 = Uranus, 7 = Neptune.

#[allow(clippy::all)]
pub const P_ARGS: [[Radian; 2]; 8] = [
    [
        Radian::new(4.402_608_842_402_962),
        Radian::new(2_608.790_314_157_411),
    ],
    [
        Radian::new(3.176_146_696_907_594),
        Radian::new(1_021.328_554_621_109),
    ],
    [
        Radian::new(1.753_470_343_150_658),
        Radian::new(628.307_584_962_155),
    ],
    [
        Radian::new(6.203_480_913_399_945),
        Radian::new(334.061_243_149_230),
    ],
    [
        Radian::new(0.599_546_497_388_674),
        Radian::new(52.969_096_509_472),
    ],
    [
        Radian::new(0.874_016_756_518_481),
        Radian::new(21.329_909_543_800),
    ],
    [
        Radian::new(5.481_293_871_604_991),
        Radian::new(7.478_159_856_714),
    ],
    [
        Radian::new(5.311_886_286_783_467),
        Radian::new(3.813_303_563_758),
    ],
];

// ── Lunar rotation series (W1 polynomial in Julian millennia) ─────────────
//
// `W1[k]` is the coefficient of `T^k`.

pub const W1: [Radian; 5] = [
    Radian::new(3.810_344_430_588_308), // epoch offset (218° 18′ 59.95571″)
    Radian::new(8_399.684_731_773_916), // rate: 1 732 559 343.736 04″ in rad/millennium
    Radian::new(-2.854_728_398_477_281e-5), // T² coefficient
    Radian::new(3.201_709_550_047_375e-8), // T³ coefficient
    Radian::new(-1.536_374_555_436_12e-10), // T⁴ coefficient
];
