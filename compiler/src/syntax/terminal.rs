use crate::{ACTIVE_KERNEL_SPEC_HASH, SpecHash};

/// Exact numbered specification owning this terminal contract.
pub const TERMINAL_CONTRACT_SPEC_HASH: SpecHash = ACTIVE_KERNEL_SPEC_HASH;

/// One exact raw-token spelling produced by a fixed grammar atom in the active specification.
///
/// Compound source atoms such as `&uniq` are represented by their two raw
/// token predicates. The declaration order is the stable dense predicate
/// index: the v0.17 inventory, the three spellings v0.18 added, and the two
/// v0.21 added, less the `index` spelling v0.22 released to IDENT, plus the
/// twenty-one v0.23 added — `if` and the twenty `infix_op` operator spellings —
/// and the three v0.25 counted-range spellings, plus v0.28's `ensures` and
/// v0.29's `deny_claims`.
/// New variants append so every
/// previously released dense index remains stable.
/// First grammar-occurrence order is carried by
/// [`ALL_FIXED_TERMINALS`] and is stable language data, not parser priority.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum FixedTerminal {
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
    /// `propagate`.
    Propagate,
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
    /// `as`.
    As,
    /// `external`.
    External,
    /// `blocks`.
    Blocks,
    /// `claim`.
    Claim,
    /// `because`.
    Because,
    /// `if`.
    If,
    /// `+`.
    Plus,
    /// `+wrap`.
    PlusWrap,
    /// `+checked`.
    PlusChecked,
    /// `+sat`.
    PlusSat,
    /// `-`.
    Minus,
    /// `-wrap`.
    MinusWrap,
    /// `-checked`.
    MinusChecked,
    /// `-sat`.
    MinusSat,
    /// `*`.
    Star,
    /// `*wrap`.
    StarWrap,
    /// `*checked`.
    StarChecked,
    /// `*sat`.
    StarSat,
    /// `/`.
    Slash,
    /// `/checked`.
    SlashChecked,
    /// `%`.
    Percent,
    /// `%checked`.
    PercentChecked,
    /// `for`.
    For,
    /// `in`.
    In,
    /// `..`.
    DotDot,
    /// `ensures`.
    Ensures,
    /// `deny_claims`.
    DenyClaims,
    /// `replace`.
    Replace,
}

/// Every fixed raw-token predicate in the active specification, in first occurrence order.
pub const ALL_FIXED_TERMINALS: [FixedTerminal; 91] = [
    FixedTerminal::Struct,
    FixedTerminal::LeftBrace,
    FixedTerminal::RightBrace,
    FixedTerminal::Colon,
    FixedTerminal::Semicolon,
    FixedTerminal::Enum,
    FixedTerminal::LeftParen,
    FixedTerminal::RightParen,
    FixedTerminal::Comma,
    FixedTerminal::DenyClaims,
    FixedTerminal::Fn,
    FixedTerminal::ThinArrow,
    FixedTerminal::Requires,
    FixedTerminal::Ensures,
    FixedTerminal::Contract,
    FixedTerminal::Law,
    FixedTerminal::Conform,
    FixedTerminal::Const,
    FixedTerminal::Equal,
    FixedTerminal::Doc,
    FixedTerminal::LeftAngle,
    FixedTerminal::RightAngle,
    FixedTerminal::LeftBracket,
    FixedTerminal::RightBracket,
    FixedTerminal::Dot,
    FixedTerminal::As,
    FixedTerminal::I8,
    FixedTerminal::I16,
    FixedTerminal::I32,
    FixedTerminal::I64,
    FixedTerminal::U8,
    FixedTerminal::U16,
    FixedTerminal::U32,
    FixedTerminal::U64,
    FixedTerminal::F32,
    FixedTerminal::F64,
    FixedTerminal::Unit,
    FixedTerminal::Array,
    FixedTerminal::Slice,
    FixedTerminal::Box,
    FixedTerminal::Arena,
    FixedTerminal::Buffer,
    FixedTerminal::Own,
    FixedTerminal::Ampersand,
    FixedTerminal::Uniq,
    FixedTerminal::Let,
    FixedTerminal::If,
    FixedTerminal::Else,
    FixedTerminal::Propagate,
    FixedTerminal::Replace,
    FixedTerminal::Set,
    FixedTerminal::Return,
    FixedTerminal::Loop,
    FixedTerminal::For,
    FixedTerminal::In,
    FixedTerminal::DotDot,
    FixedTerminal::Break,
    FixedTerminal::Region,
    FixedTerminal::Check,
    FixedTerminal::Trap,
    FixedTerminal::Claim,
    FixedTerminal::Because,
    FixedTerminal::Give,
    FixedTerminal::Match,
    FixedTerminal::FatArrow,
    FixedTerminal::Plus,
    FixedTerminal::PlusWrap,
    FixedTerminal::PlusChecked,
    FixedTerminal::PlusSat,
    FixedTerminal::Minus,
    FixedTerminal::MinusWrap,
    FixedTerminal::MinusChecked,
    FixedTerminal::MinusSat,
    FixedTerminal::Star,
    FixedTerminal::StarWrap,
    FixedTerminal::StarChecked,
    FixedTerminal::StarSat,
    FixedTerminal::Slash,
    FixedTerminal::SlashChecked,
    FixedTerminal::Percent,
    FixedTerminal::PercentChecked,
    FixedTerminal::Move,
    FixedTerminal::Deref,
    FixedTerminal::Pure,
    FixedTerminal::Reads,
    FixedTerminal::Writes,
    FixedTerminal::Allocates,
    FixedTerminal::Heap,
    FixedTerminal::External,
    FixedTerminal::Blocks,
    FixedTerminal::Traps,
];

impl FixedTerminal {
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
            Self::Propagate => b"propagate",
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
            Self::Dot => b".",
            Self::Pure => b"pure",
            Self::Reads => b"reads",
            Self::Writes => b"writes",
            Self::Allocates => b"allocates",
            Self::Heap => b"heap",
            Self::Traps => b"traps",
            Self::As => b"as",
            Self::External => b"external",
            Self::Blocks => b"blocks",
            Self::Claim => b"claim",
            Self::Because => b"because",
            Self::If => b"if",
            Self::Plus => b"+",
            Self::PlusWrap => b"+wrap",
            Self::PlusChecked => b"+checked",
            Self::PlusSat => b"+sat",
            Self::Minus => b"-",
            Self::MinusWrap => b"-wrap",
            Self::MinusChecked => b"-checked",
            Self::MinusSat => b"-sat",
            Self::Star => b"*",
            Self::StarWrap => b"*wrap",
            Self::StarChecked => b"*checked",
            Self::StarSat => b"*sat",
            Self::Slash => b"/",
            Self::SlashChecked => b"/checked",
            Self::Percent => b"%",
            Self::PercentChecked => b"%checked",
            Self::For => b"for",
            Self::In => b"in",
            Self::DotDot => b"..",
            Self::Ensures => b"ensures",
            Self::DenyClaims => b"deny_claims",
            Self::Replace => b"replace",
        }
    }

    /// Finds the fixed predicate with exactly these raw-token bytes.
    #[must_use]
    pub fn from_spelling(spelling: &[u8]) -> Option<Self> {
        ALL_FIXED_TERMINALS
            .iter()
            .copied()
            .find(|terminal| terminal.spelling() == spelling)
    }

    /// Reports whether this is one of the sixteen GRAM-1 operator forms.
    ///
    /// The four compound comparisons are excluded: they are compound
    /// punctuation, not operator forms, and carry no mode suffix.
    #[must_use]
    pub const fn is_operator_form(self) -> bool {
        matches!(
            self,
            Self::Plus
                | Self::PlusWrap
                | Self::PlusChecked
                | Self::PlusSat
                | Self::Minus
                | Self::MinusWrap
                | Self::MinusChecked
                | Self::MinusSat
                | Self::Star
                | Self::StarWrap
                | Self::StarChecked
                | Self::StarSat
                | Self::Slash
                | Self::SlashChecked
                | Self::Percent
                | Self::PercentChecked
        )
    }

    const fn index(self) -> u8 {
        self as u8
    }
}

/// One terminal predicate in the complete approved active token-membership set.
///
/// A formed token may satisfy more than one predicate. In particular, `unit`
/// satisfies both its fixed predicate and `Literal`; callers must retain both
/// rather than choosing one by priority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TerminalPredicate {
    /// One exact fixed raw-token spelling.
    Fixed(FixedTerminal),
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

/// Every approved active-specification token predicate: the fixed inventory in
/// first occurrence order followed by the external predicates. `SOURCE_END` is
/// intentionally absent.
pub const ALL_TERMINAL_PREDICATES: [TerminalPredicate; 99] = {
    let mut predicates = [TerminalPredicate::Identifier; 99];
    let mut index = 0;
    while index < ALL_FIXED_TERMINALS.len() {
        predicates[index] = TerminalPredicate::Fixed(ALL_FIXED_TERMINALS[index]);
        index += 1;
    }
    predicates[91] = TerminalPredicate::Identifier;
    predicates[92] = TerminalPredicate::TypeIdentifier;
    predicates[93] = TerminalPredicate::RegionIdentifier;
    predicates[94] = TerminalPredicate::Label;
    predicates[95] = TerminalPredicate::OperationName;
    predicates[96] = TerminalPredicate::Literal;
    predicates[97] = TerminalPredicate::String;
    predicates[98] = TerminalPredicate::Digits;
    predicates
};

impl TerminalPredicate {
    const fn index(self) -> u8 {
        match self {
            Self::Fixed(terminal) => terminal.index(),
            Self::Identifier => 91,
            Self::TypeIdentifier => 92,
            Self::RegionIdentifier => 93,
            Self::Label => 94,
            Self::OperationName => 95,
            Self::Literal => 96,
            Self::String => 97,
            Self::Digits => 98,
        }
    }
}

/// The complete set of the active specification terminal predicates retained for one formed token.
///
/// This is a membership set, not a selected token kind. Its compact layout is
/// runtime-local and is not an artifact encoding.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TerminalSet(u128);

impl TerminalSet {
    /// Creates an empty membership set.
    #[must_use]
    pub const fn empty() -> Self {
        Self(0)
    }

    /// Adds one matching predicate.
    pub fn insert(&mut self, predicate: TerminalPredicate) {
        self.0 |= 1_u128 << predicate.index();
    }

    /// Reports whether this token matched the given predicate.
    #[must_use]
    pub const fn contains(self, predicate: TerminalPredicate) -> bool {
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
    pub fn iter(self) -> impl Iterator<Item = TerminalPredicate> {
        ALL_TERMINAL_PREDICATES
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

/// Tests active specification `IDENT` membership, including the fixed-word
/// exclusion of every fixed lowercase spelling, `as`, `external`, `blocks`,
/// `claim`, and `because` among them.
#[must_use]
pub fn is_identifier(spelling: &[u8]) -> bool {
    lower_word(spelling) && FixedTerminal::from_spelling(spelling).is_none()
}

/// Tests active specification `TYPEID` membership.
#[must_use]
pub fn is_type_identifier(spelling: &[u8]) -> bool {
    spelling.first().is_some_and(u8::is_ascii_uppercase)
        && spelling[1..].iter().all(u8::is_ascii_alphanumeric)
}

/// Tests active specification `REGIONID` membership.
#[must_use]
pub fn is_region_identifier(spelling: &[u8]) -> bool {
    spelling.strip_prefix(b"'").is_some_and(lower_word)
}

/// Tests active specification `LABEL` membership.
#[must_use]
pub fn is_label(spelling: &[u8]) -> bool {
    spelling.strip_prefix(b"@").is_some_and(lower_word)
}

/// Tests active specification `OPNAME` membership.
#[must_use]
pub fn is_operation_name(spelling: &[u8]) -> bool {
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

/// Tests the active specification's sole `[0-9]+` pattern predicate.
#[must_use]
pub fn is_digits(spelling: &[u8]) -> bool {
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

/// Tests the active specification `literal` grammar membership before FORM-7 value checking.
///
/// Range, integer leading-zero, finite-value, and shortest-float checks are
/// deliberately outside this predicate, as required by FORM-7.
#[must_use]
pub fn is_literal(spelling: &[u8]) -> bool {
    matches!(spelling, b"unit" | b"0_T" | b"1_T")
        || integer_literal(spelling)
        || float_literal(spelling)
}

/// Tests active specification `STRING` membership.
#[must_use]
pub fn is_string(spelling: &[u8]) -> bool {
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
        ALL_FIXED_TERMINALS, FixedTerminal, TerminalPredicate, TerminalSet, is_identifier,
        is_literal, is_operation_name, is_string,
    };

    #[test]
    fn fixed_inventory_is_unique_and_round_trips() {
        let spellings: BTreeSet<&[u8]> = ALL_FIXED_TERMINALS
            .iter()
            .map(|terminal| terminal.spelling())
            .collect();
        assert_eq!(spellings.len(), ALL_FIXED_TERMINALS.len());
        for terminal in ALL_FIXED_TERMINALS {
            assert_eq!(
                FixedTerminal::from_spelling(terminal.spelling()),
                Some(terminal)
            );
        }
        assert_eq!(FixedTerminal::PercentChecked as u8, 84);
        assert_eq!(FixedTerminal::For as u8, 85);
        assert_eq!(FixedTerminal::In as u8, 86);
        assert_eq!(FixedTerminal::DotDot as u8, 87);
        assert_eq!(FixedTerminal::Ensures as u8, 88);
        assert_eq!(FixedTerminal::DenyClaims as u8, 89);
        assert_eq!(FixedTerminal::Replace as u8, 90);
    }

    #[test]
    fn fixed_lower_words_are_excluded_from_identifiers() {
        for terminal in ALL_FIXED_TERMINALS {
            if terminal
                .spelling()
                .first()
                .is_some_and(u8::is_ascii_lowercase)
            {
                assert!(!is_identifier(terminal.spelling()));
            }
        }
        for spelling in [b"x".as_slice(), b"deref_value", b"wrap", b"ieq"] {
            assert!(is_identifier(spelling));
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
            assert!(is_operation_name(spelling));
        }
        for spelling in [b".wrap".as_slice(), b"x.other", b"x.wrap_more", b"X.wrap"] {
            assert!(!is_operation_name(spelling));
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
            assert!(is_literal(spelling));
        }
        for spelling in [
            b"2_T".as_slice(),
            b"1_i128",
            b"01.0_f32",
            b"1.0e01_f32",
            b"1.0e+1_f32",
            b"1.0_f16",
        ] {
            assert!(!is_literal(spelling));
        }
    }

    #[test]
    fn string_membership_checks_exact_raw_bytes() {
        for spelling in [b"\"\"".as_slice(), b"\"text\"", b"\"\\n\\\"\\\\\""] {
            assert!(is_string(spelling));
        }
        for spelling in [b"text".as_slice(), b"\"\\t\"", b"\"line\nfeed\""] {
            assert!(!is_string(spelling));
        }
    }

    #[test]
    fn membership_set_retains_noncompeting_overlap() {
        let mut set = TerminalSet::empty();
        set.insert(TerminalPredicate::Fixed(FixedTerminal::Unit));
        set.insert(TerminalPredicate::Literal);
        assert_eq!(set.len(), 2);
        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            vec![
                TerminalPredicate::Fixed(FixedTerminal::Unit),
                TerminalPredicate::Literal,
            ]
        );
    }
}
