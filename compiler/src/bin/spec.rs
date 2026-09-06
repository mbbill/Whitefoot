#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use whitefoot::{
    ACTIVE_KERNEL_SPEC_HASH, ACTIVE_KERNEL_SPEC_PATH, ACTIVE_KERNEL_SPEC_TEXT,
    ACTIVE_KERNEL_SPEC_VERSION, computed_active_spec_hash,
};

const DERIVATION_LEDGER: &str = include_str!("../../../spec/derivation/derivation-ledger.md");

fn is_rule_id(text: &str) -> bool {
    let Some((family, number)) = text.split_once('-') else {
        return false;
    };
    if family.is_empty() || !family.bytes().all(|byte| byte.is_ascii_uppercase()) {
        return false;
    }

    let digit_count = number.bytes().take_while(u8::is_ascii_digit).count();
    digit_count > 0
        && (digit_count == number.len()
            || (digit_count + 1 == number.len()
                && number.as_bytes()[digit_count].is_ascii_lowercase()))
}

fn rule_definitions(text: &str) -> Result<BTreeSet<&str>, String> {
    let mut rules = BTreeSet::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix('[') else {
            continue;
        };
        let Some(close) = rest.find(']') else {
            continue;
        };
        let candidate = &rest[..close];
        if is_rule_id(candidate) && !rules.insert(candidate) {
            return Err(format!("duplicate rule definition [{candidate}]"));
        }
    }
    Ok(rules)
}

fn rule_references(text: &str) -> BTreeSet<&str> {
    let mut references = BTreeSet::new();
    let mut remaining = text;
    while let Some(open) = remaining.find('[') {
        remaining = &remaining[open + 1..];
        let Some(close) = remaining.find(']') else {
            break;
        };
        let candidate = &remaining[..close];
        if is_rule_id(candidate) {
            references.insert(candidate);
        }
        remaining = &remaining[close + 1..];
    }
    references
}

/// One rule's block extent inside the specification text: the `[ID]`
/// definition line through the line before the next definition or heading,
/// with trailing blank lines trimmed.
struct RuleBlock<'a> {
    id: &'a str,
    /// 1-based first line (the definition line).
    start: usize,
    /// 1-based last content line.
    end: usize,
    /// Byte length of the block's text, excluding the final newline.
    bytes: usize,
    /// Rule ids the block references, sorted, excluding the block's own id.
    refs: Vec<&'a str>,
}

/// Every rule block in definition order.
///
/// The definition predicate is exactly the one `rule_definitions` uses, so the
/// index covers exactly the rule set the integrity gate counts.
fn rule_blocks(text: &str) -> Result<Vec<RuleBlock<'_>>, String> {
    struct Line<'a> {
        offset: usize,
        text: &'a str,
    }
    let mut lines: Vec<Line<'_>> = Vec::new();
    let mut offset = 0;
    for raw in text.split_inclusive('\n') {
        lines.push(Line {
            offset,
            text: raw.strip_suffix('\n').unwrap_or(raw),
        });
        offset += raw.len();
    }

    let mut definitions: Vec<(usize, &str)> = Vec::new();
    let mut seen = BTreeSet::new();
    for (number, line) in lines.iter().enumerate() {
        let Some(rest) = line.text.strip_prefix('[') else {
            continue;
        };
        let Some(close) = rest.find(']') else {
            continue;
        };
        let candidate = &rest[..close];
        if is_rule_id(candidate) {
            if !seen.insert(candidate) {
                return Err(format!("duplicate rule definition [{candidate}]"));
            }
            definitions.push((number, candidate));
        }
    }

    let mut blocks = Vec::new();
    for (position, (start, id)) in definitions.iter().enumerate() {
        let stop = definitions
            .get(position + 1)
            .map_or(lines.len(), |(next, _)| *next);
        let limit = lines[*start + 1..stop]
            .iter()
            .position(|line| line.text.starts_with('#'))
            .map_or(stop, |found| *start + 1 + found);
        let mut last = limit;
        while last > *start + 1 && lines[last - 1].text.trim().is_empty() {
            last -= 1;
        }
        let first = &lines[*start];
        let closing = &lines[last - 1];
        let block = &text[first.offset..closing.offset + closing.text.len()];
        let refs = rule_references(block)
            .into_iter()
            .filter(|reference| reference != id)
            .collect();
        blocks.push(RuleBlock {
            id,
            start: start + 1,
            end: last,
            bytes: block.len(),
            refs,
        });
    }
    Ok(blocks)
}

/// The `--index` query: every rule's location, size, and outgoing references,
/// as one JSON object on stdout. A query, never a committed artifact.
fn index_json(text: &str) -> Result<String, String> {
    let blocks = rule_blocks(text)?;
    let mut out = String::from("{\n");
    for (position, block) in blocks.iter().enumerate() {
        let refs: Vec<String> = block
            .refs
            .iter()
            .map(|reference| format!("\"{reference}\""))
            .collect();
        out.push_str(&format!(
            "\"{}\": {{\"start\": {}, \"end\": {}, \"bytes\": {}, \"refs\": [{}]}}{}\n",
            block.id,
            block.start,
            block.end,
            block.bytes,
            refs.join(", "),
            if position + 1 == blocks.len() {
                ""
            } else {
                ","
            }
        ));
    }
    out.push('}');
    Ok(out)
}

/// The `--counts` query: per-family rule counts, the total, each markdown
/// table's row count, and the byte size of the pre-section header. These are
/// the numbers reviews otherwise re-derive by hand.
fn counts_json(text: &str) -> Result<String, String> {
    let blocks = rule_blocks(text)?;
    let mut families: std::collections::BTreeMap<&str, usize> = std::collections::BTreeMap::new();
    for block in &blocks {
        let family = block
            .id
            .split_once('-')
            .map_or(block.id, |(family, _)| family);
        *families.entry(family).or_insert(0) += 1;
    }

    let mut tables: Vec<(usize, usize)> = Vec::new();
    let mut current: Option<(usize, usize)> = None;
    for (number, line) in text.lines().enumerate() {
        if line.starts_with('|') {
            match current.as_mut() {
                Some((_, rows)) => *rows += 1,
                None => current = Some((number + 1, 1)),
            }
        } else if let Some(table) = current.take() {
            tables.push(table);
        }
    }
    if let Some(table) = current.take() {
        tables.push(table);
    }

    let mut header_bytes = text.len();
    let mut offset = 0;
    for raw in text.split_inclusive('\n') {
        if raw.starts_with("## ") {
            header_bytes = offset;
            break;
        }
        offset += raw.len();
    }

    let families: Vec<String> = families
        .iter()
        .map(|(family, count)| format!("\"{family}\": {count}"))
        .collect();
    let tables: Vec<String> = tables
        .iter()
        .map(|(line, rows)| format!("{{\"line\": {line}, \"rows\": {rows}}}"))
        .collect();
    Ok(format!(
        "{{\n\"families\": {{{}}},\n\"total_rules\": {},\n\"tables\": [{}],\n\"header_bytes\": {}\n}}",
        families.join(", "),
        blocks.len(),
        tables.join(", "),
        header_bytes
    ))
}

fn ledger_rule_ids(text: &str) -> BTreeSet<&str> {
    text.lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("| ")?;
            let (candidate, _) = rest.split_once(" |")?;
            is_rule_id(candidate).then_some(candidate)
        })
        .collect()
}

/// The version named by the specification's own title line.
fn titled_version(spec: &str) -> Option<&str> {
    spec.lines().next()?.strip_prefix("# Kernel Specification ")
}

/// The version named by the specification's own status line, the first
/// non-blank line after the title, which must read `Status: ACTIVE vN ...`.
/// There is no other status: an amended specification lands with its ACTIVE
/// identity, the archive of the outgoing bytes, and its chain line in one
/// change, so the stable file is always the installed authority.
fn active_status_version(spec: &str) -> Result<&str, String> {
    let Some(line) = spec.lines().skip(1).find(|line| !line.trim().is_empty()) else {
        return Err("the specification has no status line".to_owned());
    };
    let Some(rest) = line.strip_prefix("Status: ACTIVE ") else {
        return Err(format!(
            "the specification status is not `ACTIVE vN`: {line}"
        ));
    };
    rest.split_whitespace()
        .next()
        .ok_or_else(|| format!("the active status names no version: {line}"))
}

/// Check that the specification names one version in both places it states
/// one, and that the generated identity module names the same.
///
/// The specification's bytes are its identity; nothing else records it. The
/// title line and the status line are two independent statements of the same
/// version, so they are checked against each other and against the generated
/// module rather than against a ledger of past activations.
fn validate_spec_identity(spec: &str, version: &str) -> Result<(), Vec<String>> {
    let mut errors = Vec::new();
    let titled = titled_version(spec);
    match titled {
        Some(titled) if titled == version => {}
        Some(titled) => errors.push(format!(
            "the specification is titled {titled} but the generated identity names {version}"
        )),
        None => errors.push("the specification has no title line".to_owned()),
    }
    match active_status_version(spec) {
        Ok(status_version) => {
            if let Some(titled) = titled
                && status_version != titled
            {
                errors.push(format!(
                    "the specification is titled {titled} but its status names {status_version}"
                ));
            }
        }
        Err(error) => errors.push(error),
    }
    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_spec_integrity(spec: &str, ledger: &str) -> Result<usize, Vec<String>> {
    let mut errors = Vec::new();
    let rules = match rule_definitions(spec) {
        Ok(rules) => rules,
        Err(error) => {
            errors.push(error);
            BTreeSet::new()
        }
    };

    for reference in rule_references(spec).difference(&rules) {
        errors.push(format!("unknown rule reference [{reference}]"));
    }

    let ledger_rules = ledger_rule_ids(ledger);
    for rule in rules.difference(&ledger_rules) {
        errors.push(format!("derivation ledger has no row for [{rule}]"));
    }

    // The v0.30 header profile carries no "Specification delta:" or
    // "Selection ground:" sentences: the per-activation delta inventory lives
    // in the review packet and the approval-ledger entry, never in the
    // normative bytes. The former hard requirements on those two phrases are
    // deliberately retired with it.

    if errors.is_empty() {
        Ok(rules.len())
    } else {
        Err(errors)
    }
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match arguments.first().map(String::as_str) {
        None => run_gate(),
        Some("--index") => run_query(index_json(ACTIVE_KERNEL_SPEC_TEXT)),
        Some("--counts") => run_query(counts_json(ACTIVE_KERNEL_SPEC_TEXT)),
        Some(flag) => {
            eprintln!("whitefoot-spec: unknown flag {flag}; flags: --index, --counts");
            std::process::exit(2);
        }
    }
}

fn run_query(result: Result<String, String>) {
    match result {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("whitefoot-spec: {error}");
            std::process::exit(1);
        }
    }
}

fn run_gate() {
    let computed = computed_active_spec_hash();
    if ACTIVE_KERNEL_SPEC_HASH != computed {
        eprintln!("{ACTIVE_KERNEL_SPEC_PATH} does not hash to the recorded active identity");
        std::process::exit(1);
    }
    if let Err(errors) = validate_spec_identity(ACTIVE_KERNEL_SPEC_TEXT, ACTIVE_KERNEL_SPEC_VERSION)
    {
        for error in errors {
            eprintln!("spec identity: {error}");
        }
        std::process::exit(1);
    }
    let rule_count = match validate_spec_integrity(ACTIVE_KERNEL_SPEC_TEXT, DERIVATION_LEDGER) {
        Ok(rule_count) => rule_count,
        Err(errors) => {
            for error in errors {
                eprintln!("spec integrity: {error}");
            }
            std::process::exit(1);
        }
    };
    println!("Whitefoot {ACTIVE_KERNEL_SPEC_VERSION} frontend identity: {ACTIVE_KERNEL_SPEC_HASH}");
    println!("Whitefoot {ACTIVE_KERNEL_SPEC_VERSION} spec integrity: {rule_count} rules");
}

#[cfg(test)]
mod tests {
    use super::{
        ACTIVE_KERNEL_SPEC_TEXT, ACTIVE_KERNEL_SPEC_VERSION, DERIVATION_LEDGER, counts_json,
        index_json, is_rule_id, rule_definitions, validate_spec_identity, validate_spec_integrity,
    };

    /// One parsed `--index` entry.
    struct IndexEntry {
        start: usize,
        end: usize,
        bytes: usize,
        refs: Vec<String>,
    }

    /// Minimal reader for exactly the subset `index_json` emits: one object of
    /// objects whose values are non-negative integers and arrays of
    /// escape-free strings. Hand-rolled on purpose: the crate has no
    /// dependencies, and re-reading the emitted text through independent code
    /// is the test.
    struct Reader<'a> {
        bytes: &'a [u8],
        at: usize,
    }

    impl<'a> Reader<'a> {
        fn skip_space(&mut self) {
            while self.bytes.get(self.at).is_some_and(u8::is_ascii_whitespace) {
                self.at += 1;
            }
        }

        fn eat(&mut self, expected: u8) {
            self.skip_space();
            assert_eq!(
                self.bytes.get(self.at).copied(),
                Some(expected),
                "expected {:?} at byte {}",
                char::from(expected),
                self.at
            );
            self.at += 1;
        }

        fn peek(&mut self) -> Option<u8> {
            self.skip_space();
            self.bytes.get(self.at).copied()
        }

        fn string(&mut self) -> &'a str {
            self.eat(b'"');
            let start = self.at;
            while self.bytes.get(self.at).is_some_and(|byte| *byte != b'"') {
                assert_ne!(self.bytes[self.at], b'\\', "emitted strings never escape");
                self.at += 1;
            }
            let text = core::str::from_utf8(&self.bytes[start..self.at]).expect("emitted UTF-8");
            self.eat(b'"');
            text
        }

        fn number(&mut self) -> usize {
            self.skip_space();
            let start = self.at;
            while self.bytes.get(self.at).is_some_and(u8::is_ascii_digit) {
                self.at += 1;
            }
            assert_ne!(self.at, start, "expected a number at byte {start}");
            core::str::from_utf8(&self.bytes[start..self.at])
                .expect("digits are UTF-8")
                .parse()
                .expect("digits parse")
        }
    }

    /// Parses the complete `--index` document, asserting its JSON shape.
    fn parse_index(text: &str) -> Vec<(String, IndexEntry)> {
        let mut reader = Reader {
            bytes: text.as_bytes(),
            at: 0,
        };
        let mut entries = Vec::new();
        reader.eat(b'{');
        while reader.peek() != Some(b'}') {
            if !entries.is_empty() {
                reader.eat(b',');
            }
            let id = reader.string().to_owned();
            reader.eat(b':');
            reader.eat(b'{');
            let (mut start, mut end, mut bytes, mut refs) = (None, None, None, None);
            loop {
                let key = reader.string().to_owned();
                reader.eat(b':');
                match key.as_str() {
                    "start" => start = Some(reader.number()),
                    "end" => end = Some(reader.number()),
                    "bytes" => bytes = Some(reader.number()),
                    "refs" => {
                        let mut list = Vec::new();
                        reader.eat(b'[');
                        while reader.peek() != Some(b']') {
                            if !list.is_empty() {
                                reader.eat(b',');
                            }
                            list.push(reader.string().to_owned());
                        }
                        reader.eat(b']');
                        refs = Some(list);
                    }
                    other => panic!("unknown key {other:?}"),
                }
                if reader.peek() == Some(b',') {
                    reader.eat(b',');
                } else {
                    break;
                }
            }
            reader.eat(b'}');
            entries.push((
                id,
                IndexEntry {
                    start: start.expect("start"),
                    end: end.expect("end"),
                    bytes: bytes.expect("bytes"),
                    refs: refs.expect("refs"),
                },
            ));
        }
        reader.eat(b'}');
        reader.skip_space();
        assert_eq!(reader.at, reader.bytes.len(), "trailing bytes after JSON");
        entries
    }

    /// `--index` output parses as JSON and describes exactly the rule set the
    /// integrity scanner finds: same ids, correct definition lines, block
    /// bytes that match the named line range, and only resolvable references.
    #[test]
    fn index_query_parses_and_covers_the_scanned_rule_set() {
        let emitted = index_json(ACTIVE_KERNEL_SPEC_TEXT).expect("active spec indexes");
        let entries = parse_index(&emitted);
        let scanned = rule_definitions(ACTIVE_KERNEL_SPEC_TEXT).expect("active spec scans");

        let indexed: std::collections::BTreeSet<&str> =
            entries.iter().map(|(id, _)| id.as_str()).collect();
        assert_eq!(
            indexed,
            scanned.iter().copied().collect(),
            "the index must cover exactly the scanned rule set"
        );

        let lines: Vec<&str> = ACTIVE_KERNEL_SPEC_TEXT.lines().collect();
        for (id, entry) in &entries {
            assert!(entry.start <= entry.end, "[{id}] has an inverted range");
            assert!(
                lines[entry.start - 1].starts_with(&format!("[{id}]")),
                "[{id}] start line {} is not its definition line",
                entry.start
            );
            let block = lines[entry.start - 1..entry.end].join("\n");
            assert_eq!(block.len(), entry.bytes, "[{id}] bytes disagree");
            for reference in &entry.refs {
                assert_ne!(reference, id, "[{id}] lists itself as a reference");
                assert!(
                    scanned.contains(reference.as_str()),
                    "[{id}] references unknown [{reference}]"
                );
            }
        }
    }

    /// `--counts` agrees with the scanner's totals.
    #[test]
    fn counts_query_totals_agree_with_the_scanner() {
        let emitted = counts_json(ACTIVE_KERNEL_SPEC_TEXT).expect("active spec counts");
        let scanned = rule_definitions(ACTIVE_KERNEL_SPEC_TEXT).expect("active spec scans");
        assert!(
            emitted.contains(&format!("\"total_rules\": {}", scanned.len())),
            "total_rules must equal the scanned rule count"
        );
    }

    /// The specification shipped beside this compiler states one version in
    /// both places it states one, and the generated module agrees.
    #[test]
    fn the_embedded_specification_names_one_version() {
        assert_eq!(
            validate_spec_identity(ACTIVE_KERNEL_SPEC_TEXT, ACTIVE_KERNEL_SPEC_VERSION),
            Ok(())
        );
    }

    #[test]
    fn a_title_disagreeing_with_the_generated_identity_fails() {
        let errors = validate_spec_identity(
            "# Kernel Specification v0.2\n\nStatus: ACTIVE v0.2\n",
            "v0.3",
        )
        .expect_err("a specification titled other than the generated identity must fail");
        assert!(errors.iter().any(|error| error.contains("titled")));
    }

    #[test]
    fn a_status_disagreeing_with_the_title_fails() {
        let errors = validate_spec_identity(
            "# Kernel Specification v0.2\n\nStatus: ACTIVE v0.3\n",
            "v0.2",
        )
        .expect_err("a status naming another version must fail");
        assert!(errors.iter().any(|error| error.contains("status names")));
    }

    #[test]
    fn a_missing_or_non_active_status_fails() {
        for spec in [
            "# Kernel Specification v0.2\n",
            "# Kernel Specification v0.2\n\nStatus: REVIEW CANDIDATE v0.2\n",
        ] {
            let errors = validate_spec_identity(spec, "v0.2")
                .expect_err("the installed bytes must carry their active status");
            assert!(
                errors.iter().any(|error| error.contains("status")),
                "missing status error for {spec:?}: {errors:?}"
            );
        }
    }

    #[test]
    fn rule_id_shape_is_closed() {
        assert!(is_rule_id("TYPE-6"));
        assert!(is_rule_id("GRAM-10a"));
        assert!(!is_rule_id("type-6"));
        assert!(!is_rule_id("TYPE"));
        assert!(!is_rule_id("TYPE-6ab"));
    }

    /// The expected rule count is the generated `spec_identity` module's, and
    /// both sides scan the same embedded specification bytes, so this is a
    /// consistency check between artifacts — the committed generated module
    /// against this validator's scan — not an independent count. Green means
    /// the integrity checks (resolvable references, ledger rows, status
    /// markers) pass and the committed module is fresh; it replaced a
    /// hand-bumped `Ok(132)→Ok(133)` literal that witnessed one hand
    /// transcription, nothing more. Green does NOT establish that the count is
    /// the right number of rules: a rule deleted from the specification and
    /// then regenerated into the module still passes here, and that class is
    /// caught only by the owner's exact-byte specification approval and the
    /// reviewed `spec_identity.rs` diff riding the same change.
    #[test]
    fn active_spec_has_complete_internal_integrity() {
        let scanned = rule_definitions(ACTIVE_KERNEL_SPEC_TEXT).expect("the spec scans");
        assert_eq!(
            validate_spec_integrity(ACTIVE_KERNEL_SPEC_TEXT, DERIVATION_LEDGER),
            Ok(scanned.len())
        );
    }

    #[test]
    fn unknown_references_and_missing_ledger_rows_fail() {
        let spec = "[X-1] See [X-2].\n";
        let errors = validate_spec_integrity(spec, "").expect_err("invalid spec must fail");
        assert!(errors.iter().any(|error| error.contains("[X-2]")));
        assert!(errors.iter().any(|error| error.contains("[X-1]")));
    }
}
