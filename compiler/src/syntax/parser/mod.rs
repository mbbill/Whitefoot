mod diagnostic;
mod engine;
mod finalize;
mod outcome;
mod tree;

pub use engine::parse_v0_14;
pub use finalize::{
    BundleSourceExtent, CanonicalCompilerFailure, CanonicalIssue, CanonicalLimit, CanonicalLimits,
    CanonicalLocation, CanonicalOutcome, CanonicalResourceFailure, CanonicalStorage,
    CanonicalSyntaxUnit, FinalizeCompilerFailure, FinalizeLimit, FinalizeLimits, FinalizeOutcome,
    FinalizeResourceFailure, FinalizeStorage, FinalizedBundle, NodePath, audit_canonical_v0_14,
    finalize_v0_14,
};
pub use outcome::{
    ExpectedTerminalsV0_14, ParseCompilerFailure, ParseInvocationFailure, ParseLimit, ParseLimits,
    ParseOutcome, ParseResourceFailure, ParseStorage, ParsedBundle, SyntaxCoordinate, SyntaxIssue,
    SyntaxRuleV0_14,
};

pub(crate) use diagnostic::{
    DecisionSelection, DiagnosticResult, DiagnosticSite, ProbeContext, diagnose_decision,
    direct_mismatch, select_arm,
};
pub(crate) use finalize::topology::{FinalizedExtent, FinalizedTopology, NodeId};
pub(crate) use outcome::{ExpectedBuilder, Work};
pub(crate) use tree::{DerivationElement, DerivationTree, Frame};

#[cfg(test)]
mod tests;
