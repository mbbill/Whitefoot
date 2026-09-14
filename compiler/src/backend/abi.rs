//! The internal function ABI, shared by definitions and every call route.
//!
//! This describes value representation only. Source access modes do not select
//! aliasing permissions, and a declaration and definition use the same ABI.

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
