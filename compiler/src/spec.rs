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

/// Exact identity of `spec/kernel-spec-v0.10.md`.
pub const KERNEL_SPEC_V0_10_HASH: SpecHash = SpecHash::from_sha256([
    0x71, 0x07, 0x3e, 0x25, 0x21, 0x94, 0x55, 0x89, 0x62, 0x50, 0xe1, 0x5e, 0x13, 0xd1, 0xff, 0xdb,
    0xfc, 0x44, 0x3c, 0x87, 0xa9, 0xb2, 0x8c, 0xb9, 0x90, 0x6d, 0x73, 0xa0, 0x20, 0xdc, 0x33, 0xe9,
]);

#[cfg(test)]
mod tests {
    use super::KERNEL_SPEC_V0_10_HASH;

    #[test]
    fn v0_10_identity_is_the_approved_candidate_identity() {
        assert_eq!(
            KERNEL_SPEC_V0_10_HASH.to_string(),
            "71073e25219455896250e15e13d1ffdbfc443c87a9b28cb9906d73a020dc33e9"
        );
    }
}
