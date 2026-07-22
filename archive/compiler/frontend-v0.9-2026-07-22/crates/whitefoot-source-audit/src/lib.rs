#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Exact audit of a Whitefoot source and specification binding.
//!
//! The audit consumes only judgment-free types from `whitefoot-contract`. It
//! does not decode artifacts, parse source, or reproduce language semantics.

mod source_binding;

pub use source_binding::{
    VerifiedSource, VerifiedSourceBinding, VerifySourceBindingError, verify_source_binding,
};
