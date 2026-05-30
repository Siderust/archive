// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! Build script for `siderust-archive`.
//!
//! When the `vsop`, `elp`, `atmosphere`, `gravity`, `pluto`, or `nutation`
//! features are enabled, this script regenerates the Rust bindings for the
//! respective coefficient tables into `OUT_DIR`.  Each module then exposes
//! the generated constants via `include!` in its sub-module.

use std::env;
use std::path::PathBuf;

#[cfg(feature = "vsop")]
#[path = "src/vsop/pipeline.rs"]
mod vsop_pipeline;

#[cfg(feature = "elp")]
#[path = "src/elp/pipeline.rs"]
mod elp_pipeline;

#[cfg(feature = "atmosphere")]
#[path = "src/atmosphere/pipeline.rs"]
mod atmosphere_pipeline;

#[cfg(feature = "gravity")]
#[path = "src/gravity/pipeline.rs"]
mod gravity_pipeline;

#[cfg(feature = "pluto")]
#[path = "src/pluto/pipeline.rs"]
mod pluto_pipeline;

#[cfg(feature = "nutation")]
#[path = "src/nutation/pipeline.rs"]
mod nutation_pipeline;

fn main() {
    #[allow(unused_variables)]
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    #[allow(unused_variables)]
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR"));
    #[allow(unused_variables)]
    let archive_root = &manifest_dir;

    #[cfg(feature = "vsop")]
    {
        let vsop_raw = archive_root.join("src").join("vsop").join("raw");
        println!("cargo:rerun-if-changed={}", vsop_raw.display());
        vsop_pipeline::run_regen(&vsop_raw, &out_dir).expect("VSOP87 table generation failed");
    }

    #[cfg(feature = "elp")]
    {
        let elp_raw = archive_root.join("src").join("elp").join("raw");
        println!("cargo:rerun-if-changed={}", elp_raw.display());
        elp_pipeline::run_regen(&elp_raw, &out_dir).expect("ELP2000 table generation failed");
    }

    #[cfg(feature = "atmosphere")]
    {
        let raw = archive_root.join("src").join("atmosphere").join("raw");
        atmosphere_pipeline::run_regen(&raw, &out_dir)
            .expect("NRLMSISE-00 atmosphere table generation failed");
    }

    #[cfg(feature = "gravity")]
    {
        let raw = archive_root.join("src").join("gravity").join("raw");
        gravity_pipeline::run_regen(&raw, &out_dir)
            .expect("EGM2008 gravity table generation failed");
    }

    #[cfg(feature = "pluto")]
    {
        let raw = archive_root.join("src").join("pluto").join("raw");
        pluto_pipeline::run_regen(&raw, &out_dir).expect("Pluto series table generation failed");
    }

    #[cfg(feature = "nutation")]
    {
        let raw = archive_root.join("src").join("nutation").join("raw");
        nutation_pipeline::run_regen(&raw, &out_dir)
            .expect("IAU 2000A/B nutation table generation failed");
    }
}
