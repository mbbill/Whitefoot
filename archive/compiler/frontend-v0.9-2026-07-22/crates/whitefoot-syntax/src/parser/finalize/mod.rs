mod canonical;
mod engine;
mod outcome;
mod shape;
pub(crate) mod topology;

pub use canonical::audit_canonical_v0_9;
pub use engine::finalize_v0_9;
pub use outcome::{
    BundleSourceExtent, CanonicalCompilerFailure, CanonicalIssue, CanonicalLimit, CanonicalLimits,
    CanonicalLocation, CanonicalOutcome, CanonicalResourceFailure, CanonicalStorage,
    CanonicalSyntaxUnit, FinalizeCompilerFailure, FinalizeLimit, FinalizeLimits, FinalizeOutcome,
    FinalizeResourceFailure, FinalizeStorage, FinalizedBundle, NodePath,
};

#[cfg(test)]
mod tests;
