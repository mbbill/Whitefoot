use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn grammar_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("static auditor has grammar-verifier parent")
        .to_path_buf()
}

fn inputs() -> [Vec<u8>; 5] {
    let grammar = grammar_root();
    let repository = grammar.parent().expect("repository root");
    [
        fs::read(grammar.join("limits.txt")).expect("limits"),
        fs::read(repository.join("spec/kernel-spec-v0.8.md")).expect("current specification"),
        fs::read(grammar.join("proposal/kernel-spec-successor-candidate.md")).expect("proposal"),
        fs::read(grammar.join("cases.txt")).expect("cases"),
        fs::read(grammar.join("domains.txt")).expect("domains"),
    ]
}

fn inputs_with_synthetic_closed_table() -> [Vec<u8>; 5] {
    let mut values = inputs();
    values[2] = replace_once(
        &values[2],
        b"program      := item*",
        b"program      := table_probe item*",
    );
    values[2].extend_from_slice(
        concat!(
            "\n[GRAM-99] Closed-table extraction probe.\n\n",
            "```\n",
            "table_probe := EXTRANAME \"(\" IDENT \")\"\n",
            "```\n\n",
            "[FN-99] EXTRANAME is a closed table: `alpha(f)`, `beta(f)`.\n"
        )
        .as_bytes(),
    );
    values
}

fn frame(parts: &[Vec<u8>; 5]) -> Vec<u8> {
    let mut output = b"WFGRAMV1".to_vec();
    for part in parts {
        output.extend_from_slice(&(part.len() as u64).to_be_bytes());
    }
    for part in parts {
        output.extend_from_slice(part);
    }
    output
}

fn report(parts: &[Vec<u8>; 5]) -> String {
    String::from_utf8(whitefoot_static_grammar_auditor::process_frame(&frame(
        parts,
    )))
    .expect("report is ASCII")
}

fn replace_once(bytes: &[u8], from: &[u8], to: &[u8]) -> Vec<u8> {
    let offsets = bytes
        .windows(from.len())
        .enumerate()
        .filter_map(|(offset, window)| (window == from).then_some(offset))
        .collect::<Vec<_>>();
    assert_eq!(offsets.len(), 1, "mutation anchor must be unique");
    let offset = offsets[0];
    let mut output = Vec::new();
    output.extend_from_slice(&bytes[..offset]);
    output.extend_from_slice(to);
    output.extend_from_slice(&bytes[offset + from.len()..]);
    output
}

fn ascii_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let digit = |byte: u8| match byte {
                b'0'..=b'9' => byte - b'0',
                b'a'..=b'f' => byte - b'a' + 10,
                _ => panic!("non-canonical test hex"),
            };
            (digit(pair[0]) << 4) | digit(pair[1])
        })
        .collect()
}

#[test]
fn reviewed_partition_removes_the_real_deref_collision() {
    let output = report(&inputs());
    assert!(!output.contains("\nFAIL\t"), "{output}");
    assert!(output.contains("STATIC-CASE\tcurrent\tderef-p\t65787072\t6465726566287029\t1\n"));
    assert!(output.contains("STATIC-CASE\tproposal\tderef-p\t65787072\t6465726566287029\t0\n"));
    assert!(output.contains(concat!(
        "STATIC-DOMAIN\tcurrent\t66697865642d6c6f776572776f72642d63616c6c73\t",
        "65787072\t78\t48\tf3e54408ce7c4234bb3b61e27f2decd6c84ffcc4d7fb1b201c9583dd0190480c\n"
    )));
    assert!(output.contains(concat!(
        "STATIC-TRANSITION\tfixed-ident-partition\t1\t0\t",
        "removes-call-through-ident\t6465726566287829\n"
    )));

    let mut source_lowerwords = BTreeSet::new();
    let mut expanded_lowerwords = BTreeSet::new();
    for line in output.lines() {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.first() != Some(&"FIXED") || fields.get(1) != Some(&"current") {
            continue;
        }
        let spelling = decode_hex(fields[6]);
        if spelling.first().is_some_and(u8::is_ascii_lowercase)
            && spelling
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
        {
            source_lowerwords.insert(spelling);
        }
        let descriptor = String::from_utf8(decode_hex(fields[7])).expect("descriptor ASCII");
        for atom in descriptor.split(',') {
            if let Some(value) = atom.strip_prefix("lowerword:") {
                expanded_lowerwords.insert(decode_hex(value));
            }
        }
    }
    assert_eq!(source_lowerwords.len(), 47);
    assert_eq!(expanded_lowerwords.len(), 48);
    assert_eq!(
        expanded_lowerwords
            .difference(&source_lowerwords)
            .cloned()
            .collect::<Vec<_>>(),
        vec![b"uniq".to_vec()]
    );
}

#[test]
fn replacement_proposal_has_no_strong_ll2_conflict() {
    let output = report(&inputs());
    assert!(!output.contains("\nFAIL\t"), "{output}");
    assert!(
        !output
            .lines()
            .any(|line| line.starts_with("STATIC-CONFLICT\tproposal\t")),
        "proposal must have zero strong-LL(2) conflicts"
    );
    for identifier in [
        "ordinary-let",
        "requires-value-match",
        "statement-match",
        "try-let",
        "value-match",
    ] {
        assert!(
            output.lines().any(|line| {
                line.starts_with(&format!("STATIC-CASE\tproposal\t{identifier}\t"))
                    && line.ends_with("\t0")
            }),
            "proposal case {identifier} must be conflict-free"
        );
    }
}

#[test]
fn literal_membership_matches_the_grammar_layer_exactly() {
    let output = report(&inputs());
    let string = ascii_hex(format!("lex:{}", ascii_hex(b"STRING")).as_bytes());
    let literal = ascii_hex(format!("lex:{}", ascii_hex(b"literal")).as_bytes());
    assert!(!output.lines().any(|line| {
        line.starts_with("STATIC-INTERSECTION\t")
            && line.contains(&string)
            && line.contains(&literal)
    }));

    let mut values = inputs();
    values[2] = replace_once(
        &values[2],
        b"program      := item*",
        b"program      := numeric_probe item*",
    );
    values[2].extend_from_slice(
        b"\n[GRAM-99] Numeric terminal-shape regression.\n\n```\nnumeric_probe := \"-1_u8\"\n```\n",
    );
    let output = report(&values);
    assert!(!output.contains("\nFAIL\t"), "{output}");
    let fixed = ascii_hex(format!("fixed:{}", ascii_hex(b"-1_u8")).as_bytes());
    assert!(output.lines().any(|line| {
        line.starts_with("STATIC-INTERSECTION\tproposal\t")
            && line.contains(&fixed)
            && line.contains(&literal)
    }));
}

#[test]
fn float_membership_preserves_the_successor_grammar_and_v08_descriptor() {
    let mut values = inputs();
    let replacement = b"program      := float_probe item*";
    let probe = concat!(
        "\n[GRAM-99] Float terminal membership probes.\n\n",
        "```\n",
        "float_probe := \"0.0_f32\" | \"-1.50e-0_f64\" | \"01.5_f32\" | ",
        "\"1.5e01_f64\" | \"1.5e-01_f64\" | \"1.5e+1_f64\"\n",
        "```\n"
    )
    .as_bytes();
    values[2] = replace_once(&values[2], b"program      := item*", replacement);
    values[2].extend_from_slice(probe);
    let output = report(&values);
    assert!(!output.contains("\nFAIL\t"), "{output}");
    let current_predicate = concat!(
        "integer=-?[0-9]+_TYPE;float=-?[0-9]+\\.[0-9]+(e-?[0-9]+)?_TYPE;",
        "unit=unit;generic=0_T,1_T"
    );
    let proposal_predicate = concat!(
        "integer=-?[0-9]+_TYPE;",
        "float=-?(0|[1-9][0-9]*)\\.[0-9]+(e-?(0|[1-9][0-9]*))?_TYPE;",
        "float-value=signed-zero-or-sign*C*10^(E-F);e-0=0;round=ieee-rne;",
        "canonical=min-prefix-bytes,ascii-lex;finite=required;",
        "unit=unit;generic=0_T,1_T"
    );
    for (document, predicate) in [
        ("current", current_predicate),
        ("proposal", proposal_predicate),
    ] {
        let prefix = format!(
            "LEX\t{document}\t{}\t{}\tliteral-union\t",
            ascii_hex(b"FORM-5"),
            ascii_hex(b"literal")
        );
        assert!(output.lines().any(|line| {
            line.starts_with(&prefix) && line.ends_with(&ascii_hex(predicate.as_bytes()))
        }));
    }
    let literal = ascii_hex(format!("lex:{}", ascii_hex(b"literal")).as_bytes());
    let has_intersection = |document: &str, spelling: &[u8]| {
        let fixed = ascii_hex(format!("fixed:{}", ascii_hex(spelling)).as_bytes());
        output.lines().any(|line| {
            line.starts_with(&format!("STATIC-INTERSECTION\t{document}\t"))
                && line.contains(&fixed)
                && line.contains(&literal)
        })
    };

    for spelling in [b"0.0_f32".as_slice(), b"-1.50e-0_f64"] {
        assert!(has_intersection("proposal", spelling));
    }
    for spelling in [b"01.5_f32".as_slice(), b"1.5e01_f64", b"1.5e-01_f64"] {
        assert!(!has_intersection("proposal", spelling));
    }
    assert!(!has_intersection("proposal", b"1.5e+1_f64"));
}

#[test]
fn proposal_float_contract_clauses_fail_closed_under_hostile_mutation() {
    let mutations: &[(&[u8], &[u8], &str)] = &[
        (
            b"`f32` (IEEE 754 binary32) or `f64` (IEEE 754 binary64)",
            b"`f32` (IEEE 754 binary16) or `f64` (IEEE 754 binary64)",
            "float-contract-form5",
        ),
        (
            b"Let C be the nonnegative integer formed by concatenating",
            b"Let C be the nonpositive integer formed by concatenating",
            "float-contract-form5",
        ),
        (
            b"let F be the number of fraction digits",
            b"let F be the number of integer digits",
            "float-contract-form5",
        ),
        (
            b"let E be the signed integer formed by the exponent digits",
            b"let E be the unsigned integer formed by the exponent digits",
            "float-contract-form5",
        ),
        (
            "magnitude is C × 10^(E − F)".as_bytes(),
            "magnitude is C × 10^(E + F)".as_bytes(),
            "float-contract-form5",
        ),
        (
            b"a leading literal `-` selects negative zero and its absence selects positive zero",
            b"a leading literal `-` selects positive zero and its absence selects negative zero",
            "float-contract-form5",
        ),
        (
            b"IEEE 754 round-to-nearest, ties-to-even",
            b"IEEE 754 round-to-nearest, ties-away",
            "float-contract-form5",
        ),
        (
            b"fewest ASCII bytes before `_TYPE`",
            b"most ASCII bytes before `_TYPE`",
            "float-contract-form5",
        ),
        (
            b"lexicographically least unsigned ASCII bytes",
            b"lexicographically greatest unsigned ASCII bytes",
            "float-contract-form5",
        ),
        (
            b"denotes a finite value of its stated TYPE",
            b"denotes a non-finite value of its stated TYPE",
            "float-contract-form7",
        ),
    ];
    for (from, to, failure) in mutations {
        let mut values = inputs();
        values[2] = replace_once(&values[2], from, to);
        let output = report(&values);
        assert!(
            output.contains(&format!("FAIL\textraction\t{failure}\n")),
            "mutation {:?} was not rejected as {failure}: {output}",
            String::from_utf8_lossy(from)
        );
    }
}

#[test]
fn proposal_production_owners_pin_pre_tree_diagnostic_attribution() {
    let output = report(&inputs());
    assert!(!output.contains("\nFAIL\t"), "{output}");
    for (production, owner) in [
        ("program", "GRAM-2"),
        ("requires_block", "GRAM-2"),
        ("requires_entry", "GRAM-2"),
        ("law", "GRAM-2"),
        ("const", "CONST-1"),
        ("cvalue", "CONST-2"),
        ("effects", "EFF-1"),
        ("effect", "EFF-1"),
    ] {
        let prefix = format!(
            "PROD\tproposal\t{}\t{}\t",
            ascii_hex(owner.as_bytes()),
            ascii_hex(production.as_bytes())
        );
        assert!(
            output.lines().any(|line| line.starts_with(&prefix)),
            "missing exact owner {owner} for {production}"
        );
    }
}

#[test]
fn grammar_candidate_census_fails_closed() {
    let mut raw = inputs();
    raw[2].extend_from_slice(b"\n[GRAM-99] Raw candidate.\n\nrogue = \"x\"\n");
    assert!(report(&raw).contains("FAIL\textraction\tgrammar-assignment-unsupported\n"));

    let mut raw_without_spaces = inputs();
    raw_without_spaces[2].extend_from_slice(b"\n[GRAM-99] Raw compact candidate.\n\nrogue=\"x\"\n");
    assert!(
        report(&raw_without_spaces).contains("FAIL\textraction\tgrammar-assignment-unsupported\n")
    );

    let mut wrong_owner = inputs();
    wrong_owner[2]
        .extend_from_slice(b"\n[FORM-99] Wrong grammar owner.\n\n```\nrogue := \"x\"\n```\n");
    assert!(report(&wrong_owner).contains("FAIL\textraction\tgrammar-fence-owner\n"));
}

#[test]
fn every_accepted_production_is_reachable_from_program() {
    let mut values = inputs();
    values[2]
        .extend_from_slice(b"\n[GRAM-99] Unreachable production.\n\n```\nrogue := \"x\"\n```\n");
    assert!(report(&values).contains("FAIL\textraction\tproduction-unreachable\n"));
}

#[test]
fn report_collections_honor_the_cumulative_output_limit() {
    let mut values = inputs();
    values[0] = replace_once(
        &values[0],
        b"max_engine_output_bytes=8388608",
        b"max_engine_output_bytes=1024",
    );
    assert_eq!(
        report(&values),
        "WFGRREPORT1\nENGINE\tstatic\nFAIL\tresource\tmax-engine-output-bytes\nEND\n"
    );
}

#[test]
fn named_transition_is_bound_to_the_exact_registered_probe() {
    let original = inputs();
    let mut wrong_start = original.clone();
    wrong_start[3] = replace_once(
        &wrong_start[3],
        b"case\tderef-x\texpr\t6465726566287829",
        b"case\tderef-x\tprogram\t6465726566287829",
    );
    assert!(report(&wrong_start).contains("FAIL\tinput\ttransition-case\n"));

    let mut wrong_source = original.clone();
    wrong_source[3] = replace_once(
        &wrong_source[3],
        b"case\tderef-x\texpr\t6465726566287829",
        b"case\tderef-x\texpr\t6465726566287929",
    );
    assert!(report(&wrong_source).contains("FAIL\tinput\ttransition-case\n"));

    let mut unrelated = original;
    unrelated[2] = replace_once(
        &unrelated[2],
        b"expr           := atom | call | construct",
        b"expr           := rogue | atom | call | construct",
    );
    unrelated[2].extend_from_slice(
        concat!(
            "\n[GRAM-99] Reachable unrelated collision.\n\n",
            "```\n",
            "rogue := \"deref\" \"(\" IDENT \")\" | \"deref\" \"(\" IDENT \")\"\n",
            "```\n"
        )
        .as_bytes(),
    );
    let output = report(&unrelated);
    assert!(!output.contains("\nFAIL\t"), "{output}");
    assert!(output.contains(concat!(
        "STATIC-TRANSITION\tfixed-ident-partition\t1\t2\t",
        "does-not-remove-call-through-ident\t6465726566287829\n"
    )));
}

#[test]
fn class_patterns_annotations_and_table_signatures_fail_closed() {
    let original = inputs();
    let mut pattern = original.clone();
    pattern[2] = replace_once(
        &pattern[2],
        b"IDENT `[a-z][a-z0-9_]*` excluding",
        b"IDENT `[A-Z][a-z0-9_]*` excluding",
    );
    assert!(report(&pattern).contains("FAIL\textraction\tlexical-class-pattern\n"));

    let mut annotation = original.clone();
    annotation[2] = replace_once(
        &annotation[2],
        b"apostrophe-prefixed, the only region spelling",
        b"apostrophe prefixed, the only region spelling",
    );
    assert!(report(&annotation).contains("FAIL\textraction\tlexical-class-annotation\n"));

    let mut table = inputs_with_synthetic_closed_table();
    table[2] = replace_once(&table[2], b"alpha(f)", b"alpha(foo)");
    assert!(report(&table).contains("FAIL\textraction\tclosed-table-signature\n"));
}

#[test]
fn generic_closed_table_names_are_extracted_without_a_lawname_special_case() {
    let values = inputs_with_synthetic_closed_table();
    let output = report(&values);
    assert!(!output.contains("\nFAIL\t"), "{output}");
    let prefix = format!(
        "LEX\tproposal\t{}\t{}\tclosed-table\t",
        ascii_hex(b"FN-99"),
        ascii_hex(b"EXTRANAME")
    );
    assert!(output.lines().any(|line| {
        line.starts_with(&prefix) && line.ends_with(&ascii_hex(b"alpha(f),beta(f)"))
    }));
}

#[test]
fn terminal_after_two_fixed_tokens_remains_in_the_intersection_census() {
    let mut values = inputs();
    values[2] = replace_once(
        &values[2],
        b"program      := item*",
        b"program      := probe item*",
    );
    values[2].extend_from_slice(
        b"\n[GRAM-99] Static terminal-census regression:\n\n```\nprobe := \"a\" \"b\" \"hidden\"\n```\n",
    );
    let output = report(&values);
    assert!(!output.contains("\nFAIL\t"), "{output}");
    let descriptor = "fixed:68696464656e";
    let descriptor_hex = descriptor
        .bytes()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    assert!(output.contains(&format!(
        "STATIC-INTERSECTION\tproposal\t{descriptor_hex}\t{descriptor_hex}\t68696464656e\n"
    )));
}

#[test]
fn ll2_relations_distinguish_the_second_lookahead_token() {
    let mut separable = inputs();
    separable[2] = replace_once(
        &separable[2],
        b"program      := item*",
        b"program      := ll2_probe item*",
    );
    separable[2].extend_from_slice(
        concat!(
            "\n[GRAM-99] LL(2) relation regression.\n\n",
            "```\n",
            "ll2_probe := \"same\" \"left\" | \"same\" \"right\"\n",
            "```\n"
        )
        .as_bytes(),
    );
    let separable_output = report(&separable);
    assert!(!separable_output.contains("\nFAIL\t"), "{separable_output}");

    let production_prefix = format!(
        "PROD\tproposal\t{}\t{}\t",
        ascii_hex(b"GRAM-99"),
        ascii_hex(b"ll2_probe")
    );
    assert!(separable_output.contains(&production_prefix));
    let conflict_prefix = format!("STATIC-CONFLICT\tproposal\t{}\t", ascii_hex(b"ll2_probe"));
    assert_eq!(
        separable_output
            .lines()
            .filter(|line| line.starts_with(&conflict_prefix))
            .count(),
        0,
        "different second tokens must separate the alternatives"
    );

    let mut colliding = inputs();
    colliding[2] = replace_once(
        &colliding[2],
        b"program      := item*",
        b"program      := ll2_probe item*",
    );
    colliding[2].extend_from_slice(
        concat!(
            "\n[GRAM-99] LL(2) relation regression.\n\n",
            "```\n",
            "ll2_probe := \"same\" \"left\" | \"same\" \"left\"\n",
            "```\n"
        )
        .as_bytes(),
    );
    let colliding_output = report(&colliding);
    assert!(!colliding_output.contains("\nFAIL\t"), "{colliding_output}");
    assert!(colliding_output.contains(&production_prefix));
    assert_eq!(
        colliding_output
            .lines()
            .filter(|line| line.starts_with(&conflict_prefix))
            .count(),
        1,
        "identical two-token lookahead words must collide"
    );
}

#[test]
fn optional_at_the_program_end_emits_the_eof_exit_decision() {
    let mut values = inputs();
    values[2] = replace_once(
        &values[2],
        b"program      := item*",
        b"program      := item* \"stop\"?",
    );
    let output = report(&values);
    assert!(!output.contains("\nFAIL\t"), "{output}");
    assert!(output.contains(&format!(
        "STATIC-DECISION\tproposal\t{}\t0.1\toptional\t1\t{}\n",
        ascii_hex(b"program"),
        ascii_hex(b"end,end")
    )));
}
