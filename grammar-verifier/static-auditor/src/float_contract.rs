//! Versioned float-contract extraction and lexical-shape membership.

use crate::document::Document;
use crate::wire::Failure;

const FORM5_PREFIX: &str = concat!(
    "Literals, exhaustively: integers `-?[0-9]+_TYPE` (decimal only, mandatory suffix; a leading ",
    "`-` is legal for signed TYPE, and the signed value must lie in TYPE's range [FORM-7]; e.g. ",
    "`42_i32`, `-2147483648_i32`); "
);

const V08_FLOAT_CLAUSE: &str = concat!(
    "floats `-?[0-9]+\\.[0-9]+(e-?[0-9]+)?_TYPE` (a leading `-` ",
    "is legal for the value; the canonical spelling is the unique shortest decimal digit string ",
    "that round-trips under round-to-nearest-even, with at least one integer and one fraction digit, ",
    "lowercase `e`, and no leading zeros; `-0.0` is distinct from `0.0`; e.g. `1.5_f64`, ",
    "`6.022e23_f64`); "
);

const SUCCESSOR_FLOAT_CLAUSE: &str = concat!(
    "finite floats use the grammar `-?(0|[1-9][0-9]*)\\.[0-9]+(e-?(0|[1-9][0-9]*))?_TYPE`, ",
    "where TYPE is `f32` (IEEE 754 binary32) or `f64` (IEEE 754 binary64), positive exponents ",
    "carry no sign, negative exponents carry one `-`, and only the integer and exponent components ",
    "have the stated no-leading-zero form. Let C be the nonnegative integer formed by concatenating ",
    "the integer and fraction digits, let F be the number of fraction digits, and let E be the signed ",
    "integer formed by the exponent digits and their optional `-`; when the exponent is absent E is ",
    "zero, and `e-0` also gives E zero. A matching decimal whose C is zero denotes signed decimal ",
    "zero: a leading literal `-` selects negative zero and its absence selects positive zero, ",
    "independently of E. Every other matching decimal denotes the exact nonzero rational whose ",
    "magnitude is C × 10^(E − F), with the leading literal sign applied. For one finite bit pattern ",
    "of TYPE, consider every matching decimal that rounds from that signed zero or exact nonzero ",
    "rational to the bit pattern under IEEE 754 round-to-nearest, ties-to-even. Its canonical spelling ",
    "is the candidate with the fewest ASCII bytes before `_TYPE`; a tie is resolved by lexicographically ",
    "least unsigned ASCII bytes. This selection is total, host-independent, and unique; in particular ",
    "`0.0` and `-0.0` remain distinct. Other examples are `1.5_f64` and `6.022e23_f64`. "
);

const FORM5_SUFFIX: &str = concat!(
    "`unit`; STRING `\"...\"` whose interior is a sequence of items, each one raw ",
    "ASCII-printable byte in U+0020..U+007E other than `\"` and `\\`, or one of exactly three escapes ",
    "`\\\\ \\\" \\n`; no other byte is legal, and each character has exactly one spelling (the escape where one is ",
    "defined, the raw byte otherwise). STRING appears only in `doc` and `check` messages; non-ASCII ",
    "diagnostic text is DEFERRED. There are no boolean literals: `Bool` is a prelude enum (§15). ",
    "Generic-numeric literals `0_T` and `1_T` are legal where `T` is a gparam bound by a numeric ",
    "contract (`Int` or `Float`, §15), denoting T's additive and multiplicative identity; a concrete ",
    "type uses `0_i32` and the like, so there is no dual spelling. NaN and the infinities are not ",
    "literals; they are the nullary ops `fnan` and `finf` [OP-1]."
);

const V08_FORM7: &str = concat!(
    "Numeric-literal well-formedness (R4 check-reject). An integer literal `-?d_T` is legal where its ",
    "signed value lies in the closed range of T (signed `[-2^(K-1), 2^(K-1)-1]`, unsigned ",
    "`[0, 2^K-1]`) and it has no leading zeros: the single digit `0` is its own form, a leading `-` ",
    "is legal for signed T, and `-0` is written `0`. A float literal is legal where its ",
    "round-to-nearest-even value in T is finite. An out-of-range integer, a leading-zero integer, or ",
    "a float that rounds to a non-finite value is a hard error at check time [SCOPE-2]; a literal ",
    "never denotes a wrapped, truncated, saturated, or undefined value. The canonical decimal ",
    "spelling of a float value is gated on the FORM-1 reject-vs-canonicalize decision and DEFERRED."
);

const SUCCESSOR_FORM7: &str = concat!(
    "Numeric-literal well-formedness (R4 check-reject). An integer literal `-?d_T` is legal where its ",
    "signed value lies in the closed range of T (signed `[-2^(K-1), 2^(K-1)-1]`, unsigned ",
    "`[0, 2^K-1]`) and it has no leading zeros: the single digit `0` is its own form, a leading `-` ",
    "is legal for signed T, and `-0` is written `0`. A float literal is legal only when it has the ",
    "unique canonical spelling selected by [FORM-5] and denotes a finite value of its stated TYPE. ",
    "An out-of-range integer, a leading-zero integer, a noncanonical float spelling, or a float ",
    "decimal that rounds to a non-finite value is a hard error at check time [SCOPE-2]; a literal ",
    "never denotes a wrapped, truncated, saturated, or undefined value."
);

/// The two float languages audited by this engine.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Dialect {
    /// Immutable kernel-spec v0.8, including its deferred canonicality rule.
    V08,
    /// The non-authoritative successor contract with an exact canonical order.
    Successor,
}

impl Dialect {
    pub(crate) const fn predicate(self) -> &'static str {
        match self {
            Self::V08 => "float=-?[0-9]+\\.[0-9]+(e-?[0-9]+)?_TYPE",
            Self::Successor => concat!(
                "float=-?(0|[1-9][0-9]*)\\.[0-9]+(e-?(0|[1-9][0-9]*))?_TYPE;",
                "float-value=signed-zero-or-sign*C*10^(E-F);e-0=0;round=ieee-rne;",
                "canonical=min-prefix-bytes,ascii-lex;finite=required"
            ),
        }
    }
}

fn rule_body<'a>(document: &'a Document<'_>, identifier: &str) -> Result<&'a [u8], Failure> {
    let rule = document
        .rules
        .iter()
        .find(|rule| rule.id == identifier)
        .ok_or_else(|| Failure::extraction("float-contract-rule"))?;
    let line = document
        .lines
        .iter()
        .copied()
        .find(|line| line.start == rule.span.start)
        .ok_or_else(|| Failure::internal("float-contract-line"))?;
    let content = document.line_content(line);
    content
        .strip_prefix(b"[")
        .and_then(|tail| tail.strip_prefix(identifier.as_bytes()))
        .and_then(|tail| tail.strip_prefix(b"] "))
        .ok_or_else(|| Failure::extraction("float-contract-rule-head"))
}

fn form5_dialect(body: &[u8]) -> Option<Dialect> {
    let middle = body
        .strip_prefix(FORM5_PREFIX.as_bytes())?
        .strip_suffix(FORM5_SUFFIX.as_bytes())?;
    if middle == V08_FLOAT_CLAUSE.as_bytes() {
        Some(Dialect::V08)
    } else if middle == SUCCESSOR_FLOAT_CLAUSE.as_bytes() {
        Some(Dialect::Successor)
    } else {
        None
    }
}

/// Extract one closed FORM-5/FORM-7 dialect pair from a specification document.
pub(crate) fn extract(document: &Document<'_>) -> Result<Dialect, Failure> {
    let dialect = form5_dialect(rule_body(document, "FORM-5")?)
        .ok_or_else(|| Failure::extraction("float-contract-form5"))?;
    let form7 = rule_body(document, "FORM-7")?;
    let expected = match dialect {
        Dialect::V08 => V08_FORM7,
        Dialect::Successor => SUCCESSOR_FORM7,
    };
    if form7 != expected.as_bytes() {
        return Err(Failure::extraction("float-contract-form7"));
    }
    Ok(dialect)
}

fn component_end(bytes: &[u8], start: usize, dialect: Dialect) -> Option<usize> {
    let first = *bytes.get(start)?;
    if !first.is_ascii_digit() {
        return None;
    }
    if dialect == Dialect::Successor && first == b'0' {
        return Some(start + 1);
    }
    let mut cursor = start + 1;
    while bytes.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    Some(cursor)
}

/// Return the end of a float token beginning at `start`, if the selected grammar accepts it.
pub(crate) fn prefix_end(source: &[u8], start: usize, dialect: Dialect) -> Option<usize> {
    let mut cursor = start;
    if source.get(cursor) == Some(&b'-') {
        cursor += 1;
    }
    cursor = component_end(source, cursor, dialect)?;
    if source.get(cursor) != Some(&b'.') {
        return None;
    }
    cursor += 1;
    let fraction_start = cursor;
    while source.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    if cursor == fraction_start {
        return None;
    }
    if source.get(cursor) == Some(&b'e') {
        cursor += 1;
        if source.get(cursor) == Some(&b'-') {
            cursor += 1;
        }
        cursor = component_end(source, cursor, dialect)?;
    }
    for suffix in [b"_f32".as_slice(), b"_f64"] {
        if source
            .get(cursor..)
            .is_some_and(|tail| tail.starts_with(suffix))
        {
            return Some(cursor + suffix.len());
        }
    }
    None
}

/// Test complete-token membership in the selected float grammar.
pub(crate) fn accepts(bytes: &[u8], dialect: Dialect) -> bool {
    prefix_end(bytes, 0, dialect) == Some(bytes.len())
}

#[cfg(test)]
mod tests {
    use super::{Dialect, accepts, prefix_end};

    #[test]
    fn v08_and_successor_keep_distinct_lexical_languages() {
        for spelling in [b"0.0_f32".as_slice(), b"-1.50e-0_f64", b"6.022e23_f64"] {
            assert!(accepts(spelling, Dialect::V08));
            assert!(accepts(spelling, Dialect::Successor));
        }
        for spelling in [b"01.5_f32".as_slice(), b"1.5e01_f64", b"1.5e-01_f64"] {
            assert!(accepts(spelling, Dialect::V08));
            assert!(!accepts(spelling, Dialect::Successor));
        }
    }

    #[test]
    fn float_shape_rejects_non_contract_signs_and_suffixes() {
        for dialect in [Dialect::V08, Dialect::Successor] {
            for spelling in [
                b"+1.0_f32".as_slice(),
                b"1.0e+1_f32",
                b"1_f32",
                b"1._f32",
                b"1.0_f16",
            ] {
                assert!(!accepts(spelling, dialect));
            }
        }
        assert_eq!(prefix_end(b"1.5_f64 rest", 0, Dialect::Successor), Some(7));
    }
}
