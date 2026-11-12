# Compact VSOP87E validation and measurements

The `vsop-compact` feature embeds positions for the Sun and eight planets over
2020-01-01 through 2050-12-31 TDB (JD `[2458849.5, 2470172.5)`). Across 24,418
sampled epochs, the largest geometric Earth-relative angular error against
DE440s was **3.951″** (Neptune). The compact ESP32 application image used
**172,592 bytes**, an **87.75%** reduction from the full VSOP87E harness.

## Model and truncation

VSOP87E supplies a common solar-system barycenter (SSB), including the
barycentric Sun and Earth. At a common epoch, subtracting the Earth or Sun
vector yields geocentric or heliocentric positions in the same frame. VSOP87A
could supply same-epoch relative positions with the Sun at the origin. The
E profile also supplies barycentric positions for the downstream backend,
which can combine states evaluated at different epochs in a common inertial
frame when applying light time.

The collector reads the checked-in [IMCCE VSOP87 coefficients](https://ftp.imcce.fr/pub/ephem/planets/vsop87/).
For each body and axis, rank the terms by
`abs(A) * max(abs(T))^k`, where `T = (JD_TDB - 2451545) / 365250` and
`max(abs(T)) = (2470172.5 - 2451545) / 365250`. Break ties by order and original
index. Discard the smallest terms while the sum of their bounds is within the
axis budget. This gives an interval-wide Cartesian truncation bound relative
to full VSOP87E in exact arithmetic, since each cosine has magnitude at most
one. The generator retains source order within each polynomial block.

Each body has a separate budget, applied independently to each axis. Equal
axis budgets bound the vector error by the same amount in any direction.
The budget reflects
orbital distance and the corresponding angular sensitivity. For a
geocentric vector, the two body budgets add and the angular sensitivity
increases as distance decreases. Earth therefore has a tight budget, and
Venus and Mars have smaller budgets than the outer planets. At a nominal
minimum Venus distance of about 0.25 AU, the combined Cartesian bound
`sqrt(3) * (1.5e-5 + 8e-6)` AU corresponds to about 33″ of truncation error.
The Sun gets the smallest absolute budget because its SSB displacement can
be short. These budgets were chosen using orbital scales and the term
envelopes without fitting positions or residuals to any validation epochs.

| Body | Omission budget per axis (AU) | Full terms | Compact X/Y/Z | Compact total |
|---|---:|---:|---:|---:|
| Sun | 2e-7 | 6,634 | 338 / 341 / 51 | 730 |
| Mercury | 1e-5 | 7,880 | 121 / 124 / 22 | 267 |
| Venus | 1.5e-5 | 4,657 | 90 / 91 / 13 | 194 |
| Earth | 8e-6 | 5,556 | 201 / 203 / 15 | 419 |
| Mars | 2e-5 | 7,575 | 268 / 273 / 31 | 572 |
| Jupiter | 2e-4 | 4,542 | 108 / 108 / 17 | 233 |
| Saturn | 4e-4 | 7,522 | 189 / 189 / 27 | 405 |
| Uranus | 8e-4 | 5,125 | 144 / 141 / 13 | 298 |
| Neptune | 1e-3 | 2,389 | 29 / 28 / 8 | 65 |
| **Total** | | **51,823** | | **3,183** |

Both layouts use three `f64` values per term (24 bytes with 8-byte alignment
on the measured targets). The compact layout uses three contiguous arrays
per body and six `u16` cumulative order offsets per axis. Offsets are
compile-time constants. The representation keeps the source precision.
Evaluation uses `libm` cosine, sums each order in source order, and evaluates
the outer polynomial by Horner's method. All intermediate arithmetic stays
in `f64`, including epoch subtraction and phase calculation. The evaluator
has a fixed loop structure, requires no heap, and supports positions only.
A separate velocity budget would have to account for both the polynomial
derivative and the frequency, which can amplify discarded terms.

JPL's short-range [approximate Keplerian elements](https://ssd.jpl.nasa.gov/planets/approx_pos.html)
use 768 bytes for six elements and six rates for each of eight bodies at
`f64` precision. The 1800–2050 fit uses a Kepler solve and a few rotations per
body, typically fewer trigonometric evaluations. Its nominal heliocentric
longitude errors for Jupiter and Saturn are 400″ and 600″. It gives the
Earth–Moon barycenter and heliocentric elements, so a barycentric backend and
an Earth-center correction would require additional data or models. VSOP87E
already fits the archive generator and provides the common center.

## Independent reference

The reference is [JPL DE440s](https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp),
evaluated from a local SPK using jplephem. The [DE440/DE441 paper](https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440_and_de441.pdf)
describes the model. DE440s supplies the centers of Earth, Mercury and Venus
via their planet-barycenter segments, the Sun center, and system barycenters
for Mars through Neptune. Satellite-induced offsets between the latter
system barycenters and planet centers are included in this comparison
where applicable.

We rotate DE440s ICRF vectors into the native VSOP87 dynamical ecliptic and
equinox of J2000.0. The fixed transformation uses the inverse of the
[IMCCE VSOP87-to-FK5 J2000 rotation](https://ftp.imcce.fr/pub/ephem/planets/vsop87/vsop87.doc)
and the inverse of the [ERFA FK5-to-Hipparcos/ICRS orientation at J2000](https://github.com/liberfa/erfa/blob/master/src/fk5hip.c).
Each inverse uses the rotation transpose. We convert kilometers to AU using
149,597,870.7 km/AU. Both models take the same TDB JD, consistent with
Siderust's VSOP87 evaluation. IMCCE describes its dynamical argument as
approximately TT. Treating that as TDB at the same numerical JD is
negligible at the pointing scale here, since the periodic TT–TDB
difference is millisecond-scale.

All vectors are geometric and simultaneous. The comparison omits light
time, aberration, topocentric parallax, refraction, and Earth orientation.
The angular metric is `atan2(norm(cross), dot)`. We use the Cartesian
difference norm for distance error. The epoch set combines a daily grid,
8,192 seeded uniform samples, both coverage boundaries (the last epoch is
one representable JD below the exclusive end), and six-hour samples within
one day of local daily-grid elongation extrema and closest approaches in
DE440s. The event search targets near conjunctions, oppositions and short
Earth-relative distances. The sampled maxima apply to those dates and to geometric positions.
Continuous-time DE440s error and complete pointing error require separate
assessment. CI uses an 18-epoch DE440s fixture with both boundary cases and
also checks the Cartesian budget against the full archive tables.

The Sun's barycentric direction reaches about 29.5′ of error against
DE440s even in full VSOP87E. This direction is sensitive to small absolute
differences because the solar vector can pass close to the SSB. Compact vs
full differs by at most 9.710″ and 9.183 km for that vector, whereas both
profiles differ from DE440s by up to about 1,157 km. The nearly common
barycentric offset largely cancels in the Earth-relative comparison. For
Earth-relative Sun pointing, compact vs DE440s peaks at 0.537″. Outer-planet
distance errors are larger in kilometers than the angular errors suggest
because the distances are large.

## Linked size and performance

Each benchmark evaluates all nine bodies, so the same required VSOP87E
tables are referenced on each side. The full case uses the archive's full
`Vsop87` data and a scalar Horner evaluator with the same `libm` cosine as
compact. The archive's full feature exposes coefficients. The desktop ratio applies
to the scalar baseline. Siderust's downstream SIMD evaluator has a different
throughput.

| Metric | Full VSOP87E | Compact VSOP87E |
|---|---:|---:|
| Coefficient payload | 1,243,752 B | 76,392 B |
| Linux x86-64 ELF | 1,862,184 B | 643,160 B |
| Linux stripped ELF | 1,712,792 B | 504,808 B |
| Linux linked `.rodata` | 1,285,692 B | 118,620 B |
| Linux median latency (nine positions) | 892.330 µs | 46.526 µs |
| Linux median throughput | 10,086 positions/s | 193,440 positions/s |
| ESP32 linked ELF | 1,441,804 B | 207,864 B |
| ESP32 application image | 1,409,392 B | 172,592 B |
| ESP32 DROM `.rodata` | 1,258,736 B | 92,128 B |
| ESP32 flash `.text` | 98,581 B | 41,433 B |
| ESP32 IRAM `.rwtext` + vectors | 15,396 B | 15,396 B |
| ESP32 DRAM `.data` + `.bss` | 2,548 B | 2,548 B |
| ESP32 linker-reserved stack | 194,060 B | 194,060 B |

Desktop measurements use Rust 1.100.0, release defaults, GNU/Linux
x86-64, an Intel Core Ultra 5 325, 4,096 nine-body epochs per run and the
median of five runs. The ELF excludes linked VSOP87A tables in the full
case and excludes all full arrays in the compact case. Full generated A
and E data together contain 2,262,360 bytes of coefficients before linker
garbage collection. The Linux `.rodata` reduction is within 288 bytes of the reduction in
coefficient payload. This indicates both referenced
profiles link essentially one copy of each retained coefficient.

ESP32 measurements use `xtensa-esp32-none-elf`, espup 0.19.0's
`esp` Rust 1.97.0-nightly (8ea53bcd7, Espressif 1.97.0.0), esp-hal 1.2.2
and espflash 4.7.0. The harness uses a 4 MiB physical flash configuration
and a 2 MiB factory application slot. The compact image consumes 8.23%
of that slot. It also fits a standard 1 MiB factory slot (16.46%), where
the full image exceeds capacity. The application image sizes include
segment padding and exclude bootloader and partition-table bytes.
DROM is mapped from flash, and IRAM and DRAM use on-chip RAM.
[ESP-IDF partition documentation](https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-guides/partition-tables.html).

ESP32 firmware was cross-compiled and converted to flash images. No board
was available to measure runtime latency, flash-cache behavior or peak
stack usage. The stack row is a linker reservation. The linked ESP32
ELFs use software double-precision arithmetic helpers. Device throughput
depends on their cost on the Xtensa core. Compact evaluates 3,183 cosines
per nine-body epoch instead of 51,823. The firmware harness reports elapsed
microseconds over 128 epochs on UART when run on hardware.

## Reproduce

Build and measure from the repository root. Cargo generates the tables
into `OUT_DIR` using only the bundled IMCCE files. The aggregate SHA-256
of sorted `VSOP87*` filenames followed by a NUL and their file contents is
`65bb255dd7bd866fe1925e4f3053a274426655f6d03b94c303c2464d773d75dc`.
Set `SIDERUST_VSOP87_SHA256` to this digest to enforce it at build time.
Individual source hashes are in the [VSOP87 manifest](../src/vsop/manifest.toml).
Compact generation contains no volatile timestamps.

```sh
out="$(mktemp -d)"
for profile in vsop vsop-compact; do
  cargo build --locked --release --example vsop87_measure \
    --no-default-features --features "$profile"
  cp target/release/examples/vsop87_measure "$out/$profile"
  size -A "$out/$profile"
  strip -o "$out/$profile.stripped" "$out/$profile"
  wc -c "$out/$profile" "$out/$profile.stripped"
done
for trial in 1 2 3 4 5; do
  "$out/vsop"
  "$out/vsop-compact"
done
cargo tree --no-default-features --features vsop-compact -e features -i siderust-archive
cargo test --locked --no-default-features --features vsop-compact
cargo test --locked --no-default-features --features vsop,vsop-compact
```

Download the JPL kernel once for external validation. The script verifies
its SHA-256 and works offline after setup. CI needs only the committed
fixture. To reproduce the full report and the fixture, use Python 3.14
with the versions below.

```sh
python3 -m venv "$out/venv"
"$out/venv/bin/pip" install numpy==2.5.4 jplephem==2.24
curl -fL https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp \
  -o "$out/de440s.bsp"
"$out/venv/bin/python" scripts/validate_compact_vsop87.py "$out/de440s.bsp" \
  --models "$out/vsop" "$out/vsop-compact"
"$out/venv/bin/python" scripts/validate_compact_vsop87.py "$out/de440s.bsp" \
  --fixture "$out/de440s_vsop87e.csv"
cmp "$out/de440s_vsop87e.csv" tests/fixtures/de440s_vsop87e.csv
```

For the ESP32, install the `esp` toolchain and source the environment
script generated by [espup](https://github.com/esp-rs/espup). Run from
the harness directory so Cargo picks up its target configuration.

```sh
espup install --targets esp32 --toolchain-version 1.97.0.0
. "$HOME/export-esp.sh"
cd tools/vsop87-esp32
printf 'factory,app,factory,0x10000,2M\n' > "$out/partitions.csv"
for profile in vsop vsop-compact; do
  cargo +esp build --locked --release --no-default-features --features "$profile"
  cp target/xtensa-esp32-none-elf/release/vsop87-esp32 "$out/$profile.elf"
  xtensa-esp32-elf-size -A "$out/$profile.elf"
  espflash save-image --chip esp32 --flash-size 4mb --flash-mode dio \
    --flash-freq 40mhz --partition-table "$out/partitions.csv" \
    "$out/$profile.elf" "$out/$profile.bin"
  wc -c "$out/$profile.bin"
done
```

With a board attached, use `espflash flash --monitor --chip esp32 --flash-size 4mb
--flash-mode dio --flash-freq 40mhz --partition-table "$out/partitions.csv"
"$out/vsop-compact.elf"` and then repeat with the full ELF. Divide the
printed time by 128 for nine-body latency. The sample uses the HAL default
clock configuration. Measure both variants on the same board and clock.

## Downstream integration

[Siderust#139](https://github.com/Siderust/siderust/issues/139) needs to expose
the compact backend through its own Cargo feature and move its unconditional
archive `vsop` activation behind its full backend feature. Cargo unifies
features of the same archive version, so any other dependency activating
`vsop` will also generate full tables. The compact wrapper should pass TDB,
preserve the SSB origin, and apply the existing native-VSOP87-to-canonical
frame transform. It should provide the frame, light-time and observer
handling appropriate to its pointing API. Lunar compaction is tracked in
[archive#6](https://github.com/Siderust/archive/issues/6). The embedded
pointing requirements originated in [discussion #138](https://github.com/orgs/Siderust/discussions/138).

## Per-body metrics

Evaluated 24,418 distinct TDB epochs, JD 2458849.5 to 2470172.4999999995.

### Compact vs DE440s

| Origin | Body | Max ″ | Mean ″ | RMS ″ | P95 ″ | P99 ″ | Max km |
|---|---|---:|---:|---:|---:|---:|---:|
| SSB | Sun | 1769.533 | 238.753 | 360.707 | 779.395 | 1614.038 | 1157.375 |
| SSB | Mercury | 5.906 | 2.628 | 2.893 | 4.488 | 5.159 | 1394.776 |
| SSB | Venus | 2.617 | 1.279 | 1.388 | 2.166 | 2.402 | 1396.687 |
| SSB | Earth | 1.695 | 0.971 | 1.072 | 1.558 | 1.627 | 1268.220 |
| SSB | Mars | 1.426 | 0.709 | 0.772 | 1.190 | 1.384 | 1536.585 |
| SSB | Jupiter | 2.263 | 0.965 | 1.095 | 1.932 | 2.251 | 8253.595 |
| SSB | Saturn | 1.892 | 1.001 | 1.109 | 1.811 | 1.886 | 13178.757 |
| SSB | Uranus | 2.887 | 2.083 | 2.147 | 2.869 | 2.886 | 38933.461 |
| SSB | Neptune | 3.811 | 3.594 | 3.597 | 3.805 | 3.811 | 88532.573 |
| Earth | Sun | 0.537 | 0.187 | 0.207 | 0.373 | 0.454 | 394.428 |
| Earth | Mercury | 0.981 | 0.214 | 0.250 | 0.460 | 0.624 | 595.809 |
| Earth | Venus | 2.462 | 0.410 | 0.567 | 1.338 | 1.783 | 772.863 |
| Earth | Mars | 1.643 | 0.262 | 0.373 | 0.861 | 1.402 | 956.195 |
| Earth | Jupiter | 2.660 | 0.981 | 1.109 | 1.991 | 2.538 | 8423.446 |
| Earth | Saturn | 2.083 | 1.030 | 1.132 | 1.828 | 2.013 | 13230.003 |
| Earth | Uranus | 3.109 | 2.144 | 2.207 | 2.972 | 3.081 | 39832.306 |
| Earth | Neptune | 3.951 | 3.603 | 3.608 | 3.877 | 3.935 | 88336.083 |
| Sun | Mercury | 2.600 | 0.530 | 0.626 | 1.163 | 1.709 | 631.350 |
| Sun | Venus | 1.316 | 0.509 | 0.568 | 0.966 | 1.143 | 717.454 |
| Sun | Earth | 0.537 | 0.187 | 0.207 | 0.373 | 0.454 | 394.428 |
| Sun | Mars | 0.938 | 0.222 | 0.269 | 0.455 | 0.855 | 964.955 |
| Sun | Jupiter | 2.263 | 0.977 | 1.093 | 1.918 | 2.251 | 8323.475 |
| Sun | Saturn | 1.888 | 1.029 | 1.126 | 1.813 | 1.883 | 13211.686 |
| Sun | Uranus | 2.950 | 2.146 | 2.207 | 2.932 | 2.949 | 39791.742 |
| Sun | Neptune | 3.815 | 3.600 | 3.603 | 3.809 | 3.815 | 88187.144 |

### Full vs DE440s

| Origin | Body | Max ″ | Mean ″ | RMS ″ | P95 ″ | P99 ″ | Max km |
|---|---|---:|---:|---:|---:|---:|---:|
| SSB | Sun | 1766.986 | 238.703 | 360.543 | 779.909 | 1612.268 | 1157.049 |
| SSB | Mercury | 5.058 | 2.526 | 2.825 | 4.575 | 4.841 | 1164.542 |
| SSB | Venus | 2.241 | 1.334 | 1.481 | 2.104 | 2.160 | 1170.822 |
| SSB | Earth | 1.606 | 0.956 | 1.066 | 1.520 | 1.563 | 1176.814 |
| SSB | Mars | 1.176 | 0.634 | 0.707 | 1.098 | 1.149 | 1203.040 |
| SSB | Jupiter | 0.489 | 0.234 | 0.275 | 0.473 | 0.483 | 1923.989 |
| SSB | Saturn | 0.315 | 0.178 | 0.199 | 0.306 | 0.312 | 2176.794 |
| SSB | Uranus | 1.367 | 1.106 | 1.132 | 1.360 | 1.366 | 18810.814 |
| SSB | Neptune | 1.988 | 1.610 | 1.628 | 1.959 | 1.982 | 43602.237 |
| Earth | Sun | 0.056 | 0.046 | 0.046 | 0.052 | 0.054 | 40.886 |
| Earth | Mercury | 0.081 | 0.047 | 0.048 | 0.067 | 0.074 | 49.051 |
| Earth | Venus | 0.088 | 0.047 | 0.048 | 0.066 | 0.077 | 62.268 |
| Earth | Mars | 0.099 | 0.056 | 0.057 | 0.077 | 0.090 | 116.618 |
| Earth | Jupiter | 0.359 | 0.189 | 0.197 | 0.288 | 0.337 | 1043.302 |
| Earth | Saturn | 0.273 | 0.151 | 0.168 | 0.257 | 0.268 | 1781.168 |
| Earth | Uranus | 1.515 | 1.176 | 1.202 | 1.479 | 1.507 | 19810.984 |
| Earth | Neptune | 2.073 | 1.620 | 1.638 | 1.985 | 2.050 | 43800.480 |
| Sun | Mercury | 0.036 | 0.024 | 0.026 | 0.035 | 0.036 | 12.092 |
| Sun | Venus | 0.050 | 0.043 | 0.043 | 0.048 | 0.049 | 26.388 |
| Sun | Earth | 0.056 | 0.046 | 0.046 | 0.052 | 0.054 | 40.886 |
| Sun | Mars | 0.073 | 0.056 | 0.056 | 0.065 | 0.070 | 83.113 |
| Sun | Jupiter | 0.282 | 0.189 | 0.194 | 0.270 | 0.280 | 1021.800 |
| Sun | Saturn | 0.242 | 0.151 | 0.167 | 0.238 | 0.240 | 1753.526 |
| Sun | Uranus | 1.441 | 1.178 | 1.203 | 1.433 | 1.439 | 19801.492 |
| Sun | Neptune | 2.003 | 1.619 | 1.637 | 1.974 | 1.997 | 43770.596 |

### Compact vs full

| Origin | Body | Max ″ | Mean ″ | RMS ″ | P95 ″ | P99 ″ | Max km |
|---|---|---:|---:|---:|---:|---:|---:|
| SSB | Sun | 9.710 | 1.317 | 1.872 | 4.180 | 7.261 | 9.183 |
| SSB | Mercury | 2.621 | 0.528 | 0.628 | 1.182 | 1.755 | 634.001 |
| SSB | Venus | 1.315 | 0.518 | 0.576 | 0.972 | 1.129 | 722.479 |
| SSB | Earth | 0.522 | 0.170 | 0.192 | 0.356 | 0.443 | 383.500 |
| SSB | Mars | 0.969 | 0.214 | 0.266 | 0.469 | 0.861 | 993.311 |
| SSB | Jupiter | 2.164 | 0.940 | 1.068 | 1.813 | 2.151 | 7956.654 |
| SSB | Saturn | 1.899 | 0.997 | 1.112 | 1.823 | 1.893 | 13287.003 |
| SSB | Uranus | 2.086 | 1.222 | 1.315 | 2.064 | 2.083 | 28352.140 |
| SSB | Neptune | 3.455 | 2.837 | 2.870 | 3.419 | 3.449 | 76135.939 |
| Earth | Sun | 0.514 | 0.168 | 0.190 | 0.357 | 0.439 | 377.414 |
| Earth | Mercury | 0.970 | 0.200 | 0.237 | 0.447 | 0.605 | 568.250 |
| Earth | Venus | 2.430 | 0.404 | 0.555 | 1.300 | 1.769 | 760.898 |
| Earth | Mars | 1.678 | 0.256 | 0.369 | 0.846 | 1.404 | 951.693 |
| Earth | Jupiter | 2.540 | 0.945 | 1.083 | 1.944 | 2.413 | 8050.667 |
| Earth | Saturn | 2.091 | 0.999 | 1.117 | 1.838 | 2.018 | 13298.969 |
| Earth | Uranus | 2.201 | 1.223 | 1.318 | 2.082 | 2.173 | 28421.588 |
| Earth | Neptune | 3.564 | 2.841 | 2.875 | 3.437 | 3.534 | 76263.869 |
| Sun | Mercury | 2.624 | 0.525 | 0.625 | 1.175 | 1.734 | 636.588 |
| Sun | Venus | 1.316 | 0.516 | 0.574 | 0.966 | 1.131 | 717.823 |
| Sun | Earth | 0.514 | 0.168 | 0.190 | 0.357 | 0.439 | 377.414 |
| Sun | Mars | 0.959 | 0.213 | 0.265 | 0.468 | 0.860 | 986.963 |
| Sun | Jupiter | 2.163 | 0.940 | 1.067 | 1.813 | 2.151 | 7958.755 |
| Sun | Saturn | 1.899 | 0.997 | 1.112 | 1.824 | 1.894 | 13290.960 |
| Sun | Uranus | 2.085 | 1.222 | 1.315 | 2.064 | 2.083 | 28359.128 |
| Sun | Neptune | 3.454 | 2.837 | 2.870 | 3.418 | 3.449 | 76137.513 |
