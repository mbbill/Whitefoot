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
pub const ACTIVE_KERNEL_SPEC_VERSION: &str = "v0.20";

/// Repository-relative path of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_PATH: &str = "spec/kernel-spec-v0.20.md";

/// Exact UTF-8 text of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_TEXT: &str = include_str!("../../spec/kernel-spec-v0.20.md");

/// Exact bytes of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_BYTES: &[u8] = ACTIVE_KERNEL_SPEC_TEXT.as_bytes();

/// SHA-256 identity of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_HASH: SpecHash = SpecHash::from_sha256([
    0xb0, 0x82, 0xef, 0x3f, 0xa8, 0xd2, 0xee, 0x63, 0x0b, 0x7e, 0x5b, 0x6e, 0xcb, 0x55, 0xff, 0x00,
    0x4e, 0xd2, 0x47, 0x3c, 0x56, 0x60, 0x40, 0x15, 0x0a, 0x12, 0x97, 0xa6, 0x1b, 0x31, 0x2d, 0xc1,
]);

#[cfg(test)]
mod tests {
    use super::{
        ACTIVE_KERNEL_SPEC_BYTES, ACTIVE_KERNEL_SPEC_HASH, ACTIVE_KERNEL_SPEC_PATH,
        ACTIVE_KERNEL_SPEC_TEXT, ACTIVE_KERNEL_SPEC_VERSION,
    };

    #[test]
    fn active_spec_identity_is_the_approved_candidate_identity() {
        assert_eq!(ACTIVE_KERNEL_SPEC_VERSION, "v0.20");
        assert_eq!(ACTIVE_KERNEL_SPEC_PATH, "spec/kernel-spec-v0.20.md");
        assert_eq!(ACTIVE_KERNEL_SPEC_BYTES, ACTIVE_KERNEL_SPEC_TEXT.as_bytes());
        assert_eq!(
            ACTIVE_KERNEL_SPEC_HASH.to_string(),
            "b082ef3fa8d2ee630b7e5b6ecb55ff004ed2473c566040150a1297a61b312dc1"
        );
    }
}
