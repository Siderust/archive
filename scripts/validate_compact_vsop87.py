#!/usr/bin/env python3
# SPDX-License-Identifier: BSD-3-Clause
# Copyright (C) 2026 Vallés Puig, Ramon

"""Validate the compiled Rust VSOP87E profiles with a local JPL DE440s SPK."""

import argparse
import hashlib
import io
from pathlib import Path
import subprocess

from jplephem.spk import SPK
import numpy as np

FIRST_JD = 2458849.5
END_JD = 2470172.5
LAST_JD = np.nextafter(END_JD, FIRST_JD)
AU_KM = 149597870.7
ARCSEC_PER_RAD = 180.0 / np.pi * 3600
BODIES = ("Sun", "Mercury", "Venus", "Earth", "Mars", "Jupiter", "Saturn", "Uranus", "Neptune")
DE440S_SHA256 = "c1c7feeab882263fc493a9d5a5b2ddd71b54826cdf65d8d17a76126b260a49f2"
EARTH, SUN = 3, 0

# IMCCE vsop87.doc, REFERENCE SYSTEM.
VSOP_TO_FK5 = np.array([
    [1.0, 0.000000440360, -0.000000190919],
    [-0.000000479966, 0.917482137087, -0.397776982902],
    [0.0, 0.397776982902, 0.917482137087],
])
# ERFA eraFk5hip r5h (J2000 orientation only, fixed axes).
FK5_TO_ICRS = np.array([
    [0.9999999999999929, 1.1102233509835495e-7, 4.411803963527301e-8],
    [-1.1102233084981164e-7, 0.9999999999999892, -9.647792498531089e-8],
    [-4.4118050326662195e-8, 9.647792009628368e-8, 0.9999999999999943],
])


def reference(kernel, dates):
    """Return [epoch, body, xyz] in AU and native VSOP87E axes."""
    def position(center, target):
        return kernel[center, target].compute(dates).T

    icrs = np.stack([
        position(0, 10),
        position(0, 1) + position(1, 199),
        position(0, 2) + position(2, 299),
        position(0, 3) + position(3, 399),
        position(0, 4),  # DE440s supplies the Mars system barycenter.
        *(position(0, code) for code in range(5, 9)),
    ], axis=1)
    return (icrs / AU_KM) @ FK5_TO_ICRS @ VSOP_TO_FK5


def angle(a, b):
    return np.arctan2(np.linalg.norm(np.cross(a, b), axis=-1), np.sum(a * b, axis=-1))


def validation_dates(kernel):
    daily = np.arange(FIRST_JD, END_JD, 1.0)
    vectors = reference(kernel, daily)
    geo = vectors - vectors[:, EARTH:EARTH + 1]
    planets = [i for i in range(len(BODIES)) if i not in (SUN, EARTH)]
    elongation = angle(geo[:, SUN:SUN + 1], geo[:, planets])
    distance = np.linalg.norm(geo[:, planets], axis=-1)
    events = []
    for values, select_max in [(elongation, False), (elongation, True), (distance, False)]:
        direction = -1 if select_max else 1
        values = values * direction
        mask = (values[1:-1] < values[:-2]) & (values[1:-1] < values[2:])
        indices = np.unique(np.where(mask)[0] + 1)
        events.extend(daily[indices, None] + np.arange(-1.0, 1.25, 0.25))
    rng = np.random.default_rng(872440)
    random = rng.uniform(FIRST_JD, END_JD, 8192)
    return np.unique(np.concatenate([daily, random, np.ravel(events), [FIRST_JD, LAST_JD]]))


def compiled_model(executable, dates):
    run = subprocess.run(
        [str(executable.resolve()), "dump"],
        input="".join(f"{jd:.17g}\n" for jd in dates),
        capture_output=True, check=True, text=True,
    )
    values = np.loadtxt(io.StringIO(run.stdout), delimiter=",")
    np.testing.assert_array_equal(dates, values[:, 0])
    assert np.all(np.isfinite(values))
    return values[:, 1:].reshape(-1, len(BODIES), 3)


def earth_relative(vectors):
    return vectors - vectors[:, EARTH:EARTH + 1]


def write_fixture(path, dates, jpl, full, compact):
    """Select stress cases from the validation samples and write DE440s rows."""
    cases = {}

    def sample(index, label):
        jd = dates[index]
        cases.setdefault(jd, []).append(label)

    sample(0, "first valid JD")
    sample(-1, "last representable valid JD")
    targets = np.linspace(FIRST_JD + 17.25, END_JD - 81.0, 16)
    for index in np.searchsorted(dates, targets):
        sample(index, "representative epoch")
    geo = earth_relative(jpl)
    for body, name in enumerate(BODIES):
        if body not in (SUN, EARTH):
            elongation = angle(geo[:, SUN], geo[:, body])
            distance = np.linalg.norm(geo[:, body], axis=-1)
            if name in ("Mercury", "Venus"):
                # The two classes of conjunction have Earth distances on
                # opposite sides of 1 AU for these planets.
                inferior = np.where(distance < 1.0, elongation, np.inf)
                superior = np.where(distance >= 1.0, elongation, np.inf)
                sample(inferior.argmin(), f"{name} near inferior conjunction")
                sample(superior.argmin(), f"{name} near superior conjunction")
            else:
                sample(elongation.argmin(), f"{name} near conjunction")
                sample(elongation.argmax(), f"{name} near opposition")
            sample(distance.argmin(), f"{name} closest sampled approach")

    geo_compact, geo_full = earth_relative(compact), earth_relative(full)
    for comparison, model, baseline in [
        ("compact vs DE440s", geo_compact, geo),
        ("compact vs full", geo_compact, geo_full),
        ("full vs DE440s", geo_full, geo),
    ]:
        errors = angle(model, baseline)
        for body, name in enumerate(BODIES):
            if body != EARTH:
                sample(errors[:, body].argmax(), f"{name} geocentric peak {comparison}")

    # Earth has no geocentric direction. Include the peak Earth SSB errors
    # and the short solar SSB vector's largest absolute model discrepancy.
    sample(angle(compact[:, EARTH], jpl[:, EARTH]).argmax(), "Earth barycentric peak compact vs DE440s")
    sample(angle(compact[:, EARTH], full[:, EARTH]).argmax(), "Earth barycentric peak compact vs full")
    sample(np.linalg.norm(compact[:, SUN] - jpl[:, SUN], axis=-1).argmax(),
           "Sun barycentric peak Cartesian compact vs DE440s")

    with path.open("w", newline="\n") as out:
        print(f"# DE440s SHA-256 {DE440S_SHA256}", file=out)
        print("# JD TDB; native VSOP87E barycentric X,Y,Z (AU) for " + ", ".join(BODIES), file=out)
        for jd, labels in sorted(cases.items()):
            positions = jpl[np.searchsorted(dates, jd)]
            print("# " + "; ".join(labels), file=out)
            fields = [f"{jd:.17g}"] + [f"{value:.13e}" for value in positions.flat]
            print(",".join(fields), file=out)


def report(label, model, ref):
    print(f"\n### {label}\n")
    print("| Origin | Body | Max ″ | Mean ″ | RMS ″ | P95 ″ | P99 ″ | Max km |")
    print("|---|---|---:|---:|---:|---:|---:|---:|")
    for origin, index in [("SSB", None), ("Earth", EARTH), ("Sun", SUN)]:
        a, b = model, ref
        if index is not None:
            a = a - a[:, index:index + 1]
            b = b - b[:, index:index + 1]
        angular = angle(a, b) * ARCSEC_PER_RAD
        cartesian = np.linalg.norm(a - b, axis=-1) * AU_KM
        for i, body in enumerate(BODIES):
            if i == index:
                continue
            error = angular[:, i]
            metrics = [error.max(), error.mean(), np.sqrt(np.mean(error**2)),
                       *np.percentile(error, [95, 99]), cartesian[:, i].max()]
            print(f"| {origin} | {body} | " + " | ".join(f"{x:.3f}" for x in metrics) + " |")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("kernel", type=Path)
    parser.add_argument("--models", type=Path, nargs=2, required=True, metavar=("FULL", "COMPACT"),
                        help="two release builds of the vsop87_measure example")
    parser.add_argument("--fixture", type=Path, help="write the DE440s fixture instead of the report")
    args = parser.parse_args()
    with args.kernel.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    if digest != DE440S_SHA256:
        parser.error(f"DE440s checksum mismatch ({digest})")

    kernel = SPK.open(args.kernel)
    try:
        dates = validation_dates(kernel)
        jpl = reference(kernel, dates)
    finally:
        kernel.close()

    full, compact = (compiled_model(path, dates) for path in args.models)
    if args.fixture is not None:
        write_fixture(args.fixture, dates, jpl, full, compact)
        return

    print(f"Evaluated {len(dates):,} distinct TDB epochs, JD {dates[0]} to {dates[-1]}.")
    report("Compact vs DE440s", compact, jpl)
    report("Full vs DE440s", full, jpl)
    report("Compact vs full", compact, full)


if __name__ == "__main__":
    main()
