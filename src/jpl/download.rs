// SPDX-License-Identifier: BSD-3-Clause
// Copyright (C) 2026 Vallés Puig, Ramon

//! HTTP download with progress reporting for JPL BSP kernels.

use crate::error::ArchiveError;
use crate::jpl::refs::JplDatasetMeta;
use std::io::{Read, Write};
use std::path::Path;

/// Progress callback: receives `(bytes_downloaded, total_bytes_or_zero)`.
pub type ProgressCallback = Box<dyn Fn(u64, u64)>;

pub(super) fn download(
    name: &str,
    meta: &JplDatasetMeta,
    dest: &Path,
    progress: Option<ProgressCallback>,
) -> Result<(), ArchiveError> {
    let tmp = dest.with_extension("download");

    if let Some(parent) = tmp.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let response = ureq::get(meta.url)
        .call()
        .map_err(|e| ArchiveError::Download(format!("HTTP request failed for {name}: {e}")))?;

    let total: u64 = response
        .headers()
        .get("Content-Length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);

    let mut reader = response.into_body().into_reader();
    let mut file = std::fs::File::create(&tmp)?;
    let mut buf = vec![0u8; 1 << 20];
    let mut downloaded: u64 = 0;

    loop {
        let n = reader.read(&mut buf).map_err(|e| {
            ArchiveError::Download(format!("read error during {name} download: {e}"))
        })?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])?;
        downloaded += n as u64;

        if let Some(ref cb) = progress {
            cb(downloaded, total);
        }
    }

    file.flush()?;
    drop(file);

    let file_size = std::fs::metadata(&tmp)?.len();
    if file_size < meta.min_size {
        let _ = std::fs::remove_file(&tmp);
        return Err(ArchiveError::Integrity(format!(
            "{name}: downloaded file too small ({file_size} bytes, expected >= {})",
            meta.min_size,
        )));
    }

    std::fs::rename(&tmp, dest).map_err(|e| {
        let _ = std::fs::remove_file(&tmp);
        ArchiveError::Io(e)
    })?;

    Ok(())
}
