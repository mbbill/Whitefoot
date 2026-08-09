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
pub const ACTIVE_KERNEL_SPEC_PATH: &str = "spec/kernel-spec-v0.23.md";

/// Exact UTF-8 text of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_TEXT: &str = include_str!("../../spec/kernel-spec-v0.23.md");

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
    0xe0, 0x9b, 0x32, 0xed, 0xb5, 0xa4, 0x91, 0x70, 0xbd, 0x3f, 0xb6, 0x59, 0xe5, 0x27, 0x1e, 0xc4,
    0xdb, 0xcb, 0x6a, 0xc3, 0xfe, 0xc2, 0xf4, 0x0e, 0x2e, 0x25, 0xb8, 0x49, 0x7a, 0xac, 0xe0, 0xf5,
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

    /// The literal is the `shasum -a 256` value the owner approved for v0.23 in
    /// `governance/APPROVALS.md`, so a wrong SHA-256 implementation fails here
    /// instead of agreeing with itself.
    ///
    /// It is transcribed FROM the ledger, never from what this code computes.
    /// That direction is the whole test: the ledger entry is written when the
    /// owner approves, and this literal follows it at activation. Updating it
    /// to match a computed value would delete the only independent check that
    /// the embedded bytes are the approved ones.
    #[test]
    fn computed_identity_is_the_approved_digest() {
        assert_eq!(
            computed_active_spec_hash().to_string(),
            "e09b32edb5a49170bd3fb659e5271ec4dbcb6ac3fec2f40e2e25b8497aace0f5"
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
