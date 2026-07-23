use super::{
    DeclarationClass, OperationFamilyId, PreludeDeclarationId, PreludeDeclarationRecord,
    ReservedNameClass,
};

pub(crate) const PRELUDE_DECLARATIONS: [PreludeDeclarationRecord; 24] = [
    prelude(0, "Bool", Some(DeclarationClass::NominalType)),
    prelude(1, "True", Some(DeclarationClass::EnumVariant)),
    prelude(2, "False", Some(DeclarationClass::EnumVariant)),
    prelude(3, "Option", Some(DeclarationClass::NominalType)),
    prelude(4, "T", None),
    prelude(5, "None", Some(DeclarationClass::EnumVariant)),
    prelude(6, "Some", Some(DeclarationClass::EnumVariant)),
    prelude(7, "value", None),
    prelude(8, "Result", Some(DeclarationClass::NominalType)),
    prelude(9, "T", None),
    prelude(10, "E", None),
    prelude(11, "Ok", Some(DeclarationClass::EnumVariant)),
    prelude(12, "value", None),
    prelude(13, "Err", Some(DeclarationClass::EnumVariant)),
    prelude(14, "error", None),
    prelude(15, "Overflow", Some(DeclarationClass::NominalType)),
    prelude(16, "Overflow", Some(DeclarationClass::EnumVariant)),
    prelude(17, "DivError", Some(DeclarationClass::NominalType)),
    prelude(18, "DivideByZero", Some(DeclarationClass::EnumVariant)),
    prelude(19, "DivOverflow", Some(DeclarationClass::EnumVariant)),
    prelude(20, "NarrowError", Some(DeclarationClass::NominalType)),
    prelude(21, "NarrowError", Some(DeclarationClass::EnumVariant)),
    prelude(22, "Int", Some(DeclarationClass::Contract)),
    prelude(23, "Float", Some(DeclarationClass::Contract)),
];

const fn prelude(
    ordinal: u8,
    spelling: &'static str,
    class: Option<DeclarationClass>,
) -> PreludeDeclarationRecord {
    PreludeDeclarationRecord {
        id: PreludeDeclarationId::new(ordinal),
        spelling,
        class,
    }
}

/// Distinct OP-1 spellings in normative table order, with repeated `cvt`
/// collapsed at its first occurrence as required by OP-1.
pub(crate) const OPERATION_FAMILIES: [&str; 83] = [
    "iadd.wrap",
    "isub.wrap",
    "imul.wrap",
    "iadd.trap",
    "isub.trap",
    "imul.trap",
    "iadd.checked",
    "isub.checked",
    "imul.checked",
    "idiv.trap",
    "irem.trap",
    "idiv.checked",
    "irem.checked",
    "ineg.wrap",
    "ineg.trap",
    "ineg.checked",
    "ieq",
    "ine",
    "ilt",
    "ile",
    "igt",
    "ige",
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
    "len",
    "slice_of",
    "box_new",
    "arena_new",
    "array_new",
    "buffer_new",
    "iand",
    "ior",
    "ixor",
    "inot",
    "ishl.wrap",
    "ishr.wrap",
    "ishl.trap",
    "ishr.trap",
    "irotl",
    "irotr",
    "ipopcount",
    "iclz",
    "ictz",
    "ibswap",
    "imulhi",
    "iadd.sat",
    "isub.sat",
    "imul.sat",
    "imin",
    "imax",
    "iabs.wrap",
    "iabs.trap",
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

pub(crate) const MODE_WORDS: [&str; 5] = ["wrap", "trap", "checked", "sat", "strict"];

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
    MODE_WORDS
        .iter()
        .position(|candidate| *candidate == spelling)
        .and_then(|index| u16::try_from(index).ok())
        .map(|ordinal| (ReservedNameClass::ModeWord, ordinal))
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::{
        DeclarationClass, MODE_WORDS, OPERATION_FAMILIES, PRELUDE_DECLARATIONS, ReservedNameClass,
        reserved_name,
    };

    const EXACT_SPEC: &str = include_str!("../../../spec/kernel-spec-v0.14.md");

    #[test]
    fn exact_v0_14_catalogs_are_closed_and_unique_where_required() {
        assert_eq!(PRELUDE_DECLARATIONS.len(), 24);
        assert_eq!(OPERATION_FAMILIES.len(), 83);
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
        assert_eq!(
            reserved_name("ieq"),
            Some((ReservedNameClass::DotlessOperation, 16))
        );
        assert_eq!(
            reserved_name("cvt"),
            Some((ReservedNameClass::DotlessOperation, 38))
        );
        assert_eq!(
            reserved_name("wrap"),
            Some((ReservedNameClass::ModeWord, 0))
        );
    }

    #[test]
    fn catalogs_match_independent_extraction_from_exact_v0_14() {
        let extracted_prelude = extract_prelude_records(EXACT_SPEC);
        let catalog_prelude: Vec<_> = PRELUDE_DECLARATIONS
            .iter()
            .map(|record| (record.spelling.to_owned(), record.class))
            .collect();
        assert_eq!(catalog_prelude, extracted_prelude);

        assert_eq!(
            OPERATION_FAMILIES.as_slice(),
            extract_operation_families(EXACT_SPEC)
        );
    }

    fn extract_prelude_records(spec: &str) -> Vec<(String, Option<DeclarationClass>)> {
        let block = spec
            .split_once("[PRE-1] The prelude is exactly:\n\n```\n")
            .expect("exact PRE-1 opening")
            .1
            .split_once("\n```\n")
            .expect("exact PRE-1 closing")
            .0;
        let mut records = Vec::new();
        let mut in_enum = false;
        for line in block.lines() {
            let trimmed = line.trim();
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
            } else if let Some(contract) = trimmed.strip_prefix("contract ") {
                records.push((
                    contract
                        .strip_suffix(" {")
                        .expect("PRE-1 contract header")
                        .to_owned(),
                    Some(DeclarationClass::Contract),
                ));
            }
        }
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
