//! Shared lowercase hex and digest helpers.

use sha2::{Digest, Sha256};

/// Encodes bytes as lowercase hexadecimal.
#[must_use]
pub fn bytes_to_lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

/// Computes a lowercase hex SHA-256 hash for canonical payloads.
#[must_use]
pub fn sha256_hex(value: &[u8]) -> String {
    let digest = Sha256::digest(value);
    bytes_to_lower_hex(&digest)
}
