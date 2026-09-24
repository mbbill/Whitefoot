//! How one compiler stop is presented to the writer who has to act on it.
//!
//! A stop is one [`Record`]: an optional kind name, an optional location
//! resolved to the file the caller named, and the stage payload as labeled
//! fields. The driver adds the envelope it already owns -- category, stage,
//! and the numbered rule of a source rejection [DIAG-1] -- and renders the
//! whole record one of two ways. [`DiagnosticFormat::Text`] is a block whose
//! first line has the `file:line:column: error[RULE]: Kind` shape and whose
//! every other line is one `label: value`; [`DiagnosticFormat::Json`] prints
//! the same fields as one JSON object on one line.
//!
//! The payload types list their fields once, here, by exhaustive
//! destructuring. A field added to a payload fails to compile until it is
//! listed, so no payload value can silently drop out of what a writer reads,
//! and every label is the field's own name. A node path reaches this module
//! only beside its coordinate, and is printed as that coordinate's position:
//! child ordinals, source ordinals and byte offsets are identities a writer
//! cannot act on, and the blind-writer trial of 2026-08-28 recorded a writer
//! running `head -c` on their own program to decode them.
//!
//! Stops that are not source rejections -- resource ceilings, invocation
//! envelopes, internal invariants, target layout, backend -- carry
//! compiler-facing payloads with no writer repair. They keep their stage
//! value's `Debug` text as one `payload` field.

use core::fmt::{self, Write as _};

use crate::source::SourceBundle;
use crate::syntax::terminal::TerminalPredicate;
use crate::{
    CallRequirementDisposition, CanonicalIssue, ContractShapeIssue, DeclarationClass,
    DeclarationConflict, DeclarationDomain, DeclarationOrigin, ExpectedTerminals, LexicalUseRole,
    LookaheadPredicate, LoopInvariantProofObligation, PostconditionProofDisposition,
    ReservedDeclarationRole, ReservedNameClass, ResolutionIssueKind, SemanticIssueKind,
    SemanticLocation, SemanticUnsupported, SourceIssue, SourceOrigin, SourceProofObligation,
    StaticObligationDisposition, SyntaxCoordinate, SyntaxIssue, TerminalIssue,
    UndischargedCallRequirementDetail, UndischargedPostconditionDetail,
};

/// How a caller asks for compiler stops to be printed.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DiagnosticFormat {
    /// A summary line in the `file:line:column: error[RULE]: Kind` shape,
    /// then one indented `label: value` line per field.
    #[default]
    Text,
    /// One JSON object on one line, carrying the same field names.
    Json,
}

/// Renders a stop the command-line driver reached outside compilation: an
/// invalid option, an unreadable source, an unwritable output, or a host
/// toolchain that failed.
///
/// These stops carry one sentence and no payload, so the text form stays
/// that sentence on one line; the JSON form gives it the same envelope every
/// compilation stop has, with stage `Driver`.
#[must_use]
pub fn render_driver_failure(category: &str, message: &str, format: DiagnosticFormat) -> String {
    match format {
        DiagnosticFormat::Text => format!("whitefootc: {message}"),
        DiagnosticFormat::Json => Record {
            kind: None,
            at: None,
            detail: vec![("message", Value::Text(message.to_owned()))],
        }
        .json(&Head {
            verdict: String::new(),
            category,
            stage: "Driver",
            rule: None,
        }),
    }
}

/// The envelope the driver owns for one record.
pub(super) struct Head<'envelope> {
    /// The summary-line verdict: `error[RULE]` for a source rejection, the
    /// category in words otherwise.
    pub(super) verdict: String,
    pub(super) category: &'envelope str,
    pub(super) stage: &'envelope str,
    pub(super) rule: Option<&'envelope str>,
}

/// One compiler stop as labeled data, independent of its rendering.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Record {
    kind: Option<String>,
    at: Option<Place>,
    detail: Vec<(&'static str, Value)>,
}

impl Record {
    /// A rejection or capability stop whose payload lists its own fields,
    /// located at the coordinate its rule selected.
    pub(super) fn located<Issue: Report + ?Sized>(
        issue: &Issue,
        bundle: &SourceBundle,
        coordinate: SyntaxCoordinate,
        anchor: Anchor,
    ) -> Self {
        let mut fields = Fields {
            bundle,
            detail: Vec::new(),
        };
        let kind = issue.report(&mut fields);
        Self {
            kind: Some(kind),
            at: Place::resolve(bundle, coordinate, anchor),
            detail: fields.detail,
        }
    }

    /// A stop whose payload is a compiler-facing stage value.
    ///
    /// Its kind is the value's leading identifier, which for a derived
    /// `Debug` is the variant or type name.
    pub(super) fn opaque(payload: &dyn fmt::Debug) -> Self {
        let payload = format!("{payload:?}");
        let name_end = payload
            .find(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
            .unwrap_or(payload.len());
        let kind = payload[..name_end].to_owned();
        Self {
            kind: (!kind.is_empty()).then_some(kind),
            at: None,
            detail: vec![("payload", Value::Text(payload))],
        }
    }

    /// The payload fields in the text rendering, one `label: value` per line.
    pub(super) fn detail_text(&self) -> String {
        let mut text = String::new();
        for (index, (label, value)) in self.detail.iter().enumerate() {
            if index > 0 {
                text.push('\n');
            }
            push_field(&mut text, label, value);
        }
        text
    }

    /// The text block: one summary line, then one indented line per field.
    pub(super) fn text(&self, head: &Head<'_>) -> String {
        let mut text = String::new();
        match &self.at {
            Some(place) => push_position(&mut text, place),
            None => text.push_str("whitefootc"),
        }
        text.push_str(": ");
        text.push_str(&head.verdict);
        if let Some(kind) = &self.kind {
            text.push_str(": ");
            text.push_str(kind);
        }
        if let Some(rule) = head.rule {
            push_line(&mut text, "rule", rule);
        }
        if let Some(kind) = &self.kind {
            push_line(&mut text, "kind", kind);
        }
        push_line(&mut text, "category", head.category);
        push_line(&mut text, "stage", head.stage);
        if let Some(place) = &self.at {
            text.push_str("\n  at: ");
            push_position(&mut text, place);
            let _ = write!(text, "\n  bytes: {}..{}", place.start, place.end);
            // `source:` and `marker:` are the same width, so the marker's
            // carets sit under the bytes the coordinate names.
            text.push_str("\n  source: ");
            push_printable(&mut text, &place.text);
            text.push_str("\n  marker: ");
            let (offset, width) = place.marker();
            text.extend(core::iter::repeat_n(' ', offset));
            text.extend(core::iter::repeat_n('^', width));
        }
        for (label, value) in &self.detail {
            text.push_str("\n  ");
            push_field(&mut text, label, value);
        }
        text
    }

    /// One JSON object on one line, with the text rendering's field names.
    ///
    /// The marker is omitted: it is the text rendering of `at` and `bytes`,
    /// which the object carries as numbers.
    pub(super) fn json(&self, head: &Head<'_>) -> String {
        let mut envelope = Vec::with_capacity(4);
        if let Some(rule) = head.rule {
            envelope.push(("rule", rule));
        }
        if let Some(kind) = &self.kind {
            envelope.push(("kind", kind.as_str()));
        }
        envelope.push(("category", head.category));
        envelope.push(("stage", head.stage));
        let mut json = String::from("{");
        for (index, (key, value)) in envelope.into_iter().enumerate() {
            push_json_key(&mut json, key, index == 0);
            push_json_string(&mut json, value);
        }
        if let Some(place) = &self.at {
            json.push(',');
            push_json_place(&mut json, place, &place.text);
        }
        push_json_key(&mut json, "detail", false);
        push_json_fields(&mut json, &self.detail);
        json.push('}');
        json
    }
}

/// Which byte of a coordinate the reader is sent to.
#[derive(Clone, Copy)]
pub(super) enum Anchor {
    /// The coordinate's first byte: a written construct starts where it starts.
    Start,
    /// The first byte of a trivia gap that lies on the line the gap ends in.
    ///
    /// A gap that carries a line break begins at the end of the line before
    /// the one the writer must edit, so anchoring at its start quoted the
    /// enclosing item's header while the offending bytes sat lines below.
    LastLineOfGap,
}

/// One source coordinate in the terms the caller typed.
///
/// The file is the display path, so a record names the file the caller
/// named. The line is one-based. The column is one-based and counts
/// characters, and the marker is drawn in characters, so both agree with
/// what a terminal shows when the quoted line holds a multi-byte scalar; the
/// byte interval stays exact in `start` and `end`.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Place {
    file: String,
    line: u64,
    column: u64,
    start: u64,
    end: u64,
    /// The complete source line holding the anchor, decoded lossily.
    text: String,
    /// Characters of `text` before the anchor.
    anchor: usize,
    /// Characters of `text` the coordinate covers on this line.
    covered: usize,
}

impl Place {
    /// Resolves one coordinate, or `None` when the bundle does not hold it.
    ///
    /// This is presentation: a stage that already has a verdict must still
    /// deliver it, so an unresolvable coordinate leaves the record unlocated
    /// rather than failing the compilation.
    fn resolve(
        bundle: &SourceBundle,
        coordinate: SyntaxCoordinate,
        anchor: Anchor,
    ) -> Option<Self> {
        let file = bundle.file(coordinate.source())?;
        let bytes = file.bytes();
        let start = usize::try_from(coordinate.start().value())
            .ok()?
            .min(bytes.len());
        let end = usize::try_from(coordinate.end().value())
            .ok()?
            .min(bytes.len())
            .max(start);
        let line_start_before = |offset: usize| {
            bytes[..offset]
                .iter()
                .rposition(|byte| *byte == b'\n')
                .map_or(0, |index| index.saturating_add(1))
        };
        let anchored = match anchor {
            Anchor::Start => start,
            Anchor::LastLineOfGap => start.max(line_start_before(end)),
        };
        let line_start = line_start_before(anchored);
        let line_end = bytes[line_start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(bytes.len(), |offset| line_start.saturating_add(offset));
        let line = bytes[..line_start]
            .iter()
            .filter(|byte| **byte == b'\n')
            .count()
            .saturating_add(1);
        let characters = |range: &[u8]| String::from_utf8_lossy(range).chars().count();
        let before = characters(&bytes[line_start..anchored]);
        Some(Self {
            file: file.display_path().to_owned(),
            line: u64::try_from(line).ok()?,
            column: u64::try_from(before.saturating_add(1)).ok()?,
            start: coordinate.start().value(),
            end: coordinate.end().value(),
            text: String::from_utf8_lossy(&bytes[line_start..line_end]).into_owned(),
            anchor: before,
            covered: characters(&bytes[anchored..end.min(line_end).max(anchored)]),
        })
    }

    /// The marker's display offset and width under the printable line; a
    /// zero-width coordinate is one caret at its position.
    fn marker(&self) -> (usize, usize) {
        let width = |characters: &mut dyn Iterator<Item = char>| {
            characters
                .map(|character| {
                    let mut escaped = String::new();
                    push_printable_char(&mut escaped, character);
                    escaped.chars().count()
                })
                .sum::<usize>()
        };
        let offset = width(&mut self.text.chars().take(self.anchor));
        let covered = width(&mut self.text.chars().skip(self.anchor).take(self.covered));
        (offset, covered.max(1))
    }
}

/// One field value, typed by how a reader must read it.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Value {
    /// Prose, a name, or a rendered source relation: printed as written.
    Text(String),
    /// Exact source bytes whose spacing or escapes matter: printed quoted.
    Exact(Vec<u8>),
    Number(u64),
    List(Vec<Value>),
    /// Another source position the payload names.
    Place(Box<Place>),
    /// A structured payload element, such as one declaration conflict.
    Fields(Vec<(&'static str, Value)>),
}

/// The fields one payload contributes, in declaration order.
pub(super) struct Fields<'bundle> {
    bundle: &'bundle SourceBundle,
    detail: Vec<(&'static str, Value)>,
}

impl Fields<'_> {
    /// Adds one field; an absent optional value adds nothing.
    fn field<Field: FieldValue + ?Sized>(&mut self, label: &'static str, value: &Field) {
        if let Some(value) = value.value(self.bundle) {
            self.detail.push((label, value));
        }
    }
}

/// A payload that lists its own fields and names its own kind.
pub(super) trait Report {
    /// Adds every payload field to `fields` and returns the kind name.
    fn report(&self, fields: &mut Fields<'_>) -> String;
}

/// A payload structure whose fields flatten into the record that holds it.
trait FieldList {
    fn fields(&self, fields: &mut Fields<'_>);
}

/// One payload value as a [`Value`].
trait FieldValue {
    fn value(&self, bundle: &SourceBundle) -> Option<Value>;
}

impl<Field: FieldValue + ?Sized> FieldValue for &Field {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        (**self).value(bundle)
    }
}

impl<Field: FieldValue + ?Sized> FieldValue for Box<Field> {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        (**self).value(bundle)
    }
}

impl<Field: FieldValue> FieldValue for Option<Field> {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        self.as_ref().and_then(|value| value.value(bundle))
    }
}

impl<Field: FieldValue> FieldValue for [Field] {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        Some(Value::List(
            self.iter().filter_map(|item| item.value(bundle)).collect(),
        ))
    }
}

impl<Field: FieldValue> FieldValue for Vec<Field> {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        self.as_slice().value(bundle)
    }
}

impl FieldValue for str {
    fn value(&self, _: &SourceBundle) -> Option<Value> {
        Some(Value::Text(self.to_owned()))
    }
}

impl FieldValue for String {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        self.as_str().value(bundle)
    }
}

impl FieldValue for u16 {
    fn value(&self, _: &SourceBundle) -> Option<Value> {
        Some(Value::Number(u64::from(*self)))
    }
}

impl FieldValue for u32 {
    fn value(&self, _: &SourceBundle) -> Option<Value> {
        Some(Value::Number(u64::from(*self)))
    }
}

/// Exact bytes that must be read quoted, such as trivia a canonical-form
/// rejection wanted and found.
struct Exact<'text>(&'text str);

impl FieldValue for Exact<'_> {
    fn value(&self, _: &SourceBundle) -> Option<Value> {
        Some(Value::Exact(self.0.as_bytes().to_vec()))
    }
}

/// The source bytes one coordinate covers, which a lexical, terminal or
/// grammar rejection found where it expected something else.
struct Spelled(SyntaxCoordinate);

impl FieldValue for Spelled {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        let bytes = bundle.file(self.0.source())?.bytes();
        let start = usize::try_from(self.0.start().value()).ok()?;
        let end = usize::try_from(self.0.end().value()).ok()?;
        if start == end && end == bytes.len() {
            return Some(Value::Text("end of source".to_owned()));
        }
        Some(Value::Exact(bytes.get(start..end)?.to_vec()))
    }
}

impl FieldValue for SyntaxCoordinate {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        Some(Place::resolve(bundle, *self, Anchor::Start).map_or_else(
            || Value::Text("an unresolved coordinate".to_owned()),
            |place| Value::Place(Box::new(place)),
        ))
    }
}

impl FieldValue for SemanticLocation {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        self.coordinate().value(bundle)
    }
}

impl FieldValue for SourceOrigin {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        self.coordinate().value(bundle)
    }
}

impl FieldValue for DeclarationOrigin {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        match self {
            Self::Source(origin) => origin.value(bundle),
            // The colliding spelling is already in the record; the prelude
            // ordinal adds nothing a writer can act on.
            Self::Prelude(_) => Some(Value::Text("prelude".to_owned())),
        }
    }
}

impl FieldValue for DeclarationConflict {
    fn value(&self, bundle: &SourceBundle) -> Option<Value> {
        let mut fields = Fields {
            bundle,
            detail: Vec::new(),
        };
        fields.field("domain", &self.domain());
        fields.field("class", &self.class());
        fields.field("origin", self.origin());
        Some(Value::Fields(fields.detail))
    }
}

/// The expected set as a writer would write it: a fixed terminal quoted as
/// its exact spelling, a token class by its specification name, and the end
/// of the source in words.
impl FieldValue for ExpectedTerminals {
    fn value(&self, _: &SourceBundle) -> Option<Value> {
        Some(Value::List(
            self.iter()
                .map(|predicate| match predicate {
                    LookaheadPredicate::Terminal(TerminalPredicate::Fixed(terminal)) => {
                        Value::Exact(terminal.spelling_bytes().to_vec())
                    }
                    LookaheadPredicate::Terminal(class) => Value::Text(class.spelling().to_owned()),
                    LookaheadPredicate::SourceEnd => Value::Text("end of source".to_owned()),
                })
                .collect(),
        ))
    }
}

/// Closed classifications whose `Debug` spelling is their variant name and
/// whose data, where any, is a small source ordinal.
macro_rules! named_by_debug {
    ($($name:ty),* $(,)?) => {
        $(
            impl FieldValue for $name {
                fn value(&self, _: &SourceBundle) -> Option<Value> {
                    Some(Value::Text(format!("{self:?}")))
                }
            }
        )*
    };
}

named_by_debug!(
    CallRequirementDisposition,
    ContractShapeIssue,
    DeclarationClass,
    DeclarationDomain,
    LexicalUseRole,
    LoopInvariantProofObligation,
    PostconditionProofDisposition,
    ReservedDeclarationRole,
    ReservedNameClass,
    SourceProofObligation,
    StaticObligationDisposition,
);

/// Lists every variant of one payload enum with its fields.
///
/// `Variant { a, b }` adds fields `a` and `b`; `Variant(name)` adds its one
/// value as field `name`; `Variant[detail]` flattens a detail structure's
/// fields into the record. The match has no wildcard and the patterns have no
/// rest, so the listing cannot fall behind the enum.
macro_rules! report_variants {
    ($enum:ident, $value:expr, $fields:ident;
     $( $variant:ident
        $( { $($field:ident),* $(,)? } )?
        $( ( $single:ident ) )?
        $( [ $flat:ident ] )?
     ; )*
    ) => {
        match $value {
            $(
                $enum::$variant $( { $($field),* } )? $( ( $single ) )? $( ( $flat ) )? => {
                    $( $( $fields.field(stringify!($field), $field); )* )?
                    $( $fields.field(stringify!($single), $single); )?
                    $( FieldList::fields(&**$flat, $fields); )?
                    stringify!($variant).to_owned()
                }
            )*
        }
    };
}

/// Lists every field of one payload structure, in declaration order.
macro_rules! list_fields {
    ($value:expr, $fields:ident; $name:ident { $($field:ident),* $(,)? }) => {{
        let $name { $($field),* } = $value;
        $( $fields.field(stringify!($field), $field); )*
    }};
}

impl FieldList for UndischargedCallRequirementDetail {
    fn fields(&self, fields: &mut Fields<'_>) {
        list_fields!(self, fields; UndischargedCallRequirementDetail {
            concrete_callee,
            requires_clause,
            instantiated_goal,
            disposition,
            mechanical_fix,
        });
    }
}

impl FieldList for UndischargedPostconditionDetail {
    fn fields(&self, fields: &mut Fields<'_>) {
        list_fields!(self, fields; UndischargedPostconditionDetail {
            concrete_function,
            postcondition,
            conjunct,
            selector,
            relation,
            disposition,
        });
    }
}

impl Report for SemanticIssueKind {
    fn report(&self, fields: &mut Fields<'_>) -> String {
        report_variants!(SemanticIssueKind, self, fields;
            InvalidIntegerLiteral;
            InvalidFloatLiteral;
            InvalidConstValue;
            ConstEvalOverflow { operation };
            ConstRuntimeArithmeticMode { mechanical_fix };
            TypeMismatch { expected, found };
            ImmutableSetTarget;
            InvalidSetTarget { root_class, required_classes };
            ImmutableWrittenArgument { binding, mechanical_fix };
            LinearValueNotConsumed { binding, obligation, mechanical_fix };
            LinearValuePartiallyConsumed { obligation, residual, mechanical_fix };
            LinearityBoundMismatch { parameter, bound, argument, actual };
            LivenessJoinDisagreement { binding, live_predecessor, dead_predecessor, mechanical_fix };
            LinearAssignmentTarget { target_type, mechanical_fix };
            InlineRuntimeCapacityShape { spelling, mechanical_fix };
            SwapOverCopyPlace { place_type, mechanical_fix };
            HeapTypeUnderNoHeap { spelling, mechanical_fix };
            InvalidElementMove { mechanical_fix };
            MoveThroughReference { mechanical_fix };
            ReservedPseudoField { spelling, mechanical_fix };
            ReadonlyWriteTarget { spelling, mechanical_fix };
            MoveOfCopy { mechanical_fix };
            BareAffineUse { mechanical_fix };
            ContainerConstruction { nominal, mechanical_fix };
            UseAfterMove { mechanical_fix };
            InvalidReferenceUse { binder, event, mechanical_fix };
            ReferenceToReferenceVariable { mechanical_fix };
            EscapingReference { mechanical_fix };
            RangeOverRing { mechanical_fix };
            MoveOutOfSlot { mechanical_fix };
            MissingDereference { mechanical_fix };
            MoveOuterBindingInLoop { binding, mechanical_fix };
            InvalidOperation;
            InvalidPredicateCondition;
            InvalidConditionalForm { mechanical_fix };
            UndischargedBoundsObligation { residual, mechanical_fix };
            UndischargedEmptyRunRelease { residual, mechanical_fix };
            UndischargedIntegerDomainObligation { residual, disposition, mechanical_fix };
            UndischargedConversionDomainObligation { residual, disposition, mechanical_fix };
            UndischargedAllocationFitObligation { residual, mechanical_fix };
            UndischargedRangeFormationObligation { residual, mechanical_fix };
            UndischargedCallSeparation { residual, mechanical_fix };
            OverlappingCallEffects { first, second, mechanical_fix };
            AtomicUpdateReachesTargetPrefix { target, effect, mechanical_fix };
            UndischargedCallRequirement[detail];
            InvalidCountedEndpoint { mechanical_fix };
            BreakOutsideLoop { mechanical_fix };
            InvalidInvariant { reason, mechanical_fix };
            UndischargedLoopInvariant { name, obligation, required_relation, mechanical_fix };
            UndischargedLocalInvariant { name, mechanical_fix };
            InvalidSourceProof { reason, mechanical_fix };
            UndischargedSourceProof { name, obligation, mechanical_fix };
            ReturnMismatch;
            InvalidMusttail { condition, subject };
            PolymorphicRecursion { cycle, mechanical_fix };
            UnreachableStatement;
            FunctionFallthrough;
            InvalidRequires;
            InvalidPostconditionSelector;
            AmbiguousResultRoute;
            InvalidPostconditionFields { required_fields };
            PostconditionCandidateNotFresh { spelling, conflicts };
            PostconditionLocalShadowsResult { spelling, selector };
            InvalidPostconditionClause;
            InvalidPostconditionRelation;
            InvalidEntryFormer { mechanical_fix };
            ContradictoryPublishedRelations { relations, mechanical_fix };
            InvalidPostconditionReturn;
            NoSelectedNormalExit { residual };
            UndischargedPostcondition[detail];
            InvalidNamedArguments { callee, declared_parameters };
            DuplicateFieldLabel { label };
            InvalidConstructionFields { constructor, declared_fields };
            InvalidMatchFields { variant, declared_fields };
            ForeignMatchVariant;
            NonExhaustiveMatch { missing_variants };
            InvalidPropagation;
            InvalidGive;
            InvalidEffectRow { reason, mechanical_fix };
            EffectMismatch { expected_row, found_row, missing, extra, mechanical_fix };
            SourceContractGenericBound;
        )
    }
}

impl Report for ResolutionIssueKind {
    fn report(&self, fields: &mut Fields<'_>) -> String {
        report_variants!(ResolutionIssueKind, self, fields;
            ContractShape(shape);
            MisplacedHeapDeclaration { admitted };
            ReservedName { spelling, declaration_role, class, inventory_ordinal };
            MatchBinderFreshness { spelling, paired_field, earlier_binder, arm_entry_conflicts };
            DeclarationCollision { spelling, conflicts, mechanical_fix };
            InvisibleUse { spelling, role, admissible, origins };
            NonEnclosingLabel { spelling, role, origins };
            UnresolvedUse { spelling, role, admissible, available };
        )
    }
}

/// [DIAG-1]'s raw lexical defects: the kind is the defect shape, and the
/// offending bytes are printed escaped because they are often invisible.
impl Report for SourceIssue<'_> {
    fn report(&self, fields: &mut Fields<'_>) -> String {
        let span = self.span();
        fields.field(
            "found",
            &Spelled(SyntaxCoordinate::new(
                span.source(),
                span.start(),
                span.end(),
            )),
        );
        format!("{:?}", self.kind())
    }
}

/// A formed token that satisfies no terminal predicate [GRAM-1].
impl Report for TerminalIssue<'_> {
    fn report(&self, fields: &mut Fields<'_>) -> String {
        let token = self.token();
        fields.field(
            "found",
            &Spelled(SyntaxCoordinate::new(
                token.source(),
                token.start(),
                token.end(),
            )),
        );
        "UnclassifiedToken".to_owned()
    }
}

/// The grammar's expected-terminal set at the failure boundary and what the
/// source has there instead [DIAG-1].
impl Report for SyntaxIssue {
    fn report(&self, fields: &mut Fields<'_>) -> String {
        let Self {
            rule: _,
            coordinate,
            expected,
            mechanical_fix,
        } = self;
        fields.field("expected", expected);
        fields.field("found", &Spelled(*coordinate));
        fields.field("mechanical_fix", mechanical_fix);
        "UnexpectedToken".to_owned()
    }
}

/// The trivia bytes [FORM-2] requires between two terminals and the bytes the
/// source carries there.
impl Report for CanonicalIssue {
    fn report(&self, fields: &mut Fields<'_>) -> String {
        let Self {
            location: _,
            expected,
            found,
        } = self;
        fields.field("expected", &Exact(expected));
        fields.field("found", &Exact(found));
        "NonCanonicalTrivia".to_owned()
    }
}

/// A valid source that needs a semantic family the compiler has not built.
impl Report for SemanticUnsupported {
    fn report(&self, _: &mut Fields<'_>) -> String {
        format!("{:?}", self.feature())
    }
}

fn push_line(text: &mut String, label: &str, value: &str) {
    text.push_str("\n  ");
    text.push_str(label);
    text.push_str(": ");
    text.push_str(value);
}

fn push_field(text: &mut String, label: &str, value: &Value) {
    text.push_str(label);
    text.push_str(": ");
    write_text(text, value);
}

fn push_position(text: &mut String, place: &Place) {
    let _ = write!(text, "{}:{}:{}", place.file, place.line, place.column);
}

/// Text-rendering of one value; a value never spans two lines.
fn write_text(text: &mut String, value: &Value) {
    match value {
        Value::Text(value) => push_printable(text, value),
        Value::Exact(bytes) => push_quoted(text, bytes),
        Value::Number(number) => {
            let _ = write!(text, "{number}");
        }
        Value::List(items) => {
            text.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    text.push_str(", ");
                }
                write_text(text, item);
            }
            text.push(']');
        }
        // Another position a payload names: where it is, and its line with
        // the indentation removed, which is the clause or declaration there.
        Value::Place(place) => {
            push_position(text, place);
            text.push(' ');
            push_quoted(text, place.text.trim().as_bytes());
        }
        Value::Fields(fields) => {
            text.push('{');
            for (index, (label, value)) in fields.iter().enumerate() {
                if index > 0 {
                    text.push_str(", ");
                }
                push_field(text, label, value);
            }
            text.push('}');
        }
    }
}

/// Source text as written, with control bytes escaped so one field stays one
/// line and a tab cannot shift the marker.
fn push_printable(text: &mut String, value: &str) {
    for character in value.chars() {
        push_printable_char(text, character);
    }
}

fn push_printable_char(text: &mut String, character: char) {
    match character {
        '\n' => text.push_str("\\n"),
        '\r' => text.push_str("\\r"),
        '\t' => text.push_str("\\t"),
        control if control.is_ascii_control() => {
            let _ = write!(text, "\\x{:02x}", u32::from(control));
        }
        other => text.push(other),
    }
}

/// Exact bytes in quotes, every byte that is not printable ASCII escaped: a
/// non-ASCII scalar as `\u{...}` and a byte that begins no valid scalar as
/// `\x..`, so an invisible or confusable defect is visible.
fn push_quoted(text: &mut String, bytes: &[u8]) {
    text.push('"');
    for chunk in bytes.utf8_chunks() {
        for character in chunk.valid().chars() {
            match character {
                '"' => text.push_str("\\\""),
                '\\' => text.push_str("\\\\"),
                ' '..='~' => text.push(character),
                control if control.is_ascii_control() => push_printable_char(text, control),
                other => {
                    let _ = write!(text, "\\u{{{:x}}}", u32::from(other));
                }
            }
        }
        for byte in chunk.invalid() {
            let _ = write!(text, "\\x{byte:02x}");
        }
    }
    text.push('"');
}

fn push_json_key(json: &mut String, key: &str, first: bool) {
    if !first {
        json.push(',');
    }
    push_json_string(json, key);
    json.push(':');
}

/// A JSON string [RFC 8259]: quote, backslash and control characters are
/// escaped; every other scalar is written as UTF-8.
fn push_json_string(json: &mut String, value: &str) {
    json.push('"');
    for character in value.chars() {
        match character {
            '"' => json.push_str("\\\""),
            '\\' => json.push_str("\\\\"),
            '\n' => json.push_str("\\n"),
            '\r' => json.push_str("\\r"),
            '\t' => json.push_str("\\t"),
            control if u32::from(control) < 0x20 || control == '\u{7f}' => {
                let _ = write!(json, "\\u{:04x}", u32::from(control));
            }
            other => json.push(other),
        }
    }
    json.push('"');
}

/// A position as the three fields every located record carries: `at`,
/// `bytes`, and `source`.
fn push_json_place(json: &mut String, place: &Place, source: &str) {
    push_json_key(json, "at", true);
    json.push('{');
    push_json_key(json, "file", true);
    push_json_string(json, &place.file);
    let _ = write!(
        json,
        ",\"line\":{},\"column\":{}}},\"bytes\":{{\"start\":{},\"end\":{}}}",
        place.line, place.column, place.start, place.end
    );
    push_json_key(json, "source", false);
    push_json_string(json, source);
}

fn push_json_fields(json: &mut String, fields: &[(&'static str, Value)]) {
    json.push('{');
    for (index, (label, value)) in fields.iter().enumerate() {
        push_json_key(json, label, index == 0);
        push_json_value(json, value);
    }
    json.push('}');
}

fn push_json_value(json: &mut String, value: &Value) {
    match value {
        Value::Text(value) => push_json_string(json, value),
        Value::Exact(bytes) => push_json_string(json, &String::from_utf8_lossy(bytes)),
        Value::Number(number) => {
            let _ = write!(json, "{number}");
        }
        Value::List(items) => {
            json.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    json.push(',');
                }
                push_json_value(json, item);
            }
            json.push(']');
        }
        Value::Place(place) => {
            json.push('{');
            push_json_place(json, place, place.text.trim());
            json.push('}');
        }
        Value::Fields(fields) => push_json_fields(json, fields),
    }
}

#[cfg(test)]
mod tests;
