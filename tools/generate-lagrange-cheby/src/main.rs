// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! Generate Sun-Earth Lagrange Chebyshev archive files for `siderust-archive`.
//!
//! Fits L1–L5 over a Julian-Date span using a selected ephemeris backend and
//! writes Siderust Chebyshev Kernel (`.sck`) binary files plus `manifest.toml`.
//!
//! ## Usage
//!
//! ```sh
//! cd archive
//! cargo run --manifest-path tools/generate-lagrange-cheby/Cargo.toml -- \
//!     --source vsop87 \
//!     --out src/lagrange/vsop87 \
//!     --from 2415020.5 \
//!     --to 2488070.5
//! ```

use siderust::ephemeris::lagrange::fit::FitConfig;
use siderust::ephemeris::{RuntimeEphemeris, Vsop87Ephemeris};
use siderust::qtty::Seconds;
use siderust_archive::checksum;
use std::env;
use std::fs;
use std::path::PathBuf;

/// SCK magic bytes identifying a Siderust Chebyshev Kernel v1 file.
const SCK_MAGIC: &[u8; 8] = b"SCKERN01";

/// Center ID: Solar System Barycenter.
const CENTER_ID_SSB: u32 = 0;

/// Frame ID: Barycentric Ecliptic Mean J2000.
const FRAME_ID_ECLIPTIC_MEAN_J2000: u32 = 0;

/// Time scale ID: TDB-compatible seconds since J2000.
const TIME_SCALE_ID_TDB_J2000_S: u32 = 0;

fn sun_earth_target_id(label: &str) -> u32 {
    match label {
        "l1" => 3_911,
        "l2" => 3_912,
        "l3" => 3_913,
        "l4" => 3_914,
        "l5" => 3_915,
        _ => 0,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let source = value_arg(&args, "--source").unwrap_or("vsop87");
    let bsp_path = value_arg(&args, "--bsp-path");
    let out = PathBuf::from(value_arg(&args, "--out").unwrap_or("src/lagrange/vsop87"));
    let from = parse_jd(
        value_arg(&args, "--from"),
        siderust::ephemeris::lagrange::FIT_FROM_JD,
    )?;
    let to = parse_jd(
        value_arg(&args, "--to"),
        siderust::ephemeris::lagrange::FIT_TO_JD,
    )?;

    fs::create_dir_all(&out)?;
    let block_days_raw = value_arg(&args, "--block-days").unwrap_or("8");
    let block_days: f64 = block_days_raw.parse()?;
    let config = FitConfig {
        from,
        to,
        block: Seconds::new(block_days * siderust::qtty::time::SECONDS_PER_DAY),
        validation_step: Seconds::new(6.0 * 3_600.0),
        ..FitConfig::default()
    };

    match source {
        "vsop87" => generate_for("vsop87", &Vsop87Ephemeris, &out, config)?,
        "de440" | "de441" => {
            let path = bsp_path.ok_or("--bsp-path is required for de440/de441")?;
            let eph = RuntimeEphemeris::from_bsp(path)?;
            generate_for(source, &eph, &out, config)?;
        }
        other => return Err(format!("unsupported --source value: {other}").into()),
    }
    Ok(())
}

fn value_arg<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find_map(|pair| (pair[0] == name).then_some(pair[1].as_str()))
}

fn parse_jd(
    value: Option<&str>,
    default: f64,
) -> Result<siderust::JulianDate, Box<dyn std::error::Error>> {
    let raw = value.map_or(Ok(default), str::parse::<f64>)?;
    siderust::time::try_jd_f64(raw).map_err(|err| err.into())
}

/// Serialise a SCK header into a 64-byte buffer.
fn sck_header(
    ncoeff: u32,
    record_count: u32,
    target_id: u32,
    valid_from_jd: f64,
    valid_to_jd: f64,
) -> [u8; 64] {
    let mut buf = [0u8; 64];
    buf[0..8].copy_from_slice(SCK_MAGIC);
    buf[8..12].copy_from_slice(&ncoeff.to_le_bytes());
    buf[12..16].copy_from_slice(&record_count.to_le_bytes());
    buf[16..20].copy_from_slice(&CENTER_ID_SSB.to_le_bytes());
    buf[20..24].copy_from_slice(&target_id.to_le_bytes());
    buf[24..28].copy_from_slice(&FRAME_ID_ECLIPTIC_MEAN_J2000.to_le_bytes());
    buf[28..32].copy_from_slice(&TIME_SCALE_ID_TDB_J2000_S.to_le_bytes());
    buf[32..40].copy_from_slice(&valid_from_jd.to_le_bytes());
    buf[40..48].copy_from_slice(&valid_to_jd.to_le_bytes());
    buf
}

struct PointManifest {
    label: &'static str,
    file: String,
    ncoeff: usize,
    record_count: usize,
    checksum_sha256: String,
    max_abs_error_m: f64,
    rms_error_m: f64,
    target_id: u32,
}

fn generate_for(
    source_label: &str,
    ephemeris: &dyn siderust::ephemeris::DynEphemeris,
    out: &std::path::Path,
    config: FitConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    use siderust::ephemeris::lagrange::SunEarthLagrangePoint;

    let points: &[(SunEarthLagrangePoint, &str)] = &[
        (SunEarthLagrangePoint::L1, "l1"),
        (SunEarthLagrangePoint::L2, "l2"),
        (SunEarthLagrangePoint::L3, "l3"),
        (SunEarthLagrangePoint::L4, "l4"),
        (SunEarthLagrangePoint::L5, "l5"),
    ];

    let generated_at = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
    let git_commit = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_owned())
        .unwrap_or_else(|| "unknown".to_owned());

    let valid_from_jd = config.from.raw().value();
    let valid_to_jd = config.to.raw().value();

    let mut point_manifests: Vec<PointManifest> = Vec::new();

    for (point, name) in points {
        eprintln!("Fitting {} …", point.label());
        let fitted = match siderust::ephemeris::lagrange::fit::fit_sun_earth_lagrange(
            ephemeris, *point, config,
        ) {
            Ok(f) => f,
            Err(err) => {
                eprintln!(
                    "  {} → SKIPPED (fit failed: {err:?}); \
                     saddle/triangular points require a more robust solver",
                    point.label()
                );
                continue;
            }
        };

        let record_count = fitted.records.len() / (2 + 3 * fitted.ncoeff);
        let target_id = sun_earth_target_id(name);

        let header = sck_header(
            fitted.ncoeff as u32,
            record_count as u32,
            target_id,
            valid_from_jd,
            valid_to_jd,
        );
        let mut file_bytes: Vec<u8> = Vec::with_capacity(64 + fitted.records.len() * 8);
        file_bytes.extend_from_slice(&header);
        for v in &fitted.records {
            file_bytes.extend_from_slice(&v.to_le_bytes());
        }

        let file_name = format!("{name}.sck");
        let file_path = out.join(&file_name);
        fs::write(&file_path, &file_bytes)?;
        eprintln!(
            "  {} → {} ({} records, ncoeff={}, max_err={:.3e} m, rms={:.3e} m)",
            point.label(),
            file_name,
            record_count,
            fitted.ncoeff,
            fitted.stats.max_abs_error.value(),
            fitted.stats.rms_error.value(),
        );

        point_manifests.push(PointManifest {
            label: point.label(),
            file: file_name,
            ncoeff: fitted.ncoeff,
            record_count,
            checksum_sha256: checksum::sha256_hex(&file_bytes),
            max_abs_error_m: fitted.stats.max_abs_error.value(),
            rms_error_m: fitted.stats.rms_error.value(),
            target_id,
        });
    }

    let chebyshev_degree = point_manifests.first().map_or(0, |pm| pm.ncoeff);

    let manifest_toml = build_manifest_toml(
        source_label,
        &generated_at,
        &git_commit,
        valid_from_jd,
        valid_to_jd,
        config.block.value(),
        config.validation_step.value(),
        chebyshev_degree,
        &point_manifests,
    );

    let manifest_path = out.join("manifest.toml");
    fs::write(&manifest_path, &manifest_toml)?;
    eprintln!("Wrote manifest → {}", manifest_path.display());

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn build_manifest_toml(
    source_label: &str,
    generated_at: &str,
    git_commit: &str,
    valid_from_jd: f64,
    valid_to_jd: f64,
    block_seconds: f64,
    validation_step_seconds: f64,
    chebyshev_degree: usize,
    points: &[PointManifest],
) -> String {
    let generator_version = env!("CARGO_PKG_VERSION");
    let dataset_id = format!("lagrange-sun-earth-{source_label}");

    let mut s = String::new();

    s.push_str("schema_version = 1\n");
    s.push_str(&format!("dataset_id = \"{dataset_id}\"\n"));
    s.push_str("dataset_kind = \"lagrange-chebyshev\"\n");
    s.push_str(&format!("source_ephemeris = \"{source_label}\"\n"));
    s.push_str("generator = \"siderust-archive/tools/generate-lagrange-cheby\"\n");
    s.push_str(&format!("generator_version = \"{generator_version}\"\n"));
    s.push_str(&format!("git_commit = \"{git_commit}\"\n"));
    s.push_str(&format!("generated_at = \"{generated_at}\"\n"));
    s.push('\n');
    s.push_str("time_scale = \"TDB-compatible JD\"\n");
    s.push_str("time_offset_epoch = \"J2000.0\"\n");
    s.push_str("frame = \"EclipticMeanJ2000\"\n");
    s.push_str("center = \"Solar-System-Barycenter\"\n");
    s.push_str("center_id = 0\n");
    s.push_str("units = \"km\"\n");
    s.push('\n');
    s.push_str(&format!("valid_from_jd = {valid_from_jd}\n"));
    s.push_str(&format!("valid_to_jd = {valid_to_jd}\n"));
    s.push_str(&format!("block_seconds = {block_seconds}\n"));
    s.push_str(&format!(
        "validation_step_seconds = {validation_step_seconds}\n"
    ));
    s.push_str(&format!("chebyshev_degree = {chebyshev_degree}\n"));
    s.push('\n');
    s.push_str("dynamical_model = \"Restricted-Three-Body (Sun-Earth, analytic)\"\n");
    s.push_str("record_layout = \"sck-v1: header(64 bytes) + [mid_seconds, radius_seconds, x_c0..x_cn, y_c0..y_cn, z_c0..z_cn]\"\n");
    s.push_str("provenance = \"Generated from Siderust Sun-Earth Lagrange solver; coefficients fitted to analytic VSOP87 ephemeris.\"\n");
    s.push('\n');
    s.push_str("[frame_ids]\n");
    s.push_str("EclipticMeanJ2000 = 0\n");
    s.push('\n');
    s.push_str("[time_scale_ids]\n");
    s.push_str("TDB_J2000_seconds = 0\n");
    s.push('\n');
    s.push_str("[target_ids]\n");
    for pm in points {
        s.push_str(&format!("{} = {}\n", pm.label, pm.target_id));
    }
    s.push('\n');
    s.push_str("[[references]]\n");
    s.push_str("citation = \"Szebehely, V. (1967). Theory of Orbits. Academic Press.\"\n");
    s.push('\n');
    s.push_str("[[references]]\n");
    s.push_str(
        "citation = \"Bretagnon, P., Francou, G. (1988). Planetary theories in rectangular and spherical variables. A&A 202, 309-315.\"\n",
    );
    s.push('\n');
    for pm in points {
        s.push_str("[[points]]\n");
        s.push_str(&format!("point = \"{}\"\n", pm.label));
        s.push_str(&format!("target_id = {}\n", pm.target_id));
        s.push_str(&format!("file = \"{}\"\n", pm.file));
        s.push_str(&format!("ncoeff = {}\n", pm.ncoeff));
        s.push_str(&format!("record_count = {}\n", pm.record_count));
        s.push_str(&format!("checksum_sha256 = \"{}\"\n", pm.checksum_sha256));
        s.push_str(&format!("max_abs_error_m = {:.6e}\n", pm.max_abs_error_m));
        s.push_str(&format!("rms_error_m = {:.6e}\n", pm.rms_error_m));
        s.push('\n');
    }

    s
}
