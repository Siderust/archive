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
//! 3. Prepares the complete validated update before installing it into
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
//! cargo run --features fetch --bin siderust-archive-update-time-data -- --archive-root .
//! ```

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use siderust_archive::time::{
    parse_delta_t_observed, parse_delta_t_predictions, parse_eop_finals, parse_utc_tai_segments,
    TimeDataBundle, TimeDataManager,
};

const RAW_FILES: [&str; 4] = [
    "UTC-TAI.history",
    "deltat.data",
    "deltat.preds",
    "finals2000A.all",
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

    let staging = archive_root.join(".time-update-staging");
    fs::create_dir(&staging)
        .map_err(|e| format!("cannot create staging directory {staging:?}: {e}"))?;

    let result = (|| {
        let manager = TimeDataManager::with_dir(&staging)
            .map_err(|e| format!("cannot initialise TimeDataManager: {e}"))?;
        println!("Downloading and verifying IERS/USNO bundle");
        let bundle = manager.refresh_and_load().map_err(|e| e.to_string())?;
        install_snapshot(&archive_root, &staging, &bundle)
    })();
    if let Err(error) = fs::remove_dir_all(&staging) {
        eprintln!("could not remove staging directory {staging:?}: {error}");
    }
    result
}

fn install_snapshot(
    archive_root: &Path,
    staging: &Path,
    bundle: &TimeDataBundle,
) -> Result<String, String> {
    let eop_dir = archive_root.join("src/time/eop");
    let raw_dir = eop_dir.join("raw");
    let cached = staging.join("bundle");
    validate_coverage(&raw_dir, &cached)?;

    let mut changed = Vec::new();
    for name in RAW_FILES {
        let candidate = fs::read_to_string(cached.join(name))
            .map_err(|e| format!("cannot read staged {name}: {e}"))?;
        let committed = fs::read_to_string(raw_dir.join(name))
            .map_err(|e| format!("cannot read committed {name}: {e}"))?;
        if !same_payload(name, &committed, &candidate) {
            changed.push(name);
        }
    }
    if changed.is_empty() {
        return Ok("All four datasets validated; scientific payloads unchanged.".into());
    }

    // Generate everything in staging before touching the checkout. The workflow
    // validates the resulting manifest and commits all paths together.
    let snapshot = staging.join("snapshot.rs");
    write_bundled_snapshot(&snapshot, bundle)?;
    let manifest = staging.join("manifest.toml");
    copy(&eop_dir.join("manifest.toml"), &manifest)?;
    fs::rename(&cached, staging.join("raw")).map_err(|e| format!("cannot stage raw files: {e}"))?;
    update_manifest_files_block(&manifest, staging)?;

    for name in RAW_FILES.into_iter().chain([PROVENANCE_FILE]) {
        install(&staging.join("raw").join(name), &raw_dir.join(name))?;
    }
    install(&manifest, &eop_dir.join("manifest.toml"))?;
    install(
        &snapshot,
        &archive_root.join("src/time/bundled/snapshot.rs"),
    )?;
    Ok(format!(
        "Validated and refreshed datasets: {}",
        changed.join(", ")
    ))
}

fn same_payload(name: &str, committed: &str, candidate: &str) -> bool {
    if name == "finals2000A.all" {
        // Preserve fixed columns, including uncertainty and Bulletin B fields
        // that the runtime parser does not consume.
        fn rows(text: &str) -> impl Iterator<Item = &str> {
            text.lines()
                .map(str::trim_end)
                .filter(|line| !line.is_empty())
        }
        rows(committed).eq(rows(candidate))
    } else {
        fn rows(text: &str) -> impl Iterator<Item = std::str::SplitAsciiWhitespace<'_>> {
            text.lines()
                .map(str::split_ascii_whitespace)
                .filter(|fields| fields.clone().next().is_some())
        }
        let mut candidate_rows = rows(candidate);
        rows(committed).all(|fields| candidate_rows.next().is_some_and(|row| fields.eq(row)))
            && candidate_rows.next().is_none()
    }
}

fn copy(from: &Path, to: &Path) -> Result<(), String> {
    fs::copy(from, to).map_err(|e| format!("cannot copy {from:?} to {to:?}: {e}"))?;
    Ok(())
}

fn install(from: &Path, to: &Path) -> Result<(), String> {
    fs::rename(from, to).map_err(|e| format!("cannot install {to:?}: {e}"))
}

/// The public parsers also accept short historical fixtures. Maintenance must
/// reject a parseable prefix or a dataset that lost committed coverage.
fn validate_coverage(committed: &Path, candidate: &Path) -> Result<(), String> {
    fn check<T>(
        committed: &Path,
        candidate: &Path,
        name: &str,
        parse: impl Fn(&str) -> Result<Vec<T>, String>,
        complete: impl Fn(&[T], &[T]) -> bool,
    ) -> Result<(), String> {
        let load = |dir: &Path| {
            let path = dir.join(name);
            let text =
                fs::read_to_string(&path).map_err(|e| format!("cannot read {path:?}: {e}"))?;
            parse(&text).map_err(|e| format!("invalid scientific data in {path:?}: {e}"))
        };
        if !complete(&load(committed)?, &load(candidate)?) {
            return Err(format!(
                "invalid scientific data in {name}: snapshot coverage regressed"
            ));
        }
        Ok(())
    }

    check(
        committed,
        candidate,
        "UTC-TAI.history",
        parse_utc_tai_segments,
        |old, new| {
            new.len() >= old.len()
                && new[0].start_mjd <= old[0].start_mjd
                && new.last().unwrap().end_mjd.is_none()
        },
    )?;
    check(
        committed,
        candidate,
        "deltat.data",
        |text| {
            let points = parse_delta_t_observed(text)?;
            if points.len() != text.lines().filter(|line| !line.trim().is_empty()).count() {
                return Err("malformed row in observed Delta T".into());
            }
            Ok(points)
        },
        |old, new| {
            new.len() >= old.len()
                && old.iter().all(|row| {
                    new.binary_search_by(|point| point.0.total_cmp(&row.0))
                        .is_ok()
                })
        },
    )?;
    check(
        committed,
        candidate,
        "deltat.preds",
        |text| {
            let points = parse_delta_t_predictions(text)?;
            let rows = text
                .lines()
                .filter(|line| {
                    line.split_whitespace()
                        .next()
                        .is_some_and(|field| field != "MJD")
                })
                .count();
            if points.len() != rows {
                return Err("malformed row in Delta T predictions".into());
            }
            Ok(points)
        },
        |old, new| {
            // The prediction window can roll forward as historical predictions expire.
            let last_old = old.last().unwrap().0;
            let max_gap = old
                .windows(2)
                .map(|rows| rows[1].0 - rows[0].0)
                .fold(0.0, f64::max);
            new[0].0 <= last_old
                && new.last().unwrap().0 >= last_old
                && new.len() >= 2
                && new
                    .windows(2)
                    .all(|rows| rows[1].0 - rows[0].0 <= max_gap + 1.0)
        },
    )?;
    check(
        committed,
        candidate,
        "finals2000A.all",
        parse_eop_finals,
        |old, new| {
            if new[0].mjd > old[0].mjd || new.last().unwrap().mjd < old.last().unwrap().mjd {
                return false;
            }
            // The parser guarantees contiguous daily coverage. Verify that an
            // overlapping row has not lost observed status or a measured field.
            let offset = (old[0].mjd - new[0].mjd) as usize;
            old.iter().zip(&new[offset..]).all(|(old, new)| {
                (!old.ut1_observed || new.ut1_observed)
                    && (!old.pm_observed || new.pm_observed)
                    && (!old.nutation_observed || new.nutation_observed)
                    && (old.pm_xp.is_none() || new.pm_xp.is_some())
                    && (old.pm_yp.is_none() || new.pm_yp.is_some())
                    && (old.lod.is_none() || new.lod.is_some())
                    && (old.dx.is_none() || new.dx.is_some())
                    && (old.dy.is_none() || new.dy.is_some())
            })
        },
    )
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
        "pub const PRE_1961_TAI_MINUS_UTC_APPROX: f64 = {PRE_1961_TAI_MINUS_UTC_APPROX:.1};\n"
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_requires_a_scientific_change_and_preserves_snapshot_on_rejection() {
        let root = env::temp_dir().join(format!(
            "archive-update-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&root).unwrap();
        struct TempDir(PathBuf);
        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
        let _temp = TempDir(root.clone());
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/time");
        let eop = root.join("src/time/eop");
        let raw = eop.join("raw");
        let snapshot = root.join("src/time/bundled/snapshot.rs");
        let staging = root.join("staging");
        let cached = staging.join("bundle");
        fs::create_dir_all(&raw).unwrap();
        fs::create_dir_all(&cached).unwrap();
        fs::create_dir_all(snapshot.parent().unwrap()).unwrap();
        for name in RAW_FILES.into_iter().chain([PROVENANCE_FILE]) {
            fs::copy(fixture.join("eop/raw").join(name), raw.join(name)).unwrap();
            fs::copy(raw.join(name), cached.join(name)).unwrap();
        }
        let manifest = eop.join("manifest.toml");
        fs::copy(fixture.join("eop/manifest.toml"), &manifest).unwrap();
        fs::copy(fixture.join("bundled/snapshot.rs"), &snapshot).unwrap();
        let committed = || {
            [
                &raw.join(PROVENANCE_FILE),
                &raw.join("deltat.data"),
                &manifest,
                &snapshot,
            ]
            .map(|path| fs::read(path).unwrap())
        };
        let before = committed();
        let provenance = fs::read_to_string(raw.join(PROVENANCE_FILE)).unwrap();
        let mut metadata = provenance.parse::<toml::Table>().unwrap();
        metadata.insert("fetched_utc".into(), "2026-11-12T00:00:00".into());
        let manager = TimeDataManager::with_dir(&staging).unwrap();
        let install = |metadata: &toml::Table| {
            fs::write(
                cached.join(PROVENANCE_FILE),
                toml::to_string(metadata).unwrap(),
            )
            .unwrap();
            let bundle = manager.load_cached().unwrap();
            install_snapshot(&root, &staging, &bundle)
        };

        assert!(install(&metadata).unwrap().contains("unchanged"));
        assert_eq!(committed(), before);

        let finals = fs::read_to_string(raw.join("finals2000A.all")).unwrap();
        let mut missing_pm = finals.clone();
        missing_pm.replace_range(18..27, "         ");
        fs::write(cached.join("finals2000A.all"), &missing_pm).unwrap();
        metadata.insert(
            "eop_finals_sha256".into(),
            sha256_hex(missing_pm.as_bytes()).into(),
        );
        assert!(install(&metadata)
            .unwrap_err()
            .contains("finals2000A.all: snapshot coverage regressed"));
        assert_eq!(committed(), before);
        assert_eq!(
            fs::read_to_string(raw.join("finals2000A.all")).unwrap(),
            finals
        );
        fs::write(cached.join("finals2000A.all"), &finals).unwrap();
        metadata.insert(
            "eop_finals_sha256".into(),
            sha256_hex(finals.as_bytes()).into(),
        );

        let observed = fs::read_to_string(raw.join("deltat.data")).unwrap();
        let reformatted = observed.replace(' ', "\t");
        fs::write(cached.join("deltat.data"), &reformatted).unwrap();
        metadata.insert(
            "delta_t_observed_sha256".into(),
            sha256_hex(reformatted.as_bytes()).into(),
        );
        assert!(install(&metadata).unwrap().contains("unchanged"));
        assert_eq!(committed(), before);

        let truncated = observed.lines().take(2).collect::<Vec<_>>().join("\n");
        fs::write(cached.join("deltat.data"), &truncated).unwrap();
        assert!(manager.load_cached().is_err()); // Payload no longer matches the checksum.
        metadata.insert(
            "delta_t_observed_sha256".into(),
            sha256_hex(truncated.as_bytes()).into(),
        );
        assert!(install(&metadata)
            .unwrap_err()
            .contains("coverage regressed"));
        assert_eq!(committed(), before);

        let corrected = observed.replacen("43.4724", "43.4725", 1);
        assert_ne!(corrected, observed);
        fs::write(cached.join("deltat.data"), &corrected).unwrap();
        metadata.insert(
            "delta_t_observed_sha256".into(),
            sha256_hex(corrected.as_bytes()).into(),
        );
        assert_eq!(
            install(&metadata).unwrap(),
            "Validated and refreshed datasets: deltat.data"
        );
        assert_eq!(
            fs::read_to_string(raw.join("deltat.data")).unwrap(),
            corrected
        );
        assert_ne!(committed(), before);
    }
}
