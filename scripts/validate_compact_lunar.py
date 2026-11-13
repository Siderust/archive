#!/usr/bin/env python3
# SPDX-License-Identifier: BSD-3-Clause
# Copyright (C) 2026 Vallés Puig, Ramon

"""Validate Rust lunar positions and two alternative models with local DE440s."""

import argparse
import hashlib
import io
from pathlib import Path
import subprocess

import erfa
from jplephem.spk import SPK
import numpy as np

FIRST = 2458849.5
END = 2470172.5
J2000 = 2451545.0
AU_KM = 149597870.7
ARCSEC = 180 * 3600 / np.pi
SHA256 = "c1c7feeab882263fc493a9d5a5b2ddd71b54826cdf65d8d17a76126b260a49f2"
# ELP2000-82B notice, section 8: native dynamical ecliptic to FK5.
NATIVE_TO_ICRS = erfa.fk5hip()[0] @ np.array([
    [1.0, 0.000000437913, -0.000000189859],
    [-0.000000477299, 0.917482137607, -0.397776981701],
    [0.0, 0.397776981701, 0.917482137607],
])
SITES = {
    "Equator (0°, 0°, 0 m)": (0, 0, 0),
    "Mid latitude (45°, 100° E, 2,000 m)": (45, 100, 2000),
    "High latitude (75° N, 30° W, 100 m)": (75, -30, 100),
    "South highlands (23° S, 67° W, 5,000 m)": (-23, -67, 5000),
}


def moon(kernel, dates):
    # Both DE440s segments are relative to the Earth-Moon barycenter (3).
    # NAIF 301 is the lunar center, 399 is the Earth center.
    return (kernel[3, 301].compute(dates) - kernel[3, 399].compute(dates)).T


def angle(a, b):
    return np.arctan2(np.linalg.norm(np.cross(a, b), axis=-1), np.sum(a * b, axis=-1))


def validation_dates(kernel):
    grid = np.arange(FIRST, END, 0.25)
    lunar = moon(kernel, grid)
    earth = kernel[0, 3].compute(grid) + kernel[3, 399].compute(grid)
    solar = kernel[0, 10].compute(grid).T - earth.T
    elongation = angle(solar, lunar)
    radius = np.linalg.norm(lunar, axis=-1)
    declination = np.arctan2(lunar[:, 2], np.hypot(lunar[:, 0], lunar[:, 1]))
    event_dates = []
    for values in [radius, -radius, declination, -declination, elongation,
                   -elongation, abs(elongation - np.pi / 2)]:
        local = np.flatnonzero((values[1:-1] < values[:-2]) & (values[1:-1] < values[2:])) + 1
        best = local[np.argsort(values[local], kind="stable")[:8]]
        event_dates.extend(grid[best, None] + np.arange(-8, 9) / 48)
    rng = np.random.default_rng(82440)
    dates = np.unique(np.r_[grid, rng.uniform(FIRST, END, 8192), np.ravel(event_dates),
                            FIRST, np.nextafter(END, FIRST)])
    return dates[(dates >= FIRST) & (dates < END)]


def observer_frames(dates):
    # The same TT≈TDB, UT1≈TT−69 s and zero polar motion on both sides.
    days = dates - J2000
    celestial_to_terrestrial = erfa.c2t06a(J2000, days, J2000, days - 69 / 86400, 0, 0)
    for name, (lat, lon, height) in SITES.items():
        lat, lon = np.deg2rad([lat, lon])
        position = erfa.gd2gc(1, lon, lat, height) / 1000
        zenith = np.array([np.cos(lat) * np.cos(lon), np.cos(lat) * np.sin(lon), np.sin(lat)])
        yield name, position @ celestial_to_terrestrial, zenith @ celestial_to_terrestrial


def compiled_model(binary, dates):
    process = subprocess.run(
        [str(binary.resolve()), "dump"], check=True, capture_output=True, text=True,
        input="".join(f"{jd:.17g}\n" for jd in dates),
    )
    rows = np.loadtxt(io.StringIO(process.stdout), delimiter=",")
    np.testing.assert_array_equal(rows[:, 0], dates)
    assert np.isfinite(rows).all()
    return rows[:, 1:] @ NATIVE_TO_ICRS.T


def chebyshev(kernel, dates, width=16, degree=8):
    # Fit at first-kind Chebyshev nodes, independently of the validation set.
    segments = int(np.ceil((END - FIRST) / width))
    nodes = np.cos(np.pi * (np.arange(degree + 1) + 0.5) / (degree + 1))
    training = FIRST + width * (np.arange(segments)[:, None] + (nodes + 1) / 2)
    vectors = moon(kernel, training.ravel()).reshape(segments, len(nodes), 3)
    coefficients = np.linalg.solve(np.polynomial.chebyshev.chebvander(nodes, degree), vectors)
    # The central training node can coincide with a regular test epoch.
    holdout = ~np.isin(dates, training)
    relative = (dates[holdout] - FIRST) / width
    indices = relative.astype(int)
    x = 2 * (relative - indices) - 1
    # NumPy's chebval uses Clenshaw. Compare f32 storage with the f64 fit.
    def evaluate(table):
        return np.polynomial.chebyshev.chebval(x[:, None], table[indices].transpose(1, 0, 2),
                                              tensor=False)
    packed = coefficients.astype(np.float32)
    return evaluate(packed.astype(np.float64)), evaluate(coefficients), packed.nbytes, holdout


def metrics(label, values):
    absolute = abs(values)
    data = [absolute.max(), absolute.mean(), np.sqrt(np.mean(absolute**2)),
            *np.percentile(absolute, [95, 99])]
    print(f"| {label} | " + " | ".join(f"{value:.6f}" for value in data) + " |")


def report(title, model, reference, sites):
    native_model = model @ NATIVE_TO_ICRS
    native_ref = reference @ NATIVE_TO_ICRS

    def spherical(vectors):
        x, y, z = vectors.T
        return np.arctan2(y, x), np.arctan2(z, np.hypot(x, y)), np.linalg.norm(vectors, axis=-1)

    lon, lat, radius = spherical(native_model)
    lon_ref, lat_ref, radius_ref = spherical(native_ref)
    print(f"\n### {title}\n")
    print("| Error | Max | Mean absolute | RMS | P95 | P99 |")
    print("|---|---:|---:|---:|---:|---:|")
    metrics("Longitude (″)", ((lon - lon_ref + np.pi) % (2 * np.pi) - np.pi) * ARCSEC)
    metrics("Latitude (″)", (lat - lat_ref) * ARCSEC)
    metrics("Geocentric angle (″)", angle(model, reference) * ARCSEC)
    metrics("Cartesian (km)", np.linalg.norm(model - reference, axis=-1))
    metrics("Radius (km)", radius - radius_ref)
    metrics("Relative radius (ppm)", (radius - radius_ref) / radius_ref * 1e6)
    for name, observer, zenith in sites:
        reference_topo = reference - observer
        separation = angle(model - observer, reference_topo) * ARCSEC
        metrics(f"Topocentric {name} (″)", separation)
        if title == "Compact ELP vs DE440s":
            altitude = np.rad2deg(np.pi / 2 - angle(reference_topo, zenith))
            for mask, label in [
                ((altitude >= 0) & (altitude < 15), "0–15°"),
                ((altitude >= 15) & (altitude < 45), "15–45°"),
                (altitude >= 45, "≥45°"),
            ]:
                if mask.any():
                    metrics(f"Topocentric {name}, altitude {label} (″)", separation[mask])
            radial_only = reference * (radius / radius_ref)[:, None]
            metrics(f"Distance-only parallax {name} (″)",
                    angle(radial_only - observer, reference_topo) * ARCSEC)


def fixture(path, kernel, dates, reference, compact, sites):
    radius = np.linalg.norm(reference, axis=-1)
    declination = np.arctan2(reference[:, 2], np.hypot(reference[:, 0], reference[:, 1]))
    solar = (kernel[0, 10].compute(dates) - kernel[0, 3].compute(dates)
             - kernel[3, 399].compute(dates)).T
    elongation = angle(solar, reference)
    indices = {0, len(dates) - 1, *np.linspace(0, len(dates) - 1, 16, dtype=int)}
    for values in [radius, declination, elongation]:
        indices.update([values.argmin(), values.argmax()])
    indices.add(abs(elongation - np.pi / 2).argmin())
    for error in [angle(reference, compact), np.linalg.norm(compact - reference, axis=-1),
                  abs(radius - np.linalg.norm(compact, axis=-1))]:
        indices.add(error.argmax())
    for _, observer, zenith in sites:
        direction = reference - observer
        indices.add(angle(compact - observer, direction).argmax())
        indices.add(abs(angle(direction, zenith) - np.pi / 2).argmin())
    indices = sorted(indices)
    columns = [dates[indices], (reference @ NATIVE_TO_ICRS)[indices]]
    columns.extend((observer @ NATIVE_TO_ICRS)[indices] for _, observer, _ in sites)
    with path.open("w", newline="\n") as out:
        print(f"# DE440s SHA-256 {SHA256}", file=out)
        print("# JD TDB, lunar XYZ, four observer XYZ vectors in report order (native ELP J2000, km)", file=out)
        np.savetxt(out, np.column_stack(columns), delimiter=",", fmt=["%.17g"] + ["%.11e"] * 15)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kernel", type=Path)
    parser.add_argument("--models", type=Path, nargs=2, required=True, metavar=("FULL", "COMPACT"))
    parser.add_argument("--fixture", type=Path)
    args = parser.parse_args()
    with args.kernel.open("rb") as stream:
        if hashlib.file_digest(stream, "sha256").hexdigest() != SHA256:
            parser.error("DE440s SHA-256 mismatch")

    kernel = SPK.open(args.kernel)
    try:
        dates = validation_dates(kernel)
        reference = moon(kernel, dates)
        full, compact = (compiled_model(path, dates) for path in args.models)
        sites = list(observer_frames(dates))
        packed, unpacked, size, holdout = chebyshev(kernel, dates)
        if args.fixture:
            fixture(args.fixture, kernel, dates, reference, compact, sites)
    finally:
        kernel.close()
    print(f"Validated {len(dates):,} TDB epochs from {dates[0]} through {dates[-1]}.")
    report("Compact ELP vs DE440s", compact, reference, sites)
    report("Full ELP vs DE440s", full, reference, sites)
    report("Compact ELP vs full ELP", compact, full, sites)
    report("ERFA Moon98 vs DE440s", erfa.moon98(J2000, dates - J2000)["p"] * AU_KM, reference, sites)
    print(f"\nChebyshev holdout: {holdout.sum():,} epochs after removing fitting nodes.")
    cheb_sites = [(name, observer[holdout], zenith[holdout]) for name, observer, zenith in sites]
    report(f"Chebyshev f32 vs DE440s ({size:,} coefficient bytes)", packed, reference[holdout], cheb_sites)
    report("Chebyshev f32 vs f64 fit", packed, unpacked, cheb_sites)


if __name__ == "__main__":
    main()
