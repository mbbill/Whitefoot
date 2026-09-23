# Proposed module syntax

This is the complete grammar candidate over this investigation's v0.62 base,
not the active language. The compiler's own grammar generator qualifies it
through `qualify.rs`; semantic restrictions are in [LANGUAGE.md](LANGUAGE.md).
Keep this input until the grammar is integrated into the active specification,
then remove it and use the specification's ordinary generator tests.

The source start is `program`; graph qualification substitutes `graph_file`
for that start. File role is an explicit parser input. The shared source grammar
admits a function declaration or definition; formation requires declarations in
`.wfm`, definitions in `.wf`, and `public` only in `.wfm`. No symbol-table lookup
selects a grammar arm. A parsed `operand` containing a call cannot have an
ordinary `infix_tail`; contract affine expressions retain their own admission.
`heap_decl` is retained solely for the existing source-bundle entry mode. Graph
module files reject it and use target requirements; it cannot be public.

```wf-ebnf GRAM-2
program := alias_decl* item*
alias_decl := "alias" (IDENT | TYPEID) "=" alias_path ";"
alias_path := "pkg" ("::" name_segment)*
name_segment := IDENT | TYPEID
item := "public"? item_decl
item_decl := fn_decl | struct_decl | enum_decl | interface_decl | binding_decl | const_decl | heap_decl
heap_decl := "program" "no_heap" ";"
struct_decl := "opaque"? ("nocopy" | "nodrop")? "struct" TYPEID generics? "{" doc? struct_member* "}"
struct_member := "public"? (field | footprint_decl)
field := "readonly"? IDENT ":" type ";"
footprint_decl := "footprint" IDENT "=" footprint_path ";"
footprint_path := IDENT footprint_suffix*
footprint_suffix := "." IDENT | "." TYPEID "." IDENT
enum_decl := ("nocopy" | "nodrop")? "enum" TYPEID generics? "{" doc? variant* "}"
variant := "public"? TYPEID "(" vfield_list? ")" ";"
vfield_list := vfield ("," vfield)*
vfield := "public"? IDENT ":" type
fn_decl := "observe"? "fn" IDENT generics? "(" param_list? ")" "->" (result_binding | "(" result_binding ("," result_binding)+ ")") effects contract_block? fn_tail
fn_tail := ";" | "{" doc? stmt* "}"
result_binding := IDENT ":" rtype
contract_block := "contract" "{" contract_define* requires_clause* ensures_clause* "}"
contract_define := "define" IDENT "=" expr ";"
requires_clause := "requires" clause_expr ";"
ensures_clause := "ensures" ("when" result_route ":")? clause_expr ";"
result_route := (IDENT "is")? TYPEID "(" fieldbind ")"
interface_decl := "interface" TYPEID generics? "{" doc? (fn_sig ";")* "}"
binding_decl := "binding" TYPEID ":" pack_use "{" doc? fn_bind* "}"
fn_sig := "observe"? "fn" IDENT "(" param_list? ")" "->" (result_binding | "(" result_binding ("," result_binding)+ ")") effects contract_block?
pack_use := type_name targs?
function_arg := "fn" callee ("::" targs)?
const_decl := "const" IDENT ":" type "=" cvalue ";"
fn_bind := IDENT "=" callee ("::" targs)? ";"
doc := "doc" STRING ";"
generics := "<" gparam ("," gparam)* ">"
gparam := TYPEID (":" (TYPEID | capability_bound))? | "const" IDENT ":" type | fn_sig | "interface" pack_use
capability_bound := "copy" | "drop"
param_list := param ("," param)*
param := IDENT ":" (mode type | "&" "[" type "]")
graph_file := module_row+ target_decl*
module_row := module_path ":" "[" (module_path ("," module_path)*)? "]" ";"
module_path := "pkg" ("::" IDENT)*
target_decl := "target" IDENT "{" "entry" value_name ";" ("no_heap" ";")? "}"
```

```wf-ebnf GRAM-3
type := primitive_type | type_name targs?
primitive_type := "i8" | "i16" | "i32" | "i64" | "u8" | "u16" | "u32" | "u64" | "f32" | "f64" | "unit"
type_name := TYPEID | ("pkg" "::")? IDENT "::" type_name_tail
type_name_tail := TYPEID | IDENT "::" type_name_tail
rtype := "own" type
mode := "own" | "&"
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
proof_use := "use" ("view" call | (("[0-9]+" | IDENT) "times")? use_premise) ";"
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

The reserved spellings added to this base are `public`, `alias`, `pkg`,
`target`, `observe`, `footprint`, and `view`. `entry` and `no_heap` already
exist. There is no new punctuation or comment syntax. `use_premise` is a real
production already present in the base grammar, despite the base generator's
older production-count assertion. Qualification derives the count from the
candidate and never presents it as the final META-5 delta.
