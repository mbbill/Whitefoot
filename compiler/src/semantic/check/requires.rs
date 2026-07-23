use std::collections::HashMap;

use crate::syntax::NodeId;
use crate::syntax::terminal::FixedTerminal;
use crate::{
    DeclarationClass, DeclarationId, LexicalUseRole, Production, ResolvedTarget,
    SemanticCompilerFailure, SemanticIssueKind, SemanticRule,
};

use super::super::model::{CheckedMode, CheckedStatement};
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

    fn validate_requires_let(&self, entry: NodeId, node: NodeId) -> Result<(), CheckStop> {
        let mode = self
            .tree
            .first_child_with(node, Production::Mode)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if self.parse_mode(mode)? != CheckedMode::Own {
            return self.invalid_requires(entry);
        }
        let ty = self
            .tree
            .first_child_with(node, Production::Type)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        if !self.is_copy_type(self.parse_type(ty)?)? {
            return self.invalid_requires(entry);
        }
        let rhs = self
            .tree
            .first_child_with(node, Production::OrdinaryLetRhs)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let expression = self
            .tree
            .first_child_with(rhs, Production::Expr)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        let Some(call) = self.tree.first_child_with(expression, Production::Call)? else {
            return self.invalid_requires(entry);
        };
        self.validate_requires_operation(entry, call)
    }

    fn validate_requires_condition(
        &self,
        entry: NodeId,
        expression: NodeId,
    ) -> Result<(), CheckStop> {
        if let Some(call) = self.tree.first_child_with(expression, Production::Call)? {
            return self.validate_requires_operation(entry, call);
        }
        let Some(atom) = self.tree.first_child_with(expression, Production::Atom)? else {
            return self.invalid_requires(entry);
        };
        self.validate_requires_atom(entry, atom)
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
        if self.has_fixed(pbase, FixedTerminal::Index)? {
            return self.invalid_requires(entry);
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
