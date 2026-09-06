pub(in crate::semantic::check) mod calls;
pub(in crate::semantic::check) mod flat_storage;
mod places;

use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};
use crate::{
    DeclarationClass, DeclarationId, DeferredUseRole, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule, UnsupportedSemanticFeature,
};

use super::super::model::{
    CheckedConst, CheckedConstant, CheckedExpression, CheckedIntegerOperation, CheckedMode,
    CheckedNominalKind, CheckedProjectedDrop, CheckedSetTarget, CheckedType, CheckedValue,
    CheckedWritablePlace, FloatType, IntegerType,
};
use super::borrows::{AccessKind, OwnedContent, ReborrowPosition, ResolvedPlace};
use super::{
    CheckStop, Checker, Constructor, EffectSet, FunctionSignature, LocalBinding, TypedExpression,
};

#[derive(Clone, Copy)]
pub(in crate::semantic::check) enum PlaceUseContext {
    Ordinary,
    Consuming,
}

#[derive(Clone, Copy)]
struct PlaceUseOptions {
    explicit_move: bool,
    context: PlaceUseContext,
    loop_depth: usize,
}

/// Which mutation statement is forming this target.
///
/// [SET-1] and [SET-2] share the whole writability relation and differ only in
/// the final selected type's required [OWN-1] class, so one judgment serves
/// both and this says which side of it applies. `replace` demands a
/// region-free affine type at formation, while a commit's affine admission is
/// [LIV-2]'s first condition and is judged at the commit, where the read-out
/// is known; only the region-free demand, which no commit reinitializes
/// either, is decidable from the type alone.
#[derive(Clone, Copy)]
pub(super) enum MutationForm {
    /// `set p = e`, whose affine admission [LIV-2] judges at the commit.
    Set,
    /// `replace p = e`.
    Replace,
}

/// One formed and judged mutation target [SET-1, SET-2, LIV-2].
///
/// The resolved place is what the commit writes and what every judgment
/// stated over places reads: [LIV-2]'s pairwise disjointness, its read-out
/// matching, and [OWN-5]'s loan state. It is not the written spelling: a
/// `deref` target resolves through its holder to the borrowed place, so two
/// targets that overlap are refused however they are spelled.
pub(in crate::semantic::check) struct MutationTarget {
    /// The source declaration the written place is rooted at: the value
    /// binding for a bare, field or subscript target, the holder for a
    /// `deref` target.
    pub(in crate::semantic::check) declaration: DeclarationId,
    /// The resolved place this target writes [OWN-6].
    pub(in crate::semantic::check) place: ResolvedPlace,
    /// Whether the write selects one element of `place` rather than `place`
    /// itself, which is the granularity [MSR-2] states over storage.
    pub(in crate::semantic::check) element: bool,
    pub(in crate::semantic::check) target: CheckedSetTarget,
    pub(in crate::semantic::check) effects: EffectSet,
    /// A capability this compiler does not implement at this target, carried
    /// rather than raised so that [DIAG-1]'s order holds: every source
    /// rejection of the statement, [LIV-2]'s commit conditions included, is
    /// judged before the stop, and no capability limit stands in front of a
    /// rejection.
    pub(in crate::semantic::check) unsupported: Option<UnsupportedSemanticFeature>,
}

impl MutationForm {
    /// Whether this is the [SET-2] side, whose commit also reads the target.
    pub(super) const fn is_replace(self) -> bool {
        matches!(self, Self::Replace)
    }
}

/// [STOR-1]'s restructuring, which [LIV-2] leaves as the rule's only one:
/// `replace` names the previous owner.
///
/// The second sentence this constant had beside it — the fresh `let` offered
/// when the right-hand side consumed the target root — is retired with
/// [LIV-2]. That shape is no longer a rejection at a complete binding: the
/// consuming `move` is the target's read-out and the statement is accepted.
/// At a projected target the root is genuinely dead at the commit, so the
/// rejection there is [OWN-1]'s dead root, which offers the same fresh `let`
/// in its own sentence.
pub(in crate::semantic::check) const STOR1_REPLACE: &str =
    "use replace: let old = replace p = e; binds the previous owner";

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    /// [SET-2] target formation: exactly [SET-1]'s relation with the
    /// copy/affine class judgment inverted and the region-free demand added.
    pub(super) fn check_replace_target(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<MutationTarget, CheckStop> {
        self.check_mutation_target(function, node, bindings, loop_depth, MutationForm::Replace)
    }

    pub(super) fn check_set_target(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<MutationTarget, CheckStop> {
        self.check_mutation_target(function, node, bindings, loop_depth, MutationForm::Set)
    }

    /// The source declaration a written place is rooted at, when its base is a
    /// bare name.
    ///
    /// A `deref` base is rooted in a holder rather than in the storage the
    /// place selects, so it answers `None`: the storage that place selects is
    /// the referent's, not the holder's. [LIV-2] reads this to decide the one
    /// target shape it reinitializes from dead, the complete binding.
    pub(in crate::semantic::check) fn complete_binding_target(
        &self,
        place: NodeId,
    ) -> Result<Option<DeclarationId>, CheckStop> {
        let Some(pbase) = self.tree.first_child_with(place, Production::Pbase)? else {
            return Ok(None);
        };
        if self.has_fixed(pbase, FixedTerminal::Deref)? || !self.tree.children(pbase)?.is_empty() {
            return Ok(None);
        }
        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        Ok(match usage.target() {
            ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Value,
            } => Some(declaration),
            _ => None,
        })
    }

    /// One judgment of a place's [SET-1]/[SET-2] mutation-target class: the
    /// two statements share the complete writability relation and differ only
    /// in the final selected type's required [OWN-1] class.
    fn check_mutation_target(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        form: MutationForm,
    ) -> Result<MutationTarget, CheckStop> {
        let pbase = self
            .tree
            .first_child_with(node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if !self.has_fixed(pbase, FixedTerminal::Deref)? && self.tree.children(pbase)?.is_empty() {
            let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
            if let ResolvedTarget::Source {
                declaration,
                class: DeclarationClass::Value,
            } = usage.target()
                && bindings
                    .get(&declaration)
                    .is_some_and(|local| local.compiler_updated)
            {
                return self.issue_node(
                    SemanticRule::Set1,
                    node,
                    SemanticIssueKind::InvalidSetTarget {
                        root_class: "compiler-updated counted binder".to_owned(),
                        required_classes:
                            "source-writable live own storage or a live usable &uniq referent",
                    },
                );
            }
        }
        let suffixes = self.tree.children_with(node, Production::Psuffix)?;
        if let Some(subscript) = self.last_subscript(&suffixes)? {
            return self.check_indexed_set_target(
                function, node, &suffixes, subscript, bindings, loop_depth, form,
            );
        }
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            return self.check_dereferenced_set_target(node, pbase, bindings, form);
        }
        if !self.tree.children(pbase)?.is_empty() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }

        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source { declaration, class } = usage.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        if class == DeclarationClass::NamedConst {
            return self.issue_node(
                SemanticRule::Const2,
                node,
                SemanticIssueKind::ImmutableSetTarget,
            );
        }
        if class != DeclarationClass::Value {
            return self.issue_node(
                SemanticRule::Set1,
                node,
                SemanticIssueKind::InvalidSetTarget {
                    root_class: format!("{class:?}"),
                    required_classes: "live own storage or a live usable &uniq referent",
                },
            );
        }

        let local = bindings
            .get(&declaration)
            .cloned()
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        // [LIV-2] a commit whose target is a complete binding reinitializes
        // that binding, so a dead one is the one root this formation admits;
        // every projected, dereferenced or subscripted target of a dead root
        // stays [OWN-1]'s rejection, because reinitializing one component of a
        // dead root would leave the rest uninitialized.
        let reinitializes = matches!(form, MutationForm::Set) && suffixes.is_empty();
        if !local.live && !reinitializes {
            return self.issue_node(
                SemanticRule::Own1,
                node,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }

        let (fields, ty) = self.resolve_struct_path(&suffixes, local.ty)?;
        if local.mode != CheckedMode::Own {
            return self.issue_node(
                SemanticRule::Set1,
                node,
                SemanticIssueKind::InvalidSetTarget {
                    root_class: match local.mode {
                        CheckedMode::Shared(_) => "shared borrow",
                        CheckedMode::Unique(_) => "unique borrow holder",
                        CheckedMode::Own => "owned value",
                    }
                    .to_owned(),
                    required_classes: "live own storage or a live usable &uniq referent",
                },
            );
        }
        let resolved = ResolvedPlace {
            root: declaration,
            fields: fields.clone(),
        };
        self.check_loan_access(bindings, None, &resolved, AccessKind::Write, node)?;

        self.check_mutation_target_class(node, ty, form)?;
        let mut effects = EffectSet::NONE;
        for path in self.effect_paths_for_place(&resolved, bindings)? {
            effects.add_write(path.clone());
            if form.is_replace() {
                effects.add_read(path);
            }
        }

        Ok(MutationTarget {
            declaration,
            place: resolved,
            element: false,
            target: CheckedSetTarget::Place(CheckedWritablePlace {
                binding: local.binding,
                fields,
                ty,
                declares: false,
            }),
            effects,
            unsupported: None,
        })
    }

    /// The final selected type's [OWN-1] class judgment shared by the
    /// [SET-1] and [SET-2] target paths: `set` demands copy [STOR-1], and
    /// `replace` demands region-free affine [SET-2].
    fn check_mutation_target_class(
        &self,
        node: NodeId,
        ty: CheckedType,
        form: MutationForm,
    ) -> Result<(), CheckStop> {
        let MutationForm::Set = form else {
            // [SET-2, VIEW-4] a loan-bearing target is judged as the
            // region-bearing target it is, before the copy class is read:
            // [S27] made the shared view copy, and "use set for a copy place"
            // is exactly the repair [VIEW-4] refuses at this same place.
            if !Self::checked_type_is_loan_bearing(ty)
                && self.is_copy_type(ty)?
                && self.judges_class_spelling()
            {
                return self.issue_node(
                    SemanticRule::Set2,
                    node,
                    SemanticIssueKind::InvalidReplaceTarget {
                        target_type: self.checked_type_name(ty)?,
                        mechanical_fix: "use set for a copy place; read the previous value bare",
                    },
                );
            }
            // [SET-2] rejects a region-bearing target type at any depth of
            // T, which is [STOR-5]'s relation over the selected type rather
            // than an enumerated set of spellings: a slice, an arena, and
            // anything reaching one.
            if self.checked_type_is_region_bearing(ty)? {
                return self.issue_node(
                    SemanticRule::Set2,
                    node,
                    SemanticIssueKind::InvalidReplaceTarget {
                        target_type: self.checked_type_name(ty)?,
                        mechanical_fix: "a slice's static origin set and an arena's confinement \
                                         are fixed at initialization; bind a new slice or arena \
                                         under a new let",
                    },
                );
            }
            return Ok(());
        };
        // [LIV-2] an affine target's admission is judged at the commit, where
        // the read-out is known; only the region-free demand [SET-2] states of
        // a replacement target is decidable from the type alone, and a commit
        // reinitializes no origin set or arena confinement either.
        if !self.is_copy_type(ty)? && self.checked_type_is_region_bearing(ty)? {
            return self.issue_node(
                SemanticRule::Liv2,
                node,
                SemanticIssueKind::RegionBearingCommitTarget {
                    target_type: self.checked_type_name(ty)?,
                    mechanical_fix: "a slice's static origin set and an arena's confinement \
                                     are fixed at initialization; bind a new slice or arena \
                                     under a new let",
                },
            );
        }
        Ok(())
    }

    /// One value's exact written mode and type, as `own u64`, `&'r
    /// buffer<u8>`, or `&uniq 'r Output`.
    pub(in crate::semantic::check) fn checked_value_name(
        &self,
        mode: CheckedMode,
        ty: CheckedType,
    ) -> Result<String, CheckStop> {
        let mode = self.checked_mode_name(mode)?;
        let ty = self.checked_type_name(ty)?;
        // [FORM-2] attaches `&` to what follows it, and a mode whose region
        // [FORM-8] leaves unwritten ends in that `&`, so the rendering must
        // not insert the separator the written form needs.
        Ok(if mode.ends_with('&') {
            format!("{mode}{ty}")
        } else {
            format!("{mode} {ty}")
        })
    }

    /// One written mode, with the region spelled as the source spells it.
    pub(in crate::semantic::check) fn checked_mode_name(
        &self,
        mode: CheckedMode,
    ) -> Result<String, CheckStop> {
        Ok(match mode {
            CheckedMode::Own => "own".to_owned(),
            CheckedMode::Shared(region) => match self.region_spelling(region).as_str() {
                "" => "&".to_owned(),
                spelling => format!("&{spelling}"),
            },
            CheckedMode::Unique(region) => match self.region_spelling(region).as_str() {
                "" => "&uniq".to_owned(),
                spelling => format!("&uniq {spelling}"),
            },
        })
    }

    /// One region as the source spells it, or its dense identity when the
    /// declaration is not reachable.
    ///
    /// A rendering is presentation: a region a diagnostic cannot name must not
    /// turn a source rejection into a compiler failure. A region [FORM-8]
    /// leaves unwritten has no source spelling at all: resolution mints it
    /// under a name no source token can form, and rendering that name would
    /// name a region the writer cannot write. It renders as the empty string,
    /// which is exactly how the source spells it, and every caller that
    /// splices a region into a longer form drops the separator with it.
    pub(in crate::semantic::check) fn region_spelling(&self, region: DeclarationId) -> String {
        // [PROV-1] the entry heap's store region has no written spelling at
        // all: `main` declares no region parameter, so every position that
        // names it names it by elision, and rendering the identity the
        // compiler holds it under would name a region the writer cannot
        // write.
        if region.is_entry_heap_region() {
            return String::new();
        }
        let spelling = self
            .declaration_spelling(region)
            .unwrap_or_else(|_| format!("'region#{}", region.index()));
        if spelling.starts_with("'0_") {
            return String::new();
        }
        spelling
    }

    pub(super) fn checked_type_name(&self, ty: CheckedType) -> Result<String, CheckStop> {
        Ok(match ty {
            CheckedType::Unit => "unit".to_owned(),
            CheckedType::Bool => "Bool".to_owned(),
            CheckedType::Integer(integer) => match integer {
                IntegerType::I8 => "i8",
                IntegerType::I16 => "i16",
                IntegerType::I32 => "i32",
                IntegerType::I64 => "i64",
                IntegerType::U8 => "u8",
                IntegerType::U16 => "u16",
                IntegerType::U32 => "u32",
                IntegerType::U64 => "u64",
            }
            .to_owned(),
            CheckedType::Float(float) => match float {
                FloatType::F32 => "f32",
                FloatType::F64 => "f64",
            }
            .to_owned(),
            CheckedType::Generic(declaration) => {
                format!("<type-parameter:{}>", declaration.index())
            }
            CheckedType::GenericInt(declaration) => {
                format!("<Int-parameter:{}>", declaration.index())
            }
            CheckedType::GenericFloat(declaration) => {
                format!("<Float-parameter:{}>", declaration.index())
            }
            // [S20] a nominal's region arguments are components of its type
            // name [TYPE-2], so a diagnostic that reports two instances of one
            // declaration has to spell them: the two sides of a [TYPE-5]
            // mismatch between `BlockPool<'a>` and `BlockPool<'b>` are
            // otherwise the same word twice.
            CheckedType::Nominal(id) => {
                let name = self.nominal(id)?.name.clone();
                match self.nominal_region_axis(id)? {
                    Some(axis) if !axis.is_empty() => {
                        let arguments = axis
                            .iter()
                            .map(|(_, actual)| self.region_spelling(*actual))
                            .collect::<Vec<_>>()
                            .join(", ");
                        format!("{name}<{arguments}>")
                    }
                    _ => name,
                }
            }
            CheckedType::Array { element, length } => {
                let length = self.checked_const_name(length)?;
                format!("array<{}, {length}>", self.checked_type_name(element.ty())?)
            }
            CheckedType::Slice {
                region,
                element,
                strength,
            } => {
                let element = self.checked_type_name(element.ty())?;
                let view = strength.spelling();
                match self.region_spelling(region).as_str() {
                    "" => format!("{view}<{element}>"),
                    region => format!("{view}<{region}, {element}>"),
                }
            }
            CheckedType::Buffer { element } => {
                format!("buffer<{}>", self.checked_type_name(element.ty())?)
            }
            CheckedType::FixedVector { element, length } => {
                let length = self.checked_const_name(length)?;
                format!(
                    "FixedVector<{}, {length}>",
                    self.checked_type_name(element.ty())?
                )
            }
            CheckedType::Vector {
                region, element, ..
            } => {
                let element = self.checked_type_name(element.ty())?;
                match self.region_spelling(region).as_str() {
                    "" => format!("Vector<{element}>"),
                    region => format!("Vector<{region}, {element}>"),
                }
            }
            CheckedType::Heap { region } => match self.region_spelling(region).as_str() {
                "" => "Heap".to_owned(),
                region => format!("Heap<{region}>"),
            },
            CheckedType::Extent {
                region,
                bytes,
                align,
            } => {
                let bytes = self.checked_const_name(bytes)?;
                let align = self.checked_const_name(align)?;
                match self.region_spelling(region).as_str() {
                    "" => format!("Arena<{bytes}, {align}>"),
                    region => format!("Arena<{region}, {bytes}, {align}>"),
                }
            }
        })
    }

    /// A field-suffix chain rooted at a struct-typed const [CONST-2
    /// candidate]. The path is resolved by the ordinary projection judgment,
    /// then folded against the constant's total value: a copy scalar
    /// selection copies out as a constant, and a composite selection keeps
    /// the whole-composite read rules.
    fn check_struct_constant_projection(
        &self,
        use_node: NodeId,
        constant: &CheckedConstant,
        suffixes: &[NodeId],
    ) -> Result<TypedExpression, CheckStop> {
        let (fields, ty) = self.resolve_struct_path(suffixes, constant.ty)?;
        let mut value = &constant.value;
        for index in &fields {
            let CheckedValue::Struct { fields: values, .. } = value else {
                return Err(SemanticCompilerFailure::InvalidResolution.into());
            };
            value = values
                .get(*index as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        }
        if value.ty() != ty {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        }
        match value {
            CheckedValue::Struct { .. } => self.issue_node(
                SemanticRule::Own1,
                use_node,
                SemanticIssueKind::BareAffineUse {
                    mechanical_fix: "read a const struct through its fields",
                },
            ),
            CheckedValue::Array { .. } => self.issue_node(
                SemanticRule::Own1,
                use_node,
                SemanticIssueKind::BareAffineUse {
                    mechanical_fix: "read a const array through `index` or `len`",
                },
            ),
            scalar => Ok(TypedExpression::owned(
                CheckedExpression::Constant(scalar.clone()),
                EffectSet::NONE,
            )),
        }
    }

    pub(super) fn checked_const_name(&self, value: CheckedConst) -> Result<String, CheckStop> {
        Ok(match value {
            CheckedConst::Value(value) => value.to_string(),
            CheckedConst::Parameter(declaration) => {
                format!("<const-parameter:{}>", declaration.index())
            }
            CheckedConst::Derived(id) => {
                let derived = self.derived_const(id)?;
                format!(
                    "{} {} {}",
                    self.checked_const_name(derived.left)?,
                    derived.operation.spelling(),
                    self.checked_const_name(derived.right)?
                )
            }
        })
    }

    /// Resolves a run of field-selection suffixes over one starting type.
    /// Callers pass the suffix chain to walk — every suffix for a whole
    /// place, or the chain before a subscript for that subscript's base. A
    /// subscript suffix inside the walked run selects through a composite
    /// element value, which this version does not implement.
    pub(super) fn resolve_struct_path(
        &self,
        suffixes: &[NodeId],
        mut ty: CheckedType,
    ) -> Result<(Vec<u32>, CheckedType), CheckStop> {
        let mut fields = Vec::new();
        for &suffix in suffixes {
            if self.subscript_offset(suffix)?.is_some() {
                return self.unsupported(UnsupportedSemanticFeature::CompositeValues, suffix);
            }
            let name = self
                .deferred_use_at(suffix, DeferredUseRole::ProjectedField)?
                .spelling();
            let CheckedType::Nominal(nominal_id) = ty else {
                return self.issue_node(
                    SemanticRule::Type5,
                    suffix,
                    SemanticIssueKind::type_mismatch(
                        "a source struct, whose declared field this suffix selects",
                        self.checked_type_name(ty)?,
                    ),
                );
            };
            let CheckedNominalKind::Struct {
                fields: declared_fields,
            } = &self.nominal(nominal_id)?.kind
            else {
                return self.issue_node(
                    SemanticRule::Type5,
                    suffix,
                    SemanticIssueKind::type_mismatch(
                        "a source struct, whose declared field this suffix selects",
                        self.checked_type_name(ty)?,
                    ),
                );
            };
            let Some((index, field)) = declared_fields
                .iter()
                .enumerate()
                .find(|(_, field)| field.name == name)
            else {
                return self.issue_node(
                    SemanticRule::Type5,
                    suffix,
                    SemanticIssueKind::type_mismatch(
                        format!("a declared field of {}", self.checked_type_name(ty)?),
                        format!("the field name `{name}`, which that struct does not declare"),
                    ),
                );
            };
            fields
                .push(u32::try_from(index).map_err(|_| SemanticCompilerFailure::CounterOverflow)?);
            ty = field.ty;
        }
        Ok((fields, ty))
    }

    pub(super) fn check_expression(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_expression_in_context(
            function,
            node,
            bindings,
            loop_depth,
            PlaceUseContext::Ordinary,
        )
    }

    pub(super) fn check_consuming_expression(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_expression_in_context(
            function,
            node,
            bindings,
            loop_depth,
            PlaceUseContext::Consuming,
        )
    }

    fn check_expression_in_context(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        place_context: PlaceUseContext,
    ) -> Result<TypedExpression, CheckStop> {
        // [GRAM-5] `clause_expr` is the contract-clause shape: one operand,
        // or two operands around one operator token. Its operands are the
        // same three written forms an `expr` selects between, so the two
        // shapes share every judgment below and differ only in where the
        // operator and the second operand hang.
        if self.tree.production(node)? == Production::ClauseExpr {
            return self.check_clause_expression(
                function,
                node,
                bindings,
                loop_depth,
                place_context,
            );
        }
        // [GRAM-5] `expr := atom infix_tail? | call | construct`, so the only
        // shape with more than one child is the infix one.
        if let Some(tail) = self.tree.first_child_with(node, Production::InfixTail)? {
            return self.check_infix(function, node, tail, bindings, loop_depth);
        }
        let child = self.tree.only_child(node)?;
        self.check_written_operand(function, child, bindings, loop_depth, place_context)
    }

    /// [GRAM-5] one `clause_expr`: one `affine_expr`, or two around one
    /// `clause_op`. Each side is [GRAM-4]'s own affine expression, whose
    /// factors may be a `call` and which therefore admits a measure term
    /// displaced by an affine expression on either side of the operator
    /// [MSR-5].
    fn check_clause_expression(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        place_context: PlaceUseContext,
    ) -> Result<TypedExpression, CheckStop> {
        match self.tree.children(node)? {
            [side] => {
                let side = *side;
                self.check_clause_affine(function, side, None, bindings, loop_depth, place_context)
            }
            [left, operator, right] => {
                let (left, operator, right) = (*left, *operator, *right);
                let operation = self.infix_operation(self.clause_operator_node(operator)?)?;
                let left = (
                    left,
                    self.check_clause_affine(
                        function,
                        left,
                        None,
                        bindings,
                        loop_depth,
                        PlaceUseContext::Ordinary,
                    )?,
                );
                let right = (
                    right,
                    self.check_clause_affine(
                        function,
                        right,
                        None,
                        bindings,
                        loop_depth,
                        PlaceUseContext::Ordinary,
                    )?,
                );
                self.check_integer_operation_operands(node, operation, vec![left, right])
            }
            _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
    }

    /// The operator token's owning node inside one `clause_op` [GRAM-5]: the
    /// `compare_op` node it selected, or the `clause_op` itself when the
    /// operator is one of the five infix `defined` domain queries.
    pub(super) fn clause_operator_node(&self, operator: NodeId) -> Result<NodeId, CheckStop> {
        Ok(self
            .tree
            .first_child_with(operator, Production::CompareOp)?
            .unwrap_or(operator))
    }

    /// One `affine_expr`, `affine_term`, or `affine_factor` of a contract
    /// clause [MSR-5].
    ///
    /// `terms` bounds an `affine_expr`'s left-associative fold to its first
    /// `terms` `affine_term` children, so `a + b - c` is `(a + b) - c` with
    /// no rewriting of the source tree. Its `+`, `-`, and `*` denote the
    /// mathematical integer expression [INV-1] fixes; the [OP-1] rows named
    /// here are the exact ones, which carry no domain obligation of their own
    /// because a clause is never evaluated.
    fn check_clause_affine(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        terms: Option<usize>,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        place_context: PlaceUseContext,
    ) -> Result<TypedExpression, CheckStop> {
        match self.tree.production(node)? {
            Production::AffineExpr => {
                let children = self.tree.children(node)?.to_vec();
                let count = terms.unwrap_or_else(|| children.len().div_ceil(2));
                let last = count
                    .checked_mul(2)
                    .and_then(|doubled| doubled.checked_sub(2))
                    .and_then(|index| children.get(index).copied())
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                if count == 1 {
                    return self.check_clause_affine(
                        function,
                        last,
                        None,
                        bindings,
                        loop_depth,
                        place_context,
                    );
                }
                let operator = children
                    .get(
                        count
                            .checked_mul(2)
                            .and_then(|doubled| doubled.checked_sub(3))
                            .ok_or(SemanticCompilerFailure::CounterOverflow)?,
                    )
                    .copied()
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                let operation = self.affine_add_operation(operator)?;
                let left = (
                    node,
                    self.check_clause_affine(
                        function,
                        node,
                        Some(count - 1),
                        bindings,
                        loop_depth,
                        PlaceUseContext::Ordinary,
                    )?,
                );
                let right = (
                    last,
                    self.check_clause_affine(
                        function,
                        last,
                        None,
                        bindings,
                        loop_depth,
                        PlaceUseContext::Ordinary,
                    )?,
                );
                self.check_integer_operation_operands(node, operation, vec![left, right])
            }
            Production::AffineTerm => {
                let factors = self.tree.children_with(node, Production::AffineFactor)?;
                match factors.as_slice() {
                    [factor] => self.check_clause_affine(
                        function,
                        *factor,
                        None,
                        bindings,
                        loop_depth,
                        place_context,
                    ),
                    [left_node, right_node] => {
                        let left = (
                            *left_node,
                            self.check_clause_affine(
                                function,
                                *left_node,
                                None,
                                bindings,
                                loop_depth,
                                PlaceUseContext::Ordinary,
                            )?,
                        );
                        let right = (
                            *right_node,
                            self.check_clause_affine(
                                function,
                                *right_node,
                                None,
                                bindings,
                                loop_depth,
                                PlaceUseContext::Ordinary,
                            )?,
                        );
                        self.check_integer_operation_operands(
                            node,
                            CheckedIntegerOperation::MultiplyExact,
                            vec![left, right],
                        )
                    }
                    _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
                }
            }
            Production::AffineFactor => {
                let child = self.tree.only_child(node)?;
                self.check_clause_affine(function, child, None, bindings, loop_depth, place_context)
            }
            _ => self.check_written_operand(function, node, bindings, loop_depth, place_context),
        }
    }

    /// The [OP-1] row one `affine_add_op` names [GRAM-4].
    fn affine_add_operation(&self, operator: NodeId) -> Result<CheckedIntegerOperation, CheckStop> {
        let [terminal] = self.tree.direct_token_indices(operator)? else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        Ok(match self.tree.token_bytes(*terminal)? {
            b"+" => CheckedIntegerOperation::AddExact,
            b"-" => CheckedIntegerOperation::SubtractExact,
            _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        })
    }

    /// One written operand of an `expr` or a `clause_expr` [GRAM-5]: the
    /// `atom`, `call`, or `construct` the grammar selected.
    pub(in crate::semantic::check) fn check_written_operand(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        place_context: PlaceUseContext,
    ) -> Result<TypedExpression, CheckStop> {
        match self.tree.production(node)? {
            Production::Atom => self.check_atom_in_context(
                function,
                node,
                bindings,
                loop_depth,
                place_context,
                ReborrowPosition::Forbidden,
            ),
            Production::Call => self.check_call(function, node, bindings, loop_depth),
            Production::Construct => self.check_construct(function, node, bindings, loop_depth),
            _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
    }

    /// [OP-1] (ii) infix resolution: the operator token selects the row.
    ///
    /// [GRAM-9] admits exactly one operation per expression, so there is no
    /// precedence to apply — the left operand is the `expr`'s own atom and
    /// the right is the tail's. The row then takes the same judgment the
    /// named spelling takes.
    fn check_infix(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        tail: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let left = self
            .tree
            .first_child_with(node, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let operator = self.infix_operator_node(tail)?;
        let right = self
            .tree
            .first_child_with(tail, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let operation = self.infix_operation(operator)?;
        self.check_integer_operation_row(
            node,
            operation,
            &[left, right],
            function,
            bindings,
            loop_depth,
        )
    }

    /// The operator child of an `infix_tail`: its `infix_op` or its
    /// `compare_op` node, whichever the tail selected [GRAM-5].
    pub(super) fn infix_operator_node(&self, tail: NodeId) -> Result<NodeId, CheckStop> {
        if let Some(operator) = self.tree.first_child_with(tail, Production::InfixOp)? {
            return Ok(operator);
        }
        self.tree
            .first_child_with(tail, Production::CompareOp)?
            .ok_or_else(|| SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    /// [OP-1] the exact operator token, and the row it spells.
    ///
    /// Bare `+ - * / %` are proof-required exact rows; `defined` names their
    /// total Bool domain queries. The remaining suffixes keep their existing
    /// value-result policies. The six `compare_op` spellings are the total
    /// integer comparison rows.
    pub(super) fn infix_operation(
        &self,
        operator: NodeId,
    ) -> Result<CheckedIntegerOperation, CheckStop> {
        let [terminal] = self.tree.direct_token_indices(operator)? else {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        };
        Ok(match self.tree.token_bytes(*terminal)? {
            b"+" => CheckedIntegerOperation::AddExact,
            b"+defined" => CheckedIntegerOperation::AddDefined,
            b"+wrap" => CheckedIntegerOperation::AddWrap,
            b"+checked" => CheckedIntegerOperation::AddChecked,
            b"+sat" => CheckedIntegerOperation::AddSaturating,
            b"-" => CheckedIntegerOperation::SubtractExact,
            b"-defined" => CheckedIntegerOperation::SubtractDefined,
            b"-wrap" => CheckedIntegerOperation::SubtractWrap,
            b"-checked" => CheckedIntegerOperation::SubtractChecked,
            b"-sat" => CheckedIntegerOperation::SubtractSaturating,
            b"*" => CheckedIntegerOperation::MultiplyExact,
            b"*defined" => CheckedIntegerOperation::MultiplyDefined,
            b"*wrap" => CheckedIntegerOperation::MultiplyWrap,
            b"*checked" => CheckedIntegerOperation::MultiplyChecked,
            b"*sat" => CheckedIntegerOperation::MultiplySaturating,
            b"/" => CheckedIntegerOperation::DivideExact,
            b"/defined" => CheckedIntegerOperation::DivideDefined,
            b"/checked" => CheckedIntegerOperation::DivideChecked,
            b"%" => CheckedIntegerOperation::RemainderExact,
            b"%defined" => CheckedIntegerOperation::RemainderDefined,
            b"%checked" => CheckedIntegerOperation::RemainderChecked,
            b"==" => CheckedIntegerOperation::Equal,
            b"!=" => CheckedIntegerOperation::NotEqual,
            b"<" => CheckedIntegerOperation::Less,
            b"<=" => CheckedIntegerOperation::LessEqual,
            b">" => CheckedIntegerOperation::Greater,
            b">=" => CheckedIntegerOperation::GreaterEqual,
            _ => return Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        })
    }

    pub(super) fn check_atom(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_atom_in_context(
            function,
            node,
            bindings,
            loop_depth,
            PlaceUseContext::Ordinary,
            ReborrowPosition::Forbidden,
        )
    }

    /// Checks an atom in a position whose owning rule decides whether the
    /// selected value is admissible. This delays OWN-1's bare-affine spelling
    /// rejection long enough for an earlier TYPE-7 implicit-read judgment to
    /// take exclusive ownership of a holder used for its referent.
    pub(super) fn check_consuming_atom(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_atom_in_context(
            function,
            node,
            bindings,
            loop_depth,
            PlaceUseContext::Consuming,
            ReborrowPosition::Forbidden,
        )
    }

    pub(super) fn check_call_argument_atom(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        own_result: bool,
        result_candidate: bool,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_atom_in_context(
            function,
            node,
            bindings,
            loop_depth,
            PlaceUseContext::Ordinary,
            ReborrowPosition::CallArgument {
                own_result,
                result_candidate,
            },
        )
    }

    fn check_atom_in_context(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        place_context: PlaceUseContext,
        reborrow_position: ReborrowPosition,
    ) -> Result<TypedExpression, CheckStop> {
        if let Some(value) = self.postcondition_result_placeholder(node)? {
            return Ok(TypedExpression::owned(
                CheckedExpression::Constant(value),
                EffectSet::NONE,
            ));
        }
        if let Some(literal) = self
            .tree
            .direct_token_with(node, TerminalPredicate::Literal)?
        {
            let bytes = self.tree.token_bytes(literal)?;
            if matches!(bytes, b"0_T" | b"1_T") {
                return self.check_generic_numeric_identity(function, node, bytes == b"1_T");
            }
            return Ok(TypedExpression::owned(
                CheckedExpression::Constant(self.parse_literal(node, bytes)?),
                EffectSet::NONE,
            ));
        }
        if let Some(place) = self.tree.first_child_with(node, Production::Place)? {
            let value = self.check_place_use(
                function,
                node,
                place,
                bindings,
                PlaceUseOptions {
                    explicit_move: self.has_fixed(node, FixedTerminal::Move)?,
                    context: place_context,
                    loop_depth,
                },
            )?;
            return Ok(value);
        }
        if let Some(borrow) = self.tree.first_child_with(node, Production::BorrowExpr)? {
            return self.check_borrow(borrow, function, bindings, loop_depth, reborrow_position);
        }
        Err(SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    /// The `borrow_expr` that is the complete written content of `expression`,
    /// if any: the position [OWN-14] names for the returned reborrow.
    ///
    /// An infix expression is a fresh operation result rather than a written
    /// borrow, so it answers `None` like any other non-borrow shape.
    pub(super) fn complete_borrow_expression(
        &self,
        expression: NodeId,
    ) -> Result<Option<NodeId>, CheckStop> {
        let Some(child) = self.tree.sole_expression_child(expression)? else {
            return Ok(None);
        };
        if self.tree.production(child)? != Production::Atom {
            return Ok(None);
        }
        Ok(self.tree.first_child_with(child, Production::BorrowExpr)?)
    }

    fn check_generic_numeric_identity(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        one: bool,
    ) -> Result<TypedExpression, CheckStop> {
        let usage = self.use_at(node, LexicalUseRole::GenericNumericSuffix)?;
        let ResolvedTarget::Source {
            declaration,
            class: DeclarationClass::GenericType,
        } = usage.target()
        else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let ty = function
            .substitution
            .type_argument(declaration)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let value = match ty {
            CheckedType::Integer(ty) => CheckedValue::Integer {
                ty,
                bits: u64::from(one),
            },
            CheckedType::Float(FloatType::F32) => CheckedValue::Float {
                ty: FloatType::F32,
                bits: if one { 0x3f80_0000 } else { 0 },
            },
            CheckedType::Float(FloatType::F64) => CheckedValue::Float {
                ty: FloatType::F64,
                bits: if one { 0x3ff0_0000_0000_0000 } else { 0 },
            },
            CheckedType::GenericInt(_) | CheckedType::GenericFloat(_) => {
                CheckedValue::NumericIdentity { ty, one }
            }
            _ => {
                return self.issue_node(
                    SemanticRule::Form5,
                    node,
                    SemanticIssueKind::type_mismatch(
                        "an integer or float type, whose 0 and 1 this form names",
                        self.checked_type_name(ty)?,
                    ),
                );
            }
        };
        Ok(TypedExpression::owned(
            CheckedExpression::Constant(value),
            EffectSet::NONE,
        ))
    }

    fn check_place_use(
        &self,
        function: &FunctionSignature,
        use_node: NodeId,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        options: PlaceUseOptions,
    ) -> Result<TypedExpression, CheckStop> {
        let pbase = self
            .tree
            .first_child_with(node, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let suffixes = self.tree.children_with(node, Production::Psuffix)?;
        if let Some(subscript) = self.last_subscript(&suffixes)? {
            return self.check_index_use(
                function, use_node, node, &suffixes, subscript, bindings, options,
            );
        }
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            return self.check_dereferenced_place_use(use_node, node, pbase, bindings, options);
        }
        if !self.tree.children(pbase)?.is_empty() {
            return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
        }
        let usage = self.use_at(pbase, LexicalUseRole::PlaceBase)?;
        let ResolvedTarget::Source { declaration, class } = usage.target() else {
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        match class {
            DeclarationClass::Value => {
                let local = bindings
                    .get(&declaration)
                    .cloned()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                if !local.live {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::UseAfterMove {
                            mechanical_fix: "introduce a new `let` binding before reuse",
                        },
                    );
                }
                if local.mode != CheckedMode::Own {
                    if !suffixes.is_empty() {
                        return self.issue_node(
                            SemanticRule::Type7,
                            use_node,
                            SemanticIssueKind::MissingDereference {
                                mechanical_fix: "write `deref(holder)`",
                            },
                        );
                    }
                    let copy = matches!(local.mode, CheckedMode::Shared(_));
                    if options.explicit_move && copy {
                        return self.issue_node(
                            SemanticRule::Own1,
                            use_node,
                            SemanticIssueKind::MoveOfCopy {
                                mechanical_fix: "use the copy place without `move`",
                            },
                        );
                    }
                    if !copy
                        && !options.explicit_move
                        && matches!(options.context, PlaceUseContext::Ordinary)
                    {
                        return self.issue_node(
                            SemanticRule::Own1,
                            use_node,
                            SemanticIssueKind::BareAffineUse {
                                mechanical_fix: "write `move p` for the affine place",
                            },
                        );
                    }
                    // A suspended holder admits no move, copy, or
                    // call-transfer of itself [OWN-5, OWN-13]; OWN-1's
                    // spelling judgments above are defined first and cite
                    // first at this node [DIAG-1].
                    self.check_holder_not_suspended(&local, use_node)?;
                    if !copy {
                        bindings
                            .get_mut(&declaration)
                            .ok_or(SemanticCompilerFailure::InvalidResolution)?
                            .live = false;
                    }
                    let slice = local.slice;
                    let slice_origins = slice
                        .as_ref()
                        .map(|slice| slice.origins.clone())
                        .unwrap_or_default();
                    return Ok(TypedExpression {
                        expression: CheckedExpression::Binding {
                            carrier: self.tree.path(use_node)?.clone(),
                            binding: local.binding,
                            state_origins: local.state_origins.clone(),
                            ty: local.ty,
                            slice_origins,
                            consume_root: !copy,
                        },
                        mode: local.mode,
                        borrow: local.borrow,
                        slice,
                        holder: Some(declaration),
                        // A bare borrow holder selects the holder, not its
                        // referent [TYPE-7, SET-1].
                        reference_value: true,
                        effects: EffectSet::NONE,
                        accesses: Vec::new(),
                    });
                }
                let (fields, ty) = self.resolve_struct_path(&suffixes, local.ty)?;
                let copy = self.is_copy_type(ty)?;
                if options.explicit_move && copy && self.judges_class_spelling() {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::MoveOfCopy {
                            mechanical_fix: "use the copy place without `move`",
                        },
                    );
                }
                if !copy
                    && !options.explicit_move
                    && matches!(options.context, PlaceUseContext::Ordinary)
                {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::BareAffineUse {
                            mechanical_fix: "write `move p` for the affine place",
                        },
                    );
                }
                // [LIV-2] a `move` of a target place of this statement's
                // commit, or of a place reached through one, is that target's
                // read-out: the previous value leaves, the root stays live,
                // and the same statement reinitializes the target. It is not
                // [OWN-1]'s root-killing consume and derives no residual
                // cleanup of the root's unselected content.
                let read_out = !copy
                    && options.explicit_move
                    && self.take_commit_read_out(&ResolvedPlace {
                        root: declaration,
                        fields: fields.clone(),
                    });
                // OWN-1 makes an affine projection consume its whole root.
                // Its residual cleanup destroys every unselected resource
                // field, so the loan access is the root rather than only the
                // selected projection. A read-out consumes exactly its own
                // place, so its access is that place.
                let access_fields = if copy || read_out {
                    fields.clone()
                } else {
                    Vec::new()
                };
                let access_kind = if copy {
                    AccessKind::Read
                } else {
                    AccessKind::Move
                };
                self.check_loan_access(
                    bindings,
                    None,
                    &ResolvedPlace {
                        root: declaration,
                        fields: access_fields.clone(),
                    },
                    access_kind,
                    use_node,
                )?;
                // [PROV-6] a consume of a proper sub-place of a value linear
                // in this scope, with no commit reinitialising that sub-place,
                // is a partial consume: the residual leaf is abandoned in a
                // scope that has no derived release to reclaim it.
                if !copy && !read_out && !fields.is_empty() {
                    self.reject_partial_consume(local.ty, &fields, use_node)?;
                }
                let residual_drops = if copy || read_out || fields.is_empty() {
                    Vec::new()
                } else {
                    let paths = self.residual_drop_paths(local.ty, &fields)?;
                    self.released_paths(paths)?
                        .into_iter()
                        .map(|(fields, ty, release)| CheckedProjectedDrop {
                            state_origins: local
                                .state_origins
                                .clone()
                                .map(|origins| origins.projected(&fields)),
                            fields,
                            ty,
                            release,
                        })
                        .collect()
                };
                // [LIV-2] after its read-out the target is dead for the
                // remainder of the right-hand side, and the commit reinitializes
                // it. At a complete binding that is exactly this binding's own
                // liveness, so the ordinary kill stands and the commit revives
                // it; at a projection the root keeps its other content and only
                // the target place is spent, which the commit's own read-out
                // record carries.
                if !copy && (!read_out || fields.is_empty()) {
                    bindings
                        .get_mut(&declaration)
                        .ok_or(SemanticCompilerFailure::InvalidResolution)?
                        .live = false;
                }
                let access = ResolvedPlace {
                    root: declaration,
                    fields: access_fields,
                };
                let mut effects = EffectSet::NONE;
                // [LIV-2, EFF-2] a read-out reads the target's own storage,
                // exactly as [SET-2]'s exchange does, and the commit writes it.
                //
                // [EFF-1] a loan-bearing value's effect path names the viewed
                // backing state and not the descriptor, and merely moving,
                // returning or structurally repacking that value observes
                // none of it: a read *through* the view is the subscript's own
                // attribution. Before [S27] made the shared view copy this
                // guard was invisible, because a consume exhibited no read at
                // all; the copy spelling is what would otherwise have made
                // `return value;` declare a read of storage it never touches.
                if (matches!(access_kind, AccessKind::Read) || read_out)
                    && !Self::checked_type_is_loan_bearing(ty)
                {
                    for path in self.effect_paths_for_place(&access, bindings)? {
                        effects.add_read(path);
                    }
                }
                if fields.is_empty() {
                    let slice = local.slice;
                    let slice_origins = slice
                        .as_ref()
                        .map(|slice| slice.origins.clone())
                        .unwrap_or_default();
                    let mut expression = TypedExpression::owned_with_access(
                        CheckedExpression::Binding {
                            carrier: self.tree.path(use_node)?.clone(),
                            binding: local.binding,
                            state_origins: local
                                .state_origins
                                .clone()
                                .map(|origins| origins.projected(&fields)),
                            ty,
                            slice_origins,
                            consume_root: !copy,
                        },
                        effects,
                        access,
                        access_kind,
                    );
                    expression.slice = slice;
                    Ok(expression)
                } else {
                    Ok(TypedExpression::owned_with_access(
                        CheckedExpression::Project {
                            carrier: self.tree.path(use_node)?.clone(),
                            binding: local.binding,
                            state_origins: local
                                .state_origins
                                .clone()
                                .map(|origins| origins.projected(&fields)),
                            fields,
                            ty,
                            consume_root: !copy && !read_out,
                            residual_drops,
                        },
                        effects,
                        access,
                        access_kind,
                    ))
                }
            }
            // [MSR-6] an in-scope const generic is a value wherever a named
            // const is. It is one `pbase` with no suffix and no `deref`, its
            // exact type is the `gparam`'s written integer type, and reading
            // it performs no operation and has the empty effect row.
            DeclarationClass::ConstGeneric => {
                if options.explicit_move {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::MoveOfCopy {
                            mechanical_fix: "use the copy place without `move`",
                        },
                    );
                }
                if !suffixes.is_empty() {
                    return self.issue_node(
                        SemanticRule::Type5,
                        use_node,
                        SemanticIssueKind::type_mismatch(
                            "a const generic read with no suffix",
                            "a suffix chain on an integer const generic",
                        ),
                    );
                }
                let ty = self.const_generic_type(declaration)?;
                let value = match function.substitution.const_argument(declaration) {
                    Some(CheckedConst::Value(value)) => CheckedValue::Integer { ty, bits: value },
                    // [FN-2, MSR-6] a const parameter this instance's caller
                    // supplied from a const parameter of its own is that
                    // caller's parameter here. Keeping this declaration would
                    // anchor the constant to a parameter nothing outside this
                    // instance can name, and every relation published over it
                    // would be dropped at the call.
                    Some(CheckedConst::Parameter(supplied)) => CheckedValue::ConstGeneric {
                        declaration: supplied,
                        ty,
                    },
                    // The one source-canonical symbolic instance keeps the
                    // declaration-anchored constant [ENT-2] clause (c) fixes.
                    _ => CheckedValue::ConstGeneric { declaration, ty },
                };
                Ok(TypedExpression::owned(
                    CheckedExpression::Constant(value),
                    EffectSet::NONE,
                ))
            }
            DeclarationClass::NamedConst => {
                if options.explicit_move {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::MoveOfCopy {
                            mechanical_fix: "use the copy place without `move`",
                        },
                    );
                }
                let constant = self
                    .constants
                    .get(&declaration)
                    .copied()
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let constant = self.constant(constant)?;
                if !suffixes.is_empty() {
                    // A field-suffix chain rooted at a struct-typed const
                    // [CONST-2 candidate] copies the selected value out; the
                    // selection is total at compile time, so the read folds
                    // to the selected constant.
                    if matches!(constant.value, CheckedValue::Struct { .. }) {
                        return self
                            .check_struct_constant_projection(use_node, constant, &suffixes);
                    }
                    return self.unsupported(UnsupportedSemanticFeature::CompositeValues, node);
                }
                if matches!(
                    constant.ty,
                    CheckedType::Array { .. }
                        | CheckedType::Slice { .. }
                        | CheckedType::Buffer { .. }
                ) {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::BareAffineUse {
                            mechanical_fix: "read a const array through `index` or `len`",
                        },
                    );
                }
                if matches!(constant.value, CheckedValue::Struct { .. }) {
                    return self.issue_node(
                        SemanticRule::Own1,
                        use_node,
                        SemanticIssueKind::BareAffineUse {
                            mechanical_fix: "read a const struct through its fields",
                        },
                    );
                }
                Ok(TypedExpression::owned(
                    CheckedExpression::NamedConstant {
                        declaration,
                        value: constant.value.clone(),
                    },
                    EffectSet::NONE,
                ))
            }
            _ => Err(SemanticCompilerFailure::InvalidResolution.into()),
        }
    }

    /// A [SET-1] or [SET-2] target whose `deref` reaches storage the root
    /// binding owns rather than a referent behind a holder. [SET-1]'s
    /// writability relation admits both roots, and this one directly: the
    /// target is rooted in a live own-mode value binding whose storage is
    /// box-owned or arena-owned [STOR-1]. There is no holder here to be live,
    /// usable, `&uniq`, or unsuspended, so the judgment is the ordinary
    /// own-rooted one — liveness [OWN-1], the loan state [OWN-5], then the
    /// final selected type's class.
    ///
    /// Past the judgment nothing writes owned indirection content: the target
    /// names the root binding, which lowers to the content pointer under the
    /// box's own IR type, and arena storage has no runtime at all. The target
    /// therefore stops at an explicit capability gate rather than publishing a
    /// checked program whose single store would overwrite the pointer.
    fn check_owned_content_set_target(
        &self,
        node: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        form: MutationForm,
        root: (LocalBinding, OwnedContent),
    ) -> Result<MutationTarget, CheckStop> {
        let (local, content) = root;
        if !local.live {
            return self.issue_node(
                SemanticRule::Own1,
                node,
                SemanticIssueKind::UseAfterMove {
                    mechanical_fix: "introduce a new `let` binding before reuse",
                },
            );
        }
        // The written suffix chain still selects a real field of the content
        // type, so a wrong spelling stays a source rejection rather than being
        // masked by the capability stop below [DIAG-1].
        let suffixes = self.tree.children_with(node, Production::Psuffix)?;
        let (fields, ty) = self.resolve_struct_path(&suffixes, content.ty())?;
        // Owned indirection content is reached from the owning binding, so the
        // resolved place is that root plus the selected field path — the same
        // place the read path resolves for a `deref` of this binding.
        self.check_loan_access(
            bindings,
            None,
            &ResolvedPlace {
                root: local.declaration,
                fields: fields.clone(),
            },
            AccessKind::Write,
            node,
        )?;
        self.check_mutation_target_class(node, ty, form)?;
        // TEMPORARY capability stop, carried rather than raised: [LIV-2]'s
        // commit conditions are source rejections and are judged first, so a
        // live affine content target still reports [STOR-1] and only a form
        // this compiler cannot lower reaches the stop.
        let unsupported = Some(match content {
            OwnedContent::Arena { .. } => UnsupportedSemanticFeature::ArenaRuntime,
            OwnedContent::Boxed(_) => UnsupportedSemanticFeature::RegionsAndBorrows,
        });
        Ok(MutationTarget {
            declaration: local.declaration,
            place: ResolvedPlace {
                root: local.declaration,
                fields: fields.clone(),
            },
            element: false,
            target: CheckedSetTarget::Place(CheckedWritablePlace {
                binding: local.binding,
                fields,
                ty,
                declares: false,
            }),
            effects: EffectSet::NONE,
            unsupported,
        })
    }

    fn check_dereferenced_set_target(
        &self,
        node: NodeId,
        pbase: NodeId,
        bindings: &HashMap<DeclarationId, LocalBinding>,
        form: MutationForm,
    ) -> Result<MutationTarget, CheckStop> {
        // [SET-1] makes a `deref` target writable through either of two roots:
        // an explicit `deref` of a live usable `&uniq` holder, or a live
        // own-mode binding whose storage the `deref` reaches [STOR-1]. Only
        // the first has a holder to resolve. Routing every `deref` target
        // through `resolve_dereference_holder` demanded one of an own-mode
        // `box` or `arena` binding and cited TYPE-7 `MissingDereference`
        // against source that wrote no holder — a compiler capability gap
        // misreported as invalid source, and the mutation-target twin of the
        // same defect in the borrow dispatch.
        if let Some(root) = self.owned_content_deref_root(pbase, bindings)? {
            return self.check_owned_content_set_target(node, bindings, form, root);
        }
        let (declaration, local, borrow) =
            self.resolve_dereference_holder(node, pbase, bindings)?;
        // [SET-1] states the shared-borrow referent as an [OWN-5] violation
        // and gives that rule the citation; SET-1 owns only the residue of its
        // writability relation.
        if borrow.kind != super::borrows::BorrowKind::Unique {
            return self.issue_node(SemanticRule::Own5, node, SemanticIssueKind::BorrowConflict);
        }
        self.check_holder_not_suspended(&local, node)?;
        let suffixes = self.tree.children_with(node, Production::Psuffix)?;
        let (fields, ty) = self.resolve_struct_path(&suffixes, local.ty)?;
        let mut resolved = borrow.place;
        resolved.fields.extend_from_slice(&fields);
        self.check_loan_access(
            bindings,
            Some(declaration),
            &resolved,
            AccessKind::Write,
            node,
        )?;
        self.check_mutation_target_class(node, ty, form)?;
        let mut effects = EffectSet::NONE;
        for path in self.effect_paths_for_place(&resolved, bindings)? {
            effects.add_write(path.clone());
            if form.is_replace() {
                // [SET-2, EFF-2]: the commit is one read and one write of
                // the target's ultimate storage origin.
                effects.add_read(path);
            }
        }
        Ok(MutationTarget {
            declaration,
            place: resolved,
            element: false,
            target: CheckedSetTarget::Place(CheckedWritablePlace {
                binding: local.binding,
                fields,
                ty,
                declares: false,
            }),
            effects,
            unsupported: None,
        })
    }

    pub(super) fn check_match_expression(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_consuming_expression(function, node, bindings, loop_depth)
    }

    /// [PROV-6] the operand of a `dispose` statement or of a destructuring
    /// consume: an ordinary consuming place use, judged by [OWN-1] exactly as
    /// every other consuming position is.
    pub(super) fn check_consumed_place(
        &self,
        function: &FunctionSignature,
        use_node: NodeId,
        place: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        explicit_move: bool,
    ) -> Result<TypedExpression, CheckStop> {
        self.check_place_use(
            function,
            use_node,
            place,
            bindings,
            PlaceUseOptions {
                explicit_move,
                context: PlaceUseContext::Consuming,
                loop_depth,
            },
        )
    }

    /// One `construct` of a nominal carrying `region_params` [FORM-8].
    ///
    /// Every other construct forms its instance from the written argument
    /// list and then checks its operands against that instance's fields.
    /// Here the operands come first, because they are what determines the
    /// instance: a field whose declared type names one of the declaration's
    /// region parameters supplies that region from its own actual, exactly as
    /// a parameter position supplies a callee's formal region at a call, and
    /// the position writes only the region parameters no field's declared
    /// type mentions. [TYPE-5]'s ground is untouched — construction still
    /// consults no expected nominal type, and it is the operands and the
    /// written list, never a destination, that fix the instance.
    ///
    /// The instance is formed once the regions are known and every operand is
    /// then compared against *its* declared field types by the ordinary exact
    /// [TYPE-5] equality, so a second operand naming a second store is a
    /// mismatch and not a second binding [PROV-1].
    fn check_regional_construct(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
        site: &super::ConstructorSite,
        constructor_name: String,
    ) -> Result<TypedExpression, CheckStop> {
        let written_fields = match self
            .tree
            .first_child_with(node, Production::FieldinitList)?
        {
            Some(list) => self.tree.children_with(list, Production::Fieldinit)?,
            None => Vec::new(),
        };
        if written_fields.len() != site.shape.fields.len() {
            return self.issue_node(
                SemanticRule::Gram8,
                node,
                SemanticIssueKind::InvalidConstructionFields {
                    constructor: constructor_name,
                    declared_fields: site.shape.fields.clone(),
                },
            );
        }
        // [FORM-8] a written region argument this construct's own operands
        // determine carries no fact a reader can check, and the diagnostic
        // names the one repair rather than an argument count.
        self.reject_determined_region_arguments(node, site)?;
        let mut atoms = Vec::with_capacity(written_fields.len());
        let mut operands = Vec::with_capacity(written_fields.len());
        let mut effects = EffectSet::NONE;
        for (written, declared) in written_fields.into_iter().zip(&site.shape.fields) {
            if self
                .deferred_use_at(written, DeferredUseRole::FieldInitializer)?
                .spelling()
                != *declared
            {
                return self.issue_node(
                    SemanticRule::Gram8,
                    written,
                    SemanticIssueKind::InvalidConstructionFields {
                        constructor: constructor_name,
                        declared_fields: site.shape.fields.clone(),
                    },
                );
            }
            let atom = self
                .tree
                .first_child_with(written, Production::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let value = self.check_atom(function, atom, bindings, loop_depth)?;
            effects = effects.union(value.effects.clone());
            atoms.push(atom);
            operands.push(value);
        }
        let mut determined = Vec::with_capacity(site.region_parameters.len());
        for (slot, formal) in site.region_parameters.iter().enumerate() {
            let Some(field) = site.shape.determining_field.get(slot).copied().flatten() else {
                continue;
            };
            let operand = operands
                .get(field)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let atom = *atoms
                .get(field)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let ty = operand.expression.ty();
            let Some(actual) = self.written_type_region(ty)? else {
                return self.issue_node(
                    SemanticRule::Type5,
                    atom,
                    SemanticIssueKind::type_mismatch(
                        "a value whose type names the store this field's declared type brands"
                            .to_owned(),
                        self.checked_type_name(ty)?,
                    ),
                );
            };
            determined.push((*formal, actual));
        }
        let nominal = self.constructed_nominal(node, site, &determined, &function.substitution)?;
        let declared_fields = match (&self.nominal(nominal)?.kind, site.variant) {
            (CheckedNominalKind::Struct { fields }, None) => fields.clone(),
            (CheckedNominalKind::Enum { variants }, Some(variant)) => variants
                .get(variant as usize)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?
                .fields
                .clone(),
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        let mut fields = Vec::with_capacity(operands.len());
        for ((value, atom), declared) in operands.into_iter().zip(atoms).zip(&declared_fields) {
            if value.expression.ty() != declared.ty {
                return self.issue_node(
                    SemanticRule::Type5,
                    atom,
                    SemanticIssueKind::type_mismatch(
                        self.checked_type_name(declared.ty)?,
                        self.checked_type_name(value.expression.ty())?,
                    ),
                );
            }
            if value.mode != CheckedMode::Own {
                return self.issue_node(
                    SemanticRule::Type7,
                    atom,
                    SemanticIssueKind::MissingDereference {
                        mechanical_fix: "write `deref(holder)`",
                    },
                );
            }
            fields.push(value.expression);
        }
        let carrier = self.tree.path(node)?.clone();
        let expression = match site.variant {
            None => CheckedExpression::ConstructStruct {
                carrier,
                nominal,
                fields,
            },
            Some(variant) => CheckedExpression::ConstructEnum {
                carrier,
                nominal,
                variant,
                fields,
            },
        };
        Ok(TypedExpression::owned(expression, effects))
    }

    /// [FORM-8] a construct that writes a region argument its own field
    /// operands determine.
    ///
    /// The old spelling wrote every region parameter, so the shape this
    /// refuses is the complete list where a shorter one is legal, and the
    /// repair is to delete the members the operands supply. A list that is
    /// wrong in some other way stays the ordinary [TYPE-5] argument fault.
    fn reject_determined_region_arguments(
        &self,
        node: NodeId,
        site: &super::ConstructorSite,
    ) -> Result<(), CheckStop> {
        let determined = site
            .shape
            .determining_field
            .iter()
            .filter(|field| field.is_some())
            .count();
        if determined == 0 {
            return Ok(());
        }
        let Some(targs) = self.tree.first_child_with(node, Production::Targs)? else {
            return Ok(());
        };
        let arguments = self.tree.children_with(targs, Production::Targ)?;
        let expected = site
            .generic_parameters
            .len()
            .saturating_add(site.region_parameters.len())
            .saturating_sub(determined);
        if arguments.len() <= expected {
            return Ok(());
        }
        for argument in arguments.iter().take(site.region_parameters.len()) {
            if self
                .tree
                .first_child_with(*argument, Production::Type)?
                .is_some()
                || self
                    .tree
                    .first_child_with(*argument, Production::Const)?
                    .is_some()
            {
                return Ok(());
            }
        }
        self.issue_node(
            SemanticRule::Form8,
            node,
            SemanticIssueKind::RegionSpelling {
                mechanical_fix: "drop the region argument",
            },
        )
    }

    pub(super) fn check_construct(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        loop_depth: usize,
    ) -> Result<TypedExpression, CheckStop> {
        let usage = self.use_at(node, LexicalUseRole::Construct)?;
        let constructor_name = usage.spelling().to_owned();
        if let ResolvedTarget::Prelude(id) = usage.target()
            && matches!(id.ordinal(), 1 | 2)
        {
            let value = match id.ordinal() {
                1 => CheckedValue::Bool(true),
                2 => CheckedValue::Bool(false),
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            };
            if self
                .tree
                .first_child_with(node, Production::FieldinitList)?
                .is_some()
            {
                return self.issue_node(
                    SemanticRule::Gram8,
                    node,
                    SemanticIssueKind::InvalidConstructionFields {
                        constructor: constructor_name,
                        declared_fields: Vec::new(),
                    },
                );
            }
            return Ok(TypedExpression::owned(
                CheckedExpression::Constant(value),
                EffectSet::NONE,
            ));
        }
        // [BLK-1] the four compiler-owned nominals contribute a constructor
        // entry that exists to be refused: no `construct` produces a run, a
        // provider, or a store.
        if let ResolvedTarget::Container(id) = usage.target() {
            let nominal =
                crate::container_nominal(id).ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let restructuring = match nominal.shape {
                crate::ContainerShape::Vector | crate::ContainerShape::FixedVector => {
                    "form the run with a formation operation"
                }
                crate::ContainerShape::Heap | crate::ContainerShape::Arena => {
                    "receive the provider as a parameter"
                }
                crate::ContainerShape::Box => "form the cell with heap_box or arena_box",
            };
            return self.issue_node(
                SemanticRule::Blk1,
                node,
                SemanticIssueKind::ContainerConstruction {
                    nominal: constructor_name,
                    mechanical_fix: restructuring,
                },
            );
        }
        let constructor = match usage.target() {
            ResolvedTarget::Source { declaration, .. } => {
                // [FORM-8] a nominal carrying `region_params` has its region
                // arguments determined by its field operands, so its
                // instance is formed after they are checked and not before.
                if let Some(site) = self.constructor_shape(declaration)? {
                    return self.check_regional_construct(
                        function,
                        node,
                        bindings,
                        loop_depth,
                        &site,
                        constructor_name,
                    );
                }
                self.source_constructor(node, declaration, &function.substitution)?
            }
            ResolvedTarget::Prelude(id) => match id.ordinal() {
                // [TYPE-5] the prelude generic nominals are constructed
                // through these variant constructors, and they write the
                // nominal's arguments in every position, mandatorily:
                // `None()` has no operand to supply them and construction
                // never consults an expected nominal type [TYPE-6]. The
                // written arguments are read here exactly as
                // `generic_substitution` reads a source generic's, so both
                // classes cite TYPE-5 at the complete `construct`.
                5 | 6 => {
                    let value = self.option_type_argument_with(node, &function.substitution)?;
                    Constructor::Enum {
                        nominal: self.prelude_nominal(super::PreludeType::Option(value))?,
                        variant: u32::from(id.ordinal() == 6),
                    }
                }
                11 | 13 => {
                    let (ok, error) =
                        self.result_type_arguments_with(node, &function.substitution)?;
                    Constructor::Enum {
                        nominal: self.prelude_nominal(super::PreludeType::Result(ok, error))?,
                        variant: u32::from(id.ordinal() == 13),
                    }
                }
                16 => Constructor::Enum {
                    nominal: self.prelude_nominal(super::PreludeType::Overflow)?,
                    variant: 0,
                },
                18 | 19 => Constructor::Enum {
                    nominal: self.prelude_nominal(super::PreludeType::DivError)?,
                    variant: u32::from(id.ordinal() == 19),
                },
                21 => Constructor::Enum {
                    nominal: self.prelude_nominal(super::PreludeType::NarrowError)?,
                    variant: 0,
                },
                _ => {
                    return self
                        .unsupported(UnsupportedSemanticFeature::PreludeNominalValues, node);
                }
            },
            ResolvedTarget::System(id) => {
                let index = crate::system_constructor_index(id, self.inventory())
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let record = crate::SYSTEM_CONSTRUCTORS
                    .get(usize::from(index))
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?;
                let tag = crate::SYSTEM_CONSTRUCTORS[..usize::from(index)]
                    .iter()
                    .filter(|candidate| candidate.owner == record.owner)
                    .count();
                Constructor::Enum {
                    nominal: self.system_nominal(record.owner)?,
                    variant: u32::try_from(tag)
                        .map_err(|_| SemanticCompilerFailure::CounterOverflow)?,
                }
            }
            _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
        };
        let declared_fields = match constructor {
            Constructor::Struct(nominal) => match &self.nominal(nominal)?.kind {
                CheckedNominalKind::Struct { fields } => fields.clone(),
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            },
            Constructor::Enum { nominal, variant } => match &self.nominal(nominal)?.kind {
                CheckedNominalKind::Enum { variants } => variants
                    .get(variant as usize)
                    .ok_or(SemanticCompilerFailure::InvalidResolution)?
                    .fields
                    .clone(),
                _ => return Err(SemanticCompilerFailure::InvalidResolution.into()),
            },
        };
        let written_fields = if let Some(list) = self
            .tree
            .first_child_with(node, Production::FieldinitList)?
        {
            self.tree.children_with(list, Production::Fieldinit)?
        } else {
            Vec::new()
        };
        let declared_field_names = declared_fields
            .iter()
            .map(|field| field.name.clone())
            .collect::<Vec<_>>();
        if written_fields.len() != declared_fields.len() {
            return self.issue_node(
                SemanticRule::Gram8,
                node,
                SemanticIssueKind::InvalidConstructionFields {
                    constructor: constructor_name,
                    declared_fields: declared_field_names,
                },
            );
        }
        let mut fields = Vec::with_capacity(written_fields.len());
        let mut effects = EffectSet::NONE;
        for (written, declared) in written_fields.into_iter().zip(&declared_fields) {
            if self
                .deferred_use_at(written, DeferredUseRole::FieldInitializer)?
                .spelling()
                != declared.name
            {
                return self.issue_node(
                    SemanticRule::Gram8,
                    written,
                    SemanticIssueKind::InvalidConstructionFields {
                        constructor: constructor_name,
                        declared_fields: declared_field_names,
                    },
                );
            }
            let atom = self
                .tree
                .first_child_with(written, Production::Atom)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            let value = self.check_atom(function, atom, bindings, loop_depth)?;
            if value.expression.ty() != declared.ty {
                return self.issue_node(
                    SemanticRule::Type5,
                    atom,
                    SemanticIssueKind::type_mismatch(
                        self.checked_type_name(declared.ty)?,
                        self.checked_type_name(value.expression.ty())?,
                    ),
                );
            }
            if value.mode != CheckedMode::Own {
                return self.issue_node(
                    SemanticRule::Type7,
                    atom,
                    SemanticIssueKind::MissingDereference {
                        mechanical_fix: "write `deref(holder)`",
                    },
                );
            }
            effects = effects.union(value.effects);
            fields.push(value.expression);
        }
        let expression = match constructor {
            Constructor::Struct(nominal) => CheckedExpression::ConstructStruct {
                carrier: self.tree.path(node)?.clone(),
                nominal,
                fields,
            },
            Constructor::Enum { nominal, variant } => CheckedExpression::ConstructEnum {
                carrier: self.tree.path(node)?.clone(),
                nominal,
                variant,
                fields,
            },
        };
        Ok(TypedExpression::owned(expression, effects))
    }
}
