// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! Shared error type for archive dataset operations.
//!
//! [`ArchiveError`] is the root error returned by all archive I/O, download,
//! integrity, and parse operations. Individual modules re-export or wrap it as
//! appropriate.

use std::fmt;

/// Error raised during an archive dataset operation.
#[derive(Debug)]
pub enum ArchiveError {
    /// File-system or OS I/O error.
    Io(std::io::Error),
    /// HTTP or network download failed.
    Download(String),
    /// Data integrity check failed (checksum mismatch, wrong size, …).
    Integrity(String),
    /// Text or binary parse error.
    Parse(String),
}

impl fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "I/O error: {err}"),
            Self::Download(msg) => write!(f, "download error: {msg}"),
            Self::Integrity(msg) => write!(f, "integrity error: {msg}"),
            Self::Parse(msg) => write!(f, "parse error: {msg}"),
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ArchiveError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_io_error() {
        let err = ArchiveError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "missing file",
        ));
        assert!(err.to_string().contains("I/O error"));
    }

    #[test]
    fn display_integrity_error() {
        let err = ArchiveError::Integrity("sha256 mismatch".into());
        assert!(err.to_string().contains("integrity error"));
        assert!(err.to_string().contains("sha256 mismatch"));
    }

    #[test]
    fn display_parse_error() {
        let err = ArchiveError::Parse("unexpected token".into());
        assert!(err.to_string().contains("parse error"));
    }

    #[test]
    fn display_download_error() {
        let err = ArchiveError::Download("timeout".into());
        assert!(err.to_string().contains("download error"));
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let arch_err = ArchiveError::from(io_err);
        assert!(matches!(arch_err, ArchiveError::Io(_)));
    }
}
