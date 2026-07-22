//! Closed host-class assumptions for the first measurement candidate.

/// The compiler-host target selected for the first evidence run.
pub const SUPPORTED_TARGET: &str = "aarch64-apple-darwin";
/// The exact Rust release selected by the storage model.
pub const SUPPORTED_RUSTC_RELEASE: &str = "1.91.1";
/// The allocator assumption that a supervised run must independently pin.
pub const ALLOCATOR_ASSUMPTION: &str = "Rust standard library system allocator path";
/// Minimum physical memory required of the supervised host.
pub const MINIMUM_PHYSICAL_MEMORY_BYTES: u64 = 8 * 1024 * 1024 * 1024;
/// Compiler-process RSS ceiling used by the candidate measurement protocol.
pub const SUPERVISED_RSS_CEILING_BYTES: u64 = 4 * 1024 * 1024 * 1024;
/// Candidate combined modeled-process target, not an approved maximum.
pub const MODELED_PROCESS_TARGET_BYTES: u64 = 3 * 1024 * 1024 * 1024;

/// Build identities captured by the standalone witness.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuildIdentity {
    /// Rust compilation target of this witness binary.
    pub target: &'static str,
    /// Host on which Cargo invoked rustc.
    pub build_host: &'static str,
    /// Complete `rustc -vV` output flattened onto one line.
    pub rustc: &'static str,
}

impl BuildIdentity {
    /// Returns the build identity embedded by `build.rs`.
    #[must_use]
    pub const fn embedded() -> Self {
        Self {
            target: env!("WHITEFOOT_WITNESS_TARGET"),
            build_host: env!("WHITEFOOT_WITNESS_BUILD_HOST"),
            rustc: env!("WHITEFOOT_WITNESS_RUSTC_ID"),
        }
    }

    /// Checks only assumptions that the binary can prove about its own build.
    pub fn validate_supported_build(self) -> Result<(), HostClassError> {
        if self.target != SUPPORTED_TARGET {
            return Err(HostClassError::WrongTarget);
        }
        let expected = format!("rustc {SUPPORTED_RUSTC_RELEASE} ");
        if !self.rustc.starts_with(&expected) {
            return Err(HostClassError::WrongRustcRelease);
        }
        Ok(())
    }
}

/// A build-level supported-host mismatch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HostClassError {
    /// The witness was not compiled for the candidate compiler-host target.
    WrongTarget,
    /// The witness was not compiled by the pinned Rust release.
    WrongRustcRelease,
}

/// Obligations that this process cannot establish for itself.
pub const SUPERVISOR_OBLIGATIONS: [&str; 5] = [
    "exact operating-system build identity",
    "at least 8 GiB physical memory",
    "actual global allocator and allocator configuration",
    "compiler-process high-water RSS from outside the process",
    "witness executable SHA-256 before execution",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn candidate_memory_arithmetic_is_exact() {
        assert_eq!(MINIMUM_PHYSICAL_MEMORY_BYTES, 8_589_934_592);
        assert_eq!(SUPERVISED_RSS_CEILING_BYTES, 4_294_967_296);
        assert_eq!(MODELED_PROCESS_TARGET_BYTES, 3_221_225_472);
        assert_eq!(
            SUPERVISED_RSS_CEILING_BYTES - MODELED_PROCESS_TARGET_BYTES,
            1_073_741_824
        );
    }
}
