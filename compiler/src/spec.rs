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
pub const ACTIVE_KERNEL_SPEC_VERSION: &str = "v0.18";

/// Repository-relative path of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_PATH: &str = "spec/kernel-spec-v0.18.md";

/// Exact UTF-8 text of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_TEXT: &str = include_str!("../../spec/kernel-spec-v0.18.md");

/// Exact bytes of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_BYTES: &[u8] = ACTIVE_KERNEL_SPEC_TEXT.as_bytes();

/// SHA-256 identity of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_HASH: SpecHash = SpecHash::from_sha256([
    0x30, 0x7a, 0x75, 0x8e, 0x41, 0x36, 0x65, 0x31, 0xc7, 0x1d, 0xc8, 0x73, 0x6b, 0xdd, 0xc4, 0x66,
    0x05, 0x4d, 0xbe, 0xba, 0x37, 0xf6, 0xe6, 0xdb, 0x13, 0xf0, 0x85, 0x97, 0x87, 0x71, 0x1a, 0x28,
]);

#[cfg(test)]
mod tests {
    use super::{
        ACTIVE_KERNEL_SPEC_BYTES, ACTIVE_KERNEL_SPEC_HASH, ACTIVE_KERNEL_SPEC_PATH,
        ACTIVE_KERNEL_SPEC_TEXT, ACTIVE_KERNEL_SPEC_VERSION,
    };

    #[test]
    fn active_spec_identity_is_the_approved_candidate_identity() {
        assert_eq!(ACTIVE_KERNEL_SPEC_VERSION, "v0.18");
        assert_eq!(ACTIVE_KERNEL_SPEC_PATH, "spec/kernel-spec-v0.18.md");
        assert_eq!(ACTIVE_KERNEL_SPEC_BYTES, ACTIVE_KERNEL_SPEC_TEXT.as_bytes());
        assert_eq!(
            ACTIVE_KERNEL_SPEC_HASH.to_string(),
            "307a758e41366531c71dc8736bddc466054dbeba37f6e6db13f0859787711a28"
        );
    }
}
