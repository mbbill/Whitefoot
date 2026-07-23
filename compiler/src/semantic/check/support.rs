use crate::syntax::NodeId;
use crate::syntax::terminal::{FixedTerminal, TerminalPredicate};
use crate::{
    DeclarationRole, DeferredUseRole, DependentDeclarationRole, LexicalUseRole,
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
        let path = self.tree.path(node)?;
        self.resolved
            .lexical_uses()
            .iter()
            .find(|usage| usage.role() == role && usage.origin().node() == path)
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
            (Ok(path), Ok(coordinate)) => CheckStop::Issue(SemanticIssue {
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
        Err(CheckStop::Issue(SemanticIssue {
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
