// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! Network integration test for the runtime IERS download path.
//!
//! This test actually contacts IERS / USNO servers, so it is `#[ignore]`d by
//! default. Run it explicitly with:
//!
//! ```text
//! cargo test -p siderust-archive-data --features fetch -- --ignored
//! ```

#![cfg(feature = "fetch")]

use siderust_archive_data::time::TimeDataManager;

#[test]
#[ignore = "performs live network downloads from IERS/USNO"]
fn downloads_and_verifies_iers_bundle() {
    let dir = std::env::temp_dir().join(format!("siderust-archive-it-{}", std::process::id()));
    let manager = TimeDataManager::with_dir(&dir).expect("create manager");

    let bundle = manager
        .refresh_and_load()
        .expect("download, verify, and parse the IERS bundle");

    assert!(
        !bundle.utc_tai_segments().is_empty(),
        "expected leap-second segments"
    );
    assert!(!bundle.eop_points().is_empty(), "expected EOP points");
    assert!(
        bundle.eop_end_mjd() > bundle.eop_observed_end_mjd(),
        "predictions should extend past the observed window"
    );

    // Provenance must carry non-placeholder SHA-256 digests after a fetch.
    let prov = bundle.provenance();
    assert_eq!(prov.eop_finals_sha256().len(), 64);
    assert_ne!(prov.eop_finals_sha256(), "compiled");

    let _ = std::fs::remove_dir_all(&dir);
}
