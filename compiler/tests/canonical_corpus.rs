#![forbid(unsafe_code)]

//! The corpus is canonical by construction, not by assertion.
//!
//! Every `.wf` file the repository keeps as a program — the conformance cases
//! and the program corpus — is rendered from its own derivation tree and
//! compared with itself. A file that is canonical must reproduce its exact
//! bytes; a file that is deliberately not must render to something else, and
//! that something else must be canonical.
//!
//! This is what gives the renderer a permanent reason to exist and what makes
//! a scripted corpus migration safe: the transform may produce any layout that
//! parses, because this gate holds the result to FORM-2 for every file.

use std::path::{Path, PathBuf};

use whitefoot::{
    ACTIVE_KERNEL_SPEC_HASH, CanonicalOutcome, CompilerLimits, FinalizeOutcome, LexOutcome,
    ParseOutcome, RenderOutcome, SourceBundle, SourceInput, TerminalOutcome, audit_canonical,
    classify_terminals, finalize, lex, parse, render_canonical,
};

/// The two corpus roots, reached from the compiler package.
fn corpus_roots() -> [PathBuf; 2] {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler package must live directly under the repository root");
    [
        repository.join("tests").join("conformance").join("cases"),
        repository.join("tests").join("programs"),
    ]
}

/// Every corpus `.wf` file, in one stable order.
fn corpus_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    for root in corpus_roots() {
        let entries = std::fs::read_dir(&root)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", root.display()));
        for entry in entries {
            let path = entry
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", root.display()))
                .path();
            if path.extension().is_some_and(|extension| extension == "wf") {
                files.push(path);
            }
        }
    }
    files.sort();
    assert!(!files.is_empty(), "the corpus roots must hold programs");
    files
}

/// What one corpus file's own derivation says about it.
struct Reading {
    /// The file's bytes are exactly what FORM-2 requires.
    canonical: bool,
    /// The bytes the file's derivation tree denotes.
    rendered: Vec<u8>,
}

/// Derives a source and renders it, or reports that it reaches no tree.
///
/// A source issue is `None` rather than a panic: a corpus that deliberately
/// holds rejected programs must be able to hold programs that do not parse.
fn read_through_the_tree(logical_path: &str, source: &[u8]) -> Option<Reading> {
    let limits = CompilerLimits::default();
    let inputs = [SourceInput::new(logical_path, source)];
    let bundle = SourceBundle::with_limits(&inputs, limits.source)
        .unwrap_or_else(|error| panic!("{logical_path}: source envelope must be valid: {error:?}"));
    let LexOutcome::Complete(lexed) = lex(&bundle, limits.lexer) else {
        return None;
    };
    let TerminalOutcome::Complete(classified) =
        classify_terminals(&lexed, ACTIVE_KERNEL_SPEC_HASH, limits.terminals)
    else {
        return None;
    };
    let ParseOutcome::Complete(parsed) = parse(&classified, limits.parser) else {
        return None;
    };
    let FinalizeOutcome::Complete(finalized) = finalize(parsed, limits.finalizer) else {
        return None;
    };
    let rendered = match render_canonical(&finalized, limits.canonical) {
        RenderOutcome::Complete(sources) => {
            let [rendered] = <[_; 1]>::try_from(sources)
                .unwrap_or_else(|_| panic!("{logical_path}: one input renders one source"));
            rendered.bytes
        }
        other => panic!("{logical_path}: a finalized tree must render: {other:?}"),
    };
    let canonical = matches!(
        audit_canonical(finalized, limits.canonical),
        CanonicalOutcome::Complete(_)
    );
    Some(Reading {
        canonical,
        rendered,
    })
}

fn file_name(path: &Path) -> String {
    path.file_name()
        .expect("a corpus entry has a file name")
        .to_string_lossy()
        .into_owned()
}

/// The corpus files that are deliberately not canonical.
///
/// Both are FORM-2 negative cases: the corpus needs non-canonical bytes to
/// assert that FORM-2 rejects them. They are named rather than skipped by a
/// pattern so that a third one cannot appear unnoticed — a file drifting out
/// of canonical form is exactly what this gate exists to catch.
const DELIBERATELY_NONCANONICAL: &[&str] =
    &["form2-neg-noncanonical-ws.wf", "x-form-form2-tab-indent.wf"];

#[test]
fn every_canonical_corpus_file_re_renders_to_itself() {
    let mut round_tripped = Vec::new();
    let mut noncanonical = Vec::new();
    let mut underived = Vec::new();

    for path in corpus_files() {
        let name = file_name(&path);
        let source = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
        let Some(reading) = read_through_the_tree(&name, &source) else {
            underived.push(name);
            continue;
        };

        // Whatever went in, what comes out is canonical. This is the property
        // a migration relies on, so it is checked for every file rather than
        // only for the ones that were canonical already.
        let rendered_again = read_through_the_tree(&name, &reading.rendered)
            .unwrap_or_else(|| panic!("{name}: rendered bytes must derive"));
        assert!(
            rendered_again.canonical,
            "{name}: rendered bytes must be canonical"
        );
        assert_eq!(
            rendered_again.rendered, reading.rendered,
            "{name}: rendering must be idempotent"
        );

        if reading.canonical {
            assert_eq!(
                String::from_utf8_lossy(&reading.rendered),
                String::from_utf8_lossy(&source),
                "{name}: a canonical file must re-render to itself"
            );
            round_tripped.push(name);
        } else {
            assert_ne!(
                reading.rendered, source,
                "{name}: a non-canonical file must render to different bytes"
            );
            noncanonical.push(name);
        }
    }

    // Reported together: while the corpus is mid-migration the interesting
    // number is how many files still fail to derive, and stopping at the first
    // assertion would hide it.
    let total = round_tripped.len() + noncanonical.len() + underived.len();
    let report = format!(
        "{total} corpus files: {} round-tripped, {} deliberately non-canonical, {} did not derive",
        round_tripped.len(),
        noncanonical.len(),
        underived.len(),
    );
    assert!(
        underived.is_empty(),
        "{report}\nevery corpus file must derive under the active grammar; these did not: {underived:?}"
    );
    assert_eq!(
        noncanonical, DELIBERATELY_NONCANONICAL,
        "{report}\nexactly the FORM-2 negative cases may be non-canonical"
    );
    assert_eq!(
        total,
        corpus_files().len(),
        "{report}\nevery corpus file is accounted for"
    );
}

/// The anchor that locates [EX-1]'s program block in the active specification.
///
/// It is the rule's own normative sentence rather than a section number or a
/// line offset, so a renumbered section keeps working and a rewritten rule
/// fails loudly instead of matching some other fenced block.
const EX1_ANCHOR: &str = "[EX-1] The following complete program is byte-exact canonical form:";

/// [EX-1]'s program block, taken from the specification the compiler embeds.
fn ex1_block() -> &'static str {
    let occurrences = whitefoot::ACTIVE_KERNEL_SPEC_TEXT
        .matches(EX1_ANCHOR)
        .count();
    assert_eq!(
        occurrences, 1,
        "[EX-1]'s sentence must occur exactly once, or this check is reading the wrong block"
    );
    let after = whitefoot::ACTIVE_KERNEL_SPEC_TEXT
        .split_once(EX1_ANCHOR)
        .expect("the anchor was just counted")
        .1;
    let opened = after
        .split_once("```\n")
        .expect("[EX-1]'s sentence must be followed by a fenced block")
        .1;
    opened
        .split_once("```")
        .expect("[EX-1]'s fenced block must close")
        .0
}

/// The one conformance case that claims to embody [EX-1].
///
/// Identified through the manifest rather than by filename: the case is the
/// row naming `EX-1` among its rules, and exactly one row does, so renaming or
/// replacing the file cannot leave this check pointing at nothing.
fn ex1_case() -> PathBuf {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("compiler package must live directly under the repository root");
    let manifest = repository
        .join("tests")
        .join("conformance")
        .join("manifest.jsonl");
    let text = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", manifest.display()));
    let mut named: Vec<String> = Vec::new();
    for row in text.lines() {
        let row = row.trim();
        if row.is_empty() || row.starts_with('#') {
            continue;
        }
        let Some(rules) = row.split_once("\"rules\"") else {
            continue;
        };
        let rules = rules.1.split_once(']').expect("a rules array must close").0;
        if !rules.contains("\"EX-1\"") {
            continue;
        }
        let id = row
            .split_once("\"id\"")
            .expect("a case row states an id")
            .1
            .split('"')
            .nth(1)
            .expect("an id is a quoted string")
            .to_owned();
        named.push(id);
    }
    assert_eq!(
        named.len(),
        1,
        "exactly one conformance case may claim [EX-1]; found {named:?}"
    );
    repository
        .join("tests")
        .join("conformance")
        .join("cases")
        .join(format!("{}.wf", named[0]))
}

/// [EX-1] is normative bytes — §19 is titled "Worked example (normative
/// bytes)", the rule requires byte-exact canonical form, and [SCOPE-2] accepts
/// a program only if it satisfies every rule — and one conformance case exists
/// to reproduce them. Nothing compared the two until a four-byte divergence in
/// a `doc` string survived from candidate assembly into an owner review packet.
///
/// **This pins the copy to the block, never the block to reality.** It says
/// nothing about whether [EX-1]'s bytes are the right bytes. It also must not
/// be extended to `run-ex1-value-match`, an EX-1-*class* program that names no
/// EX-1 among its rules and is deliberately a superset of the block.
#[test]
fn the_case_claiming_ex1_reproduces_its_normative_bytes() {
    let case = ex1_case();
    let bytes = std::fs::read(&case)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", case.display()));
    let source = String::from_utf8(bytes).expect("a corpus file is UTF-8");
    assert_eq!(
        source,
        ex1_block(),
        "{} must reproduce [EX-1]'s program block byte for byte",
        case.display()
    );
}
