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
pub const ACTIVE_KERNEL_SPEC_VERSION: &str = "v0.19";

/// Repository-relative path of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_PATH: &str = "spec/kernel-spec-v0.19.md";

/// Exact UTF-8 text of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_TEXT: &str = include_str!("../../spec/kernel-spec-v0.19.md");

/// Exact bytes of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_BYTES: &[u8] = ACTIVE_KERNEL_SPEC_TEXT.as_bytes();

/// SHA-256 identity of the active immutable kernel specification.
pub const ACTIVE_KERNEL_SPEC_HASH: SpecHash = SpecHash::from_sha256([
    0x01, 0xfb, 0x10, 0xd2, 0xd6, 0x1c, 0xc8, 0x7c, 0xce, 0x72, 0xcc, 0x98, 0x07, 0x1e, 0xda, 0x98,
    0xc7, 0x41, 0x1f, 0xdc, 0x95, 0xaf, 0x4e, 0xf2, 0x9b, 0x79, 0xac, 0x9a, 0x49, 0xcb, 0x53, 0x98,
]);

#[cfg(test)]
mod tests {
    use super::{
        ACTIVE_KERNEL_SPEC_BYTES, ACTIVE_KERNEL_SPEC_HASH, ACTIVE_KERNEL_SPEC_PATH,
        ACTIVE_KERNEL_SPEC_TEXT, ACTIVE_KERNEL_SPEC_VERSION,
    };

    #[test]
    fn active_spec_identity_is_the_approved_candidate_identity() {
        assert_eq!(ACTIVE_KERNEL_SPEC_VERSION, "v0.19");
        assert_eq!(ACTIVE_KERNEL_SPEC_PATH, "spec/kernel-spec-v0.19.md");
        assert_eq!(ACTIVE_KERNEL_SPEC_BYTES, ACTIVE_KERNEL_SPEC_TEXT.as_bytes());
        assert_eq!(
            ACTIVE_KERNEL_SPEC_HASH.to_string(),
            "01fb10d2d61cc87cce72cc98071eda98c7411fdc95af4ef29b79ac9a49cb5398"
        );
    }
}
