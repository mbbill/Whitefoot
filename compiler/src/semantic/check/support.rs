use crate::syntax::NodeId;
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};
use crate::{
    DeclarationRole, DeferredUseRole, DependentDeclarationRole, LexicalUseRole, Production,
    SemanticCompilerFailure, SemanticIssue, SemanticIssueKind, SemanticLocation, SemanticRule,
    SemanticUnsupported, UnsupportedSemanticFeature,
};

use super::{CheckStop, Checker};

impl<'unit, 'classified, 'lexed, 'source> Checker<'unit, 'classified, 'lexed, 'source> {
    pub(super) fn has_fixed(
        &self,
        node: NodeId,
        terminal: FixedTerminal,
    ) -> Result<bool, CheckStop> {
        Ok(self
            .tree
            .direct_token_with(node, TerminalPredicate::Fixed(terminal))?
            .is_some())
    }

    /// The offset `atom` of a subscript `psuffix`, or `None` for a field
    /// suffix: the two [GRAM-5] `psuffix` alternatives differ exactly in
    /// carrying an offset atom child.
    pub(super) fn subscript_offset(&self, suffix: NodeId) -> Result<Option<NodeId>, CheckStop> {
        Ok(self.tree.first_child_with(suffix, Production::Atom)?)
    }

    /// Position of the last subscript `psuffix` in one place's suffix chain,
    /// if any. The place reads or writes through that subscript; the chain
    /// before it is the subscript's base place [OP-4].
    pub(super) fn last_subscript(&self, suffixes: &[NodeId]) -> Result<Option<usize>, CheckStop> {
        let mut last = None;
        for (position, suffix) in suffixes.iter().enumerate() {
            if self.subscript_offset(*suffix)?.is_some() {
                last = Some(position);
            }
        }
        Ok(last)
    }

    pub(super) fn identifier(&self, node: NodeId) -> Result<String, CheckStop> {
        let terminal = self
            .tree
            .direct_token_with(node, TerminalPredicate::Identifier)?
            .ok_or(SemanticCompilerFailure::InvalidCanonicalTree)?;
        std::str::from_utf8(self.tree.token_bytes(terminal)?)
            .map(str::to_owned)
            .map_err(|_| SemanticCompilerFailure::InvalidSourceEncoding.into())
    }

    pub(super) fn declaration_at(
        &self,
        node: NodeId,
        role: DeclarationRole,
    ) -> Result<&crate::DeclarationRecord, CheckStop> {
        let path = self.tree.path(node)?;
        self.resolved
            .declarations()
            .iter()
            .find(|declaration| declaration.role() == role && declaration.origin().node() == path)
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    pub(super) fn use_at(
        &self,
        node: NodeId,
        role: LexicalUseRole,
    ) -> Result<&crate::LexicalUseRecord, CheckStop> {
        self.use_at_roles(node, &[role])
    }

    /// Every lexical use of one role at one node, ordered by role ordinal
    /// (source order). The single-use reader above suits the carriers with
    /// one name; a candidate-grammar `const` operation carries two [CONST-1].
    pub(super) fn uses_at_ordered(
        &self,
        node: NodeId,
        role: LexicalUseRole,
    ) -> Result<Vec<&crate::LexicalUseRecord>, CheckStop> {
        let path = self.tree.path(node)?;
        if let Some(context) = self.active_postcondition.get() {
            let record = self
                .resolved
                .postconditions()
                .get(context.record)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            let mut uses = record
                .provisional_uses
                .iter()
                .filter(|usage| usage.role() == role && usage.origin().node() == path)
                .collect::<Vec<_>>();
            if !uses.is_empty() {
                uses.sort_by_key(|usage| usage.origin().role_ordinal());
                return Ok(uses);
            }
        }
        let mut uses = self
            .resolved
            .lexical_uses()
            .iter()
            .filter(|usage| usage.role() == role && usage.origin().node() == path)
            .collect::<Vec<_>>();
        uses.sort_by_key(|usage| usage.origin().role_ordinal());
        Ok(uses)
    }

    pub(super) fn use_at_roles(
        &self,
        node: NodeId,
        roles: &[LexicalUseRole],
    ) -> Result<&crate::LexicalUseRecord, CheckStop> {
        let path = self.tree.path(node)?;
        if let Some(context) = self.active_postcondition.get() {
            let record = self
                .resolved
                .postconditions()
                .get(context.record)
                .ok_or(SemanticCompilerFailure::InvalidResolution)?;
            if let Some(usage) = record
                .provisional_uses
                .iter()
                .find(|usage| roles.contains(&usage.role()) && usage.origin().node() == path)
            {
                return Ok(usage);
            }
        }
        self.resolved
            .lexical_uses()
            .iter()
            .find(|usage| roles.contains(&usage.role()) && usage.origin().node() == path)
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    pub(super) fn dependent_declaration_at(
        &self,
        node: NodeId,
        role: DependentDeclarationRole,
    ) -> Result<&crate::DependentDeclarationRecord, CheckStop> {
        let path = self.tree.path(node)?;
        self.resolved
            .dependent_declarations()
            .iter()
            .find(|declaration| declaration.role() == role && declaration.origin().node() == path)
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    pub(super) fn deferred_use_at(
        &self,
        node: NodeId,
        role: DeferredUseRole,
    ) -> Result<&crate::DeferredUseRecord, CheckStop> {
        let path = self.tree.path(node)?;
        self.resolved
            .deferred_uses()
            .iter()
            .find(|usage| usage.role() == role && usage.origin().node() == path)
            .ok_or(SemanticCompilerFailure::InvalidResolution.into())
    }

    pub(super) fn issue_value(
        &self,
        rule: SemanticRule,
        node: NodeId,
        kind: SemanticIssueKind,
    ) -> CheckStop {
        match (self.tree.path(node), self.tree.coordinate(node)) {
            (Ok(path), Ok(coordinate)) => CheckStop::source_issue(SemanticIssue {
                rule,
                location: SemanticLocation::SourceNode(path.clone(), coordinate),
                kind,
            }),
            _ => CheckStop::Compiler(SemanticCompilerFailure::InvalidCanonicalTree),
        }
    }

    pub(super) fn issue_node<ResultValue>(
        &self,
        rule: SemanticRule,
        node: NodeId,
        kind: SemanticIssueKind,
    ) -> Result<ResultValue, CheckStop> {
        Err(self.issue_value(rule, node, kind))
    }

    pub(super) fn issue_at<ResultValue>(
        &self,
        rule: SemanticRule,
        node: NodeId,
        coordinate: crate::SyntaxCoordinate,
        kind: SemanticIssueKind,
    ) -> Result<ResultValue, CheckStop> {
        let path = self.tree.path(node)?.clone();
        Err(CheckStop::source_issue(SemanticIssue {
            rule,
            location: SemanticLocation::SourceNode(path, coordinate),
            kind,
        }))
    }

    pub(super) fn unsupported<ResultValue>(
        &self,
        feature: UnsupportedSemanticFeature,
        node: NodeId,
    ) -> Result<ResultValue, CheckStop> {
        let node = self.tree.path(node)?.clone();
        Err(CheckStop::Unsupported(SemanticUnsupported {
            feature,
            node,
        }))
    }
}
