// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon
//
//! Maintenance binary that refreshes the canonical IERS time-data files inside
//! the Siderust Archive repository.
//!
//! What it does
//! ------------
//! 1. Downloads the four upstream IERS / USNO files (UTC-TAI.history,
//!    deltat.data, deltat.preds, finals2000A.all) using
//!    [`TimeDataManager`](siderust_archive::time::TimeDataManager) into a fresh
//!    cache directory.
//! 2. Verifies that the bundle parses correctly and computes SHA-256 hashes.
//! 3. Atomically swaps the downloaded raw files into
//!    `<archive>/src/time/eop/raw/`.
//! 4. Writes a `time_data.provenance.toml` recording fetch timestamp and per-
//!    file SHA-256.
//! 5. Regenerates the compiled bundled snapshot at
//!    `src/time/bundled/snapshot.rs`.
//! 6. Exits non-zero if anything fails so CI can react.
//!
//! Downstream crates (e.g. tempoch, siderust) consume datasets via
//! `siderust-archive` at runtime or through the bundled snapshot.
//!
//! Usage
//! -----
//! ```text
//! siderust-archive-update-time-data --archive-root <path-to-archive-repo>
//! ```
//!
//! The default `--archive-root` is `CARGO_MANIFEST_DIR`, which makes the tool
//! work out of the box when run from inside the archive workspace:
//! ```text
//! cargo run --bin update-time-data --features fetch
//! ```

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use siderust_archive::time::TimeDataManager;

const RAW_FILES: [(&str, &str); 4] = [
    ("UTC-TAI.history", "utc_tai_sha256"),
    ("deltat.data", "delta_t_observed_sha256"),
    ("deltat.preds", "delta_t_predictions_sha256"),
    ("finals2000A.all", "eop_finals_sha256"),
];

const PROVENANCE_FILE: &str = "time_data.provenance.toml";

/// Paths and format identifiers for all committed time-data files.
/// Paths are relative to the `src/time/eop/` directory (the manifest directory).
const MANIFEST_FILE_SPECS: [(&str, &str); 5] = [
    ("raw/UTC-TAI.history", "iers-utc-tai-history"),
    ("raw/deltat.data", "usno-delta-t-observed"),
    ("raw/deltat.preds", "usno-delta-t-predictions"),
    ("raw/finals2000A.all", "iers-finals2000a"),
    (
        "raw/time_data.provenance.toml",
        "siderust-time-provenance-toml",
    ),
];

const MARKER_BEGIN: &str = "# BEGIN GENERATED FILE ENTRIES \u{2014} update-time-data";
const MARKER_END: &str = "# END GENERATED FILE ENTRIES \u{2014} update-time-data";

struct CliArgs {
    archive_root: PathBuf,
}

fn print_usage() {
    eprintln!(
        "siderust-archive-update-time-data\n\
         \n\
         USAGE:\n\
             siderust-archive-update-time-data [--archive-root <PATH>]\n\
         \n\
         OPTIONS:\n\
             --archive-root <PATH>   Path to the archive repository root.\n\
                                     Defaults to CARGO_MANIFEST_DIR if run from\n\
                                     inside the workspace, otherwise the current\n\
                                     directory.\n\
             -h, --help              Show this message.\n"
    );
}

fn parse_args() -> Result<CliArgs, String> {
    let mut args = env::args().skip(1);
    let mut archive_root: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--archive-root" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--archive-root requires a value".to_string())?;
                archive_root = Some(PathBuf::from(value));
            }
            "-h" | "--help" => {
                print_usage();
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument: {other}")),
        }
    }
    let archive_root = archive_root.unwrap_or_else(default_archive_root);
    Ok(CliArgs { archive_root })
}

fn default_archive_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is the archive crate root (archive/Cargo.toml).
    if let Some(manifest_dir) = option_env!("CARGO_MANIFEST_DIR") {
        return PathBuf::from(manifest_dir);
    }
    env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(err) => {
            eprintln!("error: {err}");
            print_usage();
            return ExitCode::from(2);
        }
    };

    match run(&args) {
        Ok(summary) => {
            println!("{summary}");
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: &CliArgs) -> Result<String, String> {
    let archive_root = args.archive_root.canonicalize().map_err(|e| {
        format!(
            "cannot canonicalize archive root {:?}: {e}",
            args.archive_root
        )
    })?;

    if !archive_root.join("MANIFEST.toml").exists() {
        return Err(format!(
            "archive root {:?} does not look like a Siderust Archive checkout \
             (MANIFEST.toml not found)",
            archive_root
        ));
    }

    let eop_dir = archive_root.join("src").join("time").join("eop");
    let raw_dir = eop_dir.join("raw");
    fs::create_dir_all(&raw_dir)
        .map_err(|e| format!("cannot create raw dir {:?}: {e}", raw_dir))?;

    let staging = archive_root.join(".time-update-staging");
    if staging.exists() {
        fs::remove_dir_all(&staging)
            .map_err(|e| format!("cannot clear staging dir {:?}: {e}", staging))?;
    }
    fs::create_dir_all(&staging)
        .map_err(|e| format!("cannot create staging dir {:?}: {e}", staging))?;

    let manager = TimeDataManager::with_dir(&staging)
        .map_err(|e| format!("cannot initialise TimeDataManager: {e}"))?;

    println!("Downloading and verifying IERS bundle into {:?}", staging);
    let bundle = manager
        .refresh_and_load()
        .map_err(|e| format!("download or verification failed: {e}"))?;

    let cached = staging.join("bundle");
    let mut changed_files: Vec<&'static str> = Vec::new();

    for (name, _sha_field) in RAW_FILES {
        let src = cached.join(name);
        if !src.exists() {
            return Err(format!("expected cached file missing: {:?}", src));
        }
        let dst = raw_dir.join(name);
        if file_contents_differ(&src, &dst) {
            fs::copy(&src, &dst).map_err(|e| format!("cannot copy {:?} -> {:?}: {e}", src, dst))?;
            changed_files.push(name);
        }
    }

    let src_prov = cached.join(PROVENANCE_FILE);
    let dst_prov = raw_dir.join(PROVENANCE_FILE);
    if !src_prov.exists() {
        return Err(format!(
            "TimeDataManager did not write {} into the staging bundle",
            PROVENANCE_FILE
        ));
    }
    let prov_changed = file_contents_differ(&src_prov, &dst_prov);
    if prov_changed {
        fs::copy(&src_prov, &dst_prov).map_err(|e| {
            format!(
                "cannot copy provenance {:?} -> {:?}: {e}",
                src_prov, dst_prov
            )
        })?;
    }

    // Snapshot path is now directly under src/ (not under crates/siderust-archive/).
    let snapshot_path = archive_root
        .join("src")
        .join("time")
        .join("bundled")
        .join("snapshot.rs");
    write_bundled_snapshot(&snapshot_path, &bundle)?;

    let manifest_path = eop_dir.join("manifest.toml");
    update_manifest_files_block(&manifest_path, &eop_dir)?;

    fs::remove_dir_all(&staging).ok();

    let summary = format!(
        "Bundle refreshed successfully.\n\
         Archive root:   {archive:?}\n\
         Raw dir:        {raw:?}\n\
         UTC-TAI segments: {leap}\n\
         Modern ΔT points: {delta}\n\
         EOP points:       {eop}\n\
         Data changed:     {changed}\n\
         Provenance file:  {prov}\n\
         Bundled snapshot: {snap:?}",
        archive = archive_root,
        raw = raw_dir,
        leap = bundle.utc_tai_segments().len(),
        delta = bundle.modern_delta_t_points().len(),
        eop = bundle.eop_points().len(),
        changed = if changed_files.is_empty() {
            "none".to_string()
        } else {
            changed_files.join(", ")
        },
        prov = if prov_changed { "updated" } else { "unchanged" },
        snap = snapshot_path,
    );
    Ok(summary)
}

fn sha256_hex(data: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data);
    let hash = hasher.finalize();
    hash.iter().map(|b| format!("{b:02x}")).collect()
}

/// Rewrites the managed `[[files]]` block in `manifest_path` with up-to-date
/// SHA-256 checksums and byte counts for every committed time-data file.
///
/// The managed region is delimited by:
/// ```text
/// # BEGIN GENERATED FILE ENTRIES — update-time-data
/// # END GENERATED FILE ENTRIES — update-time-data
/// ```
/// Both markers must already be present in the manifest; the function fails if
/// either is missing or if the begin marker follows the end marker.
fn update_manifest_files_block(manifest_path: &Path, manifest_dir: &Path) -> Result<(), String> {
    let mut entries = String::new();
    for (rel_path, format) in MANIFEST_FILE_SPECS {
        let file_path = manifest_dir.join(rel_path);
        let data = fs::read(&file_path).map_err(|e| {
            format!(
                "cannot read {:?} for manifest update (run the updater to populate raw files): {e}",
                file_path
            )
        })?;
        let bytes = data.len() as u64;
        let sha256 = sha256_hex(&data);
        entries.push_str(&format!(
            "[[files]]\npath   = \"{rel_path}\"\nformat = \"{format}\"\nsha256 = \"{sha256}\"\nbytes  = {bytes}\n\n"
        ));
    }
    // Drop the trailing blank line after the last entry.
    let entries = entries.trim_end_matches('\n').to_string() + "\n";

    let content = fs::read_to_string(manifest_path)
        .map_err(|e| format!("cannot read manifest {:?}: {e}", manifest_path))?;

    let begin_pos = content
        .find(MARKER_BEGIN)
        .ok_or_else(|| format!("manifest {:?} is missing the begin marker", manifest_path))?;
    let end_pos = content
        .find(MARKER_END)
        .ok_or_else(|| format!("manifest {:?} is missing the end marker", manifest_path))?;
    if begin_pos >= end_pos {
        return Err(format!(
            "manifest {:?}: begin marker appears after end marker",
            manifest_path
        ));
    }

    let before = &content[..begin_pos];
    let after_end = end_pos + MARKER_END.len();
    let after = &content[after_end..];

    let new_content = format!("{before}{MARKER_BEGIN}\n{entries}{MARKER_END}{after}");

    let tmp = manifest_path.with_file_name("manifest.toml.new");
    fs::write(&tmp, &new_content).map_err(|e| format!("cannot write manifest {:?}: {e}", tmp))?;
    fs::rename(&tmp, manifest_path)
        .map_err(|e| format!("cannot install manifest {:?}: {e}", manifest_path))?;
    Ok(())
}

fn write_bundled_snapshot(
    path: &Path,
    bundle: &siderust_archive::time::TimeDataBundle,
) -> Result<(), String> {
    use siderust_archive::time::{
        DELTA_T_OBSERVED_URL, DELTA_T_PREDICTIONS_URL, PRE_1961_TAI_MINUS_UTC_APPROX,
        UTC_TAI_HISTORY_URL,
    };

    let prov = bundle.provenance();
    let mut out = String::new();
    out.push_str("// SPDX-License-Identifier: BSD-3-Clause\n");
    out.push_str("// Copyright (C) 2026 Vallés Puig, Ramon\n//\n");
    out.push_str("// @generated by siderust-archive-update-time-data\n");
    out.push_str("// Do not edit this file manually.\n//\n");
    out.push_str(&format!(
        "//   UTC-TAI history  SHA-256: {}\n",
        prov.utc_tai_sha256()
    ));
    out.push_str(&format!(
        "//   ΔT observed      SHA-256: {}\n",
        prov.delta_t_observed_sha256()
    ));
    out.push_str(&format!(
        "//   ΔT predictions   SHA-256: {}\n\n",
        prov.delta_t_predictions_sha256()
    ));

    out.push_str("#[derive(Debug, Clone, Copy, PartialEq)]\n");
    out.push_str("pub struct UtcTaiSegment {\n");
    out.push_str("    pub start_mjd: i32,\n");
    out.push_str("    pub end_mjd: Option<i32>,\n");
    out.push_str("    pub base_seconds: f64,\n");
    out.push_str("    pub reference_mjd: f64,\n");
    out.push_str("    pub slope_seconds_per_day: f64,\n");
    out.push_str("}\n\n");

    out.push_str(&format!(
        "pub const PRE_1961_TAI_MINUS_UTC_APPROX: f64 = {PRE_1961_TAI_MINUS_UTC_APPROX};\n"
    ));
    out.push_str(&format!(
        "pub const UTC_TAI_HISTORY_URL: &str = \"{UTC_TAI_HISTORY_URL}\";\n"
    ));
    out.push_str(&format!(
        "pub const DELTA_T_OBSERVED_URL: &str = \"{DELTA_T_OBSERVED_URL}\";\n"
    ));
    out.push_str(&format!(
        "pub const DELTA_T_PREDICTIONS_URL: &str = \"{DELTA_T_PREDICTIONS_URL}\";\n"
    ));

    let segments = bundle.utc_tai_segments();
    let start_mjd = segments.first().map(|s| s.start_mjd).unwrap_or(0);
    out.push_str(&format!(
        "pub const UTC_TAI_HISTORY_START_MJD: i32 = {start_mjd};\n"
    ));

    let delta_points = bundle.modern_delta_t_points();
    let delta_start = delta_points.first().map(|(m, _)| *m).unwrap_or(0.0);
    let delta_obs_end = bundle.modern_delta_t_observed_end_mjd();
    let delta_end = delta_points
        .last()
        .map(|(m, _)| *m)
        .unwrap_or(delta_obs_end);
    out.push_str(&format!(
        "pub const MODERN_DELTA_T_START_MJD: f64 = {delta_start:.3};\n"
    ));
    out.push_str(&format!(
        "pub const MODERN_DELTA_T_OBSERVED_END_MJD: f64 = {delta_obs_end:.3};\n"
    ));
    out.push_str(&format!(
        "pub const MODERN_DELTA_T_END_MJD: f64 = {delta_end:.3};\n\n"
    ));

    out.push_str("#[rustfmt::skip]\n");
    out.push_str(&format!(
        "pub const UTC_TAI_SEGMENTS: [UtcTaiSegment; {}] = [\n",
        segments.len()
    ));
    for seg in segments {
        let end = match seg.end_mjd {
            Some(v) => format!("Some({v})"),
            None => "None".to_string(),
        };
        out.push_str("    UtcTaiSegment {\n");
        out.push_str(&format!("        start_mjd: {},\n", seg.start_mjd));
        out.push_str(&format!("        end_mjd: {end},\n"));
        out.push_str(&format!("        base_seconds: {:.7},\n", seg.base.value()));
        out.push_str(&format!(
            "        reference_mjd: {:.1},\n",
            seg.reference_mjd
        ));
        out.push_str(&format!(
            "        slope_seconds_per_day: {:.7},\n",
            seg.slope_seconds_per_day
        ));
        out.push_str("    },\n");
    }
    out.push_str("];\n\n");

    out.push_str("#[rustfmt::skip]\n");
    out.push_str(&format!(
        "pub const MODERN_DELTA_T_POINTS: [(f64, f64); {}] = [\n",
        delta_points.len()
    ));
    for (mjd, dt) in delta_points {
        out.push_str(&format!("    ({mjd:.3}, {dt:.4}),\n"));
    }
    out.push_str("];\n");

    let tmp = path.with_extension("rs.new");
    fs::write(&tmp, out).map_err(|e| format!("cannot write bundled snapshot {:?}: {e}", tmp))?;
    fs::rename(&tmp, path)
        .map_err(|e| format!("cannot install bundled snapshot {:?}: {e}", path))?;
    Ok(())
}

fn file_contents_differ(a: &Path, b: &Path) -> bool {
    if !b.exists() {
        return true;
    }
    match (fs::read(a), fs::read(b)) {
        (Ok(va), Ok(vb)) => va != vb,
        _ => true,
    }
}
