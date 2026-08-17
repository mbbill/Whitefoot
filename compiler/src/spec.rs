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
pub const ACTIVE_KERNEL_SPEC_VERSION: &str = "v0.30";

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
    0xdb, 0x2b, 0x4b, 0x69, 0x06, 0xf6, 0x30, 0x9a, 0x4f, 0xe0, 0x45, 0x68, 0xfa, 0x5c, 0x2b, 0xeb,
    0x0f, 0xec, 0xfa, 0xe7, 0x24, 0x05, 0x59, 0x18, 0x72, 0xe6, 0xe9, 0xc6, 0xc7, 0x0c, 0x5e, 0xf2,
]);

/// SHA-256 of the embedded active specification, computed from its bytes.
///
/// The one quantity in this module that no one transcribes. Comparing it with
/// [`ACTIVE_KERNEL_SPEC_HASH`] compares two independently derived values, and
/// comparing it with the digest recorded in `governance/APPROVALS.md` compares
/// this implementation against the owner's `shasum -a 256`.
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
    /// review bytes. The activation chain supplies the protected owner-approved
    /// value once these bytes are activated, so a wrong SHA-256 implementation
    /// cannot agree only with itself.
    ///
    /// It is transcribed from `shasum`, never from what this code computes. At
    /// activation the approval ledger must independently name the same digest;
    /// changing this literal to follow a computed value would delete the
    /// external check of the runtime implementation.
    #[test]
    fn computed_identity_is_the_independently_measured_digest() {
        assert_eq!(
            computed_active_spec_hash().to_string(),
            "db2b4b6906f6309a4fe04568fa5c2beb0fecfae72405591872e6e9c6c70c5ef2"
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
