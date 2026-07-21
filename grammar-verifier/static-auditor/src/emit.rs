//! Deterministic common-ledger and static-evidence projection.

use std::collections::BTreeSet;

use crate::document::Document;
use crate::ebnf::{NodeKind, expanded_fixed_lowerwords, fixed_expansion};
use crate::grammar::{Grammar, SurfaceKind};
use crate::hash::{Sha256, hex};
use crate::lexical::{Lexical, extract as extract_lexical};
use crate::ll2::{Analysis, Conflict, analyze as analyze_ll2, matching_conflicts};
use crate::wire::{Failure, Frame, Limits, Work};

pub(crate) struct Report(pub(crate) Vec<u8>);

struct DocumentAnalysis<'a> {
    document: Document<'a>,
    grammar: Grammar<'a>,
    lexical: Vec<Lexical<'a>>,
    static_analysis: Analysis,
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct CommonRecord {
    start: usize,
    rank: usize,
    line: String,
}

struct BoundedCommonRecords {
    records: Vec<CommonRecord>,
    bytes: usize,
    limit: usize,
}

impl BoundedCommonRecords {
    const fn new(limit: usize) -> Self {
        Self {
            records: Vec::new(),
            bytes: 0,
            limit,
        }
    }

    fn push(&mut self, start: usize, rank: usize, line: String) -> Result<(), Failure> {
        let needed = self
            .bytes
            .checked_add(line.len())
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| Failure::resource("max-engine-output-bytes"))?;
        if needed > self.limit {
            return Err(Failure::resource("max-engine-output-bytes"));
        }
        self.records
            .try_reserve(1)
            .map_err(|_| Failure::allocation())?;
        self.records.push(CommonRecord { start, rank, line });
        self.bytes = needed;
        Ok(())
    }
}

struct BoundedLines {
    lines: Vec<String>,
    bytes: usize,
    limit: usize,
}

impl BoundedLines {
    const fn new(limit: usize) -> Self {
        Self {
            lines: Vec::new(),
            bytes: 0,
            limit,
        }
    }

    fn push(&mut self, line: String) -> Result<(), Failure> {
        let needed = self
            .bytes
            .checked_add(line.len())
            .and_then(|value| value.checked_add(1))
            .ok_or_else(|| Failure::resource("max-engine-output-bytes"))?;
        if needed > self.limit {
            return Err(Failure::resource("max-engine-output-bytes"));
        }
        self.lines
            .try_reserve(1)
            .map_err(|_| Failure::allocation())?;
        self.lines.push(line);
        self.bytes = needed;
        Ok(())
    }
}

fn analyze_document<'a>(bytes: &'a [u8], limits: &Limits) -> Result<DocumentAnalysis<'a>, Failure> {
    let mut work = Work::new(limits.static_max_work);
    let document = Document::parse(bytes, limits, &mut work)?;
    let mut grammar = crate::grammar::extract(&document, limits, &mut work)?;
    let lexical = extract_lexical(&document, &mut grammar, limits, &mut work)?;
    crate::grammar::require_program_reachability(&grammar, &mut work)?;
    let static_analysis = analyze_ll2(&grammar, &lexical, limits, &mut work)?;
    Ok(DocumentAnalysis {
        document,
        grammar,
        lexical,
        static_analysis,
    })
}

fn node_records(
    doc: &str,
    lhs: &str,
    grammar: &Grammar<'_>,
    node_id: usize,
    path: &str,
    records: &mut BoundedCommonRecords,
) -> Result<(), Failure> {
    let node = grammar
        .nodes
        .get(node_id)
        .ok_or_else(|| Failure::internal("report-node"))?;
    let value = node
        .value
        .map_or_else(|| "-".to_owned(), |item| hex(item.as_bytes()));
    records.push(
        node.span.start,
        3,
        format!(
            "NODE\t{doc}\t{}\t{path}\t{}\t{}\t{}\t{value}",
            hex(lhs.as_bytes()),
            node.kind.name(),
            node.span.start,
            node.span.end
        ),
    )?;
    if node.kind == NodeKind::Fixed {
        let spelling = node.value.expect("fixed value");
        let descriptor = fixed_expansion(spelling);
        records.push(
            node.span.start,
            5,
            format!(
                "FIXED\t{doc}\t{}\t{path}\t{}\t{}\t{}\t{}",
                hex(lhs.as_bytes()),
                node.span.start,
                node.span.end,
                hex(spelling.as_bytes()),
                hex(descriptor.as_bytes())
            ),
        )?;
    } else if node.kind == NodeKind::Ref {
        let name = node.value.expect("ref value");
        records.push(
            node.span.start,
            6,
            format!(
                "REF\t{doc}\t{}\t{path}\t{}\t{}\t{}",
                hex(lhs.as_bytes()),
                node.span.start,
                node.span.end,
                hex(name.as_bytes())
            ),
        )?;
    }
    for (index, child) in node.children.iter().enumerate() {
        node_records(
            doc,
            lhs,
            grammar,
            *child,
            &format!("{path}.{index}"),
            records,
        )?;
    }
    Ok(())
}

fn common_records(
    doc: &str,
    analysis: &DocumentAnalysis<'_>,
    limit: usize,
) -> Result<Vec<CommonRecord>, Failure> {
    let mut records = BoundedCommonRecords::new(limit);
    for rule in &analysis.document.rules {
        records.push(
            rule.span.start,
            0,
            format!(
                "RULE\t{doc}\t{}\t{}\t{}",
                hex(rule.id.as_bytes()),
                rule.span.start,
                rule.span.end
            ),
        )?;
    }
    for surface in &analysis.grammar.surfaces {
        records.push(
            surface.span.start,
            1,
            format!(
                "SURFACE\t{doc}\t{}\t{}\t{}\t{}",
                surface.kind.name(),
                surface.span.start,
                surface.span.end,
                hex(surface.owner.as_bytes())
            ),
        )?;
    }
    for production in &analysis.grammar.productions {
        records.push(
            production.span.start,
            2,
            format!(
                "PROD\t{doc}\t{}\t{}\t{}\t{}\t{}\t{}",
                hex(production.owner.as_bytes()),
                hex(production.lhs.as_bytes()),
                production.span.start,
                production.span.end,
                production.rhs.start,
                production.rhs.end
            ),
        )?;
        node_records(
            doc,
            production.lhs,
            &analysis.grammar,
            production.root,
            "0",
            &mut records,
        )?;
    }
    for lexical in &analysis.lexical {
        records.push(
            lexical.span.start,
            4,
            format!(
                "LEX\t{doc}\t{}\t{}\t{}\t{}\t{}\t{}",
                hex(lexical.owner.as_bytes()),
                hex(lexical.name.as_bytes()),
                lexical.kind.name(),
                lexical.span.start,
                lexical.span.end,
                hex(lexical.predicate.as_bytes())
            ),
        )?;
    }
    let assignment_count = analysis
        .grammar
        .surfaces
        .iter()
        .filter(|surface| surface.kind == SurfaceKind::Assignment)
        .count();
    let fence_count = analysis
        .grammar
        .surfaces
        .iter()
        .filter(|surface| surface.kind == SurfaceKind::GrammarFence)
        .count();
    let inline_count = analysis
        .grammar
        .surfaces
        .iter()
        .filter(|surface| surface.kind == SurfaceKind::GrammarInline)
        .count();
    let lexical_count = analysis
        .grammar
        .surfaces
        .iter()
        .filter(|surface| surface.kind == SurfaceKind::LexicalCue)
        .count();
    records.push(
        usize::MAX,
        7,
        format!(
            "COVERAGE\t{doc}\t{assignment_count}\t{fence_count}\t{inline_count}\t{lexical_count}\t{}",
            analysis.grammar.unclassified_count
        ),
    )?;
    records.records.sort_unstable();
    Ok(records.records)
}

fn tagged_witness(tokens: &[Vec<u8>]) -> String {
    let inner = tokens
        .iter()
        .map(|token| {
            if token.is_empty() {
                "-".to_owned()
            } else {
                hex(token)
            }
        })
        .collect::<Vec<_>>()
        .join(",");
    hex(inner.as_bytes())
}

fn intersection_identity(item: &crate::ll2::Intersection) -> String {
    format!(
        "{}\t{}",
        hex(item.left.descriptor().as_bytes()),
        hex(item.right.descriptor().as_bytes())
    )
}

fn intersection_payload(item: &crate::ll2::Intersection) -> String {
    format!(
        "{}\t{}",
        intersection_identity(item),
        if item.witness.is_empty() {
            "-".to_owned()
        } else {
            hex(&item.witness)
        }
    )
}

fn conflict_payload(conflict: &Conflict) -> String {
    format!(
        "{}\t{}\t{}\t{}\t{}\t{}\t{}",
        hex(conflict.lhs.as_bytes()),
        conflict.path,
        conflict.kind,
        conflict.left_arm,
        conflict.right_arm,
        hex(conflict.left_word.descriptor().as_bytes()),
        hex(conflict.right_word.descriptor().as_bytes()),
    )
}

fn conflict_record_payload(conflict: &Conflict) -> String {
    format!(
        "{}\t{}",
        conflict_payload(conflict),
        tagged_witness(&conflict.witness_tokens)
    )
}

fn static_records(
    doc: &str,
    analysis: &DocumentAnalysis<'_>,
    output: &mut BoundedLines,
) -> Result<(), Failure> {
    for (lhs, value) in &analysis.static_analysis.nullable {
        output.push(format!(
            "STATIC-NULLABLE\t{doc}\t{}\t{}",
            hex(lhs.as_bytes()),
            usize::from(*value)
        ))?;
    }
    for (lhs, words) in &analysis.static_analysis.first {
        for word in words {
            output.push(format!(
                "STATIC-FIRST\t{doc}\t{}\t{}",
                hex(lhs.as_bytes()),
                hex(word.descriptor().as_bytes())
            ))?;
        }
    }
    for (lhs, words) in &analysis.static_analysis.follow {
        for word in words {
            output.push(format!(
                "STATIC-FOLLOW\t{doc}\t{}\t{}",
                hex(lhs.as_bytes()),
                hex(word.descriptor().as_bytes())
            ))?;
        }
    }
    for item in &analysis.static_analysis.intersections {
        output.push(format!(
            "STATIC-INTERSECTION\t{doc}\t{}",
            intersection_payload(item)
        ))?;
    }
    for item in &analysis.static_analysis.decisions {
        output.push(format!(
            "STATIC-DECISION\t{doc}\t{}\t{}\t{}\t{}\t{}",
            hex(item.lhs.as_bytes()),
            item.path,
            item.kind,
            item.arm,
            hex(item.word.descriptor().as_bytes())
        ))?;
    }
    for item in &analysis.static_analysis.conflicts {
        output.push(format!(
            "STATIC-CONFLICT\t{doc}\t{}",
            conflict_record_payload(item)
        ))?;
    }
    Ok(())
}

fn delta_records(
    current: &Analysis,
    proposal: &Analysis,
    output: &mut BoundedLines,
) -> Result<(), Failure> {
    fn keys(analysis: &Analysis) -> BTreeSet<(&'static str, String)> {
        analysis
            .intersections
            .iter()
            .map(|item| ("intersection", hex(intersection_identity(item).as_bytes())))
            .chain(
                analysis
                    .conflicts
                    .iter()
                    .map(|item| ("conflict", hex(conflict_payload(item).as_bytes()))),
            )
            .collect()
    }
    let current = keys(current);
    let proposal = keys(proposal);
    for item in current.union(&proposal) {
        let (kind, key) = item;
        let status = match (current.contains(item), proposal.contains(item)) {
            (true, true) => "retained",
            (true, false) => "removed",
            (false, true) => "introduced",
            (false, false) => unreachable!(),
        };
        output.push(format!("STATIC-DELTA\t{kind}\t{status}\t{key}"))?;
    }
    Ok(())
}

fn transition_record(
    frame: &Frame<'_>,
    current: &[(String, usize)],
    proposal: &[(String, usize)],
) -> Result<String, Failure> {
    let case = frame
        .cases
        .iter()
        .find(|case| case.id == "deref-x")
        .ok_or_else(|| Failure::input("transition-case"))?;
    if case.start != "expr" || case.source != b"deref(x)" {
        return Err(Failure::input("transition-case"));
    }
    let count = |observations: &[(String, usize)]| {
        observations
            .iter()
            .find(|(id, _)| id == "deref-x")
            .map(|(_, count)| *count)
            .ok_or_else(|| Failure::internal("transition-observation"))
    };
    let current_count = count(current)?;
    let proposal_count = count(proposal)?;
    let status = if current_count == 1 && proposal_count == 0 {
        "removes-call-through-ident"
    } else {
        "does-not-remove-call-through-ident"
    };
    Ok(format!(
        "STATIC-TRANSITION\tfixed-ident-partition\t{current_count}\t{proposal_count}\t{status}\t{}",
        hex(&case.source)
    ))
}

fn case_records(
    doc: &str,
    frame: &Frame<'_>,
    analysis: &DocumentAnalysis<'_>,
    output: &mut BoundedLines,
    work: &mut Work,
) -> Result<Vec<(String, usize)>, Failure> {
    let mut observations = Vec::new();
    observations
        .try_reserve_exact(frame.cases.len())
        .map_err(|_| Failure::allocation())?;
    for case in &frame.cases {
        if !analysis.grammar.symbols.contains_key(case.start.as_str()) {
            return Err(Failure::input("case-start-symbol"));
        }
        let count = matching_conflicts(
            &case.source,
            &case.start,
            &analysis.grammar,
            &analysis.lexical,
            &analysis.static_analysis,
            work,
        )?;
        output.push(format!(
            "STATIC-CASE\t{doc}\t{}\t{}\t{}\t{count}",
            case.id,
            hex(case.start.as_bytes()),
            hex(&case.source)
        ))?;
        observations.push((case.id.clone(), count));
    }
    Ok(observations)
}

fn domain_records(
    doc: &str,
    frame: &Frame<'_>,
    analysis: &DocumentAnalysis<'_>,
    output: &mut BoundedLines,
) -> Result<(), Failure> {
    let words = expanded_fixed_lowerwords(&analysis.grammar.nodes);
    let total = words
        .len()
        .checked_mul(frame.domains.len())
        .ok_or_else(|| Failure::resource("max-generated-streams"))?;
    if total > frame.limits.max_generated_streams {
        return Err(Failure::resource("max-generated-streams"));
    }
    for domain in &frame.domains {
        if !analysis.grammar.symbols.contains_key(domain.start.as_str()) {
            return Err(Failure::input("domain-start-symbol"));
        }
        let mut digest = Sha256::new();
        for word in &words {
            let length = word
                .len()
                .checked_add(domain.argument.len())
                .and_then(|value| value.checked_add(2))
                .ok_or_else(|| Failure::resource("generated-stream-length"))?;
            digest.update(&(length as u64).to_be_bytes());
            digest.update(word.as_bytes());
            digest.update(b"(");
            digest.update(&domain.argument);
            digest.update(b")");
        }
        output.push(format!(
            "STATIC-DOMAIN\t{doc}\t{}\t{}\t{}\t{}\t{}",
            hex(domain.id.as_bytes()),
            hex(domain.start.as_bytes()),
            hex(&domain.argument),
            words.len(),
            hex(&digest.finalize())
        ))?;
    }
    Ok(())
}

fn append_line(output: &mut Vec<u8>, line: &str, limit: usize) -> Result<(), Failure> {
    let needed = output
        .len()
        .checked_add(line.len())
        .and_then(|value| value.checked_add(1))
        .ok_or_else(|| Failure::resource("max-engine-output-bytes"))?;
    if needed > limit {
        return Err(Failure::resource("max-engine-output-bytes"));
    }
    output
        .try_reserve(line.len() + 1)
        .map_err(|_| Failure::allocation())?;
    output.extend_from_slice(line.as_bytes());
    output.push(b'\n');
    Ok(())
}

fn static_rank(line: &str) -> usize {
    [
        "STATIC-NULLABLE\t",
        "STATIC-FIRST\t",
        "STATIC-FOLLOW\t",
        "STATIC-INTERSECTION\t",
        "STATIC-DECISION\t",
        "STATIC-CONFLICT\t",
        "STATIC-DELTA\t",
        "STATIC-CASE\t",
        "STATIC-DOMAIN\t",
        "STATIC-TRANSITION\t",
    ]
    .iter()
    .position(|prefix| line.starts_with(prefix))
    .expect("closed static tag")
}

pub(crate) fn analyze(frame: Frame<'_>) -> Result<Report, Failure> {
    let current = analyze_document(frame.current_binding.bytes, &frame.limits)?;
    let proposal = analyze_document(frame.proposal_binding.bytes, &frame.limits)?;
    let output_limit = frame
        .limits
        .max_engine_output_bytes
        .checked_sub(b"WFGRREPORT1\nENGINE\tstatic\n".len() + b"END\n".len())
        .ok_or_else(|| Failure::resource("max-engine-output-bytes"))?;
    let mut output = Vec::new();
    append_line(&mut output, "COMMON-BEGIN", output_limit)?;
    for binding in frame.bindings() {
        append_line(
            &mut output,
            &format!(
                "BIND\t{}\t{}\t{}",
                binding.name,
                binding.bytes.len(),
                hex(&binding.digest)
            ),
            output_limit,
        )?;
    }
    let common_current_limit = output_limit
        .checked_sub(output.len())
        .ok_or_else(|| Failure::resource("max-engine-output-bytes"))?;
    for record in common_records("current", &current, common_current_limit)? {
        append_line(&mut output, &record.line, output_limit)?;
    }
    let common_proposal_limit = output_limit
        .checked_sub(output.len())
        .ok_or_else(|| Failure::resource("max-engine-output-bytes"))?;
    for record in common_records("proposal", &proposal, common_proposal_limit)? {
        append_line(&mut output, &record.line, output_limit)?;
    }
    append_line(&mut output, "COMMON-END", output_limit)?;
    append_line(&mut output, "STATIC-BEGIN", output_limit)?;
    let static_limit = output_limit
        .checked_sub(output.len())
        .and_then(|value| value.checked_sub(b"STATIC-END\n".len()))
        .ok_or_else(|| Failure::resource("max-engine-output-bytes"))?;
    let mut static_lines = BoundedLines::new(static_limit);
    static_records("current", &current, &mut static_lines)?;
    static_records("proposal", &proposal, &mut static_lines)?;
    delta_records(
        &current.static_analysis,
        &proposal.static_analysis,
        &mut static_lines,
    )?;
    let mut current_case_work = Work::new(frame.limits.static_max_work);
    let mut proposal_case_work = Work::new(frame.limits.static_max_work);
    let current_case_observations = case_records(
        "current",
        &frame,
        &current,
        &mut static_lines,
        &mut current_case_work,
    )?;
    let proposal_case_observations = case_records(
        "proposal",
        &frame,
        &proposal,
        &mut static_lines,
        &mut proposal_case_work,
    )?;
    domain_records("current", &frame, &current, &mut static_lines)?;
    domain_records("proposal", &frame, &proposal, &mut static_lines)?;
    let transition = transition_record(
        &frame,
        &current_case_observations,
        &proposal_case_observations,
    )?;
    static_lines.push(transition)?;
    static_lines.lines.sort_unstable_by(|left, right| {
        static_rank(left)
            .cmp(&static_rank(right))
            .then_with(|| left.as_bytes().cmp(right.as_bytes()))
    });
    static_lines.lines.dedup();
    for line in static_lines.lines {
        append_line(&mut output, &line, output_limit)?;
    }
    append_line(&mut output, "STATIC-END", output_limit)?;
    Ok(Report(output))
}

pub(crate) fn success_report(report: Report) -> Vec<u8> {
    let mut output = b"WFGRREPORT1\nENGINE\tstatic\n".to_vec();
    output.extend(report.0);
    output.extend_from_slice(b"END\n");
    output
}

pub(crate) fn failure_report(failure: Failure) -> Vec<u8> {
    format!(
        "WFGRREPORT1\nENGINE\tstatic\nFAIL\t{}\t{}\nEND\n",
        failure.family.name(),
        failure.code
    )
    .into_bytes()
}

#[cfg(test)]
mod tests {
    use super::tagged_witness;

    #[test]
    fn conflict_witness_preserves_token_boundaries() {
        assert_eq!(
            tagged_witness(&[b"deref".to_vec(), b"(".to_vec()]),
            "363436353732363536362c3238"
        );
        assert_eq!(tagged_witness(&[Vec::new()]), "2d");
        assert_eq!(tagged_witness(&[b"x".to_vec(), Vec::new()]), "37382c2d");
    }
}
