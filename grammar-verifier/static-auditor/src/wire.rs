//! Exact frame, resource-profile, authored-case, and domain decoding.

use crate::hash::sha256;

const MAGIC: &[u8; 8] = b"WFGRAMV1";
const V08_SHA256: [u8; 32] = [
    0xd0, 0x43, 0x36, 0xf7, 0xfa, 0x8d, 0x1a, 0x6a, 0x0f, 0x03, 0xfe, 0x58, 0xa1, 0x7f, 0x97, 0x2b,
    0x65, 0x82, 0x17, 0xa7, 0x3a, 0x3d, 0xff, 0x91, 0xa9, 0x06, 0xb4, 0xba, 0x29, 0x53, 0x28, 0xa8,
];
const LIMIT_NAMES: [&[u8]; 26] = [
    b"cpu_timeout_seconds",
    b"max_case_bytes",
    b"max_cases",
    b"max_definitions",
    b"max_document_bytes",
    b"max_domain_bytes",
    b"max_domains",
    b"max_ebnf_depth",
    b"max_engine_output_bytes",
    b"max_final_report_bytes",
    b"max_generated_streams",
    b"max_grammar_nodes",
    b"max_lexical_definitions",
    b"max_line_bytes",
    b"max_lines",
    b"max_rules",
    b"max_symbol_bytes",
    b"max_terminal_occurrences",
    b"oracle_max_chart_items",
    b"oracle_max_packed_edges",
    b"oracle_max_proof_nodes",
    b"oracle_max_source_tokens",
    b"static_max_lookahead_words",
    b"static_max_product_states",
    b"static_max_work",
    b"wall_timeout_seconds",
];
const LIMIT_MAXIMA: [usize; 26] = [
    60, 131_072, 1_024, 1_024, 524_288, 131_072, 64, 128, 8_388_608, 16_777_216, 100_000, 65_536,
    128, 16_384, 4_096, 1_024, 256, 8_192, 1_000_000, 1_000_000, 1_000_000, 256, 262_144,
    1_000_000, 10_000_000, 60,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Family {
    Input,
    Extraction,
    Resource,
    Internal,
}

impl Family {
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Input => "input",
            Self::Extraction => "extraction",
            Self::Resource => "resource",
            Self::Internal => "internal",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Failure {
    pub(crate) family: Family,
    pub(crate) code: &'static str,
}

impl Failure {
    pub(crate) const fn input(code: &'static str) -> Self {
        Self {
            family: Family::Input,
            code,
        }
    }

    pub(crate) const fn extraction(code: &'static str) -> Self {
        Self {
            family: Family::Extraction,
            code,
        }
    }

    pub(crate) const fn resource(code: &'static str) -> Self {
        Self {
            family: Family::Resource,
            code,
        }
    }

    pub(crate) const fn internal(code: &'static str) -> Self {
        Self {
            family: Family::Internal,
            code,
        }
    }

    pub(crate) const fn allocation() -> Self {
        Self::resource("allocation")
    }
}

#[allow(
    dead_code,
    reason = "the shared profile includes Oracle-only limits that this independent engine binds"
)]
#[derive(Clone, Debug)]
pub(crate) struct Limits {
    pub(crate) cpu_timeout_seconds: usize,
    pub(crate) max_case_bytes: usize,
    pub(crate) max_cases: usize,
    pub(crate) max_definitions: usize,
    pub(crate) max_document_bytes: usize,
    pub(crate) max_domain_bytes: usize,
    pub(crate) max_domains: usize,
    pub(crate) max_ebnf_depth: usize,
    pub(crate) max_engine_output_bytes: usize,
    pub(crate) max_final_report_bytes: usize,
    pub(crate) max_generated_streams: usize,
    pub(crate) max_grammar_nodes: usize,
    pub(crate) max_lexical_definitions: usize,
    pub(crate) max_line_bytes: usize,
    pub(crate) max_lines: usize,
    pub(crate) max_rules: usize,
    pub(crate) max_symbol_bytes: usize,
    pub(crate) max_terminal_occurrences: usize,
    pub(crate) oracle_max_chart_items: usize,
    pub(crate) oracle_max_packed_edges: usize,
    pub(crate) oracle_max_proof_nodes: usize,
    pub(crate) oracle_max_source_tokens: usize,
    pub(crate) static_max_lookahead_words: usize,
    pub(crate) static_max_product_states: usize,
    pub(crate) static_max_work: usize,
    pub(crate) wall_timeout_seconds: usize,
}

#[derive(Clone, Debug)]
pub(crate) struct Case {
    pub(crate) id: String,
    pub(crate) start: String,
    pub(crate) source: Vec<u8>,
}

#[derive(Clone, Debug)]
pub(crate) struct Domain {
    pub(crate) id: String,
    pub(crate) start: String,
    pub(crate) argument: Vec<u8>,
}

#[derive(Clone, Copy)]
pub(crate) struct Binding<'a> {
    pub(crate) name: &'static str,
    pub(crate) bytes: &'a [u8],
    pub(crate) digest: [u8; 32],
}

pub(crate) struct Frame<'a> {
    pub(crate) limits_binding: Binding<'a>,
    pub(crate) current_binding: Binding<'a>,
    pub(crate) proposal_binding: Binding<'a>,
    pub(crate) cases_binding: Binding<'a>,
    pub(crate) domains_binding: Binding<'a>,
    pub(crate) limits: Limits,
    pub(crate) cases: Vec<Case>,
    pub(crate) domains: Vec<Domain>,
}

impl<'a> Frame<'a> {
    pub(crate) fn parse(input: &'a [u8]) -> Result<Self, Failure> {
        if input.get(..8) != Some(MAGIC) || input.len() < 48 {
            return Err(Failure::input("frame-header"));
        }
        let mut lengths = [0_usize; 5];
        for (index, slot) in lengths.iter_mut().enumerate() {
            let start = 8 + index * 8;
            let raw: [u8; 8] = input[start..start + 8]
                .try_into()
                .map_err(|_| Failure::input("frame-header"))?;
            *slot = usize::try_from(u64::from_be_bytes(raw))
                .map_err(|_| Failure::input("frame-length"))?;
        }
        if lengths[0] > 8_192
            || lengths[1] > 1_048_576
            || lengths[2] > 1_048_576
            || lengths[3] > 262_144
            || lengths[4] > 262_144
        {
            return Err(Failure::input("frame-outer-limit"));
        }
        let payload = lengths.iter().try_fold(48_usize, |total, length| {
            total
                .checked_add(*length)
                .ok_or_else(|| Failure::input("frame-length"))
        })?;
        if payload != input.len() {
            return Err(Failure::input("frame-length"));
        }
        let mut cursor = 48;
        let mut next = |length: usize| {
            let start = cursor;
            cursor += length;
            &input[start..cursor]
        };
        let limits_bytes = next(lengths[0]);
        let current = next(lengths[1]);
        let proposal = next(lengths[2]);
        let case_bytes = next(lengths[3]);
        let domain_bytes = next(lengths[4]);
        let limits = parse_limits(limits_bytes)?;
        if current.len() > limits.max_document_bytes || proposal.len() > limits.max_document_bytes {
            return Err(Failure::resource("max-document-bytes"));
        }
        if case_bytes.len() > limits.max_case_bytes {
            return Err(Failure::resource("max-case-bytes"));
        }
        if domain_bytes.len() > limits.max_domain_bytes {
            return Err(Failure::resource("max-domain-bytes"));
        }
        if sha256(current) != V08_SHA256 {
            return Err(Failure::input("current-document-identity"));
        }
        let cases = parse_cases(case_bytes, &limits)?;
        let domains = parse_domains(domain_bytes, &limits)?;
        Ok(Self {
            limits_binding: binding("limits", limits_bytes),
            current_binding: binding("current", current),
            proposal_binding: binding("proposal", proposal),
            cases_binding: binding("cases", case_bytes),
            domains_binding: binding("domains", domain_bytes),
            limits,
            cases,
            domains,
        })
    }

    pub(crate) fn bindings(&self) -> [Binding<'a>; 5] {
        [
            self.limits_binding,
            self.current_binding,
            self.proposal_binding,
            self.cases_binding,
            self.domains_binding,
        ]
    }
}

fn binding<'a>(name: &'static str, bytes: &'a [u8]) -> Binding<'a> {
    Binding {
        name,
        bytes,
        digest: sha256(bytes),
    }
}

fn parse_decimal(bytes: &[u8]) -> Result<usize, Failure> {
    if bytes.is_empty()
        || (bytes.len() > 1 && bytes[0] == b'0')
        || !bytes.iter().all(u8::is_ascii_digit)
    {
        return Err(Failure::input("limits-decimal"));
    }
    let mut value = 0_usize;
    for digit in bytes {
        value = value
            .checked_mul(10)
            .and_then(|current| current.checked_add(usize::from(*digit - b'0')))
            .ok_or_else(|| Failure::input("limits-decimal"))?;
    }
    if value == 0 {
        return Err(Failure::input("limits-decimal"));
    }
    Ok(value)
}

fn parse_limits(bytes: &[u8]) -> Result<Limits, Failure> {
    if !bytes.ends_with(b"\n") || !bytes.is_ascii() {
        return Err(Failure::input("limits-format"));
    }
    let lines: Vec<&[u8]> = bytes[..bytes.len() - 1]
        .split(|byte| *byte == b'\n')
        .collect();
    if lines.len() != LIMIT_NAMES.len() {
        return Err(Failure::input("limits-fields"));
    }
    let mut values = [0_usize; 26];
    for ((line, name), slot) in lines.into_iter().zip(LIMIT_NAMES).zip(&mut values) {
        let Some(value) = line
            .strip_prefix(name)
            .and_then(|tail| tail.strip_prefix(b"="))
        else {
            return Err(Failure::input("limits-fields"));
        };
        *slot = parse_decimal(value)?;
    }
    if values
        .iter()
        .zip(LIMIT_MAXIMA)
        .any(|(value, maximum)| *value > maximum)
    {
        return Err(Failure::input("limits-maximum"));
    }
    Ok(Limits {
        cpu_timeout_seconds: values[0],
        max_case_bytes: values[1],
        max_cases: values[2],
        max_definitions: values[3],
        max_document_bytes: values[4],
        max_domain_bytes: values[5],
        max_domains: values[6],
        max_ebnf_depth: values[7],
        max_engine_output_bytes: values[8],
        max_final_report_bytes: values[9],
        max_generated_streams: values[10],
        max_grammar_nodes: values[11],
        max_lexical_definitions: values[12],
        max_line_bytes: values[13],
        max_lines: values[14],
        max_rules: values[15],
        max_symbol_bytes: values[16],
        max_terminal_occurrences: values[17],
        oracle_max_chart_items: values[18],
        oracle_max_packed_edges: values[19],
        oracle_max_proof_nodes: values[20],
        oracle_max_source_tokens: values[21],
        static_max_lookahead_words: values[22],
        static_max_product_states: values[23],
        static_max_work: values[24],
        wall_timeout_seconds: values[25],
    })
}

fn valid_id(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

fn valid_symbol(bytes: &[u8]) -> bool {
    !bytes.is_empty()
        && bytes[0].is_ascii_lowercase()
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}

fn decode_hex(bytes: &[u8]) -> Result<Vec<u8>, Failure> {
    if bytes.is_empty() || !bytes.len().is_multiple_of(2) {
        return Err(Failure::input("manifest-hex"));
    }
    fn nibble(byte: u8) -> Option<u8> {
        match byte {
            b'0'..=b'9' => Some(byte - b'0'),
            b'a'..=b'f' => Some(byte - b'a' + 10),
            _ => None,
        }
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(bytes.len() / 2)
        .map_err(|_| Failure::allocation())?;
    for pair in bytes.chunks_exact(2) {
        let high = nibble(pair[0]).ok_or_else(|| Failure::input("manifest-hex"))?;
        let low = nibble(pair[1]).ok_or_else(|| Failure::input("manifest-hex"))?;
        output.push((high << 4) | low);
    }
    Ok(output)
}

fn manifest_lines<'a>(bytes: &'a [u8], version: &[u8]) -> Result<Vec<&'a [u8]>, Failure> {
    if !bytes.ends_with(b"\n") || !bytes.is_ascii() {
        return Err(Failure::input("manifest-format"));
    }
    let mut lines = bytes[..bytes.len() - 1].split(|byte| *byte == b'\n');
    if lines.next() != Some(version) {
        return Err(Failure::input("manifest-version"));
    }
    Ok(lines.collect())
}

fn parse_cases(bytes: &[u8], limits: &Limits) -> Result<Vec<Case>, Failure> {
    let lines = manifest_lines(bytes, b"whitefoot.grammar-cases.v1")?;
    if lines.len() > limits.max_cases {
        return Err(Failure::resource("max-cases"));
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(lines.len())
        .map_err(|_| Failure::allocation())?;
    let mut previous: Option<&[u8]> = None;
    for line in lines {
        let fields: Vec<&[u8]> = line.split(|byte| *byte == b'\t').collect();
        if fields.len() != 4
            || fields[0] != b"case"
            || !valid_id(fields[1])
            || !valid_symbol(fields[2])
            || previous.is_some_and(|last| last >= fields[1])
        {
            return Err(Failure::input("cases-record"));
        }
        if fields[1].len() > limits.max_symbol_bytes || fields[2].len() > limits.max_symbol_bytes {
            return Err(Failure::resource("max-symbol-bytes"));
        }
        previous = Some(fields[1]);
        output.push(Case {
            id: String::from_utf8(fields[1].to_vec())
                .map_err(|_| Failure::input("cases-record"))?,
            start: String::from_utf8(fields[2].to_vec())
                .map_err(|_| Failure::input("cases-record"))?,
            source: decode_hex(fields[3])?,
        });
    }
    Ok(output)
}

fn parse_domains(bytes: &[u8], limits: &Limits) -> Result<Vec<Domain>, Failure> {
    let lines = manifest_lines(bytes, b"whitefoot.grammar-domains.v1")?;
    if lines.len() > limits.max_domains {
        return Err(Failure::resource("max-domains"));
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(lines.len())
        .map_err(|_| Failure::allocation())?;
    let mut previous: Option<&[u8]> = None;
    for line in lines {
        let fields: Vec<&[u8]> = line.split(|byte| *byte == b'\t').collect();
        if fields.len() != 5
            || fields[0] != b"domain"
            || !valid_id(fields[1])
            || fields[2] != b"fixed-lowerword-call"
            || !valid_symbol(fields[3])
            || previous.is_some_and(|last| last >= fields[1])
        {
            return Err(Failure::input("domains-record"));
        }
        if fields[1].len() > limits.max_symbol_bytes || fields[3].len() > limits.max_symbol_bytes {
            return Err(Failure::resource("max-symbol-bytes"));
        }
        previous = Some(fields[1]);
        output.push(Domain {
            id: String::from_utf8(fields[1].to_vec())
                .map_err(|_| Failure::input("domains-record"))?,
            start: String::from_utf8(fields[3].to_vec())
                .map_err(|_| Failure::input("domains-record"))?,
            argument: decode_hex(fields[4])?,
        });
    }
    Ok(output)
}

pub(crate) struct Work {
    remaining: usize,
}

impl Work {
    pub(crate) const fn new(limit: usize) -> Self {
        Self { remaining: limit }
    }

    pub(crate) fn spend(&mut self, amount: usize) -> Result<(), Failure> {
        self.remaining = self
            .remaining
            .checked_sub(amount)
            .ok_or_else(|| Failure::resource("static-max-work"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{Family, LIMIT_MAXIMA, LIMIT_NAMES, parse_cases, parse_domains, parse_limits};

    fn profile(values: &[usize; 26]) -> Vec<u8> {
        let mut output = Vec::new();
        for (name, value) in LIMIT_NAMES.iter().zip(values) {
            output.extend_from_slice(name);
            output.push(b'=');
            output.extend_from_slice(value.to_string().as_bytes());
            output.push(b'\n');
        }
        output
    }

    #[test]
    fn committed_limits_are_hard_maxima_but_reductions_are_valid() {
        assert!(parse_limits(&profile(&LIMIT_MAXIMA)).is_ok());
        let mut reduced = LIMIT_MAXIMA;
        reduced[0] -= 1;
        assert!(parse_limits(&profile(&reduced)).is_ok());
        let mut raised = LIMIT_MAXIMA;
        raised[14] += 1;
        let failure = parse_limits(&profile(&raised)).expect_err("one over must fail");
        assert_eq!(failure.family, Family::Input);
        assert_eq!(failure.code, "limits-maximum");
    }

    #[test]
    fn manifest_identifiers_and_starts_share_the_symbol_bound() {
        let limits = parse_limits(&profile(&LIMIT_MAXIMA)).expect("profile");
        let exact = "a".repeat(limits.max_symbol_bytes);
        let cases = format!("whitefoot.grammar-cases.v1\ncase\t{exact}\texpr\t78\n");
        assert!(parse_cases(cases.as_bytes(), &limits).is_ok());
        let over = "a".repeat(limits.max_symbol_bytes + 1);
        let cases = format!("whitefoot.grammar-cases.v1\ncase\t{over}\texpr\t78\n");
        assert_eq!(
            parse_cases(cases.as_bytes(), &limits)
                .expect_err("one over")
                .code,
            "max-symbol-bytes"
        );
        let domain = b"whitefoot.grammar-domains.v1\ndomain\td\tfixed-lowerword-call\tExpr\t78\n";
        assert_eq!(
            parse_domains(domain, &limits)
                .expect_err("uppercase start")
                .code,
            "domains-record"
        );
    }
}
