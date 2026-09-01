use core::fmt;

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

/// Version label of the active kernel specification.
pub const ACTIVE_KERNEL_SPEC_VERSION: &str = "v0.40";

/// Repository-relative stable path of the active kernel specification.
pub const ACTIVE_KERNEL_SPEC_PATH: &str = "spec/kernel-spec.md";

/// Exact UTF-8 text of the active kernel specification.
pub const ACTIVE_KERNEL_SPEC_TEXT: &str = include_str!("../../spec/kernel-spec.md");

/// Exact bytes of the active kernel specification.
pub const ACTIVE_KERNEL_SPEC_BYTES: &[u8] = ACTIVE_KERNEL_SPEC_TEXT.as_bytes();

/// SHA-256 identity of the active kernel specification.
///
/// Recorded rather than computed here only because a constant is re-evaluated
/// in every crate that reads it, and hashing the whole specification in the
/// constant evaluator costs about twelve seconds per crate. It is checked
/// against the bytes rather than trusted: [`computed_active_spec_hash`] hashes
/// them at runtime and the `whitefoot-spec` gate rejects any disagreement, so
/// installing a specification cannot leave this naming the previous one.
pub const ACTIVE_KERNEL_SPEC_HASH: SpecHash = SpecHash::from_sha256([
    0x50, 0x79, 0xef, 0x2e, 0xfa, 0x78, 0x62, 0x18, 0x4f, 0x06, 0xcc, 0xf7, 0xdc, 0x27, 0x3a, 0xe9,
    0x7e, 0xda, 0x79, 0x16, 0x79, 0xa4, 0x4f, 0x66, 0xc8, 0x6e, 0x75, 0xaf, 0xbc, 0x46, 0xc6, 0xe0,
]);

/// SHA-256 of the embedded active specification, computed from its bytes.
///
/// The one quantity in this module that no one transcribes. Comparing it with
/// [`ACTIVE_KERNEL_SPEC_HASH`] compares two independently derived values, and
/// comparing it with the digest recorded in `governance/APPROVALS.md` compares
/// this implementation against the independently measured `shasum -a 256`.
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

    /// The literal is the independently measured `shasum -a 256` value for the
    /// installed bytes. The activation chain supplies the same independently
    /// recorded value, so a wrong SHA-256 implementation cannot agree only
    /// with itself.
    ///
    /// It is transcribed from `shasum`, never from what this code computes. The
    /// approval ledger independently names the same digest; changing this
    /// literal to follow a computed value would delete the
    /// external check of the runtime implementation.
    #[test]
    fn computed_identity_is_the_independently_measured_digest() {
        assert_eq!(
            computed_active_spec_hash().to_string(),
            "5079ef2efa7862184f06ccf7dc273ae97eda791679a44f66c86e75afbc46c6e0"
        );
    }

    /// The recorded constant is checked against the bytes, never trusted. The
    /// `whitefoot-spec` gate makes the same comparison.
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
