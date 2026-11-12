# Compact embedded lunar ephemeris

`elp-compact` returns the geometric Earth-to-Moon XYZ vector in kilometers,
in the native ELP2000-82B dynamical ecliptic and inertial equinox of J2000.0.
The checked API accepts TDB Julian dates in `[2458849.5, 2470172.5)`, covering
2020-01-01 through 2050-12-31. The direct `moon_geocentric` function evaluates
unchecked. The coefficients are compiled in and evaluation requires no
heap or runtime I/O.

The compact profile's maximum sampled errors against DE440s were **2.254″
geocentric**, **2.293″ topocentric** across four sites, and **1.441 km radial**.
The lunar-only ESP32 application image is **112,432 bytes**. The combined
compact Sun, Moon, and planetary image is **181,760 bytes**.

## Model and selection

We implemented the published [ELP2000-82B equations and frame specification](https://ftp.imcce.fr/pub/ephem/moon/elp82b/elp82b.ps)
in Rust. The [IMCCE reference algorithm](https://ftp.imcce.fr/pub/ephem/moon/elp82b/elp82b_1)
identifies the DE200/LE200 constants, argument families and time powers.
The implementation uses the existing [IMCCE/CDS numerical tables](https://cdsarc.cds.unistra.fr/viz-bin/cat/VI/79).
It contains no downstream Siderust implementation code. The full scalar
benchmark uses the archive's existing static arrays and matches the
IMCCE notice's Table H check vectors to within 2 cm, the tolerance of the
regression test. The generated full table declarations match `main` byte
for byte.

Each coordinate uses Julian centuries `T = (JD_TDB - 2451545) / 36525`.
The first three sub-series use integer multiples of four polynomial
Delaunay arguments. The distance main problem uses cosine, while the
longitude and latitude main problems use sine. The generator evaluates the
DE200/LE200 amplitude corrections once, including the scale correction
specific to the main distance series. It stores each corrected amplitude
as `f64` and four exactly representable signed `i8` multipliers. Each
16-byte record includes four padding bytes on the measured targets.
The raw 88-byte main term includes four `i64` multipliers, the amplitude,
and six derivative values.

Earth-family and planetary perturbations use linear combinations of fixed
phase and rate coefficients. The generator folds the integer multipliers
into a phase and rate and stores the amplitude, phase, and rate as three
`f64` values (24 bytes). Terms are grouped by coordinate and amplitude
power `k = 0, 1, 2`. The raw 64-byte Earth-family and 112-byte planetary
records use wider multipliers and include an auxiliary period column that
the position evaluator does not need. The compact representation retains
double precision. Folding the arguments changes floating-point rounding,
and narrowing the integer multipliers is exact.

The generator orders terms separately for longitude, latitude, and distance
by the absolute corrected amplitude times `max(|T|)^k`. The main problem
uses `k = 0` even though its **arguments** contain higher powers. For each
coordinate it discards the smallest contributions while their sum remains
within budget, breaking ties by sub-series number and original term index.
This bounds the omitted sum in exact arithmetic over the entire interval
because each sine or cosine is bounded by one. Longitude and latitude
budgets are 12″ and 8″, and the pre-scale distance budget is 10 km.
The retained terms are evaluated in static arrays, followed by conversion
to Cartesian coordinates and the Laskar rotation into native J2000 axes.

| Coordinate | Main retained | Perturbations retained | Omission budget |
|---|---:|---:|---:|
| Longitude | 137 | 73 | 12″ |
| Latitude | 90 | 9 | 8″ |
| Distance | 109 | 39 | 10 km |
| **Total** | **336** | **121** | |

Full term counts by sub-series (ranges are consecutive in numeric order):

| Series | Counts |
|---|---|
| 1–9 | 1023, 918, 704, 347, 316, 237, 14, 11, 8 |
| 10–18 | 14328, 5233, 6631, 4384, 833, 1715, 170, 150, 114 |
| 19–27 | 226, 188, 169, 3, 2, 2, 6, 4, 5 |
| 28–36 | 20, 12, 14, 11, 4, 10, 28, 13, 19 |

The intrinsic angular truncation bound is at most `hypot(12″, 8″) ≈ 14.43″`.
On the unit sphere, the path obtained by linearly interpolating longitude
and latitude has length at most the Euclidean length of those changes
because the longitude component is scaled by `cos(latitude)`. The Laskar
rotation preserves the angle. If the full radius is `r`, a conservative
Cartesian bound is `B = |Δr| + 2r sin(Δθ/2)`, where the distance scale
`A0/ATH` is slightly below one. Subtracting the same observer leaves this
Cartesian bound unchanged. For a full-model topocentric distance `d > B`,
the maximum possible angular deviation is `asin(B/d)`. For example,
`r ≤ 410,000 km` and `d ≥ 350,000 km` give a truncation bound below 23″.
These are analytical bounds **relative to full ELP** and conditional on
the stated distances. The DE440s maxima are empirical and also include
the full model's discrepancy with DE440s.

### Alternatives

- [ERFA Moon98](https://github.com/liberfa/erfa/blob/master/src/moon98.c)
  is a permissively licensed implementation of the Meeus analytical model.
  It has 60 longitude/distance arguments and 60 latitude arguments. Its
  source C tables occupy 3,360 bytes with 32-bit integers and 64-bit
  doubles, excluding other constants and code. It is a viable smaller
  candidate with fewer arguments. ERFA's implementation also evaluates
  velocity. Measured maxima were about 15.10″ topocentric and 11.59 km
  radial. Compact ELP gives tighter pointing and distance agreement with
  the existing data pipeline and a bounded omission budget.
- The validation script fits **16-day, degree-8** Chebyshev segments to
  DE440s Cartesian XYZ at nine first-kind nodes per segment. It evaluates
  the stored coefficients with Clenshaw at separate validation epochs,
  removing any node coincident with the validation grid. The 708 segments
  need 76,464 bytes with `f32` storage. Clenshaw needs roughly 24 recurrence
  steps for the vector and no trig, which would favor it on a CPU with
  software double precision. In this configuration the interpolation
  dominates the error. The separate quantization comparison measures the
  additional error from `f32` storage. Shorter segments or higher degree
  can improve accuracy at the cost of flash. The prototype only evaluates
  locally obtained JPL data and ships no fitted tables or kernels. NAIF
  publishes these data for [public use](https://naif.jpl.nasa.gov/naif/public.html).
  Only small numerical reference vectors are committed for regression.

Compact ELP was selected for its measured distance and pointing accuracy,
8,280-byte coefficient payload, deterministic offline build, and reuse of
the bundled, attributable ELP source files. Moon98 and Chebyshev were
evaluated numerically as desktop prototypes. The linked timing benchmarks
below cover the two Rust ELP profiles.

## Independent validation and observer geometry

The script reads a local [DE440s SPK](https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp).
It verifies SHA-256
`c1c7feeab882263fc493a9d5a5b2ddd71b54826cdf65d8d17a76126b260a49f2`.
Both the Moon (NAIF 301) and Earth (399) segments are relative to the
Earth–Moon barycenter (3). The reference is `SPK[3,301] - SPK[3,399]`, which
cancels that common center. The SPK provides geometric ICRF coordinates in
kilometers at TDB epochs. The comparison uses the same TDB JD for ELP and
DE440s. ELP is rotated into ICRF using the native-to-FK5 matrix in section 8
of the notice and the fixed J2000 orientation from
[ERFA fk5hip](https://github.com/liberfa/erfa/blob/master/src/fk5hip.c).
Ecliptic longitude and latitude metrics rotate both vectors into native
ELP axes. Moon98 uses TT, approximated by the same numeric TDB epoch
(the periodic difference is millisecond-scale).

The validation uses a six-hour grid, 8,192 seeded uniform epochs, both
boundaries, and half-hour samples within four hours of selected extrema
in lunar radius, declination, and solar elongation. The extrema include
perigees, apogees, near-new and near-full configurations, eclipse proxies
identified by small Sun–Moon or anti-Sun–Moon separation, and quadratures.
The 30-epoch fixture retains both boundaries, representative dates, apsidal and
declination extremes, phase cases, observed error peaks, and epochs near
the horizon for each site. Its positions come from DE440s and its observer
vectors come from the common geometry described below. CI uses the fixture
without fetching a kernel.

For each site, ERFA builds a WGS84 terrestrial position and local zenith.
The transpose of [ERFA c2t06a](https://github.com/liberfa/erfa/blob/master/src/c2t06a.c)
maps both into celestial axes. We use TT ≈ TDB, UT1 ≈ TT − 69 seconds, and
zero polar motion as shared test assumptions. Both model and reference
subtract exactly the same observer vector. Each site spans different
lunar elevations, including near-horizon observations with substantial
parallax. The distance-only comparison substitutes the compact radius
along the DE440s geocentric direction before subtracting the observer.
This isolates the effect of radius error on parallax. The largest sampled
distance-only contribution was below 0.014″.

These comparisons measure the ephemeris contribution with matched
observer geometry. Real pointing also depends on UTC/TT/TDB/UT1
conversion, Earth orientation, site coordinates, apparent-place
corrections and instrument calibration. The geometric calculation
omits light time, aberration, refraction, and the instrument model on
both sides. The benchmark and error tables characterize position only.

## Linked footprint and performance

The benchmarks evaluate complete positions and retain every table
referenced by each profile. The Linux baseline uses the unmodified
full archive arrays and a scalar IMCCE evaluator, with the same
`libm` trigonometry as compact. Desktop timing uses Rust 1.100.0,
release defaults, an Intel Core Ultra 5 325 on Linux x86-64, and the
median of seven warmed trials of 2,048 epochs. Compact takes 457
term trig evaluations plus two sine/cosine pairs per position. Full
takes 37,872 term trig evaluations plus the same conversion.

| Metric | Full lunar | Compact lunar | Combined compact |
|---|---:|---:|---:|
| Coefficient payload | 4,126,056 B | 8,280 B | 84,672 B |
| Linux ELF | 4,699,008 B | 573,760 B | — |
| Linux stripped ELF | 4,560,968 B | 438,256 B | — |
| Linux `.rodata` | 4,168,852 B | 50,540 B | — |
| Linux `.text` | 291,494 B | 289,606 B | — |
| Linux median latency | 867.099 µs | 8.568 µs | — |
| Linux throughput | 1,153 positions/s | 116,707 positions/s | — |
| ESP32 ELF | 4,267,256 B | 142,112 B | 227,140 B |
| ESP32 application image | 4,244,032 B | 112,432 B | 181,760 B |
| ESP32 DROM `.rodata` | 4,142,192 B | 23,768 B | 100,744 B |
| ESP32 mapped flash `.text` | 49,641 B | 46,809 B | 50,609 B |
| ESP32 IRAM code and vectors | 15,396 B | 15,396 B | 15,396 B |
| ESP32 DRAM `.data` + `.bss` | 2,568 B | 2,568 B | 2,568 B |
| ESP32 linker-reserved stack | 194,040 B | 194,040 B | 194,040 B |

The compact lunar image is 97.35% smaller than the full lunar harness.
The measurements use `xtensa-esp32-none-elf`, Espressif Rust 1.97.0.0,
esp-hal 1.2.2, espflash 4.7.0, and the existing 80 MHz harness. The
full application exceeds the WROOM-32's 4 MiB physical flash even
before bootloader and partition data. It is measured with an 8 MiB
flash configuration and 6 MiB application slot. The compact images
use 4 MiB physical flash and a 2 MiB application partition. DROM and
flash executable code are mapped from flash, while IRAM and DRAM use
on-chip RAM. The image counts include app segment padding and exclude
bootloader and partition-table data. The combined image measures this
harness with ten positions per epoch. A complete pointing application
also needs time and frame conversion, observer geometry and application
code.

Physical device timing and peak stack usage remain pending because no
ESP32 was attached. The stack figures are linker reservations. Both
builds use software double-precision arithmetic on this target, so the
desktop ratio describes the desktop only. The firmware runs seven
identical warmed trials with optimization barriers and prints mean
and median latency after all trials. The full image would require an
ESP32 board with sufficient flash to run this comparison.

## Reproduce

Ordinary builds read the bundled ELP1–36 files through the existing
archive parser. Individual hashes are in the [ELP manifest](../src/elp/manifest.toml).
The aggregate sorted-name/NUL/content SHA-256 is
`469864b4e798d90beffca46843d550c6c1df41b4151e39b107873e2a2523d690`.
Set `SIDERUST_ELP2000_SHA256` to enforce that digest during generation.
Compact output is timestamp-free.

From the repository root:

```sh
out="$(mktemp -d)"
for profile in elp elp-compact; do
  cargo build --locked --release --example lunar_measure \
    --no-default-features --features "$profile"
  cp target/release/examples/lunar_measure "$out/$profile"
  size -A "$out/$profile"
  strip -o "$out/$profile.stripped" "$out/$profile"
  wc -c "$out/$profile" "$out/$profile.stripped"
  "$out/$profile"
done

python3 -m venv "$out/venv"
"$out/venv/bin/pip" install numpy==2.5.4 jplephem==2.24 pyerfa==2.0.1.5
curl -fL https://naif.jpl.nasa.gov/pub/naif/generic_kernels/spk/planets/de440s.bsp \
  -o "$out/de440s.bsp"
"$out/venv/bin/python" scripts/validate_compact_lunar.py "$out/de440s.bsp" \
  --models "$out/elp" "$out/elp-compact" --fixture "$out/de440s_lunar.csv"
cmp "$out/de440s_lunar.csv" tests/fixtures/de440s_lunar.csv
```

Install and source the [espup](https://github.com/esp-rs/espup) environment
as in the [planetary report](compact-vsop87.md). From the same shell:

```sh
cd tools/vsop87-esp32
printf 'factory,app,factory,0x10000,6M\n' > "$out/partitions-8m.csv"
printf 'factory,app,factory,0x10000,2M\n' > "$out/partitions-4m.csv"
for profile in elp elp-compact vsop-compact,elp-compact; do
  cargo +esp build --locked --release --no-default-features --features "$profile"
  cp target/xtensa-esp32-none-elf/release/vsop87-esp32 "$out/$profile.elf"
  xtensa-esp32-elf-size -A "$out/$profile.elf"
  flash=4m
  if [ "$profile" = elp ]; then flash=8m; fi
  espflash save-image --chip esp32 --flash-size "${flash}b" --flash-mode dio \
    --flash-freq 40mhz --partition-table "$out/partitions-$flash.csv" \
    "$out/$profile.elf" "$out/$profile.bin"
  wc -c "$out/$profile.bin"
done
```

To capture compact lunar runtime on a WROOM-32:

```sh
espflash flash --monitor --chip esp32 --flash-size 4mb --flash-mode dio \
  --flash-freq 40mhz --partition-table "$out/partitions-4m.csv" "$out/elp-compact.elf"
```

Use an 8 MiB ESP32 board and the 6 MiB factory partition for timing the
full profile, with the same 80 MHz CPU and 40 MHz flash settings for both
profiles. Compare per-position results from lunar-only images. The
combined image reports the average across its ten-body workload.

## Downstream integration

[Siderust#139](https://github.com/Siderust/siderust/issues/139) can select
`elp-compact` and `vsop-compact` together. Move any unconditional `elp`
activation behind the full backend feature to avoid generating the full
data through Cargo feature unification. Pass TDB to the checked lunar
API and apply the existing ELP-specific native-to-canonical frame
transform. For topocentric pointing, subtract a consistently transformed
observer vector and apply the required apparent-place corrections.
Combine with the compact planetary positions from [archive PR #8](https://github.com/Siderust/archive/pull/8)
only after aligning centers and frames.

## Validation statistics

Validated 54,381 TDB epochs from 2458849.5 through 2470172.4999999995.

### Compact ELP vs DE440s

| Error | Max | Mean absolute | RMS | P95 | P99 |
|---|---:|---:|---:|---:|---:|
| Longitude (″) | 1.807907 | 0.325860 | 0.405091 | 0.773296 | 1.020311 |
| Latitude (″) | 1.624756 | 0.299967 | 0.384889 | 0.762713 | 1.096215 |
| Geocentric angle (″) | 2.253777 | 0.491270 | 0.558203 | 0.974644 | 1.292258 |
| Cartesian (km) | 3.900808 | 0.987103 | 1.091405 | 1.841079 | 2.330118 |
| Radius (km) | 1.441424 | 0.278630 | 0.353058 | 0.697644 | 0.961910 |
| Relative radius (ppm) | 3.860537 | 0.728472 | 0.927018 | 1.836940 | 2.570207 |
| Topocentric Equator (0°, 0°, 0 m) (″) | 2.272851 | 0.491291 | 0.558257 | 0.974850 | 1.292032 |
| Topocentric Equator (0°, 0°, 0 m), altitude 0–15° (″) | 2.256651 | 0.505207 | 0.575252 | 0.997406 | 1.376565 |
| Topocentric Equator (0°, 0°, 0 m), altitude 15–45° (″) | 1.986451 | 0.489161 | 0.555692 | 0.973155 | 1.274872 |
| Topocentric Equator (0°, 0°, 0 m), altitude ≥45° (″) | 2.272851 | 0.499547 | 0.566395 | 0.990682 | 1.300098 |
| Distance-only parallax Equator (0°, 0°, 0 m) (″) | 0.013020 | 0.001740 | 0.002368 | 0.004981 | 0.007225 |
| Topocentric Mid latitude (45°, 100° E, 2,000 m) (″) | 2.219613 | 0.491220 | 0.558199 | 0.975699 | 1.293971 |
| Topocentric Mid latitude (45°, 100° E, 2,000 m), altitude 0–15° (″) | 2.204675 | 0.496109 | 0.561756 | 0.972301 | 1.259898 |
| Topocentric Mid latitude (45°, 100° E, 2,000 m), altitude 15–45° (″) | 2.196930 | 0.497160 | 0.563682 | 0.977433 | 1.297050 |
| Topocentric Mid latitude (45°, 100° E, 2,000 m), altitude ≥45° (″) | 1.873390 | 0.507631 | 0.578204 | 1.006303 | 1.374829 |
| Distance-only parallax Mid latitude (45°, 100° E, 2,000 m) (″) | 0.013429 | 0.002094 | 0.002734 | 0.005554 | 0.007881 |
| Topocentric High latitude (75° N, 30° W, 100 m) (″) | 2.242249 | 0.491189 | 0.558147 | 0.974026 | 1.290774 |
| Topocentric High latitude (75° N, 30° W, 100 m), altitude 0–15° (″) | 1.854097 | 0.497180 | 0.562691 | 0.983303 | 1.286170 |
| Topocentric High latitude (75° N, 30° W, 100 m), altitude 15–45° (″) | 1.872716 | 0.500647 | 0.568136 | 0.992656 | 1.282358 |
| Distance-only parallax High latitude (75° N, 30° W, 100 m) (″) | 0.013446 | 0.002350 | 0.003012 | 0.006021 | 0.008465 |
| Topocentric South highlands (23° S, 67° W, 5,000 m) (″) | 2.293088 | 0.491285 | 0.558226 | 0.975236 | 1.294394 |
| Topocentric South highlands (23° S, 67° W, 5,000 m), altitude 0–15° (″) | 2.058514 | 0.486930 | 0.555723 | 0.986994 | 1.302381 |
| Topocentric South highlands (23° S, 67° W, 5,000 m), altitude 15–45° (″) | 2.254938 | 0.494935 | 0.561180 | 0.981961 | 1.270719 |
| Topocentric South highlands (23° S, 67° W, 5,000 m), altitude ≥45° (″) | 2.293088 | 0.496372 | 0.563919 | 0.984196 | 1.315504 |
| Distance-only parallax South highlands (23° S, 67° W, 5,000 m) (″) | 0.013401 | 0.001847 | 0.002489 | 0.005171 | 0.007467 |

### Full ELP vs DE440s

| Error | Max | Mean absolute | RMS | P95 | P99 |
|---|---:|---:|---:|---:|---:|
| Longitude (″) | 0.537863 | 0.275230 | 0.292913 | 0.449690 | 0.495691 |
| Latitude (″) | 0.063456 | 0.020865 | 0.025292 | 0.047402 | 0.054934 |
| Geocentric angle (″) | 0.537724 | 0.275944 | 0.293412 | 0.449243 | 0.495051 |
| Cartesian (km) | 0.942389 | 0.514086 | 0.544826 | 0.815446 | 0.881235 |
| Radius (km) | 0.095190 | 0.026856 | 0.033012 | 0.062162 | 0.077492 |
| Relative radius (ppm) | 0.252012 | 0.069920 | 0.086092 | 0.162551 | 0.204229 |
| Topocentric Equator (0°, 0°, 0 m) (″) | 0.546419 | 0.275946 | 0.293430 | 0.449471 | 0.496019 |
| Topocentric Mid latitude (45°, 100° E, 2,000 m) (″) | 0.545297 | 0.275925 | 0.293394 | 0.449162 | 0.494194 |
| Topocentric High latitude (75° N, 30° W, 100 m) (″) | 0.540660 | 0.275919 | 0.293377 | 0.448833 | 0.493840 |
| Topocentric South highlands (23° S, 67° W, 5,000 m) (″) | 0.543986 | 0.275941 | 0.293429 | 0.449403 | 0.496437 |

### Compact ELP vs full ELP

| Error | Max | Mean absolute | RMS | P95 | P99 |
|---|---:|---:|---:|---:|---:|
| Longitude (″) | 1.963091 | 0.279262 | 0.358132 | 0.714609 | 0.999823 |
| Latitude (″) | 1.604837 | 0.299241 | 0.383984 | 0.760456 | 1.095352 |
| Geocentric angle (″) | 2.412662 | 0.453462 | 0.524581 | 0.936859 | 1.293492 |
| Cartesian (km) | 4.176170 | 0.923719 | 1.030836 | 1.757572 | 2.344641 |
| Radius (km) | 1.500327 | 0.286478 | 0.363379 | 0.718740 | 0.986474 |
| Relative radius (ppm) | 4.018293 | 0.749115 | 0.954371 | 1.895510 | 2.631747 |
| Topocentric Equator (0°, 0°, 0 m) (″) | 2.435820 | 0.453480 | 0.524629 | 0.937259 | 1.289557 |
| Topocentric Mid latitude (45°, 100° E, 2,000 m) (″) | 2.378847 | 0.453372 | 0.524476 | 0.937830 | 1.291050 |
| Topocentric High latitude (75° N, 30° W, 100 m) (″) | 2.400290 | 0.453325 | 0.524393 | 0.937129 | 1.292097 |
| Topocentric South highlands (23° S, 67° W, 5,000 m) (″) | 2.454784 | 0.453501 | 0.524664 | 0.935744 | 1.295029 |

### ERFA Moon98 vs DE440s

| Error | Max | Mean absolute | RMS | P95 | P99 |
|---|---:|---:|---:|---:|---:|
| Longitude (″) | 14.733176 | 2.128066 | 2.721487 | 5.394884 | 7.702565 |
| Latitude (″) | 5.350862 | 0.793682 | 1.018297 | 1.984146 | 2.929064 |
| Geocentric angle (″) | 14.859823 | 2.407305 | 2.900621 | 5.560528 | 7.845058 |
| Cartesian (km) | 25.796135 | 5.362673 | 6.070552 | 10.713864 | 14.391946 |
| Radius (km) | 11.592882 | 2.295608 | 2.912561 | 5.818175 | 7.807833 |
| Relative radius (ppm) | 30.804704 | 6.003391 | 7.655260 | 15.382949 | 20.728585 |
| Topocentric Equator (0°, 0°, 0 m) (″) | 14.959177 | 2.407459 | 2.900987 | 5.555332 | 7.831877 |
| Topocentric Mid latitude (45°, 100° E, 2,000 m) (″) | 15.101376 | 2.406682 | 2.899679 | 5.563365 | 7.813714 |
| Topocentric High latitude (75° N, 30° W, 100 m) (″) | 14.900734 | 2.406522 | 2.899306 | 5.557569 | 7.839112 |
| Topocentric South highlands (23° S, 67° W, 5,000 m) (″) | 14.713617 | 2.407698 | 2.901394 | 5.555943 | 7.847827 |

Chebyshev holdout: 53,673 epochs after removing fitting nodes.

### Chebyshev f32 vs DE440s (76,464 coefficient bytes)

| Error | Max | Mean absolute | RMS | P95 | P99 |
|---|---:|---:|---:|---:|---:|
| Longitude (″) | 37.383254 | 4.009158 | 6.667746 | 15.890570 | 24.585639 |
| Latitude (″) | 3.175764 | 0.230869 | 0.428304 | 1.004314 | 1.770258 |
| Geocentric angle (″) | 37.241771 | 4.009332 | 6.668104 | 15.885389 | 24.563389 |
| Cartesian (km) | 66.388215 | 11.528563 | 17.068432 | 39.958808 | 53.999803 |
| Radius (km) | 62.949840 | 7.341183 | 12.063738 | 28.659738 | 45.054088 |
| Relative radius (ppm) | 173.969286 | 19.444956 | 32.316389 | 76.658096 | 122.542998 |
| Topocentric Equator (0°, 0°, 0 m) (″) | 37.525220 | 4.010053 | 6.668939 | 15.890696 | 24.580355 |
| Topocentric Mid latitude (45°, 100° E, 2,000 m) (″) | 37.486268 | 4.009902 | 6.667209 | 15.901210 | 24.581031 |
| Topocentric High latitude (75° N, 30° W, 100 m) (″) | 37.676555 | 4.010318 | 6.666203 | 15.879106 | 24.577649 |
| Topocentric South highlands (23° S, 67° W, 5,000 m) (″) | 37.663006 | 4.010508 | 6.669624 | 15.883993 | 24.649839 |

### Chebyshev f32 vs f64 fit

| Error | Max | Mean absolute | RMS | P95 | P99 |
|---|---:|---:|---:|---:|---:|
| Longitude (″) | 0.015410 | 0.001982 | 0.002544 | 0.005014 | 0.007077 |
| Latitude (″) | 0.008780 | 0.001302 | 0.001685 | 0.003449 | 0.004789 |
| Geocentric angle (″) | 0.015467 | 0.002591 | 0.003047 | 0.005536 | 0.007673 |
| Cartesian (km) | 0.032484 | 0.007293 | 0.008105 | 0.013728 | 0.017632 |
| Radius (km) | 0.029141 | 0.004667 | 0.005785 | 0.011238 | 0.014252 |
| Relative radius (ppm) | 0.075070 | 0.012126 | 0.015033 | 0.029093 | 0.037247 |
| Topocentric Equator (0°, 0°, 0 m) (″) | 0.015669 | 0.002591 | 0.003047 | 0.005551 | 0.007667 |
| Topocentric Mid latitude (45°, 100° E, 2,000 m) (″) | 0.015401 | 0.002591 | 0.003047 | 0.005540 | 0.007678 |
| Topocentric High latitude (75° N, 30° W, 100 m) (″) | 0.015624 | 0.002591 | 0.003047 | 0.005535 | 0.007684 |
| Topocentric South highlands (23° S, 67° W, 5,000 m) (″) | 0.015630 | 0.002591 | 0.003047 | 0.005549 | 0.007666 |
