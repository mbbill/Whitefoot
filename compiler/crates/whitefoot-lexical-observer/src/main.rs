#![forbid(unsafe_code)]
#![deny(missing_docs)]

//! Bounded binary observation adapter for the Whitefoot v0.9 lexer.
//!
//! This program reads one canonical source-bound request from standard input
//! and writes one lexical observation to standard output. It does not decide
//! language acceptance, emit normative diagnostics, or consume capability
//! metadata.

mod projection;
mod protocol;

use std::io::{self, Write};
use std::process::ExitCode;

use whitefoot_contract::{
    DecodeError, EncodeError, KERNEL_SPEC_V0_9_HASH, SourceBinding, SourceBundle,
    SourceBundleError, SourceInput, SpecHash,
};
use whitefoot_lexer::lex_v0_9;

use crate::projection::encode_observation;
use crate::protocol::{AdapterError, read_request};

/// The one specification identity accepted and emitted by this observer build.
pub(crate) const ACTIVE_KERNEL_SPEC_HASH: SpecHash = KERNEL_SPEC_V0_9_HASH;

fn observe() -> Result<(), AdapterError> {
    let input = io::stdin();
    let request = read_request(&mut input.lock())?;
    let candidate = SourceBinding::decode_canonical(&request.binding, request.source_limits)
        .map_err(map_binding_decode_error)?;
    if candidate.spec_hash() != ACTIVE_KERNEL_SPEC_HASH {
        return Err(AdapterError::SpecificationMismatch);
    }

    let mut inputs = Vec::<SourceInput<'_>>::new();
    inputs
        .try_reserve_exact(candidate.sources().len())
        .map_err(|_| AdapterError::SourceBundleStorageUnavailable)?;
    for source in candidate.sources() {
        inputs.push(SourceInput::new(
            source.logical_path().as_str(),
            source.bytes(),
        ));
    }
    let bundle = SourceBundle::with_limits(&inputs, request.source_limits)
        .map_err(map_source_bundle_error)?;
    let reconstructed =
        SourceBinding::try_from_bundle(ACTIVE_KERNEL_SPEC_HASH, &bundle, request.source_limits)
            .map_err(map_binding_encode_error)?;
    if reconstructed != candidate {
        return Err(AdapterError::SourceBindingDisagreement);
    }
    let canonical = reconstructed
        .encode_canonical(request.source_limits)
        .map_err(map_binding_encode_error)?;
    if canonical != request.binding {
        return Err(AdapterError::SourceBindingDisagreement);
    }

    let outcome = lex_v0_9(&bundle, request.lex_limits);
    let response = encode_observation(&bundle, outcome)?;
    io::stdout()
        .lock()
        .write_all(&response)
        .map_err(|_| AdapterError::OutputWrite)
}

fn map_binding_decode_error(error: DecodeError) -> AdapterError {
    match error {
        DecodeError::StorageUnavailable { .. } => AdapterError::SourceBindingStorageUnavailable,
        _ => AdapterError::BindingInvalid,
    }
}

fn map_source_bundle_error(error: SourceBundleError) -> AdapterError {
    match error {
        SourceBundleError::StorageUnavailable { .. } => {
            AdapterError::SourceBundleStorageUnavailable
        }
        _ => AdapterError::SourceBundleInvalid,
    }
}

fn map_binding_encode_error(error: EncodeError) -> AdapterError {
    match error {
        EncodeError::StorageUnavailable { .. } => AdapterError::SourceBindingStorageUnavailable,
        _ => AdapterError::SourceBindingDisagreement,
    }
}

fn main() -> ExitCode {
    match observe() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(
                io::stderr().lock(),
                "whitefoot lexical observer: {}",
                error.code()
            );
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used)]
mod tests;
