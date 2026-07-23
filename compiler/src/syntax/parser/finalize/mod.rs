mod canonical;
mod engine;
mod outcome;
mod shape;
pub(crate) mod topology;

pub use canonical::audit_canonical;
pub use engine::finalize;
pub use outcome::{
    BundleSourceExtent, CanonicalCompilerFailure, CanonicalIssue, CanonicalLimit, CanonicalLimits,
    CanonicalLocation, CanonicalOutcome, CanonicalResourceFailure, CanonicalStorage,
    CanonicalSyntaxUnit, FinalizeCompilerFailure, FinalizeLimit, FinalizeLimits, FinalizeOutcome,
    FinalizeResourceFailure, FinalizeStorage, FinalizedBundle, NodePath,
};

#[cfg(test)]
mod tests;
