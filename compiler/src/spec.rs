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

/// Exact identity of `spec/kernel-spec-v0.15.md`.
pub const KERNEL_SPEC_V0_15_HASH: SpecHash = SpecHash::from_sha256([
    0x3c, 0x92, 0x40, 0x95, 0xb2, 0xc2, 0x1f, 0x12, 0x3b, 0x71, 0x37, 0x55, 0x6f, 0x72, 0xdb, 0xe8,
    0x72, 0x75, 0x83, 0x86, 0x82, 0xc1, 0x96, 0x5e, 0x6c, 0xaf, 0x39, 0x9d, 0xd2, 0x4d, 0x13, 0xbd,
]);

#[cfg(test)]
mod tests {
    use super::KERNEL_SPEC_V0_15_HASH;

    #[test]
    fn v0_15_identity_is_the_approved_candidate_identity() {
        assert_eq!(
            KERNEL_SPEC_V0_15_HASH.to_string(),
            "3c924095b2c21f123b7137556f72dbe87275838682c1965e6caf399dd24d13bd"
        );
    }
}
