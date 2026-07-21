//! General bounded nullable, FIRST₂, FOLLOW₂, SELECT₂, and conflict analysis.

use std::collections::{BTreeMap, BTreeSet};

use crate::ebnf::NodeKind;
use crate::grammar::{Grammar, reachable_productions};
use crate::lexical::Lexical;
use crate::terminal::{
    Context as PredicateContext, Predicate, Word, accepts, fixed_predicates, intersection,
    predicate_universe, source_lookahead,
};
use crate::wire::{Failure, Limits, Work};

type Words = BTreeSet<Word>;

#[derive(Clone, Debug)]
pub(crate) struct Decision {
    pub(crate) lhs: String,
    pub(crate) path: String,
    pub(crate) kind: &'static str,
    pub(crate) arm: usize,
    pub(crate) word: Word,
}

#[derive(Clone, Debug)]
pub(crate) struct Conflict {
    pub(crate) lhs: String,
    pub(crate) path: String,
    pub(crate) kind: &'static str,
    pub(crate) left_arm: usize,
    pub(crate) right_arm: usize,
    pub(crate) witness_tokens: Vec<Vec<u8>>,
    pub(crate) left_word: Word,
    pub(crate) right_word: Word,
}

#[derive(Clone, Debug)]
pub(crate) struct Intersection {
    pub(crate) left: Predicate,
    pub(crate) right: Predicate,
    pub(crate) witness: Vec<u8>,
}

pub(crate) struct Analysis {
    pub(crate) nullable: BTreeMap<String, bool>,
    pub(crate) first: BTreeMap<String, Words>,
    pub(crate) follow: BTreeMap<String, Words>,
    pub(crate) decisions: Vec<Decision>,
    pub(crate) conflicts: Vec<Conflict>,
    pub(crate) intersections: Vec<Intersection>,
}

struct Bounds<'a, 'b> {
    limits: &'a Limits,
    work: &'b mut Work,
    words: usize,
    product_states: usize,
}

impl Bounds<'_, '_> {
    fn insert(&mut self, set: &mut Words, word: Word) -> Result<bool, Failure> {
        self.work.spend(1)?;
        if set.contains(&word) {
            return Ok(false);
        }
        self.words = self
            .words
            .checked_add(1)
            .ok_or_else(|| Failure::resource("static-max-lookahead-words"))?;
        if self.words > self.limits.static_max_lookahead_words {
            return Err(Failure::resource("static-max-lookahead-words"));
        }
        Ok(set.insert(word))
    }

    fn product(&mut self) -> Result<(), Failure> {
        self.work.spend(1)?;
        self.product_states = self
            .product_states
            .checked_add(1)
            .ok_or_else(|| Failure::resource("static-max-product-states"))?;
        if self.product_states > self.limits.static_max_product_states {
            return Err(Failure::resource("static-max-product-states"));
        }
        Ok(())
    }
}

fn epsilon() -> Words {
    BTreeSet::from([Word(Vec::new())])
}

fn concat(left: &Words, right: &Words, bounds: &mut Bounds<'_, '_>) -> Result<Words, Failure> {
    let mut output = Words::new();
    for lhs in left {
        for rhs in right {
            bounds.product()?;
            let mut values = Vec::new();
            values
                .try_reserve_exact(2)
                .map_err(|_| Failure::allocation())?;
            values.extend(lhs.0.iter().take(2).cloned());
            if values.len() < 2 {
                values.extend(rhs.0.iter().take(2 - values.len()).cloned());
            }
            bounds.insert(&mut output, Word(values))?;
        }
    }
    Ok(output)
}

fn union(target: &mut Words, source: Words, bounds: &mut Bounds<'_, '_>) -> Result<bool, Failure> {
    let mut changed = false;
    for word in source {
        changed |= bounds.insert(target, word)?;
    }
    Ok(changed)
}

fn repeat(body: &Words, bounds: &mut Bounds<'_, '_>) -> Result<Words, Failure> {
    let mut output = epsilon();
    loop {
        let extension = concat(&output, body, bounds)?;
        if !union(&mut output, extension, bounds)? {
            return Ok(output);
        }
    }
}

fn nullable_node(
    grammar: &Grammar<'_>,
    node: usize,
    nullable: &BTreeMap<String, bool>,
) -> Result<bool, Failure> {
    let value = grammar
        .nodes
        .get(node)
        .ok_or_else(|| Failure::internal("nullable-node"))?;
    match value.kind {
        NodeKind::Ref => {
            let name = value.value.expect("ref value");
            Ok(grammar.symbols.contains_key(name) && nullable.get(name).copied().unwrap_or(false))
        }
        NodeKind::Fixed | NodeKind::Pattern => Ok(false),
        NodeKind::Sequence => {
            for child in &value.children {
                if !nullable_node(grammar, *child, nullable)? {
                    return Ok(false);
                }
            }
            Ok(true)
        }
        NodeKind::Choice => {
            for child in &value.children {
                if nullable_node(grammar, *child, nullable)? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        NodeKind::Group | NodeKind::Repeat1 => nullable_node(grammar, value.children[0], nullable),
        NodeKind::Optional | NodeKind::Repeat0 => Ok(true),
    }
}

fn nullable(grammar: &Grammar<'_>, work: &mut Work) -> Result<BTreeMap<String, bool>, Failure> {
    let mut result = grammar
        .productions
        .iter()
        .map(|production| (production.lhs.to_owned(), false))
        .collect::<BTreeMap<_, _>>();
    loop {
        let mut changed = false;
        for production in &grammar.productions {
            work.spend(1)?;
            if !result[production.lhs] && nullable_node(grammar, production.root, &result)? {
                result.insert(production.lhs.to_owned(), true);
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    for node in &grammar.nodes {
        if matches!(node.kind, NodeKind::Repeat0 | NodeKind::Repeat1)
            && nullable_node(grammar, node.children[0], &result)?
        {
            return Err(Failure::extraction("nullable-repetition-body"));
        }
    }
    Ok(result)
}

fn node_prefix(
    grammar: &Grammar<'_>,
    node: usize,
    symbols: &BTreeMap<String, Words>,
    bounds: &mut Bounds<'_, '_>,
) -> Result<Words, Failure> {
    let value = grammar
        .nodes
        .get(node)
        .ok_or_else(|| Failure::internal("first-node"))?;
    match value.kind {
        NodeKind::Ref => {
            let name = value.value.expect("ref value");
            if grammar.symbols.contains_key(name) {
                Ok(symbols.get(name).cloned().unwrap_or_default())
            } else {
                Ok(BTreeSet::from([Word(vec![Predicate::Lex(
                    name.to_owned(),
                )])]))
            }
        }
        NodeKind::Fixed => Ok(BTreeSet::from([Word(fixed_predicates(
            value.value.expect("fixed value"),
        ))])),
        NodeKind::Pattern => Ok(BTreeSet::from([Word(vec![Predicate::Digits])])),
        NodeKind::Sequence => {
            let mut output = epsilon();
            for child in &value.children {
                let child_words = node_prefix(grammar, *child, symbols, bounds)?;
                output = concat(&output, &child_words, bounds)?;
            }
            Ok(output)
        }
        NodeKind::Choice => {
            let mut output = Words::new();
            for child in &value.children {
                let child_words = node_prefix(grammar, *child, symbols, bounds)?;
                union(&mut output, child_words, bounds)?;
            }
            Ok(output)
        }
        NodeKind::Group => node_prefix(grammar, value.children[0], symbols, bounds),
        NodeKind::Optional => {
            let mut output = epsilon();
            let child = node_prefix(grammar, value.children[0], symbols, bounds)?;
            union(&mut output, child, bounds)?;
            Ok(output)
        }
        NodeKind::Repeat0 => {
            let child = node_prefix(grammar, value.children[0], symbols, bounds)?;
            repeat(&child, bounds)
        }
        NodeKind::Repeat1 => {
            let child = node_prefix(grammar, value.children[0], symbols, bounds)?;
            let tail = repeat(&child, bounds)?;
            concat(&child, &tail, bounds)
        }
    }
}

fn first(
    grammar: &Grammar<'_>,
    bounds: &mut Bounds<'_, '_>,
) -> Result<BTreeMap<String, Words>, Failure> {
    let mut result = grammar
        .productions
        .iter()
        .map(|production| (production.lhs.to_owned(), Words::new()))
        .collect::<BTreeMap<_, _>>();
    loop {
        let mut changed = false;
        for production in &grammar.productions {
            let words = node_prefix(grammar, production.root, &result, bounds)?;
            changed |= union(
                result
                    .get_mut(production.lhs)
                    .ok_or_else(|| Failure::internal("first-symbol"))?,
                words,
                bounds,
            )?;
        }
        if !changed {
            return Ok(result);
        }
    }
}

fn suffix_words(
    grammar: &Grammar<'_>,
    children: &[usize],
    first: &BTreeMap<String, Words>,
    outer: &Words,
    bounds: &mut Bounds<'_, '_>,
) -> Result<Words, Failure> {
    let mut value = epsilon();
    for child in children {
        let words = node_prefix(grammar, *child, first, bounds)?;
        value = concat(&value, &words, bounds)?;
    }
    concat(&value, outer, bounds)
}

fn propagate_follow(
    grammar: &Grammar<'_>,
    node: usize,
    outer: &Words,
    first: &BTreeMap<String, Words>,
    follow: &mut BTreeMap<String, Words>,
    bounds: &mut Bounds<'_, '_>,
) -> Result<bool, Failure> {
    let value = grammar
        .nodes
        .get(node)
        .ok_or_else(|| Failure::internal("follow-node"))?;
    match value.kind {
        NodeKind::Ref => {
            let name = value.value.expect("ref value");
            if let Some(target) = follow.get_mut(name) {
                union(target, outer.clone(), bounds)
            } else {
                Ok(false)
            }
        }
        NodeKind::Fixed | NodeKind::Pattern => Ok(false),
        NodeKind::Sequence => {
            let mut changed = false;
            for (index, child) in value.children.iter().enumerate() {
                let continuation =
                    suffix_words(grammar, &value.children[index + 1..], first, outer, bounds)?;
                changed |= propagate_follow(grammar, *child, &continuation, first, follow, bounds)?;
            }
            Ok(changed)
        }
        NodeKind::Choice => {
            let mut changed = false;
            for child in &value.children {
                changed |= propagate_follow(grammar, *child, outer, first, follow, bounds)?;
            }
            Ok(changed)
        }
        NodeKind::Group | NodeKind::Optional => {
            propagate_follow(grammar, value.children[0], outer, first, follow, bounds)
        }
        NodeKind::Repeat0 | NodeKind::Repeat1 => {
            let body = node_prefix(grammar, value.children[0], first, bounds)?;
            let repeated = repeat(&body, bounds)?;
            let continuation = concat(&repeated, outer, bounds)?;
            propagate_follow(
                grammar,
                value.children[0],
                &continuation,
                first,
                follow,
                bounds,
            )
        }
    }
}

fn follow(
    grammar: &Grammar<'_>,
    first: &BTreeMap<String, Words>,
    bounds: &mut Bounds<'_, '_>,
) -> Result<BTreeMap<String, Words>, Failure> {
    let mut result = grammar
        .productions
        .iter()
        .map(|production| (production.lhs.to_owned(), Words::new()))
        .collect::<BTreeMap<_, _>>();
    let start = result
        .get_mut("program")
        .ok_or_else(|| Failure::extraction("program-start-missing"))?;
    bounds.insert(start, Word(vec![Predicate::End, Predicate::End]))?;
    loop {
        let mut changed = false;
        for production in &grammar.productions {
            let outer = result
                .get(production.lhs)
                .cloned()
                .ok_or_else(|| Failure::internal("follow-symbol"))?;
            changed |=
                propagate_follow(grammar, production.root, &outer, first, &mut result, bounds)?;
        }
        if !changed {
            return Ok(result);
        }
    }
}

fn words_intersect(
    left: &Word,
    right: &Word,
    context: &PredicateContext,
    bounds: &mut Bounds<'_, '_>,
) -> Result<Option<Vec<Vec<u8>>>, Failure> {
    if left.0.len() != right.0.len() {
        return Ok(None);
    }
    let mut witness = Vec::new();
    for (lhs, rhs) in left.0.iter().zip(&right.0) {
        bounds.product()?;
        let Some(token) = intersection(lhs, rhs, context) else {
            return Ok(None);
        };
        witness.try_reserve(1).map_err(|_| Failure::allocation())?;
        witness.push(token);
    }
    Ok(Some(witness))
}

struct DecisionCollector<'a, 'b, 'c> {
    grammar: &'a Grammar<'a>,
    first: &'b BTreeMap<String, Words>,
    context: &'b PredicateContext,
    bounds: &'c mut Bounds<'b, 'c>,
    decisions: Vec<Decision>,
    conflicts: Vec<Conflict>,
}

impl DecisionCollector<'_, '_, '_> {
    fn record(
        &mut self,
        lhs: &str,
        path: &str,
        kind: &'static str,
        arms: Vec<Words>,
    ) -> Result<(), Failure> {
        for (arm, words) in arms.iter().enumerate() {
            for word in words {
                self.decisions
                    .try_reserve(1)
                    .map_err(|_| Failure::allocation())?;
                self.decisions.push(Decision {
                    lhs: lhs.to_owned(),
                    path: path.to_owned(),
                    kind,
                    arm,
                    word: word.clone(),
                });
            }
        }
        for left_arm in 0..arms.len() {
            for right_arm in left_arm + 1..arms.len() {
                for left_word in &arms[left_arm] {
                    for right_word in &arms[right_arm] {
                        if let Some(witness_tokens) =
                            words_intersect(left_word, right_word, self.context, self.bounds)?
                        {
                            self.conflicts
                                .try_reserve(1)
                                .map_err(|_| Failure::allocation())?;
                            self.conflicts.push(Conflict {
                                lhs: lhs.to_owned(),
                                path: path.to_owned(),
                                kind,
                                left_arm,
                                right_arm,
                                witness_tokens,
                                left_word: left_word.clone(),
                                right_word: right_word.clone(),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn walk(&mut self, lhs: &str, node: usize, path: &str, outer: &Words) -> Result<(), Failure> {
        let value = self
            .grammar
            .nodes
            .get(node)
            .ok_or_else(|| Failure::internal("decision-node"))?;
        match value.kind {
            NodeKind::Ref | NodeKind::Fixed | NodeKind::Pattern => {}
            NodeKind::Sequence => {
                for (index, child) in value.children.iter().enumerate() {
                    let continuation = suffix_words(
                        self.grammar,
                        &value.children[index + 1..],
                        self.first,
                        outer,
                        self.bounds,
                    )?;
                    self.walk(lhs, *child, &format!("{path}.{index}"), &continuation)?;
                }
            }
            NodeKind::Choice => {
                let mut arms = Vec::new();
                for child in &value.children {
                    let prefix = node_prefix(self.grammar, *child, self.first, self.bounds)?;
                    arms.push(concat(&prefix, outer, self.bounds)?);
                }
                self.record(lhs, path, "choice", arms)?;
                for (index, child) in value.children.iter().enumerate() {
                    self.walk(lhs, *child, &format!("{path}.{index}"), outer)?;
                }
            }
            NodeKind::Group => {
                self.walk(lhs, value.children[0], &format!("{path}.0"), outer)?;
            }
            NodeKind::Optional => {
                let body = node_prefix(self.grammar, value.children[0], self.first, self.bounds)?;
                let consume = concat(&body, outer, self.bounds)?;
                self.record(lhs, path, "optional", vec![consume, outer.clone()])?;
                self.walk(lhs, value.children[0], &format!("{path}.0"), outer)?;
            }
            NodeKind::Repeat0 | NodeKind::Repeat1 => {
                let body = node_prefix(self.grammar, value.children[0], self.first, self.bounds)?;
                let tail = repeat(&body, self.bounds)?;
                let repeated_outer = concat(&tail, outer, self.bounds)?;
                let consume = concat(&body, &repeated_outer, self.bounds)?;
                let kind = if value.kind == NodeKind::Repeat0 {
                    "repeat0"
                } else {
                    "repeat1"
                };
                self.record(lhs, path, kind, vec![consume, outer.clone()])?;
                self.walk(
                    lhs,
                    value.children[0],
                    &format!("{path}.0"),
                    &repeated_outer,
                )?;
            }
        }
        Ok(())
    }
}

fn collect_intersections(
    grammar: &Grammar<'_>,
    lexical: &[Lexical<'_>],
    context: &PredicateContext,
    bounds: &mut Bounds<'_, '_>,
) -> Result<Vec<Intersection>, Failure> {
    let values = predicate_universe(grammar, lexical)
        .into_iter()
        .collect::<Vec<_>>();
    let mut output = Vec::new();
    for (left_index, left) in values.iter().enumerate() {
        for right in &values[left_index..] {
            bounds.product()?;
            if let Some(witness) = intersection(left, right, context) {
                output.try_reserve(1).map_err(|_| Failure::allocation())?;
                output.push(Intersection {
                    left: left.clone(),
                    right: right.clone(),
                    witness,
                });
            }
        }
    }
    Ok(output)
}

pub(crate) fn analyze(
    grammar: &Grammar<'_>,
    lexical: &[Lexical<'_>],
    limits: &Limits,
    work: &mut Work,
) -> Result<Analysis, Failure> {
    let nullable = nullable(grammar, work)?;
    let mut bounds = Bounds {
        limits,
        work,
        words: 0,
        product_states: 0,
    };
    let first = first(grammar, &mut bounds)?;
    let follow = follow(grammar, &first, &mut bounds)?;
    let context = PredicateContext::new(grammar, lexical);
    let mut collector = DecisionCollector {
        grammar,
        first: &first,
        context: &context,
        bounds: &mut bounds,
        decisions: Vec::new(),
        conflicts: Vec::new(),
    };
    for production in &grammar.productions {
        let outer = follow
            .get(production.lhs)
            .ok_or_else(|| Failure::internal("decision-follow"))?;
        collector.walk(production.lhs, production.root, "0", outer)?;
    }
    let intersections = collect_intersections(grammar, lexical, &context, collector.bounds)?;
    let decisions = core::mem::take(&mut collector.decisions);
    let conflicts = core::mem::take(&mut collector.conflicts);
    drop(collector);
    Ok(Analysis {
        nullable,
        first,
        follow,
        decisions,
        conflicts,
        intersections,
    })
}

pub(crate) fn matching_conflicts(
    source: &[u8],
    start: &str,
    grammar: &Grammar<'_>,
    lexical: &[Lexical<'_>],
    analysis: &Analysis,
    work: &mut Work,
) -> Result<usize, Failure> {
    let Some(tokens) = source_lookahead(source, grammar, lexical, work)? else {
        return Ok(0);
    };
    let context = PredicateContext::new(grammar, lexical);
    let starts = analysis
        .first
        .get(start)
        .ok_or_else(|| Failure::input("case-start-symbol"))?;
    let start_matches = starts.iter().any(|word| {
        let mut standalone = word.clone();
        while standalone.0.len() < 2 {
            standalone.0.push(Predicate::End);
        }
        standalone
            .0
            .iter()
            .zip(&tokens)
            .all(|(predicate, token)| accepts(predicate, token, &context))
    });
    if !start_matches {
        return Ok(0);
    }
    let reachable = reachable_productions(grammar, start, work)?;
    Ok(analysis
        .conflicts
        .iter()
        .filter(|conflict| {
            grammar
                .symbols
                .get(conflict.lhs.as_str())
                .is_some_and(|index| reachable[*index])
                && [&conflict.left_word, &conflict.right_word]
                    .iter()
                    .all(|word| {
                        word.0
                            .iter()
                            .zip(&tokens)
                            .all(|(predicate, token)| accepts(predicate, token, &context))
                    })
        })
        .count())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::Predicate;
    use crate::document::Span;
    use crate::ebnf::{Node, NodeKind};
    use crate::grammar::Grammar;

    #[test]
    fn terminal_census_is_not_limited_to_first_two_positions() {
        let leaf = |value| Node {
            kind: NodeKind::Fixed,
            span: Span { start: 0, end: 1 },
            value: Some(value),
            children: Vec::new(),
        };
        let grammar = Grammar {
            nodes: vec![leaf("a"), leaf("b"), leaf("hidden")],
            productions: Vec::new(),
            symbols: BTreeMap::new(),
            surfaces: Vec::new(),
            unclassified_count: 0,
        };
        assert!(
            crate::terminal::predicate_universe(&grammar, &[])
                .contains(&Predicate::Fixed("hidden".to_owned()))
        );
    }
}
