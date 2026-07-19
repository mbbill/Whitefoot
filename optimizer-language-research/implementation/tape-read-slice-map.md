# F1 slice 2  --  borrowed-struct-tape reads (implementation plan)

Status: PLANNED, not started. Baseline: main at F1 slice 1
(commit `2ad3ed3`, unit 530 / 37 clean / 493 unsupported / 0 rejected). All
line numbers below are in `compiler/src/semantic_reader.wf` at that baseline and
must be re-confirmed before editing (the file shifts).

## Why this slice, and its corrected scope

The acyclic reader currently admits `index<u8>(deref(bufparam), i)` where
`bufparam : &'r buffer<u8>`. But wfc almost never reads that way. Its dominant
data access is the structure-of-arrays tape read:

```
index<u64>(deref(ast).heads, node)        # buffer<u64> field of a &'r AstTape
```

Measured over `compiler/src/*.wf`: **811** `index<u64>`, plus enum-typed indexes
(`index<AstKind>` 57, `SemanticValueMode` 39, `SemanticOperation` 18,
`SemanticTypeKind` 12, `SymbolNamespace`/`PreludeConstructorCode` 10,
`TokenKind` 9, `PreludeTypeCode` 7) vs **1** `index<u8>`. **3111** field
accesses (`deref(x).field`). **~140** functions read tape fields while staying
acyclic and read-only (no loop, no `set`). So this slice unlocks a large chunk
of the 493 unsupported; the earlier "read a scalar field" scoping was far too
narrow.

## Mechanism map (confirmed by reading the frontend)

- `deref(p).field` parses to an **`AstFieldPlace`** node: `child[0]` = the base
  place; the node's **head token** = the field-name token
  (`parser_expressions.wf` ~461).
- A struct declaration's fields are **`AstField`** nodes; each has a child
  **`AstFieldType`** whose head token names the field type. Walk a struct's
  fields by name using the pattern in `semantic_type_members_prior_field`
  (`semantic_type_members.wf:137`): iterate `next_siblings`, compare
  `token_text_equal` on head tokens.
- Structs resolve via the `SymbolType` table exactly like enums do in
  `semantic_reader_type_node_enum_id` (`:503`): `symbol_lookup_in_scope(space
  SymbolType, scope symbol_none, name_token)` then a decl-kind check. The struct
  decl kind is **`AstStructDecl`** (`ast.wf:34`); `AstStructName` is `:35`.
- `analyze_index` (`:405`) hardcodes the type-arg to u8 and the base to an
  `AstDerefPlace` of a buffer param.
- `analyze_expr` (`~1470`) dispatches by node kind (place / table-call / index /
  constructor / user-call); an `AstFieldPlace` branch must be added.
- Effect pass: `semantic_reader_expr_effect` (`~3375`) maps `index` ->
  reads(region)+traps; `semantic_reader_function_param_region` (`~3219`) maps a
  place's param name-head to its region. A **field read** must contribute
  reads(region-of-base-param) and **NO trap** (a field load is a fixed-offset
  read, not bounds-checked).

## SOUNDNESS TRAP  --  struct/enum type-id id-space (must handle)

Enum type-ids are `enum_decl_node + semantic_reader_enum_type_base` where
`enum_type_base = 1_000_000_000` (`:1`). `is_enum` is tested as
`ige(type_id, enum_type_base)` at **`:3107`** (admitted_param own-branch) and
`enum_in_range` the same at **`:2506`** (analyze_match scrutinee). If struct
type-ids are encoded as `struct_decl + 2_000_000_000`, those two sites would
treat a struct id as an enum. Fix: bound both to `ige(1e9) AND ilt(struct_base)`
so enum ids stay `[1e9, 2e9)` and struct ids `[2e9, 2e9+astsize)` are distinct
from primitives (0..3), enums, and `facts_none` (u64 max). AST node count is
~130k, so the ranges never overlap. Audit: the only `enum_type_base` uses are
`:1, :801(return enc), :2506, :2517(decode), :3107`.

## Edit plan (9 parts; split into 2a then 2b)

**Slice 2a  --  struct params + scalar/enum field reads (no buffer fields / typed
index yet):**
1. `const semantic_reader_struct_type_base: u64 = 2000000000_u64;`.
2. `semantic_reader_struct_decl_valid` (mirror `enum_decl_valid`, check
   `AstStructDecl`) + `semantic_reader_type_node_struct_id` (mirror
   `type_node_enum_id`, encode `struct_decl + struct_base`).
3. `semantic_reader_param_type_id`: add a struct-resolution path so a
   `&'r Struct` param yields the struct id (after the u8/u64/bool/u8_buffer/enum
   attempts).
4. `semantic_reader_admitted_param`: shared branch accept `u8_buffer OR
   is_struct(ige struct_base)` with the mode-region tied to a declared region;
   own branch bound `is_enum` to `ilt(struct_base)` so own struct params are
   rejected (deferred). Keep rejecting `&uniq`.
5. Bound `enum_in_range` at `:2506` to `ilt(struct_base)`.
6. Field-place typing: new fn  --  require `child[0]` be `AstDerefPlace` of a place
   whose type is a struct id; decode struct_decl; resolve the field name (node
   head) among the struct's `AstField` children; read the matched `AstFieldType`
   head; resolve to a SCALAR (u8/u64/Bool) or ENUM type-id (reuse the enum name
   resolver); a buffer field or anything else -> fail closed. Wire as the
   `AstFieldPlace` branch of `analyze_expr`.
7. Effect: `expr_effect` AstFieldPlace -> reads(region-of-base-struct-param) via
   `function_param_region`, NO trap. Admit `reads(regions)`-without-`traps`
   effect rows in the signature gate and reconcile them (declared reads-set ==
   exhibited reads-set including field-read regions; declared traps == exhibited
   traps). Keep the multi-region-effectful-call fail-closed gate.
8. Regressions in `test_semantic_unit.py`: positive `&'r Struct` scalar-field
   reader declaring `reads('r)` no-traps -> CLEAN; negatives (nonexistent field;
   field read declaring `pure`; buffer-typed field; `&uniq Struct`) ->
   Unsupported. Update shifted census constants in `test_semantic_unit.py` /
   `test_parser_expressions.py`.

**Slice 2b  --  buffer-typed fields + generalized typed index (the 811-use bulk):**
9. Resolve `buffer<T>` field types (parse the `AstFieldType` generic arg to the
   element T). Generalize `analyze_index`: type-arg u8/u64/enum element types;
   base may be an `AstFieldPlace` yielding `buffer<T>` (as well as the existing
   `deref(bufparam)`); result T; effect reads(region-of-base) + traps.

Each part: `make -C compiler check` after it; all 37 must stay clean; verify
witnesses through the classifier; a FRESH adversarial review before merge (the
effect and id-space wiring are where a false-clean hides).

## Process note

codex (gpt-5.6-sol) STALLED on this slice three times (empty diffs)  --  it exhausts
its turn budget exploring the 3700-line analyzer even with a surgical spec. Do
NOT delegate this slice to codex; implement by hand against this map. See memory
`use-codex-for-delegation`.
