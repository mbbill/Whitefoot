#![forbid(unsafe_code)]

use std::collections::BTreeSet;

use whitefoot::{KERNEL_SPEC_V0_14_HASH, SYNTAX_DATA_SPEC_V0_14, TERMINAL_CONTRACT_SPEC_V0_14};

const ACTIVE_SPEC: &[u8] = include_bytes!("../../../spec/kernel-spec-v0.14.md");
const ACTIVE_SPEC_TEXT: &str = include_str!("../../../spec/kernel-spec-v0.14.md");
const APPROVED_CANDIDATE: &[u8] =
    include_bytes!("../../../governance/spec-evolution/kernel-spec-v0.14-candidate.md");
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

fn ledger_rule_ids(text: &str) -> BTreeSet<&str> {
    text.lines()
        .filter_map(|line| {
            let rest = line.strip_prefix("| ")?;
            let (candidate, _) = rest.split_once(" |")?;
            is_rule_id(candidate).then_some(candidate)
        })
        .collect()
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
    if ACTIVE_SPEC != APPROVED_CANDIDATE {
        eprintln!("spec/kernel-spec-v0.14.md differs from the approved candidate");
        std::process::exit(1);
    }
    if SYNTAX_DATA_SPEC_V0_14 != KERNEL_SPEC_V0_14_HASH
        || TERMINAL_CONTRACT_SPEC_V0_14 != KERNEL_SPEC_V0_14_HASH
    {
        eprintln!("frontend data is not bound to the active v0.14 identity");
        std::process::exit(1);
    }
    let rule_count = match validate_spec_integrity(ACTIVE_SPEC_TEXT, DERIVATION_LEDGER) {
        Ok(rule_count) => rule_count,
        Err(errors) => {
            for error in errors {
                eprintln!("spec integrity: {error}");
            }
            std::process::exit(1);
        }
    };
    println!("Whitefoot v0.14 frontend identity: {KERNEL_SPEC_V0_14_HASH}");
    println!("Whitefoot v0.14 spec integrity: {rule_count} rules");
}

#[cfg(test)]
mod tests {
    use super::{ACTIVE_SPEC_TEXT, DERIVATION_LEDGER, is_rule_id, validate_spec_integrity};

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
            validate_spec_integrity(ACTIVE_SPEC_TEXT, DERIVATION_LEDGER),
            Ok(93)
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
