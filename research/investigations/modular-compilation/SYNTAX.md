# Proposed module syntax

This is the complete grammar candidate over the active v0.69 grammar, not the
active language. [LANGUAGE.md](LANGUAGE.md) states the semantic restrictions;
the compiler's own grammar generator qualifies these productions through
[qualify.rs](qualify.rs). Keep this input until the grammar is integrated into
the active specification, then remove it with its adapter and use the
specification's ordinary generator tests.

## Differences from the v0.69 grammar

- A source file starts with file-local `alias_decl` headers. Items, struct
  fields and payload fields may carry `public`; formation admits it only in
  `.wfm` and only where LANGUAGE.md gives it a meaning.
- A function ends in `fn_tail`. A `.wfm` declaration ends in `;` or in its
  `doc` entry, so the interface can document every declared function; a `.wf`
  definition keeps its body with the body's own optional `doc`.
- Types, callees, constructors, values, constants, destructuring targets and
  group applications admit `pkg`-rooted or alias-rooted qualified paths. The
  forms are factored through `operand`, `type_name`, `callee_path` and
  `qualified_arg` so every decision stays strong-LL(2).
- `affine_factor` is `operand`. Contract clauses keep the constructor calls
  and domain-query rows that v0.69 admits through `call`; INV-1 and MSR-5
  still decide which operands each position accepts.
- A match arm may end its bindings with `..` (`destruct_bindings`).
- The separate `graph_file` start lists module rows and named entries using
  the existing `entry` and `no_heap` atoms; the graph adds no keyword.

File role is an explicit parser input: `program` is the start for `.wfm` and
`.wf`, `graph_file` for `modules.wfg`. The shared source grammar admits both a
function declaration and a definition; formation requires declarations in
`.wfm`, definitions in `.wf`, and `public` only in `.wfm`. No symbol-table
lookup selects a grammar arm. A parsed `operand` containing a call cannot
take an ordinary `infix_tail`. `heap_decl` is retained solely for the existing
source-bundle entry mode; graph module files reject it and use entry
requirements.

```wf-ebnf GRAM-2
program := alias_decl* item*
alias_decl := "alias" (IDENT | TYPEID) "=" alias_path ";"
alias_path := "pkg" ("::" name_segment)*
name_segment := IDENT | TYPEID
item := "public"? item_decl
item_decl := fn_decl | struct_decl | enum_decl | interface_decl | binding_decl | const_decl | heap_decl
heap_decl := "program" "no_heap" ";"
struct_decl := "opaque"? ("nocopy" | "nodrop")? "struct" TYPEID generics? "{" doc? field* "}"
field := "public"? "readonly"? IDENT ":" type ";"
enum_decl := ("nocopy" | "nodrop")? "enum" TYPEID generics? "{" doc? variant* "}"
variant := TYPEID "(" vfield_list? ")" ";"
vfield_list := vfield ("," vfield)*
vfield := "public"? IDENT ":" type
fn_decl := "fn" IDENT generics? "(" param_list? ")" "->" (result_binding | "(" result_binding ("," result_binding)+ ")") effects contract_block? fn_tail
fn_tail := ";" | doc | "{" doc? stmt* "}"
result_binding := IDENT ":" rtype
contract_block := "contract" "{" contract_define* requires_clause* ensures_clause* "}"
contract_define := "define" IDENT "=" expr ";"
requires_clause := "requires" clause_expr ";"
ensures_clause := "ensures" ("when" result_route ":")? clause_expr ";"
result_route := (IDENT "is")? TYPEID "(" fieldbind ")"
interface_decl := "interface" TYPEID generics? "{" doc? (fn_sig ";")* "}"
binding_decl := "binding" TYPEID ":" pack_use "{" doc? fn_bind* "}"
fn_sig := "fn" IDENT "(" param_list? ")" "->" (result_binding | "(" result_binding ("," result_binding)+ ")") effects contract_block?
pack_use := type_name targs?
function_arg := "fn" callee ("::" targs)?
const_decl := "const" IDENT ":" type "=" cvalue ";"
fn_bind := IDENT "=" callee ("::" targs)? ";"
doc := "doc" STRING ";"
generics := "<" gparam ("," gparam)* ">"
gparam := TYPEID (":" (TYPEID | capability_bound))? | "const" IDENT ":" type | fn_sig | "interface" pack_use
capability_bound := "copy" | "drop"
param_list := param ("," param)*
param := IDENT ":" (type | "&" (type | "[" type "]"))
graph_file := module_row+ entry_decl*
module_row := module_path ":" "[" (module_path ("," module_path)*)? "]" ";"
module_path := "pkg" ("::" IDENT)*
entry_decl := "entry" IDENT "=" module_path entry_tail
entry_tail := ";" | "{" "no_heap" ";" "}"
```

```wf-ebnf GRAM-3
type := primitive_type | type_name targs?
primitive_type := "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" | "unit"
type_name := TYPEID | ("pkg" "::")? IDENT "::" type_name_tail
type_name_tail := TYPEID | IDENT "::" type_name_tail
rtype := type
targs := "<" targ ("," targ)* ">"
targ := primitive_type | TYPEID targs? | function_arg | "[0-9]+" const_tail? | qualified_arg
qualified_arg := ("pkg" "::")? IDENT qualified_arg_tail
qualified_arg_tail := "::" (IDENT qualified_arg_tail | TYPEID targs?) | const_tail?
```

```wf-ebnf GRAM-4
stmt := let_stmt | set_stmt | expr_stmt | return_stmt | loop_stmt | for_stmt | invariant_stmt | break_stmt | if_stmt | match_stmt | give_stmt
let_stmt := "let" (IDENT "=" (ordinary_let_rhs | propagate_let_rhs | value_match | value_if) | "(" IDENT ("," IDENT)+ ")" "=" call ";" | type_name "(" destruct_bindings? ")" "=" "move" place ";")
destruct_bindings := fieldbind_list ("," "..")? | ".."
if_stmt := "if" expr "{" stmt* "}" ("else" (if_stmt | "{" stmt* "}"))?
value_if := "if" expr "{" stmt* "}" "else" (value_if | "{" stmt* "}")
ordinary_let_rhs := expr ";"
propagate_let_rhs := "propagate" expr ";"
set_stmt := "set" place "=" expr ";"
expr_stmt := call ";"
return_stmt := "return" expr ("," expr)* ";"
loop_stmt := "loop" LABEL? ("(" header_invariant ("," header_invariant)* ")")? "{" stmt* "}"
for_stmt := "for" LABEL? "(" for_binding ("," header_invariant)* ")" "{" stmt* "}"
for_binding := IDENT "in" atom ".." atom
header_invariant := "invariant" IDENT ":" affine_expr compare_op affine_expr
invariant_stmt := "invariant" IDENT ":" affine_expr compare_op affine_expr (";" | "{" proof_use+ "}")
proof_use := "use" (("[0-9]+" | IDENT) "times")? use_premise ";"
use_premise := IDENT | "(" affine_expr compare_op affine_expr ")"
affine_expr := affine_term (affine_add_op affine_term)*
affine_term := affine_factor ("*" affine_factor)?
affine_factor := operand | "(" affine_expr ")"
affine_add_op := "+" | "-"
break_stmt := "break" LABEL? ";"
give_stmt := "give" expr ";"
match_stmt := "match" expr "{" arm+ "}"
value_match := "match" expr "{" arm+ "}"
arm := TYPEID "(" destruct_bindings? ")" "=>" "{" stmt* "}"
fieldbind_list := fieldbind ("," fieldbind)*
fieldbind := IDENT ":" IDENT
```

```wf-ebnf GRAM-5
expr := operand infix_tail?
operand := literal | "move" place | borrow_expr | indirect_base psuffix* | named_operand | OPNAME ("::" targs)? call_arguments | "musttail" call_target
named_operand := ("pkg" "::")? named_operand_tail
named_operand_tail := IDENT lower_operand_tail | TYPEID upper_operand_tail
lower_operand_tail := "::" (targs call_arguments | named_operand_tail) | call_arguments | psuffix*
upper_operand_tail := targs? ("::" member_operand | call_arguments)
member_operand := IDENT ("::" targs)? call_arguments | TYPEID call_arguments
call_arguments := "(" (atom_list | fieldinit_list)? ")"
infix_tail := (infix_op | compare_op) atom
infix_op := "+" | "+wrap" | "+defined" | "+checked" | "+sat" | "-" | "-wrap" | "-defined" | "-checked" | "-sat" | "*" | "*wrap" | "*defined" | "*checked" | "*sat" | "/" | "/defined" | "/checked" | "%" | "%defined" | "%checked"
compare_op := "==" | "!=" | "<" | "<=" | ">" | ">="
atom := literal | "move" place | place | borrow_expr
call := "musttail"? call_target
call_target := callee ("::" targs)? call_arguments
callee := ("pkg" "::")? callee_path | OPNAME
callee_path := IDENT ("::" callee_path)? | TYPEID targs? ("::" (IDENT | TYPEID))?
fieldinit_list := fieldinit ("," fieldinit)*
fieldinit := IDENT ":" atom
borrow_expr := "&" place
atom_list := atom ("," atom)*
clause_expr := affine_expr (clause_op affine_expr)?
clause_op := compare_op | "+defined" | "-defined" | "*defined" | "/defined" | "%defined"
place := pbase psuffix*
pbase := value_name | indirect_base
indirect_base := "deref" "(" place ")" | "entry" "(" IDENT ")"
value_name := ("pkg" "::")? IDENT ("::" IDENT)*
psuffix := "." IDENT | "." TYPEID "." IDENT | "[" atom range_tail? "]"
range_tail := ".." atom
```

```wf-ebnf CONST-1
const := const_head const_tail?
const_head := "[0-9]+" | value_name
const_tail := infix_op const_head
```

```wf-ebnf CONST-2
cvalue := literal | "[" cvalue ("," cvalue)* "]" | ("pkg" "::")? cvalue_path
cvalue_path := IDENT ("::" cvalue_path)? | TYPEID targs? "(" (IDENT ":" cvalue ("," IDENT ":" cvalue)*)? ")"
```

```wf-ebnf EFF-1
effects := "pure" | effect ("," effect)*
effect := "reads" "(" effect_path ")" | "writes" "(" effect_path ")"
effect_path := epbase epsuffix*
epbase := IDENT
epsuffix := "." IDENT | "." TYPEID "." IDENT | "[" IDENT erange? "]"
erange := ".." IDENT
```

## Reserved spellings

`public`, `alias` and `pkg` become fixed atoms. Raw lexical formation does not
consult grammar position [GRAM-1], so each is reserved in every `.wfm`, `.wf`
and graph file. `alias` is an ordinary binding name in some current test
sources; integration renames those bindings. The graph reuses the existing
`entry` and `no_heap` atoms, so `target`, a common binding name in current
sources, stays an identifier. There is no new punctuation or comment syntax.

## Qualification

Run from the repository root under the ordinary construction guard:

```sh
perl .github/run-check.pl module-grammar sh -c 'rustc --edition=2024 research/investigations/modular-compilation/qualify.rs -o /tmp/wf-module-qualify && /tmp/wf-module-qualify'
```

The adapter builds a temporary copy of the compiler's grammar generator with
the candidate production inventory and the three new fixed atoms, then checks:

1. strong-LL(2) and overlapping-token-predicate decisions for the source start
   and the graph start;
2. that every production of the active specification still exists and keeps
   every two-token prefix it derives there, a necessary condition for not
   dropping an existing form;
3. two negative controls: restoring the unfactored qualified call/value
   alternatives must fail the strong-LL(2) check, and the earlier
   `affine_factor := atom | "(" affine_expr ")"` candidate must fail the
   prefix check.

Passing establishes those grammar properties only, not lexer/parser
integration, formation, resolution, proofs or execution. It modifies no
compiler or specification file and is not a daily gate input.
