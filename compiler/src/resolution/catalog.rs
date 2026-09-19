use super::{
    BuiltinPreludeDeclarationRecord, BuiltinPreludeId, DeclarationClass, OperationFamilyId,
    ReservedNameClass,
};

pub(crate) const PRELUDE_DECLARATIONS: [BuiltinPreludeDeclarationRecord; 24] = [
    prelude(
        BuiltinPreludeId::BOOL,
        "Bool",
        Some(DeclarationClass::NominalType),
    ),
    prelude(
        BuiltinPreludeId::TRUE,
        "True",
        Some(DeclarationClass::EnumVariant),
    ),
    prelude(
        BuiltinPreludeId::FALSE,
        "False",
        Some(DeclarationClass::EnumVariant),
    ),
    prelude(
        BuiltinPreludeId::OPTION,
        "Option",
        Some(DeclarationClass::NominalType),
    ),
    prelude(BuiltinPreludeId::OPTION_TYPE, "T", None),
    prelude(
        BuiltinPreludeId::NONE,
        "None",
        Some(DeclarationClass::EnumVariant),
    ),
    prelude(
        BuiltinPreludeId::SOME,
        "Some",
        Some(DeclarationClass::EnumVariant),
    ),
    prelude(BuiltinPreludeId::SOME_VALUE, "value", None),
    prelude(
        BuiltinPreludeId::RESULT,
        "Result",
        Some(DeclarationClass::NominalType),
    ),
    prelude(BuiltinPreludeId::RESULT_VALUE_TYPE, "T", None),
    prelude(BuiltinPreludeId::RESULT_ERROR_TYPE, "E", None),
    prelude(
        BuiltinPreludeId::OK,
        "Ok",
        Some(DeclarationClass::EnumVariant),
    ),
    prelude(BuiltinPreludeId::OK_VALUE, "value", None),
    prelude(
        BuiltinPreludeId::ERR,
        "Err",
        Some(DeclarationClass::EnumVariant),
    ),
    prelude(BuiltinPreludeId::ERR_ERROR, "error", None),
    prelude(
        BuiltinPreludeId::OVERFLOW_TYPE,
        "Overflow",
        Some(DeclarationClass::NominalType),
    ),
    prelude(
        BuiltinPreludeId::OVERFLOW,
        "Overflow",
        Some(DeclarationClass::EnumVariant),
    ),
    prelude(
        BuiltinPreludeId::DIV_ERROR_TYPE,
        "DivError",
        Some(DeclarationClass::NominalType),
    ),
    prelude(
        BuiltinPreludeId::DIVIDE_BY_ZERO,
        "DivideByZero",
        Some(DeclarationClass::EnumVariant),
    ),
    prelude(
        BuiltinPreludeId::DIV_OVERFLOW,
        "DivOverflow",
        Some(DeclarationClass::EnumVariant),
    ),
    prelude(
        BuiltinPreludeId::NARROW_ERROR_TYPE,
        "NarrowError",
        Some(DeclarationClass::NominalType),
    ),
    prelude(
        BuiltinPreludeId::NARROW_ERROR,
        "NarrowError",
        Some(DeclarationClass::EnumVariant),
    ),
    prelude(
        BuiltinPreludeId::INT,
        "Int",
        Some(DeclarationClass::NumericBound),
    ),
    prelude(
        BuiltinPreludeId::FLOAT,
        "Float",
        Some(DeclarationClass::NumericBound),
    ),
];

const fn prelude(
    id: BuiltinPreludeId,
    spelling: &'static str,
    class: Option<DeclarationClass>,
) -> BuiltinPreludeDeclarationRecord {
    BuiltinPreludeDeclarationRecord {
        id,
        spelling,
        class,
    }
}

/// Distinct OP-1 spellings in normative table order, with repeated `cvt`
/// collapsed at its first occurrence as required by OP-1.
pub(crate) const OPERATION_FAMILIES: [&str; 86] = [
    "+wrap",
    "-wrap",
    "*wrap",
    "+",
    "-",
    "*",
    "+defined",
    "-defined",
    "*defined",
    "+checked",
    "-checked",
    "*checked",
    "/",
    "%",
    "/defined",
    "%defined",
    "/checked",
    "%checked",
    "ineg.wrap",
    "ineg",
    "ineg.defined",
    "ineg.checked",
    "==",
    "!=",
    "<",
    "<=",
    ">",
    ">=",
    "eeq",
    "ene",
    "fadd.strict",
    "fsub.strict",
    "fmul.strict",
    "fdiv.strict",
    "feq",
    "flt",
    "fle",
    "fgt",
    "fge",
    "fne",
    "band",
    "bor",
    "bxor",
    "bnot",
    "cvt",
    "iand",
    "ior",
    "ixor",
    "inot",
    "ishl.wrap",
    "ishr.wrap",
    "ishl",
    "ishr",
    "ishl.defined",
    "ishr.defined",
    "irotl",
    "irotr",
    "ipopcount",
    "iclz",
    "ictz",
    "ibswap",
    "imulhi",
    "+sat",
    "-sat",
    "*sat",
    "imin",
    "imax",
    "iabs.wrap",
    "iabs",
    "iabs.defined",
    "iabs.checked",
    "reinterpret",
    "fneg",
    "fabs",
    "fcopysign",
    "fmin",
    "fmax",
    "ffloor",
    "fceil",
    "ftrunc",
    "froundeven",
    "frem",
    "fsqrt.strict",
    "ffma.strict",
    "finf",
    "fnan",
];

pub(crate) const MODE_WORDS: [&str; 5] = ["wrap", "defined", "checked", "sat", "strict"];

/// The four measure pseudo-fields [MSR-1, OP-15] followed by the four window
/// parts [WIN-2], in the FORM-3 alternative order DIAG-1 fixes as the
/// `measure-or-part` reserved ordinal.
pub(crate) const MEASURE_AND_PART_NAMES: [&str; 8] = [
    "len", "cap", "room", "head", "next", "last", "filled", "free",
];

pub(crate) fn operation_id(spelling: &str) -> Option<OperationFamilyId> {
    OPERATION_FAMILIES
        .iter()
        .position(|candidate| *candidate == spelling)
        .and_then(OperationFamilyId::from_index)
}

pub(crate) fn operation_spelling(id: OperationFamilyId) -> Option<&'static str> {
    OPERATION_FAMILIES.get(usize::from(id.0)).copied()
}

pub(crate) fn reserved_name(spelling: &str) -> Option<(ReservedNameClass, u16)> {
    if !spelling.contains('.')
        && let Some(index) = OPERATION_FAMILIES
            .iter()
            .position(|candidate| *candidate == spelling)
    {
        return u16::try_from(index)
            .ok()
            .map(|ordinal| (ReservedNameClass::DotlessOperation, ordinal));
    }
    if let Some(index) = MODE_WORDS
        .iter()
        .position(|candidate| *candidate == spelling)
    {
        return u16::try_from(index)
            .ok()
            .map(|ordinal| (ReservedNameClass::ModeWord, ordinal));
    }
    // [FORM-3] the eight measure and window-part names are reserved from every
    // declaration role OP-1 already lists, because `r.len` would otherwise
    // carry two meanings [TYPE-10].
    MEASURE_AND_PART_NAMES
        .iter()
        .position(|candidate| *candidate == spelling)
        .and_then(|index| u16::try_from(index).ok())
        .map(|ordinal| (ReservedNameClass::MeasureOrPart, ordinal))
}

#[cfg(test)]
mod tests {
    use super::{
        DeclarationClass, MEASURE_AND_PART_NAMES, MODE_WORDS, OPERATION_FAMILIES,
        PRELUDE_DECLARATIONS, ReservedNameClass, reserved_name,
    };
    use std::collections::HashSet;

    #[test]
    fn exact_catalogs_are_closed_and_unique_where_required() {
        assert_eq!(PRELUDE_DECLARATIONS.len(), 24);
        assert_eq!(OPERATION_FAMILIES.len(), 86);
        assert_eq!(
            OPERATION_FAMILIES
                .iter()
                .copied()
                .collect::<HashSet<_>>()
                .len(),
            OPERATION_FAMILIES.len()
        );
        assert!(
            MODE_WORDS
                .iter()
                .all(|word| !OPERATION_FAMILIES.contains(word))
        );
        // [DIAG-1] "Those three reserved sets are disjoint in this version."
        assert!(
            MEASURE_AND_PART_NAMES
                .iter()
                .all(|name| !OPERATION_FAMILIES.contains(name) && !MODE_WORDS.contains(name))
        );
        // OP-1's derived-set consequence of the v0.41 comparison symbols:
        // the six integer comparisons are operator spellings, so they occupy
        // their contiguous family ordinals but are no longer dotless names,
        // and the retired names are free identifiers.
        for (spelling, ordinal) in [
            ("==", 22),
            ("!=", 23),
            ("<", 24),
            ("<=", 25),
            (">", 26),
            (">=", 27),
        ] {
            assert_eq!(
                OPERATION_FAMILIES[ordinal], spelling,
                "{spelling} occupies family ordinal {ordinal}"
            );
        }
        for retired in ["ieq", "ine", "ilt", "ile", "igt", "ige"] {
            assert_eq!(
                reserved_name(retired),
                None,
                "{retired} is a free identifier"
            );
        }
        // OP-1's table no longer carries a reader, view, or acquiring row, so
        // the twelve v0.59 spellings below became free identifiers with them.
        for retired in [
            "len_of",
            "cap_of",
            "room_of",
            "head_of",
            "slice_of",
            "mut_slice_of",
            "arena_new",
            "array_new",
            "buffer_fits",
            "buffer_new",
            "buffer_vacant",
        ] {
            assert_eq!(
                reserved_name(retired),
                None,
                "{retired} is a free identifier"
            );
        }
        // `box_new` also left the operation table, but it is now an ordinary
        // PRE-1 construction function [OP-13], so it is taken by declaration
        // collision rather than by reservation.
        assert_eq!(reserved_name("box_new"), None);
        assert_eq!(
            reserved_name("cvt"),
            Some((ReservedNameClass::DotlessOperation, 44))
        );
        assert_eq!(
            reserved_name("wrap"),
            Some((ReservedNameClass::ModeWord, 0))
        );
        for (ordinal, spelling) in MEASURE_AND_PART_NAMES.iter().enumerate() {
            assert_eq!(
                reserved_name(spelling),
                Some((
                    ReservedNameClass::MeasureOrPart,
                    u16::try_from(ordinal).expect("eight reserved names")
                )),
                "{spelling} is FORM-3 reserved at ordinal {ordinal}"
            );
        }
    }

    #[test]
    fn catalogs_match_independent_extraction_from_exact() {
        let extracted_prelude = extract_prelude_records(crate::ACTIVE_KERNEL_SPEC_TEXT);
        let catalog_prelude: Vec<_> = PRELUDE_DECLARATIONS
            .iter()
            .map(|record| (record.spelling.to_owned(), record.class))
            .collect();
        assert_eq!(catalog_prelude, extracted_prelude);

        assert_eq!(
            OPERATION_FAMILIES.as_slice(),
            extract_operation_families(crate::ACTIVE_KERNEL_SPEC_TEXT)
        );

        assert_eq!(
            MEASURE_AND_PART_NAMES.as_slice(),
            extract_measure_and_part_names(crate::ACTIVE_KERNEL_SPEC_TEXT)
        );
    }

    /// FORM-3's own reservation sentence, read as the authority for both the
    /// membership and the `measure-or-part` ordinal order [DIAG-1].
    fn extract_measure_and_part_names(spec: &str) -> Vec<&str> {
        let sentence = spec
            .split_once("\nThe eight names ")
            .expect("FORM-3 reservation sentence")
            .1
            .split_once(" are reserved from every declaration role")
            .expect("FORM-3 reservation sentence ending")
            .0;
        sentence
            .split('`')
            .enumerate()
            .filter_map(|(index, part)| (index % 2 == 1).then_some(part))
            .collect()
    }

    fn extract_prelude_records(spec: &str) -> Vec<(String, Option<DeclarationClass>)> {
        let (block, after) = spec
            .split_once("[PRE-1] The prelude contributes")
            .expect("exact PRE-1 opening")
            .1
            .split_once("```\n")
            .expect("PRE-1 ordinary declaration fence")
            .1
            .split_once("\n```\n")
            .expect("exact PRE-1 closing");
        let mut records = Vec::new();
        let mut in_enum = false;
        for line in block.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("struct ") {
                break;
            }
            if let Some(header) = trimmed.strip_prefix("enum ") {
                in_enum = true;
                let name_end = header
                    .find(['<', ' '])
                    .expect("PRE-1 enum header terminator");
                records.push((
                    header[..name_end].to_owned(),
                    Some(DeclarationClass::NominalType),
                ));
                if let Some(generics) = header
                    .split_once('<')
                    .and_then(|(_, rest)| rest.split_once('>'))
                    .map(|(generics, _)| generics)
                {
                    records.extend(
                        generics
                            .split(',')
                            .map(|generic| (generic.trim().to_owned(), None)),
                    );
                }
            } else if in_enum && trimmed == "}" {
                in_enum = false;
            } else if in_enum && trimmed.ends_with(");") {
                let (variant, rest) = trimmed.split_once('(').expect("PRE-1 variant declaration");
                records.push((variant.to_owned(), Some(DeclarationClass::EnumVariant)));
                let fields = rest
                    .strip_suffix(");")
                    .expect("PRE-1 variant declaration ending");
                if !fields.is_empty() {
                    records.extend(fields.split(',').map(|field| {
                        let (name, _) = field
                            .split_once(':')
                            .expect("PRE-1 variant field declaration");
                        (name.trim().to_owned(), None)
                    }));
                }
            }
        }
        assert!(after.contains("The two built-in numeric bounds `Int` and `Float`"));
        records.push(("Int".to_owned(), Some(DeclarationClass::NumericBound)));
        records.push(("Float".to_owned(), Some(DeclarationClass::NumericBound)));
        records
    }

    fn extract_operation_families(spec: &str) -> Vec<&str> {
        let operation_section = spec.split_once("[OP-1]").expect("exact OP-1 opening").1;
        let mut rows = operation_section
            .lines()
            .skip_while(|line| !line.starts_with("| op |"))
            .skip(2);
        let mut seen = HashSet::new();
        let mut operations = Vec::new();
        for row in rows.by_ref().take_while(|line| line.starts_with("| `")) {
            let op_cell = row
                .strip_prefix('|')
                .and_then(|rest| rest.split_once('|'))
                .map(|(cell, _)| cell)
                .expect("OP-1 operation cell");
            for (index, part) in op_cell.split('`').enumerate() {
                if index % 2 == 1 && seen.insert(part) {
                    operations.push(part);
                }
            }
        }
        operations
    }
}
