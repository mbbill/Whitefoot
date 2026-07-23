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

/// Exact identity of `spec/kernel-spec-v0.14.md`.
pub const KERNEL_SPEC_V0_14_HASH: SpecHash = SpecHash::from_sha256([
    0x31, 0xc0, 0x93, 0x13, 0x36, 0x33, 0x04, 0xf4, 0x05, 0xc8, 0xdb, 0x11, 0x91, 0xd1, 0x98, 0x2e,
    0x36, 0x25, 0xb8, 0x67, 0x88, 0xbf, 0x95, 0x3e, 0xc3, 0xbb, 0x16, 0x96, 0x48, 0x46, 0x6e, 0x9f,
]);

#[cfg(test)]
mod tests {
    use super::KERNEL_SPEC_V0_14_HASH;

    #[test]
    fn v0_14_identity_is_the_approved_candidate_identity() {
        assert_eq!(
            KERNEL_SPEC_V0_14_HASH.to_string(),
            "31c09313363304f405c8db1191d1982e3625b86788bf953ec3bb169648466e9f"
        );
    }
}
