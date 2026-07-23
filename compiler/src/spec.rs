use core::fmt;

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
pub const ACTIVE_KERNEL_SPEC_VERSION: &str = "v0.16";

/// Repository-relative path of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_PATH: &str = "spec/kernel-spec-v0.16.md";

/// Exact UTF-8 text of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_TEXT: &str = include_str!("../../spec/kernel-spec-v0.16.md");

/// Exact bytes of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_BYTES: &[u8] = ACTIVE_KERNEL_SPEC_TEXT.as_bytes();

/// SHA-256 identity of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_HASH: SpecHash = SpecHash::from_sha256([
    0xf9, 0x32, 0x64, 0xfb, 0x4d, 0xf6, 0x99, 0x4a, 0x40, 0x8e, 0x13, 0x27, 0xc6, 0xe8, 0x64, 0x3c,
    0x34, 0xb4, 0xae, 0xa5, 0x5f, 0xba, 0x4b, 0x1a, 0x0b, 0x3d, 0xab, 0x35, 0x01, 0xce, 0xb9, 0x42,
]);

#[cfg(test)]
mod tests {
    use super::{
        ACTIVE_KERNEL_SPEC_BYTES, ACTIVE_KERNEL_SPEC_HASH, ACTIVE_KERNEL_SPEC_PATH,
        ACTIVE_KERNEL_SPEC_TEXT, ACTIVE_KERNEL_SPEC_VERSION,
    };

    #[test]
    fn active_spec_identity_is_the_approved_candidate_identity() {
        assert_eq!(ACTIVE_KERNEL_SPEC_VERSION, "v0.16");
        assert_eq!(ACTIVE_KERNEL_SPEC_PATH, "spec/kernel-spec-v0.16.md");
        assert_eq!(ACTIVE_KERNEL_SPEC_BYTES, ACTIVE_KERNEL_SPEC_TEXT.as_bytes());
        assert_eq!(
            ACTIVE_KERNEL_SPEC_HASH.to_string(),
            "f93264fb4df6994a408e1327c6e8643c34b4aea55fba4b1a0b3dab3501ceb942"
        );
    }
}
