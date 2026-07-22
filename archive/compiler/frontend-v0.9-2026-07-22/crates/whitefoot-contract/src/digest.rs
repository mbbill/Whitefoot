use core::fmt;

/// A raw SHA-256 value.
///
/// The type represents already computed bytes; hashing is deliberately not
/// implemented until the workspace admits a reviewed cryptographic dependency.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Sha256Digest([u8; 32]);

impl Sha256Digest {
    /// Creates a digest from its exact 32-byte representation.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Debug for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, formatter)
    }
}

impl fmt::Display for Sha256Digest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}

/// The SHA-256 identity of one immutable numbered kernel specification.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SpecHash(Sha256Digest);

impl SpecHash {
    /// Creates a specification identity from a previously computed SHA-256.
    #[must_use]
    pub const fn from_sha256(bytes: [u8; 32]) -> Self {
        Self(Sha256Digest::from_bytes(bytes))
    }

    /// Returns the domain-neutral SHA-256 value.
    #[must_use]
    pub const fn digest(self) -> Sha256Digest {
        self.0
    }
}

impl fmt::Display for SpecHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

/// The SHA-256 identity of one canonical static semantic catalog.
///
/// This type names catalog bytes. Equality with an expected value does not
/// establish that a compiler implements the catalog's semantic obligations.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CatalogHash(Sha256Digest);

impl CatalogHash {
    /// Creates a catalog identity from a previously computed SHA-256.
    #[must_use]
    pub const fn from_sha256(bytes: [u8; 32]) -> Self {
        Self(Sha256Digest::from_bytes(bytes))
    }

    /// Returns the domain-neutral SHA-256 value.
    #[must_use]
    pub const fn digest(self) -> Sha256Digest {
        self.0
    }
}

impl fmt::Display for CatalogHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, formatter)
    }
}

/// Exact identity of `spec/kernel-spec-v0.8.md`.
pub const KERNEL_SPEC_V0_8_HASH: SpecHash = SpecHash::from_sha256([
    0xd0, 0x43, 0x36, 0xf7, 0xfa, 0x8d, 0x1a, 0x6a, 0x0f, 0x03, 0xfe, 0x58, 0xa1, 0x7f, 0x97, 0x2b,
    0x65, 0x82, 0x17, 0xa7, 0x3a, 0x3d, 0xff, 0x91, 0xa9, 0x06, 0xb4, 0xba, 0x29, 0x53, 0x28, 0xa8,
]);

/// Exact identity of `spec/kernel-spec-v0.9.md`.
pub const KERNEL_SPEC_V0_9_HASH: SpecHash = SpecHash::from_sha256([
    0xbd, 0xfb, 0x46, 0x1d, 0x19, 0x01, 0xf6, 0x10, 0x63, 0x3c, 0x5c, 0xbc, 0xd2, 0x47, 0x7d, 0x24,
    0xdf, 0x3c, 0x77, 0xca, 0x90, 0x59, 0x9b, 0x95, 0x80, 0xc8, 0x28, 0x9e, 0x50, 0xb8, 0x2b, 0x68,
]);

/// Exact identity of the canonical v0.8 static semantic catalog.
pub const STATIC_SEMANTIC_CATALOG_V0_8_HASH: CatalogHash = CatalogHash::from_sha256([
    0x2f, 0xa5, 0x86, 0xa8, 0xa1, 0xd9, 0xa4, 0x9f, 0x34, 0x4d, 0x64, 0xad, 0x2b, 0x5f, 0x45, 0x0a,
    0x2a, 0xe2, 0xe8, 0x36, 0x2b, 0xc1, 0x87, 0xc7, 0x02, 0x67, 0x09, 0x7b, 0x9b, 0x42, 0x7e, 0x1d,
]);

/// Exact identity of the canonical v0.9 static semantic catalog.
pub const STATIC_SEMANTIC_CATALOG_V0_9_HASH: CatalogHash = CatalogHash::from_sha256([
    0x3f, 0xf8, 0x2e, 0x48, 0xfc, 0x86, 0x0c, 0x4a, 0x41, 0x4e, 0x8e, 0x1a, 0x16, 0xa6, 0x52, 0x42,
    0x6b, 0x75, 0x05, 0xd7, 0xb7, 0x4b, 0xee, 0xdf, 0x05, 0x7e, 0x41, 0x85, 0x33, 0x15, 0x1a, 0xae,
]);

#[cfg(test)]
mod tests {
    use super::{
        CatalogHash, KERNEL_SPEC_V0_8_HASH, KERNEL_SPEC_V0_9_HASH,
        STATIC_SEMANTIC_CATALOG_V0_8_HASH, STATIC_SEMANTIC_CATALOG_V0_9_HASH,
    };

    #[test]
    fn v0_8_hash_matches_workspace_lock() {
        let locked = include_str!("../../../kernel-spec-v0.8.sha256").trim_end_matches('\n');
        assert_eq!(KERNEL_SPEC_V0_8_HASH.to_string(), locked);
    }

    #[test]
    fn v0_9_hash_matches_workspace_lock() {
        let locked = include_str!("../../../kernel-spec-v0.9.sha256").trim_end_matches('\n');
        assert_eq!(KERNEL_SPEC_V0_9_HASH.to_string(), locked);
    }

    #[test]
    fn v0_8_static_semantic_catalog_hash_matches_exact_workspace_lock() {
        let locked = include_str!("../../../static-semantic-catalog-v0.8.sha256");
        assert_eq!(format!("{STATIC_SEMANTIC_CATALOG_V0_8_HASH}\n"), locked);

        let mut changed = *STATIC_SEMANTIC_CATALOG_V0_8_HASH.digest().as_bytes();
        changed[0] ^= 1;
        assert_ne!(
            STATIC_SEMANTIC_CATALOG_V0_8_HASH,
            CatalogHash::from_sha256(changed)
        );
    }

    #[test]
    fn v0_9_static_semantic_catalog_hash_matches_exact_workspace_lock() {
        let locked = include_str!("../../../static-semantic-catalog-v0.9.sha256");
        assert_eq!(format!("{STATIC_SEMANTIC_CATALOG_V0_9_HASH}\n"), locked);

        let mut changed = *STATIC_SEMANTIC_CATALOG_V0_9_HASH.digest().as_bytes();
        changed[0] ^= 1;
        assert_ne!(
            STATIC_SEMANTIC_CATALOG_V0_9_HASH,
            CatalogHash::from_sha256(changed)
        );
    }
}
