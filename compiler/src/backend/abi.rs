//! The internal function ABI, shared by definitions and every user-call route.
//!
//! This describes value representation only. Source access modes do not select
//! aliasing permissions, and qualified system wrappers keep their separate ABI.

use crate::{IrFunction, IrProgram, IrType};

use super::{BackendFailure, storage::is_stored_aggregate};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ParameterAbi {
    Value(IrType),
    ContentPointer(IrType),
}

impl ParameterAbi {
    pub(super) const fn ty(self) -> IrType {
        match self {
            Self::Value(ty) | Self::ContentPointer(ty) => ty,
        }
    }

    pub(super) const fn is_indirect(self) -> bool {
        matches!(self, Self::ContentPointer(_))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ResultAbi {
    Value(IrType),
    Destination(IrType),
}

impl ResultAbi {
    pub(super) const fn ty(self) -> IrType {
        match self {
            Self::Value(ty) | Self::Destination(ty) => ty,
        }
    }

    pub(super) const fn uses_destination(self) -> bool {
        matches!(self, Self::Destination(_))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct FunctionAbi {
    parameters: Vec<ParameterAbi>,
    result: ResultAbi,
}

impl FunctionAbi {
    pub(super) fn build(
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

    pub(super) fn parameters(&self) -> &[ParameterAbi] {
        &self.parameters
    }

    pub(super) const fn result(&self) -> ResultAbi {
        self.result
    }
}
