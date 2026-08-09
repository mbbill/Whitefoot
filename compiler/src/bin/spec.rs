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
        computed_active_spec_hash, is_rule_id, validate_activation_chain, validate_spec_integrity,
    };

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
            Ok(16)
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
            Ok(128)
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
