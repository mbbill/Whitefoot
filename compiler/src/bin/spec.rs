#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use whitefoot::{
    ACTIVE_KERNEL_SPEC_HASH, ACTIVE_KERNEL_SPEC_PATH, ACTIVE_KERNEL_SPEC_TEXT,
    ACTIVE_KERNEL_SPEC_VERSION, computed_active_spec_hash,
};

const DERIVATION_LEDGER: &str = include_str!("../../../spec/derivation/derivation-ledger.md");
const APPROVAL_RECORD: &str = include_str!("../../../governance/APPROVALS.md");

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

/// One `ACTIVE-SPEC:` line of the approval record's activation chain.
struct Activation<'a> {
    version: &'a str,
    digest: &'a str,
    superseded: &'a str,
}

fn is_digest(text: &str) -> bool {
    text.len() == 64 && text.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Every activation line, in file order. A line that carries the prefix but
/// not the exact shape is an error, never a line to skip.
fn activation_chain(approvals: &str) -> Result<Vec<Activation<'_>>, String> {
    let mut chain = Vec::new();
    for line in approvals.lines() {
        let Some(record) = line.strip_prefix("ACTIVE-SPEC: ") else {
            continue;
        };
        let mut fields = record.split(' ');
        let (Some(version), Some(digest), Some(superseded), None) =
            (fields.next(), fields.next(), fields.next(), fields.next())
        else {
            return Err(format!("activation record is not four fields: {line}"));
        };
        if !version.starts_with('v') || !is_digest(digest) {
            return Err(format!("activation record is malformed: {line}"));
        }
        if superseded != "-" && !is_digest(superseded) {
            return Err(format!("activation record is malformed: {line}"));
        }
        chain.push(Activation {
            version,
            digest,
            superseded,
        });
    }
    Ok(chain)
}

/// The version named by the specification's own title line.
fn titled_version(spec: &str) -> Option<&str> {
    spec.lines().next()?.strip_prefix("# Kernel Specification ")
}

/// The version named by the active status line inside the approved bytes.
fn active_status_version(spec: &str) -> Option<&str> {
    spec.lines()
        .skip(1)
        .find(|line| !line.trim().is_empty())?
        .strip_prefix("Status: ACTIVE ")?
        .split_whitespace()
        .next()
}

/// Check the activation chain against the specification actually embedded.
fn validate_activation_chain(
    approvals: &str,
    version: &str,
    spec: &str,
    digest: &str,
) -> Result<usize, Vec<String>> {
    let chain = match activation_chain(approvals) {
        Ok(chain) => chain,
        Err(error) => return Err(vec![error]),
    };
    let Some(active) = chain.last() else {
        return Err(vec!["approval record has no activation chain".to_owned()]);
    };

    let mut errors = Vec::new();
    for pair in chain.windows(2) {
        if pair[1].superseded != pair[0].digest {
            errors.push(format!(
                "{} supersedes {}, but {} was installed before it",
                pair[1].version, pair[1].superseded, pair[0].digest
            ));
        }
    }
    if chain[0].superseded != "-" {
        errors.push(format!(
            "the chain starts at {} but claims to supersede {}",
            chain[0].version, chain[0].superseded
        ));
    }

    if active.version != version {
        errors.push(format!(
            "the chain ends at {} but the active version is {version}",
            active.version
        ));
    }
    match titled_version(spec) {
        Some(titled) if titled == active.version => {}
        Some(titled) => errors.push(format!(
            "the chain ends at {} but the specification is titled {titled}",
            active.version
        )),
        None => errors.push("the specification has no title line".to_owned()),
    }
    match active_status_version(spec) {
        Some(status) if status == active.version => {}
        Some(status) => errors.push(format!(
            "the chain ends at {} but the specification status names {status}",
            active.version
        )),
        None => errors.push("the specification has no active status line".to_owned()),
    }
    if active.digest != digest {
        errors.push(format!(
            "the chain records {} for {}, but its bytes hash to {digest}",
            active.digest, active.version
        ));
    }

    if errors.is_empty() {
        Ok(chain.len())
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

    if !spec.contains("Specification delta:") {
        errors.push("status header has no Specification delta".to_owned());
    }
    if !spec.contains("Selection ground:") {
        errors.push("status header has no Selection ground".to_owned());
    }

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

/// Print one query result, or the reason the specification cannot answer it.
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
    let activations = match validate_activation_chain(
        APPROVAL_RECORD,
        ACTIVE_KERNEL_SPEC_VERSION,
        ACTIVE_KERNEL_SPEC_TEXT,
        &computed.to_string(),
    ) {
        Ok(activations) => activations,
        Err(errors) => {
            for error in errors {
                eprintln!("activation chain: {error}");
            }
            std::process::exit(1);
        }
    };
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
    println!(
        "Whitefoot {ACTIVE_KERNEL_SPEC_VERSION} activation chain: {activations} unbroken activations"
    );
}

#[cfg(test)]
mod tests {
    use super::{
        ACTIVE_KERNEL_SPEC_TEXT, ACTIVE_KERNEL_SPEC_VERSION, APPROVAL_RECORD, DERIVATION_LEDGER,
        computed_active_spec_hash, counts_json, index_json, is_rule_id, rule_definitions,
        validate_activation_chain, validate_spec_integrity,
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

    const A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn chain_of(records: &str) -> Result<usize, Vec<String>> {
        validate_activation_chain(
            records,
            "v0.2",
            "# Kernel Specification v0.2\n\nStatus: ACTIVE v0.2\n",
            B,
        )
    }

    /// The recorded chain shipped beside this compiler must describe the
    /// exact active specification bytes and contain every activation link.
    #[test]
    fn recorded_chain_ends_at_the_embedded_specification() {
        assert_eq!(
            validate_activation_chain(
                APPROVAL_RECORD,
                ACTIVE_KERNEL_SPEC_VERSION,
                ACTIVE_KERNEL_SPEC_TEXT,
                &computed_active_spec_hash().to_string(),
            ),
            Ok(21)
        );
    }

    #[test]
    fn well_formed_chain_passes() {
        assert_eq!(
            chain_of(&format!(
                "ACTIVE-SPEC: v0.1 {A} -\nprose\nACTIVE-SPEC: v0.2 {B} {A}\n"
            )),
            Ok(2)
        );
    }

    #[test]
    fn broken_link_fails() {
        let errors = chain_of(&format!(
            "ACTIVE-SPEC: v0.1 {A} -\nACTIVE-SPEC: v0.2 {B} {B}\n"
        ))
        .expect_err("a chain that skips its predecessor must fail");
        assert!(errors.iter().any(|error| error.contains("v0.2 supersedes")));
    }

    #[test]
    fn wrong_digest_for_the_installed_bytes_fails() {
        let errors = chain_of(&format!("ACTIVE-SPEC: v0.2 {A} -\n"))
            .expect_err("a chain naming other bytes must fail");
        assert!(errors.iter().any(|error| error.contains("bytes hash to")));
    }

    #[test]
    fn version_disagreement_fails() {
        let errors = validate_activation_chain(
            &format!("ACTIVE-SPEC: v0.3 {B} -\n"),
            "v0.2",
            "# Kernel Specification v0.2\n\nStatus: ACTIVE v0.2\n",
            B,
        )
        .expect_err("a chain ending at another version must fail");
        assert!(errors.iter().any(|error| error.contains("active version")));
        assert!(errors.iter().any(|error| error.contains("titled")));
    }

    #[test]
    fn malformed_and_missing_records_fail() {
        for records in [
            format!("ACTIVE-SPEC: v0.2 {B}\n"),
            format!("ACTIVE-SPEC: v0.2 {B} {A} extra\n"),
            format!("ACTIVE-SPEC: 0.2 {B} -\n"),
            "ACTIVE-SPEC: v0.2 short -\n".to_owned(),
            String::new(),
        ] {
            assert!(
                chain_of(&records).is_err(),
                "these records must not pass: {records}"
            );
        }
    }

    #[test]
    fn missing_or_non_active_status_fails() {
        for spec in [
            "# Kernel Specification v0.2\n",
            "# Kernel Specification v0.2\n\nStatus: REVIEW CANDIDATE v0.2\n",
            "# Kernel Specification v0.2\n\nStatus: ACTIVE v0.3\n",
        ] {
            let errors =
                validate_activation_chain(&format!("ACTIVE-SPEC: v0.2 {B} -\n"), "v0.2", spec, B)
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

    #[test]
    fn active_spec_has_complete_internal_integrity() {
        assert_eq!(
            validate_spec_integrity(ACTIVE_KERNEL_SPEC_TEXT, DERIVATION_LEDGER),
            Ok(133)
        );
    }

    #[test]
    fn unknown_references_and_missing_ledger_rows_fail() {
        let spec = "Specification delta: test\nSelection ground: test\n[X-1] See [X-2].\n";
        let errors = validate_spec_integrity(spec, "").expect_err("invalid spec must fail");
        assert!(errors.iter().any(|error| error.contains("[X-2]")));
        assert!(errors.iter().any(|error| error.contains("[X-1]")));
    }
}
