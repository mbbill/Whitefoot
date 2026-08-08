use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminal;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::model::CheckedStatement;
use super::{
    CheckStop, Checker, ControlCounters, ControlScope, EffectSet, FunctionSignature, LocalBinding,
};

pub(super) struct CheckedRequires {
    pub(super) statements: Vec<CheckedStatement>,
    pub(super) effects: EffectSet,
}

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn check_requires(
        &self,
        function: &FunctionSignature,
        node: NodeId,
        bindings: &mut HashMap<DeclarationId, LocalBinding>,
        counters: &mut ControlCounters<'_>,
    ) -> Result<CheckedRequires, CheckStop> {
        let entries = self.tree.children_with(node, Production::RequiresEntry)?;
        let mut statements = Vec::with_capacity(entries.len());
        let mut effects = EffectSet::NONE;
        for entry in entries {
            let wrapper = self.tree.only_child(entry)?;
            if self.tree.production(wrapper)? != Production::Stmt {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            let statement = self.tree.only_child(wrapper)?;
            self.validate_requires_statement(entry, statement)?;
            let checked = self.check_statement(
                function,
                statement,
                bindings,
                counters,
                ControlScope {
                    loops: &[],
                    give_context: None,
                },
            )?;
            if !checked.can_continue {
                return Err(SemanticCompilerFailure::InvalidCanonicalTree.into());
            }
            effects = effects.union(checked.effects);
            statements.push(checked.statement);
        }
        Ok(CheckedRequires {
            statements,
            effects,
        })
    }

    fn validate_requires_statement(
        &self,
        entry: NodeId,
        statement: NodeId,
    ) -> Result<(), CheckStop> {
        match self.tree.production(statement)? {
            Production::LetStmt => self.validate_requires_let(entry, statement),
            Production::CheckStmt => {
                let expression = self
                    .tree
                    .first_child_with(statement, Production::Expr)?
                    .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
                self.validate_requires_condition(entry, expression)
            }
            _ => Err(SemanticCompilerFailure::InvalidCanonicalTree.into()),
        }
    }

    /// Validates one clause `let` against the admitted [FN-8] subset.
    ///
    /// A clause `let` carries no written mode or type, so neither is read
    /// here. [FN-8] fixes the mode itself — "each let introduces a fresh
    /// clause-local own copy value visible to later clause statements" — and
    /// the type is derived from the initializer by the ordinary [TYPE-5]
    /// derivation that a function-body `let` uses, in `check_statement`
    /// immediately after this pass. What remains written, and what this pass
    /// examines, is the right-hand side: an ordinary initializer whose
    /// expression is a call to an admitted operation-table row.
    fn validate_requires_let(&self, entry: NodeId, node: NodeId) -> Result<(), CheckStop> {
        let rhs = self
            .tree
            .first_child_with(node, Production::OrdinaryLetRhs)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expression = self
            .tree
            .first_child_with(rhs, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        // A clause `let` initializer is a computation, so only [FN-8]'s two
        // operation spellings are admitted. A bare atom is not a computation,
        // which is what keeps `let candidate = x;` rejected.
        if self.validate_requires_computation(entry, expression)? {
            return Ok(());
        }
        self.invalid_requires(entry)
    }

    fn validate_requires_condition(
        &self,
        entry: NodeId,
        expression: NodeId,
    ) -> Result<(), CheckStop> {
        if self.validate_requires_computation(entry, expression)? {
            return Ok(());
        }
        // [FN-8] admits one further shape here that a clause `let` does not:
        // "the final check condition is either a Bool clause atom or one such
        // operation returning Bool".
        let Some(atom) = self.tree.first_child_with(expression, Production::Atom)? else {
            return self.invalid_requires(entry);
        };
        self.validate_requires_atom(entry, atom)
    }

    /// Validates a clause computation, reporting whether the expression was
    /// one of the two spellings [FN-8] admits for it.
    ///
    /// [FN-8] requires "an ANF [GRAM-9] call to, or infix spelling of, a
    /// non-trapping, total operation-table row with effect `pure`", and
    /// [GRAM-5] gives those two spellings distinct `expr` shapes. `Ok(false)`
    /// means the expression is neither, leaving each caller to say whether
    /// its position admits a bare atom.
    fn validate_requires_computation(
        &self,
        entry: NodeId,
        expression: NodeId,
    ) -> Result<bool, CheckStop> {
        if let Some(tail) = self.tree.first_child_with(expression, Production::InfixTail)? {
            self.validate_requires_infix(entry, expression, tail)?;
            return Ok(true);
        }
        if let Some(call) = self.tree.first_child_with(expression, Production::Call)? {
            self.validate_requires_operation(entry, call)?;
            return Ok(true);
        }
        Ok(false)
    }

    /// Validates the infix spelling of a row against the same [FN-8] subset
    /// the named spelling faces.
    ///
    /// The operator token selects the row under [OP-1] (ii), so admission
    /// asks the same `traps` predicate the ordinary checker asks rather than
    /// re-reading the spelling: the bare `+ - * / %` forms carry the trapping
    /// mode and no `.trap` suffix, so the named-spelling filter in
    /// `validate_requires_operation` cannot see them. Both operands are
    /// clause atoms and both are validated; [GRAM-9] admits exactly one
    /// operation per expression, so there is no deeper operand to reach.
    fn validate_requires_infix(
        &self,
        entry: NodeId,
        expression: NodeId,
        tail: NodeId,
    ) -> Result<(), CheckStop> {
        let operator = self
            .tree
            .first_child_with(tail, Production::InfixOp)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.infix_operation(operator)?.traps() {
            return self.invalid_requires(entry);
        }
        let left = self
            .tree
            .first_child_with(expression, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.validate_requires_atom(entry, left)?;
        let right = self
            .tree
            .first_child_with(tail, Production::Atom)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        self.validate_requires_atom(entry, right)
    }

    fn validate_requires_operation(&self, entry: NodeId, call: NodeId) -> Result<(), CheckStop> {
        let callee = self
            .tree
            .first_child_with(call, Production::Callee)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let callee_path = self.tree.path(callee)?;
        let usage = self
            .resolved
            .lexical_uses()
            .iter()
            .find(|usage| {
                usage.origin().node() == callee_path
                    && matches!(
                        usage.role(),
                        LexicalUseRole::IdentifierCallee | LexicalUseRole::OperationCallee
                    )
            })
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        let ResolvedTarget::Operation(operation) = usage.target() else {
            if matches!(
                usage.target(),
                ResolvedTarget::Source {
                    class: DeclarationClass::Function,
                    ..
                }
            ) {
                return self.invalid_requires(entry);
            }
            return Err(SemanticCompilerFailure::InvalidResolution.into());
        };
        let spelling = crate::operation_family_spelling(operation)
            .ok_or(SemanticCompilerFailure::InvalidResolution)?;
        if spelling.ends_with(".trap") || matches!(spelling, "buffer_new" | "box_new" | "arena_new")
        {
            return self.invalid_requires(entry);
        }
        if let Some(arguments) = self.tree.first_child_with(call, Production::AtomList)? {
            for atom in self.tree.children_with(arguments, Production::Atom)? {
                self.validate_requires_atom(entry, atom)?;
            }
        }
        Ok(())
    }

    fn validate_requires_atom(&self, entry: NodeId, atom: NodeId) -> Result<(), CheckStop> {
        if self.has_fixed(atom, FixedTerminal::Move)?
            || self
                .tree
                .first_child_with(atom, Production::BorrowExpr)?
                .is_some()
        {
            return self.invalid_requires(entry);
        }
        if let Some(place) = self.tree.first_child_with(atom, Production::Place)? {
            return self.validate_requires_place(entry, place);
        }
        if self
            .tree
            .direct_token_with(atom, crate::TerminalPredicate::Literal)?
            .is_some()
        {
            return Ok(());
        }
        Err(SemanticCompilerFailure::InvalidCanonicalTree.into())
    }

    fn validate_requires_place(&self, entry: NodeId, place: NodeId) -> Result<(), CheckStop> {
        let pbase = self
            .tree
            .first_child_with(place, Production::Pbase)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        for suffix in self.tree.children_with(place, Production::Psuffix)? {
            if self.subscript_offset(suffix)?.is_some() {
                return self.invalid_requires(entry);
            }
        }
        if self.has_fixed(pbase, FixedTerminal::Deref)? {
            let nested = self
                .tree
                .first_child_with(pbase, Production::Place)?
                .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
            self.validate_requires_place(entry, nested)?;
        }
        Ok(())
    }

    fn invalid_requires<T>(&self, node: NodeId) -> Result<T, CheckStop> {
        self.issue_node(SemanticRule::Fn8, node, SemanticIssueKind::InvalidRequires)
    }
}
