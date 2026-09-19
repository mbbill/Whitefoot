//! The internal function ABI, shared by definitions and every call route.
//!
//! This module describes value representation: which parameters and results
//! travel as values and which travel as a pointer to their storage. A
//! declaration and a definition use the same ABI.
//!
//! Representation is not the whole signature. A parameter's *source mode*
//! does select the aliasing facts its emitted signature carries
//! (compiler/backend-facts): a reference parameter is `noalias`, `nonnull`,
//! `dereferenceable` and non-capturing, because [REF-1] through [REF-3] and
//! [EFF-5] already proved each of those, and `swap` [OP-11] is the one row
//! whose two arguments may name the same place. The emitter reads the source
//! signature, not this representation table, to decide that.

use crate::{IrFunction, IrProgram, IrType};

use super::{BackendFailure, storage::is_stored_aggregate};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParameterAbi {
    Value(IrType),
    ContentPointer(IrType),
}

impl ParameterAbi {
    pub(crate) const fn ty(self) -> IrType {
        match self {
            Self::Value(ty) | Self::ContentPointer(ty) => ty,
        }
    }

    pub(crate) const fn is_indirect(self) -> bool {
        matches!(self, Self::ContentPointer(_))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ResultAbi {
    Value(IrType),
    Destination(IrType),
}

impl ResultAbi {
    pub(crate) const fn ty(self) -> IrType {
        match self {
            Self::Value(ty) | Self::Destination(ty) => ty,
        }
    }

    pub(crate) const fn uses_destination(self) -> bool {
        matches!(self, Self::Destination(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FunctionAbi {
    parameters: Vec<ParameterAbi>,
    result: ResultAbi,
}

impl FunctionAbi {
    pub(crate) fn build(
        program: &IrProgram<'_, '_, '_>,
        function: &IrFunction,
    ) -> Result<Self, BackendFailure> {
        let parameters = function
            .parameters()
            .iter()
            .map(|(_, ty)| {
                Ok(if is_stored_aggregate(program, *ty)? {
                    ParameterAbi::ContentPointer(*ty)
                } else {
                    ParameterAbi::Value(*ty)
                })
            })
            .collect::<Result<Vec<_>, BackendFailure>>()?;
        let ty = function.result();
        let result = if is_stored_aggregate(program, ty)? {
            ResultAbi::Destination(ty)
        } else {
            ResultAbi::Value(ty)
        };
        Ok(Self { parameters, result })
    }

    pub(crate) fn parameters(&self) -> &[ParameterAbi] {
        &self.parameters
    }

    pub(crate) const fn result(&self) -> ResultAbi {
        self.result
    }
}
