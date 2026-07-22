use crate::{KERNEL_SPEC_V0_10_HASH, SpecHash};

/// Exact numbered specification owning this terminal contract.
pub const TERMINAL_CONTRACT_SPEC_V0_10: SpecHash = KERNEL_SPEC_V0_10_HASH;

/// One exact raw-token spelling produced by a fixed grammar atom in v0.10.
///
/// Compound source atoms such as `&uniq` are represented by their two raw
/// token predicates. The order follows first appearance in the approved
/// grammar and is stable language data, not parser priority.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FixedTerminalV0_10 {
    /// `struct`.
    Struct,
    /// `{`.
    LeftBrace,
    /// `}`.
    RightBrace,
    /// `:`.
    Colon,
    /// `;`.
    Semicolon,
    /// `enum`.
    Enum,
    /// `(`.
    LeftParen,
    /// `)`.
    RightParen,
    /// `,`.
    Comma,
    /// `fn`.
    Fn,
    /// `->`.
    ThinArrow,
    /// `requires`.
    Requires,
    /// `contract`.
    Contract,
    /// `law`.
    Law,
    /// `conform`.
    Conform,
    /// `const`.
    Const,
    /// `=`.
    Equal,
    /// `doc`.
    Doc,
    /// `<`.
    LeftAngle,
    /// `>`.
    RightAngle,
    /// `[`.
    LeftBracket,
    /// `]`.
    RightBracket,
    /// `i8`.
    I8,
    /// `i16`.
    I16,
    /// `i32`.
    I32,
    /// `i64`.
    I64,
    /// `u8`.
    U8,
    /// `u16`.
    U16,
    /// `u32`.
    U32,
    /// `u64`.
    U64,
    /// `f32`.
    F32,
    /// `f64`.
    F64,
    /// `unit`.
    Unit,
    /// `array`.
    Array,
    /// `slice`.
    Slice,
    /// `box`.
    Box,
    /// `arena`.
    Arena,
    /// `buffer`.
    Buffer,
    /// `own`.
    Own,
    /// `&`.
    Ampersand,
    /// `uniq`.
    Uniq,
    /// `let`.
    Let,
    /// `try`.
    Try,
    /// `set`.
    Set,
    /// `return`.
    Return,
    /// `loop`.
    Loop,
    /// `break`.
    Break,
    /// `region`.
    Region,
    /// `check`.
    Check,
    /// `else`.
    Else,
    /// `trap`.
    Trap,
    /// `give`.
    Give,
    /// `match`.
    Match,
    /// `=>`.
    FatArrow,
    /// `move`.
    Move,
    /// `deref`.
    Deref,
    /// `index`.
    Index,
    /// `.`.
    Dot,
    /// `pure`.
    Pure,
    /// `reads`.
    Reads,
    /// `writes`.
    Writes,
    /// `allocates`.
    Allocates,
    /// `heap`.
    Heap,
    /// `traps`.
    Traps,
}

/// Every v0.10 fixed raw-token predicate, in first-grammar-occurrence order.
pub const ALL_FIXED_TERMINALS_V0_10: [FixedTerminalV0_10; 64] = [
    FixedTerminalV0_10::Struct,
    FixedTerminalV0_10::LeftBrace,
    FixedTerminalV0_10::RightBrace,
    FixedTerminalV0_10::Colon,
    FixedTerminalV0_10::Semicolon,
    FixedTerminalV0_10::Enum,
    FixedTerminalV0_10::LeftParen,
    FixedTerminalV0_10::RightParen,
    FixedTerminalV0_10::Comma,
    FixedTerminalV0_10::Fn,
    FixedTerminalV0_10::ThinArrow,
    FixedTerminalV0_10::Requires,
    FixedTerminalV0_10::Contract,
    FixedTerminalV0_10::Law,
    FixedTerminalV0_10::Conform,
    FixedTerminalV0_10::Const,
    FixedTerminalV0_10::Equal,
    FixedTerminalV0_10::Doc,
    FixedTerminalV0_10::LeftAngle,
    FixedTerminalV0_10::RightAngle,
    FixedTerminalV0_10::LeftBracket,
    FixedTerminalV0_10::RightBracket,
    FixedTerminalV0_10::I8,
    FixedTerminalV0_10::I16,
    FixedTerminalV0_10::I32,
    FixedTerminalV0_10::I64,
    FixedTerminalV0_10::U8,
    FixedTerminalV0_10::U16,
    FixedTerminalV0_10::U32,
    FixedTerminalV0_10::U64,
    FixedTerminalV0_10::F32,
    FixedTerminalV0_10::F64,
    FixedTerminalV0_10::Unit,
    FixedTerminalV0_10::Array,
    FixedTerminalV0_10::Slice,
    FixedTerminalV0_10::Box,
    FixedTerminalV0_10::Arena,
    FixedTerminalV0_10::Buffer,
    FixedTerminalV0_10::Own,
    FixedTerminalV0_10::Ampersand,
    FixedTerminalV0_10::Uniq,
    FixedTerminalV0_10::Let,
    FixedTerminalV0_10::Try,
    FixedTerminalV0_10::Set,
    FixedTerminalV0_10::Return,
    FixedTerminalV0_10::Loop,
    FixedTerminalV0_10::Break,
    FixedTerminalV0_10::Region,
    FixedTerminalV0_10::Check,
    FixedTerminalV0_10::Else,
    FixedTerminalV0_10::Trap,
    FixedTerminalV0_10::Give,
    FixedTerminalV0_10::Match,
    FixedTerminalV0_10::FatArrow,
    FixedTerminalV0_10::Move,
    FixedTerminalV0_10::Deref,
    FixedTerminalV0_10::Index,
    FixedTerminalV0_10::Dot,
    FixedTerminalV0_10::Pure,
    FixedTerminalV0_10::Reads,
    FixedTerminalV0_10::Writes,
    FixedTerminalV0_10::Allocates,
    FixedTerminalV0_10::Heap,
    FixedTerminalV0_10::Traps,
];

impl FixedTerminalV0_10 {
    /// Returns the exact one-token spelling of this predicate.
    #[must_use]
    pub const fn spelling(self) -> &'static [u8] {
        match self {
            Self::Struct => b"struct",
            Self::LeftBrace => b"{",
            Self::RightBrace => b"}",
            Self::Colon => b":",
            Self::Semicolon => b";",
            Self::Enum => b"enum",
            Self::LeftParen => b"(",
            Self::RightParen => b")",
            Self::Comma => b",",
            Self::Fn => b"fn",
            Self::ThinArrow => b"->",
            Self::Requires => b"requires",
            Self::Contract => b"contract",
            Self::Law => b"law",
            Self::Conform => b"conform",
            Self::Const => b"const",
            Self::Equal => b"=",
            Self::Doc => b"doc",
            Self::LeftAngle => b"<",
            Self::RightAngle => b">",
            Self::LeftBracket => b"[",
            Self::RightBracket => b"]",
            Self::I8 => b"i8",
            Self::I16 => b"i16",
            Self::I32 => b"i32",
            Self::I64 => b"i64",
            Self::U8 => b"u8",
            Self::U16 => b"u16",
            Self::U32 => b"u32",
            Self::U64 => b"u64",
            Self::F32 => b"f32",
            Self::F64 => b"f64",
            Self::Unit => b"unit",
            Self::Array => b"array",
            Self::Slice => b"slice",
            Self::Box => b"box",
            Self::Arena => b"arena",
            Self::Buffer => b"buffer",
            Self::Own => b"own",
            Self::Ampersand => b"&",
            Self::Uniq => b"uniq",
            Self::Let => b"let",
            Self::Try => b"try",
            Self::Set => b"set",
            Self::Return => b"return",
            Self::Loop => b"loop",
            Self::Break => b"break",
            Self::Region => b"region",
            Self::Check => b"check",
            Self::Else => b"else",
            Self::Trap => b"trap",
            Self::Give => b"give",
            Self::Match => b"match",
            Self::FatArrow => b"=>",
            Self::Move => b"move",
            Self::Deref => b"deref",
            Self::Index => b"index",
            Self::Dot => b".",
            Self::Pure => b"pure",
            Self::Reads => b"reads",
            Self::Writes => b"writes",
            Self::Allocates => b"allocates",
            Self::Heap => b"heap",
            Self::Traps => b"traps",
        }
    }

    /// Finds the fixed predicate with exactly these raw-token bytes.
    #[must_use]
    pub fn from_spelling(spelling: &[u8]) -> Option<Self> {
        ALL_FIXED_TERMINALS_V0_10
            .iter()
            .copied()
            .find(|terminal| terminal.spelling() == spelling)
    }

    const fn index(self) -> u8 {
        self as u8
    }
}

/// One terminal predicate in the complete approved v0.10 token-membership set.
///
/// A formed token may satisfy more than one predicate. In particular, `unit`
/// satisfies both its fixed predicate and `Literal`; callers must retain both
/// rather than choosing one by priority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalPredicateV0_10 {
    /// One exact fixed raw-token spelling.
    Fixed(FixedTerminalV0_10),
    /// FORM-3 `IDENT`.
    Identifier,
    /// FORM-3 `TYPEID`.
    TypeIdentifier,
    /// FORM-3 `REGIONID`.
    RegionIdentifier,
    /// FORM-3 `LABEL`.
    Label,
    /// FORM-3 `OPNAME`.
    OperationName,
    /// FORM-5 `literal` union membership, before FORM-7 checking.
    Literal,
    /// FORM-5 `STRING`.
    String,
    /// The sole `[0-9]+` grammar-pattern predicate.
    Digits,
}

/// Every approved v0.10 token predicate. `SOURCE_END` is intentionally absent.
pub const ALL_TERMINAL_PREDICATES_V0_10: [TerminalPredicateV0_10; 72] = [
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Struct),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::LeftBrace),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::RightBrace),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Colon),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Semicolon),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Enum),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::LeftParen),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::RightParen),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Comma),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Fn),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::ThinArrow),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Requires),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Contract),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Law),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Conform),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Const),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Equal),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Doc),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::LeftAngle),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::RightAngle),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::LeftBracket),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::RightBracket),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::I8),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::I16),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::I32),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::I64),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::U8),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::U16),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::U32),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::U64),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::F32),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::F64),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Unit),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Array),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Slice),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Box),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Arena),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Buffer),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Own),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Ampersand),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Uniq),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Let),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Try),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Set),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Return),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Loop),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Break),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Region),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Check),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Else),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Trap),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Give),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Match),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::FatArrow),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Move),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Deref),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Index),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Dot),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Pure),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Reads),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Writes),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Allocates),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Heap),
    TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Traps),
    TerminalPredicateV0_10::Identifier,
    TerminalPredicateV0_10::TypeIdentifier,
    TerminalPredicateV0_10::RegionIdentifier,
    TerminalPredicateV0_10::Label,
    TerminalPredicateV0_10::OperationName,
    TerminalPredicateV0_10::Literal,
    TerminalPredicateV0_10::String,
    TerminalPredicateV0_10::Digits,
];

impl TerminalPredicateV0_10 {
    const fn index(self) -> u8 {
        match self {
            Self::Fixed(terminal) => terminal.index(),
            Self::Identifier => 64,
            Self::TypeIdentifier => 65,
            Self::RegionIdentifier => 66,
            Self::Label => 67,
            Self::OperationName => 68,
            Self::Literal => 69,
            Self::String => 70,
            Self::Digits => 71,
        }
    }
}

/// The complete set of v0.10 terminal predicates retained for one formed token.
///
/// This is a membership set, not a selected token kind. Its compact layout is
/// runtime-local and is not an artifact encoding.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TerminalSetV0_10(u128);

impl TerminalSetV0_10 {
    /// Creates an empty membership set.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Adds one matching predicate.
    pub fn insert(&mut self, predicate: TerminalPredicateV0_10) {
        self.0 |= 1_u128 << predicate.index();
    }

    /// Reports whether this token matched the given predicate.
    #[must_use]
    pub const fn contains(self, predicate: TerminalPredicateV0_10) -> bool {
        self.0 & (1_u128 << predicate.index()) != 0
    }

    /// Reports whether no approved predicate matched.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Returns the number of matching predicates.
    #[must_use]
    pub const fn len(self) -> u32 {
        self.0.count_ones()
    }

    /// Visits matching predicates in stable storage order.
    ///
    /// This order is not the grammar-occurrence order used by syntax
    /// diagnostics. Parser tables must retain their separately approved
    /// source-grammar ranks.
    pub fn iter(self) -> impl Iterator<Item = TerminalPredicateV0_10> {
        ALL_TERMINAL_PREDICATES_V0_10
            .iter()
            .copied()
            .filter(move |predicate| self.contains(*predicate))
    }
}

fn lower_word(spelling: &[u8]) -> bool {
    spelling.first().is_some_and(u8::is_ascii_lowercase)
        && spelling
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
}

/// Tests exact v0.10 `IDENT` membership, including the fixed-word exclusion.
#[must_use]
pub fn is_identifier_v0_10(spelling: &[u8]) -> bool {
    lower_word(spelling) && FixedTerminalV0_10::from_spelling(spelling).is_none()
}

/// Tests exact v0.10 `TYPEID` membership.
#[must_use]
pub fn is_type_identifier_v0_10(spelling: &[u8]) -> bool {
    spelling.first().is_some_and(u8::is_ascii_uppercase)
        && spelling[1..].iter().all(u8::is_ascii_alphanumeric)
}

/// Tests exact v0.10 `REGIONID` membership.
#[must_use]
pub fn is_region_identifier_v0_10(spelling: &[u8]) -> bool {
    spelling.strip_prefix(b"'").is_some_and(lower_word)
}

/// Tests exact v0.10 `LABEL` membership.
#[must_use]
pub fn is_label_v0_10(spelling: &[u8]) -> bool {
    spelling.strip_prefix(b"@").is_some_and(lower_word)
}

/// Tests exact v0.10 `OPNAME` membership.
#[must_use]
pub fn is_operation_name_v0_10(spelling: &[u8]) -> bool {
    [
        b".wrap".as_slice(),
        b".trap",
        b".checked",
        b".sat",
        b".strict",
    ]
    .iter()
    .any(|suffix| spelling.strip_suffix(*suffix).is_some_and(lower_word))
}

/// Tests the sole v0.10 `[0-9]+` pattern predicate.
#[must_use]
pub fn is_digits_v0_10(spelling: &[u8]) -> bool {
    !spelling.is_empty() && spelling.iter().all(u8::is_ascii_digit)
}

fn integer_literal(spelling: &[u8]) -> bool {
    let Some(split) = spelling.iter().rposition(|byte| *byte == b'_') else {
        return false;
    };
    let suffix = &spelling[split + 1..];
    if !matches!(
        suffix,
        b"i8" | b"i16" | b"i32" | b"i64" | b"u8" | b"u16" | b"u32" | b"u64"
    ) {
        return false;
    }
    let negative = spelling.first() == Some(&b'-');
    let digits = &spelling[usize::from(negative)..split];
    !digits.is_empty() && digits.iter().all(u8::is_ascii_digit)
}

fn decimal_component_end(spelling: &[u8], start: usize) -> Option<usize> {
    let first = *spelling.get(start)?;
    if !first.is_ascii_digit() {
        return None;
    }
    if first == b'0' {
        return Some(start + 1);
    }
    let mut cursor = start + 1;
    while spelling.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    Some(cursor)
}

fn float_literal(spelling: &[u8]) -> bool {
    let mut cursor = usize::from(spelling.first() == Some(&b'-'));
    let Some(integer_end) = decimal_component_end(spelling, cursor) else {
        return false;
    };
    cursor = integer_end;
    if spelling.get(cursor) != Some(&b'.') {
        return false;
    }
    cursor += 1;
    let fraction_start = cursor;
    while spelling.get(cursor).is_some_and(u8::is_ascii_digit) {
        cursor += 1;
    }
    if cursor == fraction_start {
        return false;
    }
    if spelling.get(cursor) == Some(&b'e') {
        cursor += 1;
        if spelling.get(cursor) == Some(&b'-') {
            cursor += 1;
        }
        let Some(exponent_end) = decimal_component_end(spelling, cursor) else {
            return false;
        };
        cursor = exponent_end;
    }
    matches!(&spelling[cursor..], b"_f32" | b"_f64")
}

/// Tests v0.10 `literal` grammar membership before FORM-7 value checking.
///
/// Range, integer leading-zero, finite-value, and shortest-float checks are
/// deliberately outside this predicate, as required by FORM-7.
#[must_use]
pub fn is_literal_v0_10(spelling: &[u8]) -> bool {
    matches!(spelling, b"unit" | b"0_T" | b"1_T")
        || integer_literal(spelling)
        || float_literal(spelling)
}

/// Tests exact v0.10 `STRING` membership.
#[must_use]
pub fn is_string_v0_10(spelling: &[u8]) -> bool {
    if spelling.len() < 2 || spelling.first() != Some(&b'"') || spelling.last() != Some(&b'"') {
        return false;
    }
    let mut cursor = 1;
    while cursor + 1 < spelling.len() {
        let byte = spelling[cursor];
        if byte == b'\\' {
            if !matches!(spelling.get(cursor + 1), Some(b'\\' | b'"' | b'n')) {
                return false;
            }
            cursor += 2;
        } else if !(0x20..=0x7e).contains(&byte) || matches!(byte, b'"' | b'\\') {
            return false;
        } else {
            cursor += 1;
        }
    }
    cursor + 1 == spelling.len()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::{
        ALL_FIXED_TERMINALS_V0_10, FixedTerminalV0_10, TerminalPredicateV0_10, TerminalSetV0_10,
        is_identifier_v0_10, is_literal_v0_10, is_operation_name_v0_10, is_string_v0_10,
    };

    #[test]
    fn fixed_inventory_is_unique_and_round_trips() {
        let spellings: BTreeSet<&[u8]> = ALL_FIXED_TERMINALS_V0_10
            .iter()
            .map(|terminal| terminal.spelling())
            .collect();
        assert_eq!(spellings.len(), ALL_FIXED_TERMINALS_V0_10.len());
        for terminal in ALL_FIXED_TERMINALS_V0_10 {
            assert_eq!(
                FixedTerminalV0_10::from_spelling(terminal.spelling()),
                Some(terminal)
            );
        }
    }

    #[test]
    fn fixed_lower_words_are_excluded_from_identifiers() {
        for terminal in ALL_FIXED_TERMINALS_V0_10 {
            if terminal
                .spelling()
                .first()
                .is_some_and(u8::is_ascii_lowercase)
            {
                assert!(!is_identifier_v0_10(terminal.spelling()));
            }
        }
        for spelling in [b"x".as_slice(), b"deref_value", b"wrap", b"ieq"] {
            assert!(is_identifier_v0_10(spelling));
        }
    }

    #[test]
    fn operation_suffix_language_is_closed() {
        for spelling in [
            b"iadd.wrap".as_slice(),
            b"iadd.trap",
            b"iadd.checked",
            b"iadd.sat",
            b"iadd.strict",
        ] {
            assert!(is_operation_name_v0_10(spelling));
        }
        for spelling in [b".wrap".as_slice(), b"x.other", b"x.wrap_more", b"X.wrap"] {
            assert!(!is_operation_name_v0_10(spelling));
        }
    }

    #[test]
    fn literal_membership_stops_before_form7_value_checks() {
        for spelling in [
            b"unit".as_slice(),
            b"0_T",
            b"1_T",
            b"00_i8",
            b"-0_i64",
            b"999999999999999999999_u8",
            b"0.0_f32",
            b"1.00_f64",
            b"1.5e-0_f64",
        ] {
            assert!(is_literal_v0_10(spelling));
        }
        for spelling in [
            b"2_T".as_slice(),
            b"1_i128",
            b"01.0_f32",
            b"1.0e01_f32",
            b"1.0e+1_f32",
            b"1.0_f16",
        ] {
            assert!(!is_literal_v0_10(spelling));
        }
    }

    #[test]
    fn string_membership_checks_exact_raw_bytes() {
        for spelling in [b"\"\"".as_slice(), b"\"text\"", b"\"\\n\\\"\\\\\""] {
            assert!(is_string_v0_10(spelling));
        }
        for spelling in [b"text".as_slice(), b"\"\\t\"", b"\"line\nfeed\""] {
            assert!(!is_string_v0_10(spelling));
        }
    }

    #[test]
    fn membership_set_retains_noncompeting_overlap() {
        let mut set = TerminalSetV0_10::empty();
        set.insert(TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Unit));
        set.insert(TerminalPredicateV0_10::Literal);
        assert_eq!(set.len(), 2);
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            vec![
                TerminalPredicateV0_10::Fixed(FixedTerminalV0_10::Unit),
                TerminalPredicateV0_10::Literal,
            ]
        );
    }
}
