#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Judgment-free data contracts shared by Whitefoot compiler components.
//!
//! This crate does not decide whether source is legal Whitefoot. In particular,
//! source bundles retain arbitrary bytes for separately authorized lexical,
//! syntax, and source-form stages.

mod binding;
mod digest;
mod source;

pub use binding::{
    BoundSource, DecodeError, EncodeError, SOURCE_BINDING_CODEC_VERSION, SourceBinding,
};
pub use digest::{
    CatalogHash, KERNEL_SPEC_V0_8_HASH, KERNEL_SPEC_V0_9_HASH, STATIC_SEMANTIC_CATALOG_V0_8_HASH,
    STATIC_SEMANTIC_CATALOG_V0_9_HASH, Sha256Digest, SpecHash,
};
pub use source::{
    ByteOffset, LogicalPath, LogicalPathError, SourceBundle, SourceBundleError, SourceFile,
    SourceId, SourceInput, SourceLimit, SourceLimits, SourceSpan, SpanError,
};
