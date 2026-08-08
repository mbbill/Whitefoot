//! Migrating the Whitefoot fixtures embedded in Rust string literals.
//!
//! This module does no rewriting of its own. It locates a literal, decodes it
//! to the bytes the test actually feeds the compiler, hands those bytes to
//! [`super::migrate`] — the same function that migrated the corpus — and
//! re-encodes the result in the literal form it came from. Every spelling
//! decision therefore has exactly one implementation.
//!
//! Two gates keep it from touching a fixture whose subject is its own surface
//! form. A literal is rewritten only when the *pre-pass* changed its bytes, so
//! a fixture already in v0.23 is never re-rendered and a deliberately
//! non-canonical source is never canonicalized. And a rewrite is written back
//! only when decoding its own re-encoding reproduces the migrated bytes
//! exactly, so an escaping this module cannot round-trip is reported instead of
//! guessed at.

use std::path::Path;

use super::{lex_source, render, rewrite};

/// How a Rust string literal spells its content.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Form {
    /// `"…"` or `b"…"`, with backslash escapes.
    Escaped { byte: bool },
    /// `r#"…"#` or `br#"…"#`, verbatim, with `hashes` pound signs.
    Raw { byte: bool, hashes: usize },
}

/// One located literal: the whole token, and the content between delimiters.
struct Literal {
    end: usize,
    content_start: usize,
    content_end: usize,
    form: Form,
}

/// What happened to one literal, for the report.
pub(crate) struct Outcome {
    pub(crate) line: usize,
    pub(crate) state: State,
}

pub(crate) enum State {
    /// Migrated: the counts, and how many of its lines the rewrite touched.
    Migrated {
        counts: rewrite::Counts,
        changed_lines: usize,
        total_lines: usize,
    },
    /// Carries a v0.22 spelling the pre-pass rewrote, but the result could not
    /// be written back. The reason is the report's whole point.
    Blocked { reason: String },
    /// Held back by a [`KEEP_MARKER`] at the site, because the fixture's own
    /// surface form is what its test asserts about.
    Kept,
}

/// The comment that holds a fixture back.
///
/// A fixture whose subject is its own surface form — a Bool `match` under a
/// [GRAM-6] rejection, a forbidden form under detection — is destroyed rather
/// than migrated by a spelling rewrite, and the destruction is silent because
/// the test keeps compiling. The marker records that decision at the site so a
/// later run cannot undo it, and it must carry the reason after it.
const KEEP_MARKER: &[u8] = b"migrate: keep";

/// How far above a literal the marker may sit.
///
/// A fixture is often nested a couple of call lines deep inside the assertion
/// that reads it, so the marker goes on the statement rather than on the
/// literal's own line.
const KEEP_WINDOW_LINES: usize = 3;

/// Migrates every embedded fixture in one Rust source.
///
/// Returns the new bytes and one outcome per literal the pre-pass touched.
/// A literal the pre-pass left alone produces no outcome at all: it holds no
/// v0.22 spelling, so this module has no business re-rendering it.
pub(crate) fn migrate_rust(source: &[u8], path: &Path) -> Result<(Vec<u8>, Vec<Outcome>), String> {
    let literals = literals(source)
        .map_err(|reason| format!("{}: cannot scan Rust literals: {reason}", path.display()))?;
    let mut out = Vec::with_capacity(source.len());
    let mut outcomes = Vec::new();
    let mut cursor = 0;
    for literal in literals {
        let raw = &source[literal.content_start..literal.content_end];
        let Some(decoded) = decode(raw, literal.form) else {
            // An escape this module does not model. Silent unless the literal
            // turns out to hold a fixture, which cannot be known without
            // decoding it — so it is reported by the caller's marker sweep.
            continue;
        };
        let line = 1 + source[..literal.content_start]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count();
        if is_kept(source, literal.content_start) {
            outcomes.push(Outcome {
                line,
                state: State::Kept,
            });
            continue;
        }
        match migrate_fixture(&decoded, path) {
            FixtureOutcome::NotAFixture => continue,
            FixtureOutcome::Blocked(reason) => {
                outcomes.push(Outcome {
                    line,
                    state: State::Blocked { reason },
                });
            }
            FixtureOutcome::Migrated { bytes, counts } => {
                let Some(encoded) = encode(&bytes, literal.form) else {
                    outcomes.push(Outcome {
                        line,
                        state: State::Blocked {
                            reason: "the migrated bytes cannot be spelled in this literal form"
                                .to_owned(),
                        },
                    });
                    continue;
                };
                // The round-trip is verified rather than trusted: what the
                // compiler will read back must equal what the tool produced.
                if decode(&encoded, literal.form).as_deref() != Some(&bytes[..]) {
                    outcomes.push(Outcome {
                        line,
                        state: State::Blocked {
                            reason: "re-encoding does not round-trip byte-exactly".to_owned(),
                        },
                    });
                    continue;
                }
                let (changed_lines, total_lines) = line_delta(&decoded, &bytes);
                out.extend_from_slice(&source[cursor..literal.content_start]);
                out.extend_from_slice(&encoded);
                cursor = literal.content_end;
                outcomes.push(Outcome {
                    line,
                    state: State::Migrated {
                        counts,
                        changed_lines,
                        total_lines,
                    },
                });
            }
        }
    }
    out.extend_from_slice(&source[cursor..]);
    Ok((out, outcomes))
}

enum FixtureOutcome {
    /// Does not lex as Whitefoot, or holds no v0.22 spelling.
    NotAFixture,
    Blocked(String),
    Migrated {
        bytes: Vec<u8>,
        counts: rewrite::Counts,
    },
}

/// One decoded literal put through the corpus migration.
fn migrate_fixture(decoded: &[u8], path: &Path) -> FixtureOutcome {
    let display = path.display().to_string();
    let Ok(lexemes) = lex_source(decoded, &display) else {
        return FixtureOutcome::NotAFixture;
    };
    let (pre_pass, counts) = match rewrite::pre_pass(decoded, &lexemes) {
        Ok(result) => result,
        Err(reason) => return FixtureOutcome::Blocked(reason),
    };
    // The gate: no v0.22 spelling class fired, so there is nothing to migrate
    // and re-rendering would only churn — or destroy — the fixture's layout.
    if pre_pass == decoded {
        return FixtureOutcome::NotAFixture;
    }
    match render(&pre_pass, &display) {
        Ok(bytes) => FixtureOutcome::Migrated { bytes, counts },
        Err(reason) => FixtureOutcome::Blocked(reason),
    }
}

/// Whether a [`KEEP_MARKER`] governs the literal whose content starts here.
///
/// The window is the literal's own line plus the [`KEEP_WINDOW_LINES`] above
/// it, so the marker can sit on the assertion that reads the fixture.
fn is_kept(source: &[u8], content_start: usize) -> bool {
    let mut window_start = content_start;
    for _ in 0..=KEEP_WINDOW_LINES {
        match source[..window_start]
            .iter()
            .rposition(|byte| *byte == b'\n')
        {
            Some(newline) => window_start = newline,
            None => {
                window_start = 0;
                break;
            }
        }
    }
    source[window_start..content_start]
        .windows(KEEP_MARKER.len())
        .any(|window| window == KEEP_MARKER)
}

/// Lines that differ between two versions of a fixture, and the new line count.
///
/// A spelling migration touches a few lines; a whole-fixture re-layout touches
/// nearly all of them, so this ratio is what makes the second case visible in
/// the report instead of hiding inside a large diff.
fn line_delta(before: &[u8], after: &[u8]) -> (usize, usize) {
    let before: Vec<_> = before.split(|byte| *byte == b'\n').collect();
    let after: Vec<_> = after.split(|byte| *byte == b'\n').collect();
    let common = before
        .iter()
        .zip(after.iter())
        .filter(|(left, right)| left == right)
        .count();
    (after.len().max(before.len()) - common, after.len())
}

fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// Every string literal in a Rust source, in order.
///
/// Comments and character literals are skipped so a quotation mark inside one
/// cannot be mistaken for a delimiter. Lifetimes are the one ambiguity: `'a` is
/// not a character literal, so a `'` that does not close is consumed as one
/// byte.
fn literals(source: &[u8]) -> Result<Vec<Literal>, String> {
    let mut out = Vec::new();
    let mut index = 0;
    while index < source.len() {
        let byte = source[index];
        if byte == b'/' && source.get(index + 1) == Some(&b'/') {
            index = line_end(source, index);
            continue;
        }
        if byte == b'/' && source.get(index + 1) == Some(&b'*') {
            index = block_comment_end(source, index)?;
            continue;
        }
        if is_ident_byte(byte) {
            let word_start = index;
            while index < source.len() && is_ident_byte(source[index]) {
                index += 1;
            }
            let word = &source[word_start..index];
            if matches!(word, b"b" | b"r" | b"br") && matches!(source.get(index), Some(b'"' | b'#'))
            {
                let literal = literal_at(source, word, index)?;
                index = literal.end;
                out.push(literal);
                continue;
            }
            if word == b"b" && source.get(index) == Some(&b'\'') {
                index = character_end(source, index).unwrap_or(index + 1);
            }
            continue;
        }
        if byte == b'"' {
            let literal = literal_at(source, b"", index)?;
            index = literal.end;
            out.push(literal);
            continue;
        }
        if byte == b'\'' {
            index = character_end(source, index).unwrap_or(index + 1);
            continue;
        }
        index += 1;
    }
    Ok(out)
}

fn line_end(source: &[u8], from: usize) -> usize {
    source[from..]
        .iter()
        .position(|byte| *byte == b'\n')
        .map_or(source.len(), |offset| from + offset + 1)
}

/// Past a `/* … */`, which Rust nests.
fn block_comment_end(source: &[u8], from: usize) -> Result<usize, String> {
    let mut depth = 0_usize;
    let mut index = from;
    while index + 1 < source.len() {
        match (source[index], source[index + 1]) {
            (b'/', b'*') => {
                depth += 1;
                index += 2;
            }
            (b'*', b'/') => {
                depth -= 1;
                index += 2;
                if depth == 0 {
                    return Ok(index);
                }
            }
            _ => index += 1,
        }
    }
    Err("an unterminated block comment".to_owned())
}

/// Past a `'x'` or `b'x'`, or `None` when the quote opens a lifetime.
fn character_end(source: &[u8], quote: usize) -> Option<usize> {
    let mut index = quote + 1;
    if source.get(index) == Some(&b'\\') {
        index += 2;
        // `\x41`, and the braced Unicode form.
        if source.get(quote + 2) == Some(&b'x') {
            index += 2;
        } else if source.get(quote + 2) == Some(&b'u') {
            index = source[index..]
                .iter()
                .position(|byte| *byte == b'}')
                .map(|offset| index + offset + 1)?;
        }
    } else {
        // One UTF-8 scalar.
        index += 1;
        while source.get(index).is_some_and(|byte| byte & 0xC0 == 0x80) {
            index += 1;
        }
    }
    (source.get(index) == Some(&b'\'')).then_some(index + 1)
}

/// The literal whose prefix is `prefix` and whose delimiter run starts at
/// `after_prefix`.
fn literal_at(source: &[u8], prefix: &[u8], after_prefix: usize) -> Result<Literal, String> {
    let byte = prefix.contains(&b'b');
    if !prefix.contains(&b'r') {
        let content_start = after_prefix + 1;
        let mut index = content_start;
        while index < source.len() {
            match source[index] {
                b'\\' => index += 2,
                b'"' => {
                    return Ok(Literal {
                        end: index + 1,
                        content_start,
                        content_end: index,
                        form: Form::Escaped { byte },
                    });
                }
                _ => index += 1,
            }
        }
        return Err("an unterminated string literal".to_owned());
    }
    let hashes = source[after_prefix..]
        .iter()
        .take_while(|byte| **byte == b'#')
        .count();
    if source.get(after_prefix + hashes) != Some(&b'"') {
        return Err("a raw string prefix reached no quote".to_owned());
    }
    let content_start = after_prefix + hashes + 1;
    let mut index = content_start;
    while index < source.len() {
        if source[index] == b'"'
            && source[index + 1..]
                .iter()
                .take(hashes)
                .filter(|byte| **byte == b'#')
                .count()
                == hashes
        {
            return Ok(Literal {
                end: index + 1 + hashes,
                content_start,
                content_end: index,
                form: Form::Raw { byte, hashes },
            });
        }
        index += 1;
    }
    Err("an unterminated raw string literal".to_owned())
}

/// The bytes a literal's content denotes, or `None` for an escape this module
/// does not model.
fn decode(content: &[u8], form: Form) -> Option<Vec<u8>> {
    let Form::Escaped { .. } = form else {
        return Some(content.to_vec());
    };
    let mut out = Vec::with_capacity(content.len());
    let mut index = 0;
    while index < content.len() {
        if content[index] != b'\\' {
            out.push(content[index]);
            index += 1;
            continue;
        }
        index += 1;
        match content.get(index)? {
            b'n' => out.push(b'\n'),
            b'r' => out.push(b'\r'),
            b't' => out.push(b'\t'),
            b'0' => out.push(0),
            b'\\' => out.push(b'\\'),
            b'\'' => out.push(b'\''),
            b'"' => out.push(b'"'),
            b'x' => {
                let digits = content.get(index + 1..index + 3)?;
                let text = std::str::from_utf8(digits).ok()?;
                out.push(u8::from_str_radix(text, 16).ok()?);
                index += 2;
            }
            // The line continuation, and the braced Unicode escape, would both
            // need care this module has no fixture for.
            _ => return None,
        }
        index += 1;
    }
    Some(out)
}

/// Spells `bytes` as the content of a literal of `form`, or `None` when that
/// form cannot hold them.
fn encode(bytes: &[u8], form: Form) -> Option<Vec<u8>> {
    match form {
        Form::Raw { hashes, .. } => {
            // The content must not spell its own terminator.
            let terminator: Vec<u8> = std::iter::once(b'"')
                .chain(std::iter::repeat_n(b'#', hashes))
                .collect();
            if bytes
                .windows(terminator.len())
                .any(|w| w == &terminator[..])
                || bytes.contains(&b'\r')
            {
                return None;
            }
            Some(bytes.to_vec())
        }
        Form::Escaped { byte } => {
            let mut out = Vec::with_capacity(bytes.len() + 16);
            for value in bytes {
                match value {
                    b'\n' => out.extend_from_slice(b"\\n"),
                    b'\r' => out.extend_from_slice(b"\\r"),
                    b'\t' => out.extend_from_slice(b"\\t"),
                    b'"' => out.extend_from_slice(b"\\\""),
                    b'\\' => out.extend_from_slice(b"\\\\"),
                    0x20..=0x7E => out.push(*value),
                    // A `str` literal cannot spell a byte escape above 0x7F,
                    // and its content is already valid UTF-8, so it passes
                    // through; a byte string spells it.
                    _ if byte => out.extend_from_slice(format!("\\x{value:02x}").as_bytes()),
                    _ => out.push(*value),
                }
            }
            Some(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Form, State, decode, encode, literals, migrate_rust};

    /// The one Rust source shape this module is built for: a fixture inside a
    /// test, held back or not by the marker above it.
    fn rust_with(marker: &str, fixture: &str) -> Vec<u8> {
        format!("#[test]\nfn a_test() {{\n{marker}    let source = br#\"{fixture}\"#;\n}}\n")
            .into_bytes()
    }

    fn outcome_states(source: &[u8]) -> Vec<&'static str> {
        let (_, outcomes) =
            migrate_rust(source, std::path::Path::new("test.rs")).expect("migrates");
        outcomes
            .iter()
            .map(|outcome| match outcome.state {
                State::Migrated { .. } => "migrated",
                State::Blocked { .. } => "blocked",
                State::Kept => "kept",
            })
            .collect()
    }

    const ANNOTATED: &str =
        "fn main() -> own unit pure {\n  let value: own u64 = 1_u64;\n  return unit;\n}\n";

    /// Both halves in one test, because a marker that is never read and a
    /// marker that is always read look identical from the migrated side.
    #[test]
    fn the_marker_holds_a_fixture_back_and_its_absence_does_not() {
        let unmarked = rust_with("", ANNOTATED);
        assert_eq!(outcome_states(&unmarked), ["migrated"]);
        let (rewritten, _) = migrate_rust(&unmarked, std::path::Path::new("test.rs")).expect("ok");
        assert!(
            String::from_utf8_lossy(&rewritten).contains("let value = 1_u64;"),
            "{}",
            String::from_utf8_lossy(&rewritten)
        );

        let marked = rust_with("    // migrate: keep — a reason.\n", ANNOTATED);
        assert_eq!(outcome_states(&marked), ["kept"]);
        let (untouched, _) = migrate_rust(&marked, std::path::Path::new("test.rs")).expect("ok");
        assert_eq!(untouched, marked);
    }

    /// The marker reaches over the assertion lines a fixture is usually nested
    /// inside, and no further.
    #[test]
    fn the_marker_window_covers_the_nesting_lines_but_stops() {
        let inside = format!(
            "// migrate: keep — a reason.\nassert_eq!(\n  read(\n    br#\"{ANNOTATED}\"#\n  ),\n  1\n);\n"
        );
        assert_eq!(outcome_states(inside.as_bytes()), ["kept"]);

        let too_far = format!(
            "// migrate: keep — a reason.\nassert_eq!(\n  read(\n    outer(\n      br#\"{ANNOTATED}\"#\n    )\n  ),\n  1\n);\n"
        );
        assert_eq!(outcome_states(too_far.as_bytes()), ["migrated"]);
    }

    fn located(source: &[u8]) -> Vec<String> {
        literals(source)
            .expect("scans")
            .iter()
            .map(|literal| {
                String::from_utf8_lossy(&source[literal.content_start..literal.content_end])
                    .into_owned()
            })
            .collect()
    }

    #[test]
    fn every_literal_form_is_located_with_its_content() {
        let source = br####"let a = "one"; let b = b"two"; let c = r"three"; let d = br#"four"#; let e = r##"fi"#ve"##;"####;
        assert_eq!(located(source), ["one", "two", "three", "four", "fi\"#ve"]);
    }

    /// The hazard the scanner exists for: a quotation mark that is not a
    /// delimiter.
    #[test]
    fn quotes_in_comments_and_character_literals_are_not_delimiters() {
        let source = br#"// a " here
/* and a " here, /* nested */ still */
let quote = '"';
let escaped = '\'';
let real = "kept";
"#;
        assert_eq!(located(source), ["kept"]);
    }

    /// A lifetime opens with the same byte as a character literal and never
    /// closes; mistaking one for the other would swallow the next literal.
    #[test]
    fn a_lifetime_does_not_swallow_the_literal_after_it() {
        let source = br#"fn f<'a>(x: &'a str) -> &'static str { "kept" }"#;
        assert_eq!(located(source), ["kept"]);
    }

    /// `br` is a prefix; `bar` is an identifier that merely starts with it.
    #[test]
    fn an_identifier_ending_in_a_prefix_letter_is_not_a_prefix() {
        let source = br#"let bar = "kept"; let number = "also";"#;
        assert_eq!(located(source), ["kept", "also"]);
    }

    #[test]
    fn escapes_decode_to_the_bytes_a_test_feeds_the_compiler() {
        let form = Form::Escaped { byte: true };
        assert_eq!(
            decode(br#"fn main() {\n  let x = \"q\";\t\\\x41"#, form).expect("decodes"),
            b"fn main() {\n  let x = \"q\";\t\\A".to_vec()
        );
    }

    /// An escape this module does not model must be refused, not guessed at.
    #[test]
    fn an_unmodelled_escape_refuses_to_decode() {
        assert_eq!(decode(br"\u{41}", Form::Escaped { byte: false }), None);
        assert_eq!(
            decode(b"line \\\n  continued", Form::Escaped { byte: true }),
            None
        );
    }

    #[test]
    fn encoding_round_trips_every_form() {
        let fixture = b"fn main() -> own unit traps {\n  check a else trap \"m\\n\";\n}\n";
        for form in [
            Form::Escaped { byte: true },
            Form::Escaped { byte: false },
            Form::Raw {
                byte: true,
                hashes: 1,
            },
        ] {
            let encoded = encode(fixture, form).expect("encodes");
            assert_eq!(decode(&encoded, form).as_deref(), Some(&fixture[..]));
        }
    }

    /// A raw literal cannot hold its own terminator, and saying so is how the
    /// caller learns to leave the fixture alone.
    #[test]
    fn a_raw_literal_refuses_content_spelling_its_terminator() {
        assert_eq!(
            encode(
                b"ends with \"#",
                Form::Raw {
                    byte: true,
                    hashes: 1
                }
            ),
            None
        );
        assert!(
            encode(
                b"ends with \"#",
                Form::Raw {
                    byte: true,
                    hashes: 2
                }
            )
            .is_some()
        );
    }
}
