# Grammar verifier engine format version 1 and runner policy version 2

This file defines the complete interchange contract. Unknown fields, tags,
records, input files, or declaration shapes are errors. All offsets are
zero-based half-open byte offsets into the exact bound input.

## Invocation inputs

The installed-v0.9 entry point reads seven fixed content inputs selected by
code, never by framed input:

1. `limits.txt`
2. exact `../spec/kernel-spec-v0.8.md`
3. `proposal/kernel-spec-successor-candidate.md`
4. exact `../spec/kernel-spec-v0.9.md`
5. `cases.txt`
6. `domains.txt`
7. `expectations.txt`

Expected answers are bound into the final evidence package but are never sent
to either engine. The installed v0.9 bytes must be byte-for-byte identical to
the reviewed candidate and must have SHA-256
`bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`.
Only one copy of those equal successor bytes occupies the `proposal` frame
section. The frame otherwise contains the limits, v0.8 comparison baseline,
cases, and domains; it excludes the expectation file.

The immutable owner-review package records the earlier review-mode run, in
which the candidate supplied the same successor frame bytes before installation.
`run.py` never republishes that package. It validates every historical packet
artifact by its approved digest, then writes a distinct installed-v0.9 package.
The same pre-run validation pins the frozen v0.8 lexical snapshot and requires
the installed derivation ledger to be the pinned pre-v0.9 bytes followed by one
LF and the exact delimited body of the approved v0.9 amendment.

The current document must have SHA-256
`d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8`.

## Engine frame

The frame is:

```text
8 bytes   ASCII WFGRAMV1
8 bytes   big-endian limits length
8 bytes   big-endian current-document length
8 bytes   big-endian proposal-document length
8 bytes   big-endian authored-case length
8 bytes   big-endian generated-domain length
N bytes   exact sections in that order
```

There is exact EOF after the domain section. Before parsing `limits.txt`, each
engine enforces hard outer caps of 8192 profile bytes, 1 MiB per document, and
256 KiB per case or domain file. Length arithmetic is checked before slicing
or allocation. Each engine echoes the byte length and SHA-256 of all five
sections in its raw report.

## Limits

`limits.txt` is strict ASCII. Every line is `lower_snake_name=positive_decimal`
with one terminal LF, sorted bytewise by name. Values have no sign or leading
zero. Missing, extra, duplicate, reordered, or unknown lines are input errors.
Every engine collection whose size depends on an input is checked against its
named semantic or structural limit before growth. The runner accepts at most
`max_engine_output_bytes` per raw report and `max_line_bytes` per record before
building its validation indexes; its later semantic counts are checked against
the same authored workload limits.

A named limit hit is `resource_inconclusive`. Host allocation failure is also
inconclusive when the engine can emit its fixed small response. If allocation
failure prevents that response, the runner classifies the missing or abnormal
report as `tool_inconclusive`; an OS kill is likewise `tool_inconclusive`.

The committed format-v1 values are hard maxima, not suggestions. A test may
reduce any value while retaining a positive canonical decimal; neither the
runner nor an engine accepts a larger value. The maxima are:

```text
cpu_timeout_seconds=60
max_case_bytes=131072
max_cases=1024
max_definitions=1024
max_document_bytes=524288
max_domain_bytes=131072
max_domains=64
max_ebnf_depth=128
max_engine_output_bytes=8388608
max_final_report_bytes=16777216
max_generated_streams=100000
max_grammar_nodes=65536
max_lexical_definitions=128
max_line_bytes=16384
max_lines=4096
max_rules=1024
max_symbol_bytes=256
max_terminal_occurrences=8192
oracle_max_chart_items=1000000
oracle_max_packed_edges=1000000
oracle_max_proof_nodes=1000000
oracle_max_source_tokens=256
static_max_lookahead_words=262144
static_max_product_states=1000000
static_max_work=10000000
wall_timeout_seconds=60
```

## Authored cases and generated domains

`cases.txt` begins with `whitefoot.grammar-cases.v1`. Remaining lines are:

```text
case<TAB>ASCII-id<TAB>ASCII-start-nonterminal<TAB>hex-source-bytes
```

IDs are exactly `[a-z][a-z0-9-]*`; start nonterminals are exactly
`[a-z][a-z0-9_]*`. Both are nonempty and at most `max_symbol_bytes` bytes.
Case IDs are logically unique and records are sorted bytewise. The start
nonterminal must exist in each independently extracted document. Source bytes
are strict lowercase even-length hex and nonempty. The file contains no
expected derivation counts.

`domains.txt` begins with `whitefoot.grammar-domains.v1`. Version 1 admits:

```text
domain<TAB>ASCII-id<TAB>fixed-lowerword-call<TAB>ASCII-start-nonterminal<TAB>hex-argument
```

For each document, each engine independently validates the start nonterminal,
obtains the expanded fixed lowerword inventory from its own grammar extraction,
and enumerates `word(argument)` in spelling order. It emits the exact stream
count and the SHA-256 of the generated inventory. Each stream contributes an
eight-byte unsigned big-endian byte length followed by its exact bytes; those
records are concatenated in spelling order before hashing. A runner-generated
corpus is never substituted for independent generation.

`expectations.txt` is runner/test policy and is not engine input. Its first
line is `whitefoot.grammar-expectations.v2`; remaining tab-separated records
are kept sorted and use these closed forms:

```text
case<TAB>ASCII-id<TAB>current|proposal<TAB>zero|one|many
case-delta<TAB>ASCII-id<TAB>trace-identical|trace-subset|trace-replacement
transition<TAB>ASCII-id<TAB>ASCII-status
```

Every authored case has two `case` records and one `case-delta` record.
`trace-identical` requires equal derivation-tree sets. `trace-subset` requires
every proposal tree to be a current tree and permits only removals.
`trace-replacement` requires the tree sets to be disjoint. It is used when
source-preserving grammar factoring intentionally changes typed tree shape or
when a proposal deliberately moves a closed restriction from syntax formation
to a later semantic check; the paired case counts distinguish replacement from
intentional introduction. These are exact tree policies, not merely
derivation-count policies.
Policy version 2 adds `case-delta`; it does not change the version-1 engine
frame or raw-report wire contract.

## Closed specification meta-notation

The extractor recognizes a deliberately small notation, not arbitrary
English:

- strict UTF-8, LF only, and no trailing SP or HTAB;
- numbered rule headings at column zero, outside fenced code;
- exact untagged column-zero triple-backtick fences;
- grammar definitions only in `GRAM-*` fences or complete inline-code bodies;
- every nonblank grammar-fence line is a definition head or an indented
  continuation;
- grammar layout uses ASCII SP, never HTAB;
- every `:=` occurrence in the document belongs to exactly one definition;
- a single-`=` lowercase grammar head is an unsupported grammar candidate;
- EBNF has names, sequence, `|`, grouping, `?`, `*`, and `+`;
- quoted atoms are nonempty printable ASCII excluding quote and backslash,
  with no escape syntax; `[0-9]+` is the one pattern atom;
- line comments are accepted only in the exact existing grammar-line position;
- duplicate definitions, undefined references, and nullable repetition bodies
  are extraction errors;
- exact `program` is the root nonterminal, and every extracted production must
  be reachable from it through production references.

`Lexical classes:` is not a general regular-expression syntax. Format v1
accepts exactly these five name/pattern pairs, in this order, with exact
semicolon/space separators and the exact shown annotations:

```text
IDENT `[a-z][a-z0-9_]*`; TYPEID `[A-Z][A-Za-z0-9]*`; REGIONID `'[a-z][a-z0-9_]*` (apostrophe-prefixed, the only region spelling); LABEL `@[a-z][a-z0-9_]*`; OPNAME `[a-z][a-z0-9_]*\.(wrap|trap|checked|sat|strict)` (single token; the base is an IDENT and the mode suffix is a closed word set, so an OPNAME can never maximal-munch a field-access place `p.field` [GRAM-5]; e.g. `iadd.checked`).
```

The only permitted variation is the exact IDENT modifier below, inserted
immediately after the IDENT pattern. It may occur zero or one time:

The proposal's reviewed IDENT modifier is exactly:

```text
 excluding every lowercase token spelling produced by exact fixed grammar atoms in the complete grammar
```

It changes the IDENT membership predicate, so the lexical definition's
semantic span includes the modifier. Fixed quoted atoms are independently
expanded into lexical token shapes; for example `&uniq` contributes `&` and
`uniq`, so the source and expanded lowerword inventories are reported
separately.

`Literals, exhaustively:` is the exact FORM-5 declaration handler for the two
bound documents; it does not admit a user-selected literal syntax. Exact
uppercase `[A-Z][A-Z0-9_]* is a closed table:` declarations remain the generic
closed-table handler. High-signal variants with no exact handler are
extraction errors. Every external grammar reference resolves to an
independently extracted lexical definition. Any new or changed FORM-3 pattern
requires a new format-version decision and two independently implemented
handlers.

Unknown meta-notation stops. Adding a notation requires a format-version
decision and independent handlers; neither engine may silently ignore it.

## Raw reports

Raw reports are strict ASCII with terminal LF. The first line is
`WFGRREPORT1`; the second is `ENGINE<TAB>static` or `ENGINE<TAB>oracle`.
Fields use one TAB separator. Arbitrary source bytes are lowercase hex.
Integers are unsigned canonical decimal. SHA-256 is lowercase hexadecimal.

Both reports contain a `COMMON-BEGIN` / `COMMON-END` block. Common records
bind each document's identity, rules, complete grammar-candidate census,
definitions, structural EBNF nodes, lexical definitions and predicates, fixed
terminal occurrences and expansions, external-reference occurrences, and the
zero-unclassified-surface success result. The exact common record layouts are:

Each engine charges every LF-terminated record against
`max_engine_output_bytes` before retaining it in an output collection. Record
construction may use bounded per-record scratch space derived only from already
bounded fields, but both that record and the cumulative retained report bytes
must fit before the record is inserted.

The visual spaces between fields below each denote one TAB byte.

```text
BIND name byte_length sha256
RULE doc owner_hex start end
SURFACE doc kind start end owner_hex
PROD doc owner_hex lhs_hex definition_start definition_end rhs_start rhs_end
NODE doc lhs_hex path kind start end value_hex
LEX doc owner_hex name_hex kind start end predicate_hex
FIXED doc lhs_hex path start end spelling_hex expansion_hex
REF doc lhs_hex path start end name_hex
COVERAGE doc assignment_count fence_count inline_count lexical_cue_count unclassified_count
```

`doc` is `current` or `proposal`. `path` is dot-separated zero-based child
indices rooted at `0`; child `i` appends `.i`. The closed NODE kinds are
`ref`, `fixed`, `pattern`, `sequence`, `choice`, `group`, `optional`,
`repeat0`, and `repeat1`. A leaf span includes the exact source name or complete
quoted atom. A structural span covers its first through last syntax byte,
including group or postfix delimiters. A leaf value is the lowercase hex of
the referenced symbol, unquoted fixed spelling, or unquoted pattern spelling;
a structural value is `-`.

The closed LEX kinds and decoded canonical ASCII predicate descriptors are:

```text
regex         pattern=<exact-source-pattern>;exclude=none
regex         pattern=<exact-source-pattern>;exclude=fixed-lowerwords
literal-union integer=-?[0-9]+_TYPE;float=-?[0-9]+\.[0-9]+(e-?[0-9]+)?_TYPE;unit=unit;generic=0_T,1_T
byte-string   range=32-126;exclude=34,92;escapes=backslash,quote,n;contexts=doc,check
closed-table  associative(f),commutative(f),identity(f,e)
```

The record's `predicate_hex` field is lowercase hex encoding of the complete
descriptor bytes, not raw descriptor text. A regex lexical span ends after
its source pattern and any semantic fixed-lowerword exclusion modifier, but
before a later reviewed explanatory annotation. The literal-union span is the
complete FORM-5 payload line; the byte-string span is the exact nested STRING
membership clause; the closed-table span is its cue through line end.

`pattern` NODEs and `[0-9]+` are not FIXED records. Each FIXED expansion is a
comma-separated canonical ASCII descriptor in source-token order, with each
atom spelled `<kind>:<lowercase-hex-source-bytes>`. The closed atom kinds are
`lowerword`, `identifier`, `ampersand`, `thin-arrow`, `fat-arrow`, and
`punctuation`. The `expansion_hex` field is lowercase hex encoding of that
complete descriptor. Compound fixed atoms split only as one non-name prefix
followed by one name-shaped suffix; for example `&uniq` decodes to
`ampersand:26,lowerword:756e6971`.

The closed SURFACE kinds are `grammar-fence`, `grammar-inline`, `assignment`,
and `lexical-cue`. A fence span runs from opener line start through closer line
end including its LF. An inline span is its body, excluding backticks. An
assignment span is the exact `:=`. A lexical-cue span is the exact accepted cue
including its colon. Only grammar-candidate fences appear on a successful
ledger. RULE spans run from an outside-fence heading through the next such
heading or EOF. PROD and RHS spans end at the last EBNF token before any SP plus
`#` comment or LF.

The five BIND records appear first in `limits`, `current`, `proposal`, `cases`,
and `domains` order. Remaining records are ordered by document (`current`,
`proposal`), then primary start offset, tag rank in the layout order above,
and remaining fields bytewise; COVERAGE is last for each document. An absent
owner or value is `-`. The runner validates framing, closed tags, field counts,
canonical encodings, source bounds, uniqueness, and order, then compares the
complete common block byte for byte. It does not re-extract EBNF or compute
lexical membership, but it does decode the closed structural descriptors needed
to prevent two matching reports from agreeing on malformed evidence.

For each document, the runner requires exactly one path-`0` NODE for every PROD
and no NODE outside a PROD. That root span equals the PROD RHS span. Every other
NODE has a recorded parent; direct child paths are contiguous from zero, child
spans are contained in the parent, and siblings are source-ordered and
non-overlapping. Node child counts obey the closed kind arities. FIXED and REF
records equal the corresponding leaf NODE identities and spans. Their source
slices are respectively the exact quoted spelling and referenced name; the one
pattern leaf is the exact source bytes `[0-9]+`. Fixed expansions contain one or
two closed atoms, occur in source-token order, and reconstruct the quoted
spelling exactly. Every reference resolves to exactly one production or lexical
definition; those namespaces are disjoint.

LEX names, kinds, decoded predicates, and source spans obey the closed lexical
forms above. In particular, each regex descriptor carries its exact accepted
source pattern and one of the permitted exclusion modes, and that mode agrees
with the presence or absence of the exact IDENT modifier in its source span.
Literal-union, byte-string, and closed-table descriptors use only their closed
forms. The resulting terminal-predicate universe is therefore derived only from
validated common records.

The runner also replays the engine-facing common-ledger limits: RULE count uses
`max_rules`; PROD count uses `max_definitions`; NODE count uses
`max_grammar_nodes`; LEX count uses `max_lexical_definitions`; FIXED plus
pattern-NODE count uses `max_terminal_occurrences`; every decoded symbol uses
`max_symbol_bytes`; and recorded structural nesting uses `max_ebnf_depth`.
A successful report inconsistent with the limits bound in its own BIND records
is malformed evidence.

The static report then contains `STATIC-BEGIN` / `STATIC-END`. Its exact record
layouts are:

```text
STATIC-NULLABLE doc lhs_hex boolean
STATIC-FIRST doc lhs_hex word_hex
STATIC-FOLLOW doc lhs_hex word_hex
STATIC-INTERSECTION doc left_predicate_hex right_predicate_hex witness_hex_or_dash
STATIC-DECISION doc lhs_hex path decision_kind arm_index word_hex
STATIC-CONFLICT doc lhs_hex path decision_kind left_arm right_arm left_word_hex right_word_hex witness_stream_hex
STATIC-DELTA delta_kind status key_hex
STATIC-CASE doc id start_hex source_hex matching_conflict_count
STATIC-DOMAIN doc id_hex start_hex argument_hex stream_count stream_sha256
STATIC-TRANSITION id current_count proposal_count status witness_hex_or_dash
```

`boolean` is `0|1`; `decision_kind` is
`choice|optional|repeat0|repeat1`; `delta_kind` is
`intersection|conflict`; and delta `status` is
`introduced|removed|retained`. A predicate descriptor is exact ASCII
`fixed:<spelling_hex>`, `pattern:digits`, `lex:<name_hex>`, or `end`. A word
descriptor is `empty` or one or two predicate descriptors joined by one comma;
`word_hex`, `left_word_hex`, and `right_word_hex` encode those descriptor
bytes. An intersection witness is the direct intersecting source bytes, with
`-` meaning the empty witness. A conflict witness stream is an ASCII list with
each concrete source token in lowercase hex and an end-of-input token as `-`,
joined by commas, then lowercase-hex encoded as one field. Thus an EOF-only
witness is outer field `2d`; a token `x` followed by EOF is the outer hex of
ASCII `78,-`.

Predicates in each intersection are ordered by the closed rank `fixed`,
`pattern:digits`, `lex`, `end`, then by descriptor bytes within a rank. A
non-end intersection has a witness; a fixed predicate's witness equals its
spelling and a digits predicate's witness contains only digits. An end
intersection is exactly `end`/`end` with `-`.

A derived intersection witness is not identity. An intersection delta key is
lowercase hex of the ASCII tab-joined `left_predicate_hex` and
`right_predicate_hex` fields.
A conflict delta key is lowercase hex of the ASCII tab-joined `lhs_hex`,
`path`, `decision_kind`, `left_arm`, `right_arm`, `left_word_hex`, and
`right_word_hex` fields; its derived witness is not identity. The records must
classify the complete current/proposal union for both kinds. The current policy
admits no `introduced` delta and requires the proposal to contain zero
strong-LL(2) decision conflicts. Terminal-predicate intersections remain a
complete census rather than a global-disjointness requirement: an overlap is
permitted when the predicates never compete at one grammar decision. The sole
transition succeeds only when its
expectation label matches; the authored case `deref-x` has exact start `expr`
and exact source bytes `deref(x)`; that static probe has `current_count` one and
`proposal_count` zero; and `witness_hex_or_dash` is the lowercase hex of the
complete `deref(x)` source bytes.

`STATIC-CASE` is a static predictive-conflict probe, not a parse or derivation
count. Each document uses its independently extracted fixed, pattern, and
lexical predicates to maximal-munch at most the first two source tokens, pads
EOF to two symbols, requires that word to match `FIRST_2(start EOF EOF)`, and
counts only matching conflicts whose containing production is reachable from
the declared start. The Oracle alone supplies complete source-level derivation
counts.

Each document has exactly one NULLABLE result and at least one FIRST and FOLLOW
word for every common production. Every common `choice`, `optional`, `repeat0`,
and `repeat1` decision has its exact closed arm set, and every common terminal
predicate has its self-intersection. Every conflict names a common decision,
uses words emitted for its two ordered arms, and carries a witness with the same
one- or two-position shape and end markers as those words. Case and domain start
symbols exist in the common production schema. Cases, case-delta policies, and
transitions cover the complete authored expectation registry; the format-v1
transition is bound to the exact `deref-x` case, counts, complete source, and
witness described above.

Static records are ordered by the layout's tag order, then by the complete
line bytes. Within a tag this places `current` before `proposal`. Duplicate
semantic identities are forbidden.

The Oracle report then contains `ORACLE-BEGIN` / `ORACLE-END`. Its exact
record layouts are:

```text
CASE doc id_hex start_hex source_hex class trace_count
CASE-TRACE doc id_hex trace_ordinal trace_hex
CASE-DELTA id_hex status trace_hex
DOMAIN doc id_hex start_hex argument_hex stream_count stream_sha256
STREAM doc id_hex stream_ordinal source_hex class trace_count
STREAM-TRACE doc id_hex stream_ordinal trace_ordinal trace_hex
METRIC doc parsed_streams source_tokens chart_items packed_edges proof_nodes
```

`class` is `zero|one|many` and has retained trace count 0, 1, or 2
respectively. Ordinals are contiguous from zero. Each CASE-TRACE belongs to the
immediately preceding logical CASE identity; each STREAM-TRACE belongs to an
existing STREAM identity. CASE-DELTA classifies the exact union of current and
proposal case traces as `introduced|removed|retained`. The runner enforces each
case's closed trace-delta policy. In particular, the registered lexical
transition cases use `trace-subset`. Authored grammar-factoring cases use
`trace-replacement` when their typed production trees intentionally change,
and semantic-subset cases use it with exact `zero`/`one` counts when the
proposal intentionally forms a tree for later checker rejection.

A decoded trace is one recursive source-grammar tree. Each node is byte `T`, a
four-byte big-endian label length, that many ASCII label bytes, a four-byte
big-endian child count, then for each child an eight-byte big-endian child byte
length and exactly one recursively encoded child. Exact labels are
`production:<lhs_hex>`; `node:<lhs_hex>:<path>:<kind>:<variant>`; or
`token:<fixed|pattern|lexical>:<value_hex>:<source_start>:<source_end>`.
Node variants are the referenced name or fixed spelling hex, exact pattern
`5b302d395d2b`, decimal choice arm, `empty|present`, `empty|more`, `one|more`,
or `-` as selected by the node kind. Binary-normalization helpers never appear
in a trace. A production, ref, pattern, group, or selected choice has one
child; a fixed node has one or two; a sequence has at least one; optional has
zero for `empty` and one for `present`; repeat0 has zero for `empty` and two
for `more`; repeat1 has one for `one` and two for `more`; token events have
zero. The root is the requested start production. Token offsets are canonical
decimal, nonempty, ordered, and in bounds; together they cover the source
except for skipped SP/LF bytes. Fixed-token labels equal their exact source
slice, and pattern slices contain only decimal digits. `trace_hex` is lowercase
hex of the complete tree bytes. The runner decodes it iteratively under the
declared proof-node, encoded-byte, line, and source-token bounds and requires
exact tree EOF. Derivation nesting is bounded by proof nodes and encoded bytes;
it is not bounded by source-token count plus one production's EBNF depth because
a legal production/reference chain may be longer than either.

Oracle order is case ID, `current` then `proposal`, with each CASE followed by
its traces; then CASE-DELTA by ID, status rank `removed`, `retained`,
`introduced`, and trace bytes; then domain ID, `current` then `proposal`, with
each DOMAIN followed by streams and each stream by traces; then METRIC for
`current` and `proposal`. Every ordinal and semantic identity is unique.
The case set and case-delta policy set equal `expectations.txt` exactly, every
case and domain start is a common production, and all declared zero/one/many
trace ordinals are present.
Each trace root is that start production; every trace label and parent-child edge
is validated against the common grammar schema, and its token spans cover the
complete source apart from SP and LF.

For each `(id_hex, doc)`, both engines independently emit and the runner
compares the opaque tuple `(start_hex, argument_hex, stream_count,
stream_sha256)` byte for byte. The stream digest is computed by the domain rule
above. Engine-specific records remain evidence; the runner validates their
closed wire shapes and policy relations but does not turn them into common
grammar facts.

Every successful report ends with `END` followed by one LF and exact EOF.
Empty engine-specific sections have adjacent BEGIN/END lines; fields never use
an empty byte string. An explicitly absent optional byte field is `-` only
where a layout above says `_or_dash`. Input, extraction, resource, and
internal failures instead use `FAIL<TAB>family<TAB>code` after the engine line
and then `END`; `family` is one of `input`, `extraction`, `resource`, or
`internal`, and `code` is exactly `[a-z][a-z0-9_-]*`.

All decimal integers in a successful engine report are canonical unsigned
64-bit values. For each document, DOMAIN stream counts sum to at most
`max_generated_streams`; `parsed_streams` equals the authored-case count plus
that sum. The four remaining METRIC counters cannot exceed `parsed_streams`
times their corresponding per-stream Oracle limit. Retained traces for a
`many` result must be byte-distinct, not two ordinals for the same derivation.

Format version 1 has one closed transition adapter:
`fixed-ident-partition` is bound to case `deref-x`, start production `expr`,
and the complete source and witness bytes `deref(x)`. Its two counts must equal
that STATIC-CASE's current and proposal counts. This adapter interprets one
registered policy transition; the underlying static LL(2) analysis remains
general and case-independent.

## Runner and evidence package

Engine commands and paths are fixed in source. The runner builds the Rust
engine by reading the exact channel from the bound `rust-toolchain.toml`,
setting `RUSTUP_TOOLCHAIN` to that channel, and running `cargo build --locked
--offline --release`, then executes that exact fixed artifact. The build uses
a closed small environment, fresh `HOME`, `CARGO_HOME`, `TMPDIR`, target and
configuration-free working directory, fixed deterministic flags, and only the
required Cargo/Rustup/system-tool PATH plus exact `RUSTUP_HOME`. It runs in a
new process session with a wall timeout and unconditional process-group
cleanup. Cargo's source-to-artifact relation is trusted, not attested.

The runner executes engines sequentially with a fixed environment and closed
inherited descriptors. On supported POSIX hosts it uses temporary regular
files, `RLIMIT_FSIZE`, a process session, wall timeout, unconditional process
group cleanup/reap, exact final size checks, and bounded reads. Nonempty
stderr, spawn failure, timeout, signal, overflow, descendant survival,
malformed output, mixed extraction ledgers, or abnormal exit is
tool-inconclusive.

`RUNNER_SOURCES`, `static-auditor/SOURCES`, and `oracle/SOURCES` are sorted,
exhaustive, non-symlink source manifests. Runner discovery recursively includes
all Python files outside the independently manifested `oracle/` and
`static-auditor/` trees and excludes bytecode-cache directories. The runner
hashes canonical relative path, byte length, and contents before and after
execution. These are stable source observations under the stated
non-adversarial-workspace assumption, not executed-code attestation.

The evidence package preserves both raw reports and a canonical ASCII JSON
summary binding every input, source revision, environment receipt, raw output,
proposal, census, and final component by byte length and SHA-256. Installed
reproduction uses schema `whitefoot.grammar-evidence.v2` and adds an
`installation` object that binds the candidate and numbered v0.9 path, byte
length, SHA-256, `installed-v0.9` mode, and `byte-identical` relation. The
historical review packet remains schema `whitefoot.grammar-evidence.v1`. A separate
`.sha256` sidecar binds the complete report; the report never embeds its own
hash. Semantic output contains no timestamps, paths, PIDs, durations, runtime
addresses, or directory-order dependence.

For reviewability, the summary also projects only already validated closed
records: each case's class and trace count per document, each case's delta
status counts, the static transition fields, static intersection/conflict
delta counts, and each document/domain stream count and digest. This compact
projection is not a recomputation and does not replace the bound raw reports.

Publication is deliberately fail-closed rather than a multi-file atomic
transaction. Component files are replaced first, then canonical `package.json`,
and `package.sha256` is replaced last as the sole completeness marker. A
consumer accepts a published package only after the marker binds package.json
and package.json's byte length and SHA-256 bind every declared component. Any
missing or mismatched marker, manifest, report sidecar, or component is not a
published package.

Two fresh-work-directory runs must produce byte-identical raw reports and
combined semantic report bytes. Binary byte identity is not required.

## Proposal-only frontend-boundary evidence

The frontend-boundary evidence is a separate, non-production contract. It does
not extend the engine frame above and is never language, compiler, numbered
specification, or protected-expectation authority.

`proposal/frontend-boundary-cases.json` is canonical pretty-printed ASCII JSON
followed by one LF. Its objects use sorted keys, strings use ASCII escapes, and
the rendering is Python JSON indentation level two with the standard `,` and
`: ` separators. Its exact schema tag is
`whitefoot.frontend-boundary-cases.v2`; its exact top-level keys are
`authority`, `bundles`, `cases`, `limits`, `raw_cases`, `requirements`, and
`schema`. The inventory contains 100 structured cases and 34 raw exact-byte
cases. Structured cases refer to closed source-bundle records and expected
boundary observations. Raw cases carry source bytes as lowercase hexadecimal
and exact expected source-local scanner observations. The limits are exact
positive integers for observations, record bytes, records, and total source
bytes. Requirements are exactly `B01` through `B10`; every case identity is
unique, appears in at least one requirement, and no requirement names a missing
case. Both models independently reject malformed descriptor fields and enforce
the focused per-case invariants before evaluating them.

The primary typed-record interpreter and the independent tuple-and-event
interpreter may share only the candidate bytes, descriptor bytes, and declared
resource limits. They do not import one another. Each independently extracts
the load-bearing candidate rule text, validates the descriptor, evaluates all
cases, and emits its complete case and requirement projections. A successful
evidence build requires byte-equal candidate contracts, descriptor bindings,
case outcomes, and requirement projections. A model rejection, disagreement,
missing case, changed inventory count, resource failure, or malformed input is
tool-inconclusive and cannot support approval.

`evidence/frontend-boundary-evidence.json` is canonical ASCII JSON followed by
one LF with exact schema tag `whitefoot.frontend-boundary-evidence.v1`. Its
exact top-level keys are `agreement`, `authority`, `candidate`, `cases`,
`descriptor`, `models`, `requirements`, `schema`, `source_manifest`, and
`source_revisions`. `candidate` binds the full candidate path, byte length, and
SHA-256 plus the byte length and SHA-256 of every extracted rule. `descriptor`
binds the descriptor path, byte length, and SHA-256. `cases` preserves the 100
structured-case order followed by the 34 raw-case order, with each record
containing only `id` and its complete derived `outcome`. `requirements` is the
agreed derived projection. `models` names the two implementation and result
schemas. `source_revisions` binds every path declared by the exact sorted
`proposal/FRONTEND_BOUNDARY_SOURCES` manifest; `source_manifest` binds that
manifest itself.

`agreement` contains exact structured, raw, and total case counts; the SHA-256
of canonical JSON for the complete case projection; and four booleans for
candidate-contract, descriptor-binding, outcome, and requirement equality. All
four booleans must be true. The candidate hash is computed from the candidate
file on every build. No documentation value, previous report, or baseline may
substitute for those bytes.

`evidence/frontend-boundary-evidence.sha256` contains exactly 64 lowercase
hexadecimal digits, two SP bytes, the ASCII filename
`frontend-boundary-evidence.json`, and one LF. It is the SHA-256 of the complete
JSON report bytes. Publication replaces the report first and this checksum
marker last. `frontend_boundary_evidence.py --check` accepts only exact
regeneration of both regular, non-symlink files; `--write` atomically replaces
each artifact in that order. The Makefile frontend-boundary target invokes the
check mode and the complete grammar-verifier gate depends on that target.
