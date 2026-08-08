use core::fmt;

mod sha256;

/// The SHA-256 identity of one immutable numbered kernel specification.
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

/// Version label of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_VERSION: &str = "v0.23";

/// Repository-relative path of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_PATH: &str =
    "governance/spec-evolution/kernel-spec-v0.23-candidate.md";

/// Exact UTF-8 text of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_TEXT: &str =
    include_str!("../../governance/spec-evolution/kernel-spec-v0.23-candidate.md");

/// Exact bytes of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_BYTES: &[u8] = ACTIVE_KERNEL_SPEC_TEXT.as_bytes();

/// SHA-256 identity of the active immutable kernel specification.
///
/// Recorded rather than computed here only because a constant is re-evaluated
/// in every crate that reads it, and hashing the whole specification in the
/// constant evaluator costs about twelve seconds per crate. It is checked
/// against the bytes rather than trusted: [`computed_active_spec_hash`] hashes
/// them at runtime and the `whitefoot-spec` gate rejects any disagreement, so
/// installing a specification cannot leave this naming the previous one.
pub const ACTIVE_KERNEL_SPEC_HASH: SpecHash = SpecHash::from_sha256([
    0xab, 0x25, 0x7a, 0xa6, 0x58, 0x74, 0xc4, 0xe6, 0xde, 0x16, 0x71, 0x89, 0xb9, 0x7c, 0xf7, 0x06,
    0xb5, 0xca, 0x00, 0x45, 0xcc, 0xab, 0x86, 0xfd, 0xb5, 0x4d, 0xa8, 0x3e, 0x2b, 0xa6, 0x13, 0xda,
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
        ACTIVE_KERNEL_SPEC_HASH, ACTIVE_KERNEL_SPEC_PATH, ACTIVE_KERNEL_SPEC_VERSION,
        computed_active_spec_hash,
    };

    /// The literal is the `shasum -a 256` value the owner approved for v0.22 in
    /// `governance/APPROVALS.md`, so a wrong SHA-256 implementation fails here
    /// instead of agreeing with itself.
    #[test]
    fn computed_identity_is_the_approved_digest() {
        assert_eq!(
            computed_active_spec_hash().to_string(),
            "b133b793629d28e7ee1b7ad0ae3d49185932b9390f5c25517f0fb0ea2fc8a6e8"
        );
    }

    /// The recorded constant is checked against the bytes, never trusted. The
    /// `whitefoot-spec` gate makes the same comparison.
    #[test]
    fn recorded_identity_is_the_computed_identity() {
        assert_eq!(ACTIVE_KERNEL_SPEC_HASH, computed_active_spec_hash());
    }

    /// The path and the version label are maintained separately, so a version
    /// bump that moves only one of them is caught here.
    #[test]
    fn path_and_version_label_agree() {
        assert_eq!(
            ACTIVE_KERNEL_SPEC_PATH,
            format!("spec/kernel-spec-{ACTIVE_KERNEL_SPEC_VERSION}.md")
        );
    }
}
