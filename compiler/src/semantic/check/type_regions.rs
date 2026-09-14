//! Declaration-local region positions and their actual-type correspondence.
//!
//! [FORM-8, TYPE-5] follows the finite written type tree, never nominal fields.
//! In particular, instantiating an opaque `T` does not add formal positions.
//! The stored shape has no temporary nominal IDs: constructor templates keep
//! it after their symbolic nominal checkpoint has been restored.

use crate::syntax::NodeId;
use crate::syntax::terminal::TerminalPredicate;
use crate::{DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget};

use super::super::model::{CheckedNominalKind, CheckedType};
use super::{CheckStop, Checker, PreludeType, SemanticCompilerFailure};

#[derive(Clone, Copy, Eq, PartialEq)]
enum TypeConstructor {
    Leaf,
    Nominal(DeclarationId),
    Box,
    LegacyBox,
    Arena,
    Option,
    Result,
    Array,
    Buffer,
    Slice,
    FixedVector,
    Vector,
    Heap,
    Extent,
}

#[derive(Clone, Copy)]
pub(super) struct RegionPosition {
    pub(super) formal: DeclarationId,
    pub(super) invariant: bool,
}

#[derive(Clone)]
pub(super) struct TypeRegionShape {
    constructor: TypeConstructor,
    regions: Vec<RegionPosition>,
    arguments: Vec<Self>,
}

impl TypeRegionShape {
    pub(super) fn determines(&self, formal: DeclarationId) -> bool {
        self.regions
            .iter()
            .any(|position| position.formal == formal)
            || self
                .arguments
                .iter()
                .any(|argument| argument.determines(formal))
    }
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [BLK-4] the complete value type, including phantom nominal brands and
    /// instantiated element types. This is independent of the declaration's
    /// region-argument matching shape: an opaque `T` can carry confinement.
    pub(super) fn confinement_regions(
        &self,
        ty: CheckedType,
    ) -> Result<Vec<DeclarationId>, CheckStop> {
        let mut pending = vec![ty];
        let mut visited = std::collections::HashSet::new();
        let mut regions = Vec::new();
        while let Some(ty) = pending.pop() {
            if !visited.insert(ty) {
                continue;
            }
            let (_, axes, arguments) = self.type_region_axes(ty)?;
            regions.extend(axes.into_iter().map(|axis| axis.formal));
            pending.extend(arguments);
            if let CheckedType::Nominal(id) = ty {
                match &self.nominal(id)?.kind {
                    CheckedNominalKind::Struct { fields } => {
                        pending.extend(fields.iter().map(|field| field.ty));
                    }
                    CheckedNominalKind::Enum { variants } => {
                        pending.extend(
                            variants
                                .iter()
                                .flat_map(|variant| variant.fields.iter().map(|field| field.ty)),
                        );
                    }
                    _ => {}
                }
            }
        }
        regions.sort_by_key(|region| region.index());
        regions.dedup();
        Ok(regions)
    }

    /// An instantiated type parameter can carry a caller's store brand even
    /// though that brand's source declaration belongs to a different body.
    /// Input values and explicit type arguments supply those regions for
    /// this invocation [FN-2, OWN-3].
    pub(super) fn check_confined_destination(
        &self,
        function: &super::FunctionSignature,
        ty: CheckedType,
        destination: Option<DeclarationId>,
        node: NodeId,
    ) -> Result<(), CheckStop> {
        let regions = self.confinement_regions(ty)?;
        if regions.is_empty() {
            return Ok(());
        }
        let mut supplied = function.region_parameters.clone();
        for parameter in &function.parameters {
            supplied.extend(self.confinement_regions(parameter.ty)?);
        }
        for (_, argument) in function.substitution.entries() {
            if let super::generics::GenericArgument::Type(ty) = argument {
                supplied.extend(self.confinement_regions(*ty)?);
            }
        }
        for region in regions {
            let within = match destination {
                Some(destination) => {
                    self.declaration_is_within_region_block(destination, region)?
                }
                None => false,
            };
            if supplied.contains(&region) || within {
                continue;
            }
            return self.issue_node(
                crate::SemanticRule::Blk4,
                node,
                crate::SemanticIssueKind::ConfinedValueEscape {
                    region: self.region_phrase(region)?,
                    mechanical_fix: "keep the destination within every region named by the value's complete type, or allocate its storage in a region that outlives the destination",
                },
            );
        }
        Ok(())
    }

    /// The name's own region and type-argument axes, without its fields.
    fn type_region_axes(
        &self,
        ty: CheckedType,
    ) -> Result<(TypeConstructor, Vec<RegionPosition>, Vec<CheckedType>), CheckStop> {
        let brand = |formal| {
            vec![RegionPosition {
                formal,
                invariant: true,
            }]
        };
        let loan = |formal| {
            vec![RegionPosition {
                formal,
                invariant: false,
            }]
        };
        let axes = match ty {
            CheckedType::Nominal(id) => {
                if let Some((template, substitution)) = self.source_nominal_instance_entry(id)? {
                    let template = &self.nominal_templates[template];
                    let regions = substitution
                        .region_arguments()
                        .iter()
                        .map(|(_, formal)| RegionPosition {
                            formal: *formal,
                            invariant: true,
                        })
                        .collect();
                    let arguments = template
                        .generic_parameters
                        .iter()
                        .filter_map(|parameter| match parameter {
                            super::generics::GenericParameter::Type { declaration, .. } => {
                                substitution.type_argument(*declaration)
                            }
                            _ => None,
                        })
                        .collect();
                    (
                        TypeConstructor::Nominal(template.declaration),
                        regions,
                        arguments,
                    )
                } else {
                    match self.nominal(id)?.kind {
                        CheckedNominalKind::Box {
                            region, referent, ..
                        } => (
                            if region.is_some() {
                                TypeConstructor::Box
                            } else {
                                TypeConstructor::LegacyBox
                            },
                            region.map(brand).unwrap_or_default(),
                            vec![referent],
                        ),
                        CheckedNominalKind::Arena { region, content } => {
                            (TypeConstructor::Arena, loan(region), vec![content])
                        }
                        _ => match self.prelude_type(id) {
                            Some(PreludeType::Option(value)) => {
                                (TypeConstructor::Option, vec![], vec![value])
                            }
                            Some(PreludeType::Result(ok, error)) => {
                                (TypeConstructor::Result, vec![], vec![ok, error])
                            }
                            _ => (TypeConstructor::Leaf, vec![], vec![]),
                        },
                    }
                }
            }
            CheckedType::Array { element, .. } => (
                TypeConstructor::Array,
                vec![],
                vec![self.element_type(element)?],
            ),
            CheckedType::FixedVector { element, .. } => (
                TypeConstructor::FixedVector,
                vec![],
                vec![self.element_type(element)?],
            ),
            CheckedType::Vector {
                region, element, ..
            } => (
                TypeConstructor::Vector,
                brand(region),
                vec![self.element_type(element)?],
            ),
            CheckedType::Buffer { element } => {
                (TypeConstructor::Buffer, vec![], vec![element.ty()])
            }
            CheckedType::Slice {
                region, element, ..
            } => (TypeConstructor::Slice, loan(region), vec![element.ty()]),
            CheckedType::Heap { region } => (TypeConstructor::Heap, brand(region), vec![]),
            CheckedType::Extent { region, .. } => (TypeConstructor::Extent, brand(region), vec![]),
            _ => (TypeConstructor::Leaf, vec![], vec![]),
        };
        Ok(axes)
    }

    /// `source` is present for a concrete function instance so written `T`
    /// remains opaque. Constructor templates use symbolic checked types,
    /// where generic leaves already carry that distinction.
    pub(super) fn type_region_shape(
        &self,
        ty: CheckedType,
        source: Option<NodeId>,
    ) -> Result<TypeRegionShape, CheckStop> {
        if let Some(source) = source
            && self
                .tree
                .direct_token_with(source, TerminalPredicate::TypeIdentifier)?
                .is_some()
            && matches!(
                self.use_at(source, LexicalUseRole::Type)?.target(),
                ResolvedTarget::Source {
                    class: DeclarationClass::GenericType,
                    ..
                }
            )
        {
            return Ok(TypeRegionShape {
                constructor: TypeConstructor::Leaf,
                regions: vec![],
                arguments: vec![],
            });
        }
        let (constructor, regions, types) = self.type_region_axes(ty)?;
        let mut sources = Vec::new();
        if let Some(source) = source {
            sources = self.tree.children_with(source, Production::Type)?;
            if let Some(targs) = self.tree.argument_list(source)? {
                for argument in self.tree.children_with(targs, Production::Targ)? {
                    if let Some(ty) = self.tree.first_child_with(argument, Production::Type)? {
                        sources.extend(self.behavior_type_sources(ty)?);
                    }
                }
            }
            if sources.len() != types.len() {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            }
        }
        let arguments = types
            .into_iter()
            .enumerate()
            .map(|(index, ty)| self.type_region_shape(ty, sources.get(index).copied()))
            .collect::<Result<_, _>>()?;
        Ok(TypeRegionShape {
            constructor,
            regions,
            arguments,
        })
    }

    /// Pair every region ordinal at matching constructors. A mismatching
    /// constructor supplies no correspondence; the caller still performs
    /// ordinary exact whole-type checking and reports [TYPE-5].
    pub(super) fn match_type_regions(
        &self,
        shape: &TypeRegionShape,
        actual: CheckedType,
    ) -> Result<Vec<(RegionPosition, DeclarationId)>, CheckStop> {
        let mut matched = Vec::new();
        let mut pending = vec![(shape, actual)];
        while let Some((shape, actual)) = pending.pop() {
            let (constructor, regions, arguments) = self.type_region_axes(actual)?;
            if shape.constructor != constructor {
                continue;
            }
            matched.extend(
                shape
                    .regions
                    .iter()
                    .copied()
                    .zip(regions.into_iter().map(|p| p.formal)),
            );
            pending.extend(shape.arguments.iter().zip(arguments).rev());
        }
        Ok(matched)
    }
}
