// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 Vallés Puig, Ramon

//! SHA-256 checksum helpers used to verify archive datasets against the
//! checksums recorded in their manifests / provenance records.

/// Compute the lowercase hex SHA-256 digest of `bytes`.
///
/// ```
/// let hex = siderust_archive::checksum::sha256_hex(b"abc");
/// assert_eq!(
///     hex,
///     "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
/// );
/// ```
#[cfg(feature = "fetch")]
pub fn sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    to_hex(&hasher.finalize())
}

/// Lowercase hex encoding of a byte slice.
pub fn to_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// Error returned when an actual checksum does not match the expected one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecksumMismatch {
    pub label: String,
    pub expected: String,
    pub actual: String,
}

impl std::fmt::Display for ChecksumMismatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} SHA-256 mismatch: expected {}, got {}",
            self.label, self.expected, self.actual
        )
    }
}

impl std::error::Error for ChecksumMismatch {}

/// Verify that `bytes` hash to the `expected` lowercase-hex SHA-256 digest.
#[cfg(feature = "fetch")]
pub fn verify_sha256(label: &str, bytes: &[u8], expected: &str) -> Result<(), ChecksumMismatch> {
    let actual = sha256_hex(bytes);
    if actual != expected {
        return Err(ChecksumMismatch {
            label: label.to_string(),
            expected: expected.to_string(),
            actual,
        });
    }
    Ok(())
}

#[cfg(all(test, feature = "fetch"))]
mod tests {
    use super::*;

    #[test]
    fn sha256_of_abc_is_known_vector() {
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn verify_accepts_matching_digest() {
        let expected = sha256_hex(b"hello archive");
        assert!(verify_sha256("sample", b"hello archive", &expected).is_ok());
    }

    #[test]
    fn verify_rejects_mismatch() {
        let err = verify_sha256("sample", b"hello", "deadbeef").unwrap_err();
        assert_eq!(err.label, "sample");
        assert_eq!(err.expected, "deadbeef");
    }
}
