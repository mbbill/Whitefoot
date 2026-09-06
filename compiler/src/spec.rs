use core::fmt;

/// SHA-256, so the active specification's identity can be derived from its own
/// bytes instead of being taken on trust.
///
/// The crate has no dependencies and keeps none, so this is written by hand.
/// It is a `const fn` and usable in constant position for small inputs, but the
/// active specification is hashed at runtime: see `computed_active_spec_hash`
/// for why. Names inside the compression function follow FIPS 180-4 section 6.2
/// so the code can be read against the standard.
///
/// The documentation lives here rather than as a `//!` header inside the file
/// because `build.rs` includes those same bytes to derive the identity, and an
/// inner attribute cannot appear in an `include!` expansion.
mod sha256;

/// The SHA-256 identity of one exact kernel specification.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpecHash([u8; 32]);

impl SpecHash {
    /// Create an identity from its exact SHA-256 bytes.
    #[must_use]
    pub const fn from_sha256(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Return the exact SHA-256 bytes.
    #[must_use]
    pub const fn as_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl fmt::Debug for SpecHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for SpecHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// Version label of the active kernel specification, from the generated
/// identity module.
pub const ACTIVE_KERNEL_SPEC_VERSION: &str = crate::spec_identity::SPEC_VERSION;

/// Repository-relative stable path of the active kernel specification.
pub const ACTIVE_KERNEL_SPEC_PATH: &str = "spec/kernel-spec.md";

/// Exact UTF-8 text of the active kernel specification.
pub const ACTIVE_KERNEL_SPEC_TEXT: &str = include_str!("../../spec/kernel-spec.md");

/// Exact bytes of the active kernel specification.
pub const ACTIVE_KERNEL_SPEC_BYTES: &[u8] = ACTIVE_KERNEL_SPEC_TEXT.as_bytes();

/// SHA-256 identity of the active kernel specification, decoded at compile
/// time from the generated identity module.
///
/// Decoded rather than computed here only because a constant is re-evaluated
/// in every crate that reads it, and hashing the whole specification in the
/// constant evaluator costs about twelve seconds per crate. It is checked
/// against the bytes rather than trusted: [`computed_active_spec_hash`] hashes
/// them at runtime and the `whitefoot-spec` gate rejects any disagreement, so
/// installing a specification cannot leave this naming the previous one.
pub const ACTIVE_KERNEL_SPEC_HASH: SpecHash =
    SpecHash::from_sha256(sha256_from_hex(crate::spec_identity::SPEC_SHA256_HEX));

/// Decode 64 lowercase hex digits into 32 bytes at compile time. A malformed
/// digest is a compile-time panic, never a wrong identity.
const fn sha256_from_hex(hex: &str) -> [u8; 32] {
    let digits = hex.as_bytes();
    assert!(digits.len() == 64, "a SHA-256 digest is 64 hex digits");
    let mut bytes = [0u8; 32];
    let mut index = 0;
    while index < 32 {
        bytes[index] = (hex_value(digits[2 * index]) << 4) | hex_value(digits[2 * index + 1]);
        index += 1;
    }
    bytes
}

const fn hex_value(digit: u8) -> u8 {
    match digit {
        b'0'..=b'9' => digit - b'0',
        b'a'..=b'f' => digit - b'a' + 10,
        _ => panic!("a SHA-256 digest is lowercase hex"),
    }
}

/// SHA-256 of the embedded active specification, computed from its bytes.
///
/// The one quantity in this module that no one transcribes. Comparing it with
/// the digest the generated identity module names, which the `whitefoot-spec`
/// gate does, compares this implementation against the independently measured
/// `shasum -a 256` the record was written from.
#[must_use]
pub fn computed_active_spec_hash() -> SpecHash {
    SpecHash::from_sha256(sha256::digest(ACTIVE_KERNEL_SPEC_BYTES))
}

#[cfg(test)]
mod tests {
    use super::{
        ACTIVE_KERNEL_SPEC_HASH, ACTIVE_KERNEL_SPEC_PATH, ACTIVE_KERNEL_SPEC_TEXT,
        ACTIVE_KERNEL_SPEC_VERSION, computed_active_spec_hash,
    };

    /// The decoded constant is checked against the bytes, never trusted. The
    /// independent measurement enters through the generated identity module: its
    /// digests were recorded from `shasum -a 256`, the archive gate hashes the
    /// same file with `shasum`, and `whitefoot-spec` compares this computed
    /// value against that chain tail, so a wrong SHA-256 implementation cannot
    /// agree only with itself. The former hand-transcribed literal here said
    /// the same thing a third time and was retired when the identity module
    /// became the one decoded source.
    #[test]
    fn recorded_identity_is_the_computed_identity() {
        assert_eq!(ACTIVE_KERNEL_SPEC_HASH, computed_active_spec_hash());
    }

    /// The active path is stable, while the embedded title independently names
    /// the version. A bump that moves only the constant or title is caught.
    #[test]
    fn stable_path_and_version_title_agree() {
        assert_eq!(ACTIVE_KERNEL_SPEC_PATH, "spec/kernel-spec.md");
        assert_eq!(
            ACTIVE_KERNEL_SPEC_TEXT.lines().next(),
            Some(format!("# Kernel Specification {ACTIVE_KERNEL_SPEC_VERSION}").as_str())
        );
    }
}
