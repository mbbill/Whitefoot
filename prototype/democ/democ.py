#!/usr/bin/env python3
"""democ: demo compiler for a growing subset of kernel-spec v0.8.
source -> parse -> program check (type + ownership) -> LLVM IR (-> native).
Subset (2026-07-10): all int widths x {wrap,trap,checked,sat}, Bool/tag-only
enums as copy i1, buffers (incl. in-struct + buffer<Bool>), structs, payload
enums + Result/Option, try/ERR-3, const items + arrays, contracts/laws with
static FN-4 discharge + reduction reassociation, borrows/regions with
scoped-alias + effect-attr + derived-willreturn fact channels (--no-facts for
controls, --totality for the lint), give value-match, recursion, runnable main.
Temporary tool (owner ruling): endgame is a self-hosted compiler.
"""
import re, sys, subprocess, json
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'checker'))
from checker import (check_program, CheckError, PRELUDE_ENUMS, _parse_effect_row,
                     RESERVED_BINDING_IDENTS, OPERATION_MODE_WORDS)

TOK = re.compile(r'"[^"]*"|\'[a-z][a-z0-9_]*|@[a-z][a-z0-9_]*'
                 r'|->'
                 r'|-?[0-9]+_(?:i8|i16|i32|i64|u8|u16|u32|u64)|[0-9]+'
                 r'|[a-z][a-z0-9_]*(?:\.[a-z][a-z0-9_]*)*|[A-Z][A-Za-z0-9]*'
                 r'|=>|&uniq|&|[(){}<>:;,=\[\].]'
                 r'|(\S)')                                 # catch-all: unknown byte -> FORM-1

# ---- integer type family: width, signedness, range, LLVM type ----
INT_LLTY = {"i8": "i8", "i16": "i16", "i32": "i32", "i64": "i64",
            "u8": "i8", "u16": "i16", "u32": "i32", "u64": "i64"}
INT_WIDTH = {"i8": 8, "i16": 16, "i32": 32, "i64": 64,
             "u8": 8, "u16": 16, "u32": 32, "u64": 64}
INT_MAX = {"i8": 127, "i16": 32767, "i32": 2147483647,
           "i64": 9223372036854775807, "u8": 255, "u16": 65535,
           "u32": 4294967295, "u64": 18446744073709551615}
INT_MIN = {"i8": -128, "i16": -32768, "i32": -2147483648,
           "i64": -9223372036854775808}
INT_SUFFIXES = set(INT_LLTY)
LIT_RE = re.compile(r'(-?)([0-9]+)_([iu](?:8|16|32|64))')
REGIONID_RE = re.compile(r"'[a-z][a-z0-9_]*")

def _is_signed(suf):
    return suf in INT_SUFFIXES and suf[0] == "i"

def _litval(tok):                                  # integer value of a suffixed literal token [FORM-5]
    m = LIT_RE.fullmatch(tok)
    if not m: raise CheckError("FORM-5", f"const value '{tok}' is not a numeric literal")
    return int(m.group(1) + m.group(2))

_TAGONLY2 = {}    # 2-variant tag-only enums: Bool-isomorphic, lowered i1 (name -> (v0, v1))

# Versioned, closed analyzer set behind the first checked-automation policy.
# A site may be called `not-applicable` only after every analyzer in this set
# has run.  Codegen's fallback report is deliberately incomplete/unknown so a
# future AST rewrite cannot acquire promotion credit by skipping this pass.
_OBLIGATION_ANALYSIS_SCOPE = "frontend-implicit-bounds-v1"

def _llty(name):
    """LLVM type for a democ type name; Bool and 2-variant tag-only enums are i1;
    other non-int named types are i32 tags."""
    if name == "Bool" or name in _TAGONLY2: return "i1"
    return INT_LLTY.get(name, "i32")

def _buf_elem(name):
    return name[7:-1] if isinstance(name, str) and name.startswith("buffer<") else None

def _arr_parts(name):                              # array<T,N> -> (T, N) else None [CONST-2]
    if isinstance(name, str) and name.startswith("array<"):
        elem, n = name[6:-1].split(",", 1)
        return elem, int(n)
    return None

def _named_args(name, base):                       # Result<T,E>/Option<T> -> [args]
    prefix = base + "<"
    if not isinstance(name, str) or not name.startswith(prefix) or not name.endswith(">"):
        return None
    inner = name[len(prefix):-1]
    args, start, depth = [], 0, 0
    for index, char in enumerate(inner):
        if char == "<":
            depth += 1
        elif char == ">":
            depth -= 1
        elif char == "," and depth == 0:
            args.append(inner[start:index])
            start = index + 1
    args.append(inner[start:])
    return args

def _democ_type_spellings(structs, enums, fns, consts=()):
    """Collect every parsed type spelling that can influence codegen.

    This is a stage-0 profile inventory, not language type inference.  Walking the
    parsed trees also catches body-local Result/Option uses that signature-only
    typedef discovery used to miss.
    """
    found = []
    for fields in structs.values():
        found.extend(field["ty"] for field in fields)
    for variants in enums.values():
        for _variant, fields in variants:
            found.extend(field["ty"] for field in fields)
    found.extend(const["ty"] for const in consts)
    def walk(value):
        if isinstance(value, dict):
            for key, child in value.items():
                if key in ("ty", "rty") and isinstance(child, str):
                    found.append(child)
                walk(child)
        elif isinstance(value, (list, tuple)):
            for child in value:
                walk(child)
    walk(fns)
    return found

def _validate_erased_prelude_payloads(type_spellings, enums):
    """Fail explicitly when disposable stage 0 cannot represent a generic payload.

    democ's bootstrap ABI erases Result/Option payloads into one i64 word.  Integer
    and tag-only-enum payloads are lossless in that slot; aggregates, unit, floats,
    and nested generic values require the production compiler's monomorphized layout.
    They must never fall through to malformed LLVM or a Python exception.
    """
    prelude_copy = {
        name for name, variants in PRELUDE_ENUMS.items()
        if all(not variant["fields"] for variant in variants)
    }
    user_copy = {
        name for name, variants in enums.items()
        if all(not fields for _variant, fields in variants)
    }
    allowed = set(INT_LLTY) | prelude_copy | user_copy | {"Bool"}
    def inspect(spelling):
        for base in ("Result", "Option"):
            args = _named_args(spelling, base)
            if args is None:
                continue
            bad = [arg for arg in args if arg not in allowed]
            if bad:
                raise SystemExit(
                    "democ: aggregate Result/Option payloads are outside the "
                    f"stage-0 word-erased profile ({spelling}: {', '.join(bad)}); "
                    "compile this shape with wfc"
                )
            for arg in args:
                inspect(arg)
            return
        for container in ("buffer", "array"):
            args = _named_args(spelling, container)
            if args is not None:
                for arg in args:
                    inspect(arg)
                return
    for spelling in type_spellings:
        inspect(spelling)

def _tybytes(name):
    """sizeof(T) for buffer_new's OP-9 byte-size computation (monomorphization-time)."""
    if name == "Bool" or name in _TAGONLY2: return 1   # i1 stores as one byte
    import re as _re
    m = _re.fullmatch(r'[iuf](\d+)', name)
    return (int(m.group(1)) // 8) if m else 4

def _size_align(tyname, structs):
    """(size, align) of a pointee for dereferenceable/align param facts; None if unknown.
    Standard layout: ints by width, Bool 1 byte, buffer {ptr, i64} 16/8, structs padded
    per-field with tail padding to the max field alignment."""
    if tyname in INT_LLTY:
        b = _tybytes(tyname); return (b, b)
    if tyname == "Bool" or tyname in _TAGONLY2: return (1, 1)
    if _buf_elem(tyname): return (16, 8)
    if tyname in structs:
        off, mx = 0, 1
        for fld in structs[tyname]:
            sa = _size_align(fld["ty"], structs)
            if sa is None: return None
            sz, al = sa
            off = (off + al - 1) // al * al + sz
            mx = max(mx, al)
        return ((off + mx - 1) // mx * mx, mx)
    return None

# ---- op-name resolution over democ's integer subset [OP-1/2/6/7/8, DIAG-1] ----
# ops with a result/amount mode axis -> the modes democ lowers; dotless ops carry
# no mode. An illegal op name surfaces as a spec rejection with its rule id.
_MODE_OPS = {"iadd": {"wrap", "trap", "checked", "sat"}, "isub": {"wrap", "trap", "checked", "sat"},
             "imul": {"wrap", "trap", "checked", "sat"}, "idiv": {"trap", "checked"},
             "irem": {"trap", "checked"}, "ishl": {"wrap", "trap"},
             "ishr": {"wrap", "trap"}}
_DOTLESS_OPS = {"buffer_new", "len", "imin", "imax", "iand", "ior", "ixor", "irotl", "irotr",
                "band", "bor", "bxor", "bnot",
                "ieq", "ine", "ilt", "ile", "igt", "ige", "eeq", "ene",
                "cvt", "reinterpret"}
_KNOWN_OP_BASES = set(_MODE_OPS) | _DOTLESS_OPS

def _resolve_op(op, tyargs):
    base, _, mode = op.partition(".")
    if base == "cvt":                                  # [OP-6] cvt<T,T> is not an operation
        if len(tyargs) == 2 and tyargs[0] == tyargs[1]:
            raise CheckError("OP-6", f"cvt<{tyargs[0]}, {tyargs[1]}> is not an operation; "
                             "cvt is defined only for distinct numeric pairs")
        return
    if base in _DOTLESS_OPS:
        if mode:                                       # [OP-8] bitwise/rotate are dotless-total
            raise CheckError("OP-8", f"'{op}' is dotless-total and carries no mode suffix "
                             "(the amount is taken modulo width; there is no out-of-range edge)")
        return
    if base in _MODE_OPS:
        if mode and mode not in _MODE_OPS[base]:
            if base in ("idiv", "irem"):               # [OP-2] div/rem carry no wrap mode
                raise CheckError("OP-2", f"'{op}': division and remainder carry no wrap mode "
                                 "(no sound modular semantics for a zero divisor); the axis is {trap, checked}")
            raise CheckError("OP-8", f"'{op}': mode '{mode}' is not on this op's mode axis")
        return
    if any((pre + base) in _KNOWN_OP_BASES for pre in ("i", "f", "b", "e")):
        raise CheckError("OP-7", f"'{op}' lacks its domain prefix (i/f/b/e); "
                         f"'{base}' names no table op")

def _check_form2(src):
    """FORM-2: canonical byte formatting — 2-space indent per block level, spaces (not
    tabs), exactly one space after ':'. Raises on the first violation [DIAG-1]."""
    depth = 0
    for i, line in enumerate(src.split("\n"), 1):
        stripped = line.strip()
        if not stripped:
            continue
        lead = line[:len(line) - len(line.lstrip())]
        if "\t" in lead:
            raise CheckError("FORM-2", f"line {i}: indentation must be spaces, not tabs")
        nostr = re.sub(r'"[^"]*"', '""', line)             # brace/colon count ignores string bytes
        exp = 2 * (depth - 1) if stripped.startswith("}") else 2 * depth
        if len(lead) != exp:
            raise CheckError("FORM-2",
                f"line {i}: indent {len(lead)} != canonical {exp} (two spaces per block level)")
        if re.search(r':(?!\s)', nostr) or re.search(r':  ', nostr):
            raise CheckError("FORM-2", f"line {i}: exactly one space required after ':'")
        depth += nostr.count("{") - nostr.count("}")

def toks(src):
    _check_form2(src)                                       # FORM-2: canonical byte formatting
    if re.search(r'//|/\*', re.sub(r'"[^"]*"', '', src)):   # FORM-4: comments do not exist
        raise CheckError("FORM-4", "comments do not exist; documentation rides the doc field of a declaration")
    toks_ = []
    for m in TOK.finditer(src):
        if m.group(1) is not None:                     # FORM-1: no silent drops, ever
            raise CheckError("FORM-1",
                f"unknown byte {m.group(1)!r} is not part of any canonical token")
        toks_.append(m.group(0))
    return toks_

class P:
    def __init__(s, t): s.t = t; s.i = 0
    def peek(s, k=0): return s.t[s.i+k] if s.i+k < len(s.t) else None
    def eat(s, x=None):
        v = s.t[s.i]; s.i += 1
        assert x is None or v == x, f"expected {x}, got {v} at {s.t[max(0,s.i-4):s.i+3]}"
        return v

def is_typeid(t): return bool(t) and t[0].isupper()

def check_binding_ident(name, context):
    if name in RESERVED_BINDING_IDENTS:
        source_rule = "FN-8" if name == "requires" else "OP-1"
        raise CheckError("FORM-3",
            f"'{name}' is reserved by {source_rule} and cannot bind a {context}")
    return name

def binding_ident(p, context):
    return check_binding_ident(p.eat(), context)

def binding_region(p, context):
    token = p.eat()
    if not REGIONID_RE.fullmatch(token):
        raise CheckError("FORM-3",
            f"{context} must use the apostrophe-prefixed REGIONID spelling, got '{token}'")
    return check_binding_ident(token[1:], context)

def parse_mode(p):
    if p.peek() == '&uniq': p.eat(); return {"kind": "ref", "region": binding_region(p, "reference region"), "uniq": True}
    if p.peek() == '&': p.eat(); return {"kind": "ref", "region": binding_region(p, "reference region"), "uniq": False}
    p.eat('own'); return {"kind": "own"}

def parse_call_region_args(p):
    """Retain an explicit user-call region list without filling an omission."""
    p.eat('<')
    if p.peek() == '>':
        raise CheckError("GRAM-5", "a call region-argument list must not be empty")
    regions = []
    while True:
        token = p.peek()
        if token is None:
            raise CheckError("GRAM-5",
                "an explicit user-call region list must be closed by '>'")
        if not REGIONID_RE.fullmatch(token):
            if token in (',', '(', ')', ';', '}'):
                raise CheckError("GRAM-5",
                    "a call region-argument list expects a REGIONID")
            raise SystemExit(
                "democ: user-call type/const/mixed targs are outside the "
                "stage-0 profile; only explicit REGIONID arguments are supported")
        regions.append(binding_region(p, "call region argument"))
        if p.peek() == '>':
            p.eat('>')
            return regions
        if p.peek() is None:
            raise CheckError("GRAM-5",
                "an explicit user-call region list must be closed by '>'")
        if p.peek() != ',':
            raise CheckError("GRAM-5",
                "call region arguments must be separated by ',' and closed by '>'")
        p.eat(',')
        if p.peek() is None:
            raise CheckError("GRAM-5",
                "an explicit user-call region list must not end after ','")
        if p.peek() == '>':
            raise CheckError("GRAM-5", "a call region-argument list must not have a trailing comma")

def parse_type(p):
    base = p.eat()
    inner = []
    if p.peek() == '<':
        p.eat()
        while p.peek() != '>':
            inner.append(parse_type(p))
            if p.peek() == ',': p.eat()
        p.eat('>')
    if base == 'buffer' and inner:                     # retain the element type [TYPE-2]
        return f"buffer<{inner[0]}>"
    if base == 'array' and len(inner) == 2:            # array<T, N> const-array type [TYPE-2/CONST-2]
        return f"array<{inner[0]},{inner[1]}>"
    if base == 'Result' and len(inner) == 2:           # retain payload types [ERR-3]
        return f"Result<{inner[0]},{inner[1]}>"
    if base == 'Option' and len(inner) == 1:           # retain payload type [PRE-1]
        return f"Option<{inner[0]}>"
    return base

def parse_place(p):
    if p.peek() == 'index':                            # index<T>(place, atom) — a place [GRAM-5/OP-4]
        p.eat(); p.eat('<'); elem = parse_type(p); p.eat('>'); p.eat('(')
        inner = parse_place(p); p.eat(','); atom = parse_atom(p); p.eat(')')
        pl = {"base": inner["base"], "path": inner["path"], "deref": inner["deref"],
              "index": {"place": inner, "atom": atom, "elem": elem}}
    elif p.peek() == 'deref':
        p.eat(); p.eat('('); inner = parse_place(p); p.eat(')')
        inner["deref"] = inner.get("deref", 0) + 1
        pl = inner
    else:
        # a field path rides the identifier token: `base.f.g` tokenizes as one word [GRAM-5 psuffix]
        token = p.eat()
        parts = token.split('.')
        if len(parts) == 2 and parts[1] in OPERATION_MODE_WORDS:
            raise CheckError("FORM-3",
                f"OPNAME '{token}' cannot be used as a place; operation mode words are reserved")
        for field in parts[1:]:
            check_binding_ident(field, "field name")
        pl = {"base": parts[0], "path": parts[1:], "deref": 0}
    while p.peek() == '.':                             # psuffix after deref(...)/index<...>(...) [GRAM-5]
        p.eat('.')
        fields = p.eat().split('.')
        for field in fields:
            pl.setdefault("post", []).append(
                check_binding_ident(field, "field name"))
    return pl

def parse_expr(p):
    t = p.peek()
    if t == 'move':                                    # "move" place — affine consumption [GRAM-5/OWN-1]
        p.eat(); return {"e": "move", "p": parse_place(p)}
    if t in ('&', '&uniq'):
        uniq = p.eat() == '&uniq'; r = binding_region(p, "borrow region")
        return {"e": "borrow", "uniq": uniq, "region": r, "place": parse_place(p)}
    if t == 'unit': p.eat(); return {"e": "unit"}
    m = LIT_RE.fullmatch(t)
    if m:                                              # suffixed integer literal [FORM-5]
        p.eat(); sign, digits, suf = m.group(1), m.group(2), m.group(3)
        if len(digits) > 1 and digits[0] == '0':       # FORM-7: leading-zero form is illegal (0 is its own form)
            raise CheckError("FORM-7", f"leading-zero integer literal '{t}' is illegal; the single digit 0 is its own form")
        if sign and suf[0] == 'u':                     # FORM-5: unsigned literals carry no sign
            raise CheckError("FORM-7", f"negative literal '{t}' is illegal for unsigned {suf}")
        if sign and digits == '0':                     # FORM-5: one spelling per value; -0 is not a form
            raise CheckError("FORM-7", "'-0' is not a canonical integer form; write 0")
        v = -int(digits) if sign else int(digits)
        if v > INT_MAX[suf] or (sign and v < INT_MIN[suf]):   # FORM-7 range
            raise CheckError("FORM-7", f"integer literal '{t}' is out of range for {suf}")
        return {"e": "lit", "v": v, "ty": suf}
    if re.fullmatch(r'[0-9]+', t):                     # FORM-5: a bare integer lacks its mandatory type suffix
        raise CheckError("FORM-5", f"integer literal '{t}' must carry its mandatory type suffix (e.g. {t}_i32)")
    if is_typeid(t):                                   # construct K(field: atom, ...) [GRAM-8]
        n = p.eat(); p.eat('('); fields = []
        while p.peek() != ')':
            fname = binding_ident(p, "field name"); p.eat(':'); atom = parse_atom(p)
            fields.append({"name": fname, "atom": atom})
            if p.peek() == ',': p.eat()
        p.eat(')'); return {"e": "construct", "n": n, "fields": fields}
    if t == 'index':                                   # index is a PLACE, its sole home [GRAM-6/OP-4]
        return {"e": "place", "p": parse_place(p)}
    table_base = t.partition('.')[0]
    table_callee = ('.' in t or
                    table_base in _KNOWN_OP_BASES or
                    t in RESERVED_BINDING_IDENTS)
    has_call_regions = False
    if p.peek(1) == '<' and not table_callee:
        first_targ = p.peek(2)
        if first_targ is None:
            raise CheckError("GRAM-5",
                "an explicit user-call argument list must contain a targ and close with '>'")
        if REGIONID_RE.fullmatch(first_targ) or first_targ == '>':
            has_call_regions = True
        else:
            raise SystemExit(
                "democ: user-call type/const/mixed targs are outside the "
                "stage-0 profile; only explicit REGIONID arguments are supported")
    if p.peek(1) == '(' and t in ("eeq", "ene"):
        # Preserve the reserved operation identity when its mandatory explicit
        # type argument is absent. The type layer owns the canonical FN-2
        # diagnostic; treating this spelling as a positional user call would
        # incorrectly report GRAM-11 before operation resolution.
        op = p.eat()
        p.eat('(')
        args = []
        if p.peek() != ')':
            args.append(parse_atom(p))
            while p.peek() == ',':
                p.eat(',')
                args.append(parse_atom(p))
        p.eat(')')
        return {"e": "op", "op": op, "args": args, "tyargs": []}
    if (p.peek(1) == '(' or has_call_regions) and t not in ('deref', 'index'):
        # user call f<regions?>(param: atom, ...) [GRAM-5/GRAM-11]
        n = p.eat()
        region_args = parse_call_region_args(p) if p.peek() == '<' else None
        if p.peek() != '(':
            raise CheckError("GRAM-5",
                "an explicit user-call region list must be followed by '('")
        p.eat('('); args = []; argnames = []
        while True:
            if p.peek() is None:
                raise CheckError("GRAM-11",
                    "an unterminated user-fn call must end with ')'")
            if p.peek() == ')':
                p.eat(')')
                return {"e": "ucall", "n": n, "args": args,
                        "argnames": argnames, "region_args": region_args}
            pname = binding_ident(p, "call-argument label")
            if p.peek() != ':':                        # [GRAM-11] call args are named (param: atom), never positional
                raise CheckError("GRAM-11", "a user-fn call must name its arguments (param: atom) in declared order; positional args are illegal")
            p.eat(':')
            if p.peek() is None:
                raise CheckError("GRAM-11",
                    "an unterminated user-fn call is missing its argument atom")
            atom = parse_atom(p)
            argnames.append(pname); args.append(atom)
            if p.peek() == ',':
                p.eat(',')
                if p.peek() is None:
                    raise CheckError("GRAM-11",
                        "an unterminated user-fn call ends after ','")
                if p.peek() == ')':
                    raise CheckError("GRAM-11",
                        "a user-fn call must not have a trailing argument comma")
            elif p.peek() not in (')', None):
                raise CheckError("GRAM-11",
                    "user-fn call arguments must be separated by ',' and closed by ')'")
    if p.peek(1) == '<':                               # OPNAME<ty>(atoms) — every table op takes targs;
        op = p.eat()                                   # a dotted place token (base.field) never does
        if '.' in op and op.split('.', 1)[1] not in ("wrap", "trap", "checked", "sat", "strict"):
            raise CheckError("FORM-3", f"'{op}' is not a legal OPNAME; the mode suffix is a closed word set {{wrap,trap,checked,sat,strict}}")
        p.eat('<'); tyargs = [parse_type(p)]
        while p.peek() == ',': p.eat(); tyargs.append(parse_type(p))
        p.eat('>'); _resolve_op(op, tyargs)             # [OP-1/2/6/7/8] op-name resolution
        p.eat('(')
        args = [parse_atom(p)]                          # [GRAM-9] operands are atoms, not nested calls
        while p.peek() == ',': p.eat(); args.append(parse_atom(p))
        p.eat(')'); return {"e": "op", "op": op, "args": args, "tyargs": tyargs}
    return {"e": "place", "p": parse_place(p)}

def parse_atom(p):                                     # [GRAM-9] atom := literal | move place | place | borrow
    if is_typeid(p.peek()):
        raise CheckError("GRAM-9", "a construct in an atom position does not derive (three-address form); bind it with a preceding let")
    e = parse_expr(p)
    if e["e"] in ("op", "ucall"):                      # a call in an atom position does not derive (three-address/ANF)
        raise CheckError("GRAM-9", "a call in an atom position does not derive (three-address form); bind it with a preceding let")
    return e

def parse_match(p):
    p.eat('match'); scrut = parse_expr(p); p.eat('{'); arms = []
    while p.peek() != '}':
        vn = p.eat(); p.eat('('); binders = []
        while p.peek() != ')':                         # K(field: binder, ...) [GRAM-10]
            field = binding_ident(p, "field label"); p.eat(':'); binder = binding_ident(p, "match binder")
            binders.append({"field": field, "name": binder})
            if p.peek() == ',': p.eat()
        p.eat(')'); p.eat('=>'); p.eat('{'); body = []
        while p.peek() != '}': body.append(parse_stmt(p))
        p.eat('}'); arms.append({"v": vn, "b": binders, "body": body})
    p.eat('}'); return {"s": "match", "scrut": scrut, "arms": arms}

def parse_stmt(p):
    t = p.peek()
    if t == 'doc': p.eat(); p.eat(); p.eat(';'); return {"s": "doc"}
    if t == 'let':
        p.eat(); n = binding_ident(p, "let name"); p.eat(':'); m = parse_mode(p); ty = parse_type(p); p.eat('=')
        if p.peek() == 'match':                        # value-match initializer [GIVE-1]
            return {"s": "let", "n": n, "m": m, "ty": ty, "match": parse_match(p)}
        if p.peek() == 'try':                          # try-propagation [ERR-3]
            p.eat(); e = parse_expr(p); p.eat(';')
            return {"s": "try", "n": n, "m": m, "ty": ty, "e": e}
        e = parse_expr(p); p.eat(';')
        return {"s": "let", "n": n, "m": m, "ty": ty, "e": e}
    if t == 'give':
        p.eat(); e = parse_expr(p); p.eat(';'); return {"s": "give", "e": e}
    if t == 'set':
        p.eat(); pl = parse_place(p); p.eat('='); e = parse_expr(p); p.eat(';')
        return {"s": "set", "p": pl, "e": e}
    if t == 'return':
        p.eat(); e = parse_expr(p); p.eat(';'); return {"s": "return", "e": e}
    if t == 'loop':
        p.eat(); lb = p.eat(); p.eat('{'); b = []
        while p.peek() != '}': b.append(parse_stmt(p))
        p.eat('}'); return {"s": "loop", "l": lb, "body": b}
    if t == 'break':
        p.eat(); lb = p.eat(); p.eat(';'); return {"s": "break", "l": lb}
    if t == 'region':
        p.eat(); r = binding_region(p, "region"); p.eat('{'); b = []
        while p.peek() != '}': b.append(parse_stmt(p))
        p.eat('}'); return {"s": "region", "r": r, "body": b}
    if t == 'check':
        p.eat(); e = parse_expr(p); p.eat('else'); p.eat('trap'); msg = p.eat(); p.eat(';')
        return {"s": "check", "e": e, "msg": msg.strip('"')}
    if t == 'match':
        return parse_match(p)
    e = parse_expr(p)
    if p.peek() != ';':                                # [FORM-1] an unknown construct is a hard error
        raise CheckError("FORM-1", f"unknown construct: expected ';' to end the statement, got {p.peek()!r}")
    p.eat(';'); return {"s": "expr", "e": e}

def parse_stmt_block(p):
    """Parse a braced statement block without inventing a second AST shape.

    `requires` deliberately reuses the ordinary ANF let/check nodes; FN-8's
    checker-side validator applies the much smaller admitted sublanguage.
    """
    p.eat('{'); body = []
    while p.peek() != '}': body.append(parse_stmt(p))
    p.eat('}')
    return body

def _reserve_type_declaration(name, structs, enums):
    """Reject TYPEID redeclaration before the parser's maps can overwrite it."""
    if name in PRELUDE_ENUMS:
        raise CheckError("TYPE-6", f"type name '{name}' redeclares a prelude enum")
    if name in structs or name in enums:
        raise CheckError("TYPE-6", f"type name '{name}' is already declared")

def parse_program(src):
    p = P(toks(src)); structs = {}; enums = {}; fns = []; contracts = {}; conforms = []; consts = []
    while p.peek():
        if p.peek() == 'const':                        # const NAME: type = cvalue; [CONST-2]
            p.eat(); name = binding_ident(p, "const name"); p.eat(':'); ty = parse_type(p); p.eat('=')
            if p.peek() == '[':                        # array literal [cvalue, ...]
                p.eat(); vals = []
                while p.peek() != ']':
                    vals.append(p.eat())
                    if p.peek() == ',': p.eat()
                p.eat(']')
            else:
                vals = [p.eat()]
            p.eat(';')
            consts.append({"name": name, "ty": ty, "vals": vals})
        elif p.peek() == 'contract':                   # contract TYPEID { fn sigs; laws } [FN-3/FN-4]
            p.eat(); name = p.eat()
            if not is_typeid(name):
                raise CheckError("FORM-3", f"contract name must be a TYPEID ([A-Z]...), got '{name}'")
            p.eat('{'); cfns = {}; laws = []
            while p.peek() != '}':
                if p.peek() == 'doc':
                    p.eat(); p.eat(); p.eat(';'); continue
                if p.peek() == 'law':                  # law LAWNAME(f) | law identity(f, atom) [FN-4]
                    p.eat(); lawname = p.eat()
                    if lawname not in ("associative", "commutative", "identity"):
                        raise CheckError("FN-4", f"'{lawname}' is not a LAWNAME; the table is closed: associative, commutative, identity")
                    p.eat('('); target = p.eat(); ident = None
                    if lawname == "identity":
                        p.eat(','); ident = parse_atom(p)
                    p.eat(')'); p.eat(';')
                    if target not in cfns:
                        raise CheckError("FN-4", f"law {lawname}({target}) names no fn declared in contract {name}")
                    laws.append({"law": lawname, "fn": target, "e": ident})
                    continue
                p.eat('fn'); fname = binding_ident(p, "contract member"); sig_regions = []
                if p.peek() == '[':
                    p.eat()
                    while p.peek() != ']':
                        sig_regions.append(binding_region(p, "region parameter"))
                        if p.peek() == ',': p.eat()
                    p.eat(']')
                p.eat('(')
                sig_params = []
                while p.peek() != ')':
                    pn = binding_ident(p, "parameter"); p.eat(':'); m = parse_mode(p); ty = parse_type(p)
                    sig_params.append({"name": pn, "mode": m, "ty": ty})
                    if p.peek() == ',': p.eat()
                p.eat(')'); p.eat('->'); rm = parse_mode(p); rt = parse_type(p)
                eff = []
                while p.peek() not in (';', 'requires'): eff.append(p.eat())
                if p.peek() == 'requires':
                    raise CheckError("FN-8",
                        "requires is not admitted on contract members in the first slice; "
                        "precondition equality/refinement is not yet defined")
                p.eat(';')
                _parse_effect_row(eff, f"contract {name}.{fname}")
                cfns[fname] = {"regions": sig_regions, "params": sig_params,
                                "rmode": rm, "rty": rt, "effects": eff}
            p.eat('}'); contracts[name] = {"fns": cfns, "laws": laws}
        elif p.peek() == 'conform':                    # conform ty : Contract { member = fn; } [FN-3]
            p.eat(); ty = parse_type(p); p.eat(':'); cname = p.eat()
            p.eat('{'); binds = {}
            while p.peek() != '}':
                member = p.eat(); p.eat('='); target = p.eat(); p.eat(';')
                binds[member] = target
            p.eat('}'); conforms.append({"ty": ty, "contract": cname, "binds": binds})
        elif p.peek() == 'struct':
            p.eat(); name = p.eat()
            if not is_typeid(name):                    # [FORM-3] a struct name is a TYPEID ([A-Z]...)
                raise CheckError("FORM-3", f"struct name must be a TYPEID ([A-Z]...), got '{name}'")
            _reserve_type_declaration(name, structs, enums)
            if p.peek() == '<':                        # region-generic / generic aggregates: later step
                raise SystemExit("democ: generic/region-parameterized structs are not in the subset yet")
            p.eat('{'); fields = []                    # field := IDENT ":" type ";" (declared order)
            while p.peek() != '}':
                if p.peek() == 'doc':                  # doc? rides the front of the body [GRAM-2]
                    p.eat(); p.eat(); p.eat(';'); continue
                fname = binding_ident(p, "field name"); p.eat(':'); fty = parse_type(p); p.eat(';')
                fields.append({"name": fname, "ty": fty})
            p.eat('}'); structs[name] = fields
        elif p.peek() == 'enum':
            p.eat(); name = p.eat()
            if not is_typeid(name):                    # [FORM-3] an enum name is a TYPEID ([A-Z]...)
                raise CheckError("FORM-3", f"enum name must be a TYPEID ([A-Z]...), got '{name}'")
            _reserve_type_declaration(name, structs, enums)
            p.eat('{'); vs = []
            while p.peek() != '}':
                vn = p.eat(); p.eat('('); fields = []      # vfield := IDENT ":" type
                while p.peek() != ')':
                    fname = binding_ident(p, "variant field name"); p.eat(':'); fty = parse_type(p)
                    fields.append({"name": fname, "ty": fty})
                    if p.peek() == ',': p.eat()
                p.eat(')'); p.eat(';'); vs.append((vn, fields))
            p.eat('}'); enums[name] = vs
        else:
            p.eat('fn'); name = binding_ident(p, "function name")
            if not re.fullmatch(r'[a-z][a-z0-9_]*', name):   # [FORM-3] a fn name is an IDENT ([a-z][a-z0-9_]*)
                raise CheckError("FORM-3", f"fn name must be an IDENT ([a-z][a-z0-9_]*), got '{name}'")
            regions = []
            if p.peek() == '[':
                p.eat()
                while p.peek() != ']':
                    regions.append(binding_region(p, "region parameter"))
                    if p.peek() == ',': p.eat()
                p.eat(']')
            p.eat('('); params = []
            while p.peek() != ')':
                pn = binding_ident(p, "parameter"); p.eat(':'); m = parse_mode(p); ty = parse_type(p)
                params.append({"name": pn, "mode": m, "ty": ty})
                if p.peek() == ',': p.eat()
            p.eat(')'); p.eat('->'); rmode = parse_mode(p); rty = parse_type(p)
            eff = []
            while p.peek() not in ('requires', '{'):
                eff.append(p.eat())                       # effect row [EFF-1/EFF-2]
            requires = None
            if p.peek() == 'requires':
                p.eat('requires')
                requires = parse_stmt_block(p)            # restricted + checked by FN-8
                if not requires:
                    raise CheckError("FN-8",
                        "requires must contain zero or more lets followed by one final check")
            body = parse_stmt_block(p)
            fns.append({"name": name, "regions": regions, "params": params,
                        "rmode": rmode, "rty": rty, "effects": eff,
                        "requires": requires, "body": body})
    return structs, enums, fns, contracts, conforms, consts

# ---- v0.6 type-layer mapping: democ parse tree -> check_program `prog` dict ----
PRIM_SET = {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64"}
def ttype(base):
    if base in PRIM_SET: return {"kind": "prim", "name": base}
    if base == "unit": return {"kind": "unit"}
    if base.startswith("buffer<"):
        return {"kind": "buffer", "elem": ttype(_buf_elem(base))}
    if base.startswith("Result<"):
        ok, err = _named_args(base, "Result")
        return {"kind": "named", "name": "Result", "args": [ttype(ok), ttype(err)]}
    if base.startswith("Option<"):
        (value,) = _named_args(base, "Option")
        return {"kind": "named", "name": "Option", "args": [ttype(value)]}
    if base.startswith("array<"):
        elem, n = _arr_parts(base)
        return {"kind": "array", "elem": ttype(elem), "n": n}
    if base in ("box", "buffer"): return {"kind": base, "elem": {"kind": "any"}}
    if base in ("slice", "arena"): return {"kind": base, "region": None, "elem": {"kind": "any"}}
    if base == "array": return {"kind": "array", "elem": {"kind": "any"}, "n": None}
    return {"kind": "named", "name": base}
def tplace(pl):
    if pl.get("index"):
        ix = pl["index"]
        return {"kind": "index", "place": tplace(ix["place"]),
                "elem": ttype(ix["elem"]), "atom": texpr(ix["atom"])}
    node = {"kind": "var", "name": pl["base"]}
    for f in pl.get("path", []):
        node = {"kind": "field", "place": node, "name": f}
    for _ in range(pl.get("deref", 0)):
        node = {"kind": "deref", "place": node}
    for f in pl.get("post", []):                       # psuffix fields applied after the deref [GRAM-5]
        node = {"kind": "field", "place": node, "name": f}
    return node
def texpr(e):
    k = e["e"]
    if k == "lit": return {"kind": "lit", "ty": {"kind": "prim", "name": e["ty"]},
                           "value": e["v"]}
    if k == "unit": return {"kind": "lit", "ty": {"kind": "unit"}}
    if k == "place": return {"kind": "use", "place": tplace(e["p"])}
    if k == "move": return {"kind": "move", "place": tplace(e["p"])}
    if k == "borrow": return {"kind": "borrow", "region": e["region"], "uniq": e["uniq"],
                              "place": tplace(e["place"])}
    if k == "construct": return {"kind": "construct", "name": e["n"],
        "fields": [{"name": f["name"], "atom": texpr(f["atom"])} for f in e["fields"]]}
    if k == "ucall": return {"kind": "call", "callee": e["n"],
        "args": [texpr(a) for a in e["args"]], "argnames": e["argnames"],
        "region_args": e["region_args"]}
    if k == "op": return {"kind": "call", "callee": e["op"],
        "args": [texpr(a) for a in e["args"]], "argnames": None,
        "tyargs": [ttype(t) for t in e["tyargs"]]}
    raise SystemExit(f"democ: cannot map expr {k}")
def tmatch(m):
    return {"kind": "match", "scrut": texpr(m["scrut"]),
            "arms": [{"variant": a["v"],
                      "binders": [{"field": b["field"], "name": b["name"]} for b in a["b"]],
                      "body": tstmts(a["body"])} for a in m["arms"]]}
def tstmt(s):
    k = s["s"]
    if k == "doc": return None
    if k == "let":
        init = tmatch(s["match"]) if "match" in s else texpr(s["e"])
        return {"kind": "let", "name": s["n"], "mode": s["m"], "ty": ttype(s["ty"]), "init": init}
    if k == "set": return {"kind": "set", "place": tplace(s["p"]), "expr": texpr(s["e"])}
    if k == "return": return {"kind": "return", "expr": texpr(s["e"])}
    if k == "region": return {"kind": "region", "name": s["r"], "body": tstmts(s["body"])}
    if k == "loop": return {"kind": "loop", "label": s["l"], "body": tstmts(s["body"])}
    if k == "break": return {"kind": "break"}
    if k == "check": return {"kind": "check", "expr": texpr(s["e"])}
    if k == "try":
        return {"kind": "try", "name": s["n"], "mode": s["m"], "ty": ttype(s["ty"]),
                "expr": texpr(s["e"])}
    if k == "match": return tmatch(s)
    if k == "give": return {"kind": "give", "expr": texpr(s["e"])}
    if k == "expr": return {"kind": "expr", "expr": texpr(s["e"])}
    raise SystemExit(f"democ: cannot map stmt {k}")
def tstmts(body):
    return [x for x in (tstmt(s) for s in body) if x is not None]
def build_prog(structs, enums, fns, consts=()):
    if len([f for f in fns if f["name"] == "main"]) > 1:   # FN-7: at most one main
        raise CheckError("FN-7", "at most one fn main")
    for c in consts:                                       # [CONST-2] const-eligibility: primitive or array<prim,N>
        ct = c["ty"]
        parts = _arr_parts(ct)
        base_elem = parts[0] if parts else ct
        if base_elem not in PRIM_SET:
            raise CheckError("CONST-2",
                f"const {c['name']}: type '{ct}' is not const-eligible "
                "(a const is static rodata: only a primitive or array<primitive, N>)")
    prog = {"structs": {}, "enums": {}, "fns": {},
            "consts": {c["name"]: ttype(c["ty"]) for c in consts}}  # [CONST-2]
    for sn, flds in structs.items():                       # struct field types -> the type layer [TYPE-2]
        prog["structs"][sn] = [{"name": f["name"], "ty": ttype(f["ty"])} for f in flds]
    for en, vs in enums.items():
        prog["enums"][en] = [{"variant": vn,
            "fields": [{"name": f["name"], "ty": ttype(f["ty"])} for f in flds]}
            for (vn, flds) in vs]
    for f in fns:
        prog["fns"][f["name"]] = {
            "regions": f["regions"],
            "params": [{"name": q["name"], "mode": q["mode"], "ty": ttype(q["ty"])}
                       for q in f["params"]],
            "rmode": f["rmode"], "rty": ttype(f["rty"]),
            "effects": f.get("effects", []),
            "requires": (tstmts(f["requires"]) if f.get("requires") is not None else None),
            "body": tstmts(f["body"]),
        }
    return prog


def _has_loop(body):
    for s in body:
        k = s.get("s")
        if k == "loop": return True
        if k in ("region",) and _has_loop(s["body"]): return True
        if k == "match" and any(_has_loop(a["body"]) for a in s["arms"]): return True
        if "match" in s and any(_has_loop(a["body"]) for a in s["match"]["arms"]): return True
    return False

def _calls(body, out):
    def ex(e):
        if not isinstance(e, dict): return
        if e.get("e") == "ucall": out.add(e["n"])
        for a in e.get("args", []): ex(a)
    for s in body:
        for key in ("e", "expr", "init", "scrut"):
            if key in s: ex(s[key])
        if s.get("s") == "region": _calls(s["body"], out)
        if s.get("s") == "match":
            for a in s["arms"]: _calls(a["body"], out)
        if "match" in s:                       # give-match let: scrutinee + arm bodies
            ex(s["match"]["scrut"])
            for a in s["match"]["arms"]: _calls(a["body"], out)
    return out

def compute_total(fns):
    """Derived willreturn tier: loop-free + trap-free + all callees total.
    Trap-freedom is read off the effect row — sound because EFF-2's exhibits
    discipline (checker-enforced) forces every may-abort source (index bounds,
    .trap arithmetic, buffer_new size check, check-else-trap) to declare
    `traps`. Memory rows (reads/writes/allocates) are termination-irrelevant
    and do not block the tier."""
    total = {}
    for _ in range(len(fns) + 1):
        for f in fns:
            if f["name"] in total: continue
            if _has_loop(f["body"]): total[f["name"]] = False; continue
            if "traps" in f.get("effects", []): total[f["name"]] = False; continue
            cs = _calls(f["body"], set())
            if all(total.get(c, None) is True for c in cs):
                total[f["name"]] = True
    return {k for k, v in total.items() if v}

def totality_report(fns):
    """[EFF-3 / willreturn tier] the totality-economics lint: which fns earn the
    derived willreturn attribute, and exactly what blocks each of the rest —
    traps are INFECTIOUS (one trapping leaf strips hoisting/CSE rights from
    every transitive caller), so blockers are named per fn."""
    total = compute_total(fns)
    names = {f["name"] for f in fns}
    out = ["totality report (derived willreturn: loop-free + trap-free + total callees):"]
    w = max((len(f["name"]) for f in fns), default=1)
    for f in fns:
        if f["name"] in total:
            out.append(f"  {f['name']:<{w}}  TOTAL -> willreturn")
            continue
        blockers = []
        if _has_loop(f["body"]): blockers.append("loop (const-trip tier not yet derived)")
        if "traps" in f.get("effects", []):
            blockers.append("traps row (may abort; infectious upward)")
        cs = sorted(_calls(f["body"], set()))
        bad = [c for c in cs if c in names and c not in total]
        if bad: blockers.append("non-total callees: " + ", ".join(bad))
        out.append(f"  {f['name']:<{w}}  blocked: " + "; ".join(blockers))
    out.append(f"  {len(total)}/{len(fns)} functions willreturn")
    return "\n".join(out)

# ---- LLVM IR ----
INT_LL = {"i8", "i16", "i32", "i64"}            # the LLVM integer types democ emits

def _enum_payw(variants):
    """Widest payload (bits) across an enum's variants; 0 => tag-only enum.
    A payload-carrying enum lowers to { i32 tag, i<payw> payload }: one word-sized
    slot sized to hold any variant's payload, narrower payloads zext/trunc in/out
    [word-sized copy payloads]. Multi-field or non-int-field variants only occur in
    checker-rejected programs (never reach codegen)."""
    best = 0
    for _vn, flds in variants:
        w = sum(1 if f["ty"] == "Bool" else INT_WIDTH.get(f["ty"], 32) for f in flds)
        best = max(best, w)
    return best

def _is_tag_only_enum_spelling(name, enums):
    """Validate democ's exact nominal eeq/ene domain before LLVM emission."""
    if name == "Bool":
        return True
    variants = enums.get(name) if isinstance(enums, dict) else None
    return (isinstance(variants, list)
            and all(isinstance(variant, tuple) and len(variant) == 2
                    and isinstance(variant[1], list) and not variant[1]
                    for variant in variants))

def _field_ll(tyname, structs):
    """LLVM type for a struct field: int width, i1 for Bool, %Sub for a nested struct,
    i32 for an enum tag (the democ enum representation)."""
    if tyname in INT_LLTY: return INT_LLTY[tyname]
    if tyname == "Bool" or tyname in _TAGONLY2: return "i1"
    if tyname in structs: return "%" + tyname
    if _buf_elem(tyname): return "{ptr, i64}"   # buffer<T> field: {data-pointer, length} pair [TYPE-2]
    return "i32"                                # user enum / prelude tag-only: i32 tag

class Gen:
    def __init__(g, f, enums, structs=None, alias=True, fnret=None, decls=None, total=frozenset(),
                 mdefs=None, mdctr=None, elide=False, proof_report=None):
        g.f = f; g.enums = enums; g.structs = structs or {}; g.alias = alias
        g.fnret = fnret or {}          # fn name -> LLVM return type (for cross-fn calls)
        g.fnrty = getattr(Gen, "_fnrty", {})
        g.decls = decls if decls is not None else set()   # extra intrinsic declares used
        g.n = 0; g.lines = []; g.env = {}; g.traps = False; g.term = False
        g.loopstk = []
        g.elide = elide                # EXPERIMENT ONLY: emit no bounds checks (perfect-prover ceiling)
        g.proof_report = proof_report  # optional structured OP-4 site accounting
        g.bounds_site = 0              # deterministic source/codegen order within this function
        g.mdefs = mdefs                # module-level scoped-alias metadata lines [F003 channel]
        g.mdctr = mdctr                # shared metadata id counter (one numbering per module)
        g.pmode = {}                   # param name -> 'uniq' | 'shared' | 'own'
        g.consts = getattr(Gen, "_consts", {})   # [CONST-2] array tuple or ('scalar', value, signed, LLVM type)
        g.scopes = {}                  # provenance key -> (alias.scope list id, noalias list id)
        g.total = total; g.give_slot = None; g.give_ty = "i32"
        g.entry_allocas = []           # fixed-size stack slots, emitted once in the entry block
        g.prologue = []                # entry-block setup after stack-slot declarations
        # payload-carrying enums lower to a named aggregate %E = { i32, i<payw> }
        g.payenums = {en for en, vs in g.enums.items() if _enum_payw(vs) > 0}
        # global variant registry over prelude + user enums (TYPE-6 => names unique):
        # variant -> (enum, tag index); enum -> [variant, ...]. This is the single
        # dispatch table that subsumes Ok/Err, Some/None and user variants alike.
        g.venum, g.evariants = {}, {}
        def _reg(en, names):
            g.evariants[en] = list(names)
            for i, vn in enumerate(names):
                g.venum[vn] = (en, i)
        for en, vs in PRELUDE_ENUMS.items():
            _reg(en, [v["variant"] for v in vs])
        for en, vs in g.enums.items():
            _reg(en, [vn for vn, _ in vs])
    def llty(g, name):                             # struct/enum-aware LLVM type name
        if _buf_elem(name): return "{ptr, i64}"
        name = name.split("<")[0]                      # Result<T,E> lowers by base name
        if name in g.structs or name in g.payenums or name in ("Result", "Option"):
            return "%" + name
        return _llty(name)
    def enum_payll(g, en):                         # LLVM type of the payload word of enum en
        if en not in g.enums:                          # prelude Result/Option: word-erased payload
            return "i64"
        return "i" + str(_enum_payw(g.enums[en]))
    def variant_field_ll(g, en, variant, type_args=None):  # instantiated field LLVM type
        if en == "Result" and type_args:
            ty = type_args[0] if variant == "Ok" else type_args[1]
            return g.llty(ty), _is_signed(ty)
        if en == "Option" and type_args and variant == "Some":
            ty = type_args[0]
            return g.llty(ty), _is_signed(ty)
        if en not in g.enums:                          # erased prelude fallback
            return "i64", True
        for vn, flds in g.enums[en]:
            if vn == variant and flds:
                return _field_ll(flds[0]["ty"], g.structs), _is_signed(flds[0]["ty"])
        return g.enum_payll(en), True
    def coerce_int(g, val, src_ll, dst_ll):        # bit-preserving zext/trunc between int widths
        if src_ll == dst_ll:
            return val
        op = "zext" if int(dst_ll[1:]) > int(src_ll[1:]) else "trunc"
        t = g.tmp(); g.emit(f"  {t} = {op} {src_ll} {val} to {dst_ll}")
        return t
    def pack_enumv(g, x):                          # SSA enumv -> %E aggregate value
        en = x["en"]; pw = g.enum_payll(en)
        tag = x["tag"]
        if x.get("tty") and x["tty"] != "i32":
            t = g.tmp(); g.emit(f"  {t} = zext {x['tty']} {tag} to i32"); tag = t
        pay = x["pay"]; pty = x.get("pty", "i32")
        if pty != pw:
            t = g.tmp()
            verb = "zext" if int(pw[1:]) > int(pty[1:]) else "trunc"
            g.emit(f"  {t} = {verb} {pty} {pay} to {pw}"); pay = t
        a0 = g.tmp(); g.emit(f"  {a0} = insertvalue %{en} undef, i32 {tag}, 0")
        a1 = g.tmp(); g.emit(f"  {a1} = insertvalue %{en} {a0}, {pw} {pay}, 1")
        return a1

    def load_enum(g, x):                           # materialize an SSA aggregate from an enum value
        if x.get("slot"):
            t = g.tmp(); g.emit(f"  {t} = load %{x['en']}, ptr {x['v']}")
            return t
        return x["v"]
    def field_info(g, sname, fname):               # (index, LLVM type, signed, sub-struct-or-None, tyname)
        for i, fld in enumerate(g.structs[sname]):
            if fld["name"] == fname:
                ty = fld["ty"]
                return (i, _field_ll(ty, g.structs), _is_signed(ty),
                        ty if ty in g.structs else None, ty)
        raise SystemExit(f"democ: struct {sname} has no field {fname}")
    def place_addr(g, pl):
        """GEP a field-path place to a pointer; returns (ptr, elem_llty, signed, sub-struct, tyname)."""
        v = g.env[pl["base"]]
        assert v.get("st"), f"field access on non-struct place {pl['base']}"
        ptr, cur, llty, signed, sub, tyn = v["v"], v["st"], "%" + v["st"], True, v["st"], v["st"]
        for fname in pl.get("path", []) + pl.get("post", []):
            idx, llty, signed, sub, tyn = g.field_info(cur, fname)
            t = g.tmp()
            g.emit(f"  {t} = getelementptr %{cur}, ptr {ptr}, i32 0, i32 {idx}")
            ptr = t; cur = sub
        return ptr, llty, signed, sub, tyn
    def load_struct(g, x):                         # materialize an SSA aggregate from a struct value
        if x.get("slot"):
            t = g.tmp(); g.emit(f"  {t} = load %{x['st']}, ptr {x['v']}")
            return t
        return x["v"]
    def tmp(g): g.n += 1; return f"%t{g.n}"
    def stack_slot(g, ty):
        """Reserve one fixed-size slot at function entry, never inside a loop."""
        slot = g.tmp()
        g.entry_allocas.append(f"  {slot} = alloca {ty}")
        return slot
    def lbl(g): g.n += 1; return f"L{g.n}"
    def emit(g, s): g.lines.append(s)
    def scope_key(g, pl):
        """[F003 channel] provenance class of a place rooted at a borrow/own param, or None.
        Sound by OWN-1 (affine: distinct live buffer values have disjoint element memory),
        OWN-2/5 (&uniq excludes every other in-function path to its target for the whole
        call), and T-A (each explicit bounded-reborrow lineage retains one singleton root,
        disjoint from every other live uniq-param lineage). All shared-borrow-rooted accesses share ONE class (two shared
        params may alias the same target), so no false claim is ever made between them."""
        mode = g.pmode.get(pl["base"])
        if mode is None: return None
        if mode == "shared": return "__shared"
        return (pl["base"], tuple(pl.get("path", []) + pl.get("post", [])))
    def scope_suffix(g, pl, hdr=False):
        """metadata suffix for an access through place pl; hdr=True marks struct-memory
        (header/scalar-field) accesses, keyed at the root: buffer element memory is a
        separate primitive-only heap allocation, disjoint from every struct slot."""
        key = g.scope_key(pl)
        if key is None: return ""
        if hdr and key != "__shared": key = (pl["base"], ())
        pair = g.scopes.get(key)
        return f", !alias.scope !{pair[0]}, !noalias !{pair[1]}" if pair else ""
    def load_buffield(g, fp, elname, pl):
        """load a buffer field's {ptr, len} header (tagged struct-memory access)."""
        md = g.scope_suffix(pl, hdr=True)
        pp = g.tmp(); g.emit(f"  {pp} = getelementptr {{ptr, i64}}, ptr {fp}, i32 0, i32 0")
        bp = g.tmp(); g.emit(f"  {bp} = load ptr, ptr {pp}{md}")
        lp = g.tmp(); g.emit(f"  {lp} = getelementptr {{ptr, i64}}, ptr {fp}, i32 0, i32 1")
        bl = g.tmp(); g.emit(f"  {bl} = load i64, ptr {lp}{md}")
        return {"k": "buffer", "ptr": bp, "len": bl, "ell": _llty(elname),
                "esigned": _is_signed(elname), "key": g.scope_key(pl)}
    def index_addr(g, pl):
        """[OP-4] bounds-checked element address of an index place; traps OOB.
        Returns (ptr, elem_llty, signed, metadata-suffix)."""
        ipl = pl["index"]["place"]
        cinfo = g.consts.get(ipl["base"]) if not (ipl.get("path") or ipl.get("post")) else None
        proof = pl["index"].get("proof")
        status = "ceiling" if g.elide else ("proved" if proof else "retained")
        kind = "const" if cinfo and cinfo[0] != "scalar" else \
            ("buffer-field" if ipl.get("path") or ipl.get("post") else "buffer")
        if g.proof_report is not None:
            atom = pl["index"]["atom"]
            index_name = atom.get("p", {}).get("base") if atom.get("e") == "place" else None
            obligation = pl["index"].get(
                "obligation_info", _indeterminate_obligation_info())
            g.proof_report.append({
                "function": g.f["name"],
                "site": g.bounds_site,
                "status": status,
                "proof": proof,
                "kind": kind,
                "target": ipl["base"],
                "index": index_name,
                **obligation,
            })
            pl["index"]["_proof_report_emissions"] = (
                pl["index"].get("_proof_report_emissions", 0) + 1
            )
        g.bounds_site += 1
        if cinfo and cinfo[0] != "scalar":             # [CONST-2] const-array index: GEP the global, bounds-checked
            ell, n, sg = cinfo
            iv = g.expr(pl["index"]["atom"])
            if not g.elide and not proof:                        # [OP-4]; PROOF-1 discharges marked sites
                g.traps = True
                c = g.tmp(); g.emit(f"  {c} = icmp ult i64 {iv['v']}, {n}")
                l = g.lbl(); g.emit(f"  br i1 {c}, label %{l}, label %trap"); g.emit(f"{l}:")
            ptr = g.tmp()
            g.emit(f"  {ptr} = getelementptr [{n} x {ell}], ptr @__const_{ipl['base']}, i64 0, i64 {iv['v']}")
            return ptr, ell, sg, ""
        if ipl.get("path") or ipl.get("post"):         # buffer behind a struct field
            fp, fll, _s, _sub, tyn = g.place_addr(ipl)
            el = _buf_elem(tyn)
            assert el, "index place over a non-buffer field (democ subset)"
            buf = g.load_buffield(fp, el, ipl)
        else:
            buf = g.env[ipl["base"]]
            assert buf.get("k") == "buffer", "index place over a non-buffer binding (democ subset)"
        md = ""
        key = buf.get("key") or g.scope_key(ipl)
        if key is not None:
            pair = g.scopes.get(key)
            if pair: md = f", !alias.scope !{pair[0]}, !noalias !{pair[1]}"
        iv = g.expr(pl["index"]["atom"])
        if not g.elide and not proof:                        # [OP-4]; PROOF-1 discharges marked sites
            g.traps = True
            c = g.tmp(); g.emit(f"  {c} = icmp ult i64 {iv['v']}, {buf['len']}")
            l = g.lbl(); g.emit(f"  br i1 {c}, label %{l}, label %trap"); g.emit(f"{l}:")
        ptr = g.tmp(); g.emit(f"  {ptr} = getelementptr {buf['ell']}, ptr {buf['ptr']}, i64 {iv['v']}")
        return ptr, buf["ell"], buf["esigned"], md

    def ovf_decl(g, verb, signed, w):              # {sadd,uadd,...}.with.overflow.iW
        pfx = "s" if signed else "u"
        if w != "i32" or pfx != "s":               # sadd/ssub/smul.i32 are in the fixed header
            g.decls.add(f"declare {{{w}, i1}} @llvm.{pfx}{verb}.with.overflow.{w}({w}, {w})")
        return f"{pfx}{verb}"
    def vtag(g, name):
        for en, vs in g.enums.items():
            for i, (vn, _) in enumerate(vs):
                if vn == name: return i
        return None
    def argty(g, x):                               # LLVM type prefix for a call argument
        if x["k"] in INT_LL: return x["k"]
        if x["k"] in ("ptr", "slot"): return "ptr"
        if x["k"] == "struct": return "%" + x["st"]
        if x["k"] == "enum": return "%" + x["en"]
        return x["k"]
    def expr(g, e):
        k = e["e"]
        if k == "lit":
            return {"k": _llty(e["ty"]), "v": str(e["v"]), "signed": _is_signed(e["ty"])}
        if k == "unit": return {"k": "unit"}
        if k == "move":                                # move p: value read; the checker did the affine accounting
            return g.expr({"e": "place", "p": e["p"]})
        if k == "place":
            pl = e["p"]
            if pl.get("index"):                        # bounds-checked element read [OP-4]
                ptr, ell, sg, md = g.index_addr(pl)
                t = g.tmp(); g.emit(f"  {t} = load {ell}, ptr {ptr}{md}")
                return {"k": ell, "v": t, "signed": sg}
            if pl.get("path") or pl.get("post"):       # struct field read: GEP + load [TYPE-5]
                ptr, fll, fsigned, sub, tyn = g.place_addr(pl)
                if sub is not None:                    # leaf field is itself a struct
                    return {"k": "struct", "v": ptr, "st": sub, "slot": True}
                el = _buf_elem(tyn)
                if el:                                 # buffer-typed field: load {ptr, len} header
                    return g.load_buffield(ptr, el, pl)
                t = g.tmp(); g.emit(f"  {t} = load {fll}, ptr {ptr}{g.scope_suffix(pl, hdr=True)}")
                return {"k": fll, "v": t, "signed": fsigned}
            cinfo = g.consts.get(pl["base"])           # [CONST-2] scalar const: fold to its literal
            if cinfo and cinfo[0] == "scalar":
                return {"k": cinfo[3], "v": str(cinfo[1]), "signed": cinfo[2]}
            if cinfo:                                  # bare use of a const array: len operand marker
                return {"k": "constarr", "n": cinfo[1]}
            v = g.env[pl["base"]]
            if v["k"] in ("struct", "enum"):           # whole aggregate use stays an addressable slot
                return v
            if v["k"] == "ptr" and v.get("st"):        # bare struct-borrow use: the value IS the reference [TYPE-7]
                return v
            if v["k"] in ("ptr", "slot"):
                if v.get("en"):                        # &'r E: reading/deref yields the enum aggregate
                    return {"k": "enum", "v": v["v"], "en": v["en"], "slot": True,
                            "type_args": v.get("type_args")}
                ty = v.get("ty", "i32")
                t = g.tmp(); g.emit(f"  {t} = load {ty}, ptr {v['v']}")
                return {"k": ty, "v": t, "signed": v.get("signed", True)}
            return v
        if k == "borrow":                              # &'r p / &uniq 'r p -> pointer to place
            pl = e["place"]
            if pl.get("index"):                       # borrow of an element still performs OP-4
                ptr, ell, signed, _md = g.index_addr(pl)
                return {"k": "ptr", "v": ptr, "ty": ell, "signed": signed}
            if pl.get("path") or pl.get("post"):      # borrow of a struct field must point at that field
                ptr, fll, signed, sub, tyn = g.place_addr(pl)
                if sub is not None:
                    return {"k": "ptr", "v": ptr, "ty": "%" + sub, "st": sub}
                el = _buf_elem(tyn)
                if el:
                    return g.load_buffield(ptr, el, pl)
                if tyn in g.enums:
                    return {"k": "ptr", "v": ptr, "ty": "%" + tyn, "en": tyn}
                return {"k": "ptr", "v": ptr, "ty": fll, "signed": signed}
            src = g.env[pl["base"]]
            if src["k"] == "buffer":                  # reborrow of a buffer keeps its checked header value
                return dict(src)                       # (whole-buffer replacement is rejected by STOR-1)
            if src["k"] == "enum":                     # borrow of an own enum local: ptr to its slot
                return {"k": "ptr", "v": src["v"], "ty": "%" + src["en"], "en": src["en"],
                        "type_args": src.get("type_args")}
            if src["k"] == "struct":                   # borrow of an own struct local: ptr to its slot
                return {"k": "ptr", "v": src["v"], "ty": "%" + src["st"], "st": src["st"]}
            if src["k"] in ("slot", "ptr"):
                return {"k": "ptr", "v": src["v"], "ty": src.get("ty", "i32"),
                        "signed": src.get("signed", True), "en": src.get("en"),
                        "type_args": src.get("type_args")}
            ty = src["k"] if src["k"] in INT_LL else "i32"
            slot = g.stack_slot(ty)                         # spill own SSA to make addressable
            g.emit(f"  store {ty} {src['v']}, ptr {slot}")
            return {"k": "ptr", "v": slot, "ty": ty, "signed": src.get("signed", True)}
        if k == "construct":
            n = e["n"]; flds = e["fields"]
            # Bool is the degenerate 2-variant tag-only enum, lowered to i1 (its tag IS
            # the value); True/False are its nullary constructors.
            if n == "True": return {"k": "i1", "v": "true"}
            if n == "False": return {"k": "i1", "v": "false"}
            _en2, _idx2 = g.venum.get(n, (None, None))
            if _en2 in _TAGONLY2:
                return {"k": "i1", "v": "true" if _idx2 == 1 else "false"}
            # Prelude Option/Result are enums with one word-sized (type-erased) payload.
            # A directly-constructed variant is an SSA enum value {tag, payload}; the tag
            # index comes from the same registry as user enums [PRE-1].
            if n in ("Ok", "Some"):
                a = g.expr(flds[0]["atom"]); idx = g.venum[n][1]
                return {"k": "enumv", "tag": str(idx), "tty": "i32", "pay": a["v"],
                        "pty": a["k"] if a["k"] in INT_LL or a["k"] == "i1" else "i32",
                        "psigned": a.get("signed", True), "payidx": idx, "en": g.venum[n][0]}
            if n in ("Err", "None"):
                idx = g.venum[n][1]
                if e["fields"]:                        # Err carries its error payload
                    a = g.expr(e["fields"][0]["atom"])
                    if a.get("k") in ("enum", "enumv") and a.get("en") not in (None, "Result", "Option") \
                            and a.get("en") in g.payenums:
                        raise SystemExit("democ: a payload-carrying enum value inside Err()/Ok() is not in "
                                         "the subset; use nullary error variants (e.g. BadDigit() not BadDigit(pos: ...))")
                    return {"k": "enumv", "tag": str(idx), "tty": "i32", "pay": a["v"],
                            "pty": a["k"] if a["k"] in INT_LL or a["k"] == "i1" else "i32",
                            "psigned": a.get("signed", True), "payidx": idx, "en": g.venum[n][0]}
                return {"k": "enumv", "tag": str(idx), "tty": "i32", "pay": "0", "pty": "i32",
                        "psigned": True, "payidx": idx, "en": g.venum[n][0]}
            en, _idx = g.venum.get(n, (None, None))
            if en in g.payenums:                       # payload enum: build {i32 tag, iN payload} [GRAM-8]
                payll = g.enum_payll(en)
                slot = g.stack_slot(f"%{en}")
                tp = g.tmp(); g.emit(f"  {tp} = getelementptr %{en}, ptr {slot}, i32 0, i32 0")
                g.emit(f"  store i32 {g.vtag(n)}, ptr {tp}")
                pp = g.tmp(); g.emit(f"  {pp} = getelementptr %{en}, ptr {slot}, i32 0, i32 1")
                if flds:                               # payload variant: coerce the field into the word
                    fv = g.expr(flds[0]["atom"]); stored = g.coerce_int(fv["v"], fv["k"], payll)
                else:                                  # nullary variant of a payload enum: zero the word
                    stored = "0"
                g.emit(f"  store {payll} {stored}, ptr {pp}")
                return {"k": "enum", "v": slot, "en": en, "slot": True}
            if n in g.structs:                         # struct construct: alloca + store each field [GRAM-8]
                slot = g.stack_slot(f"%{n}")
                for fld in flds:
                    fv = g.expr(fld["atom"])
                    idx, fll, _fs, sub, _tyn = g.field_info(n, fld["name"])
                    fp = g.tmp()
                    g.emit(f"  {fp} = getelementptr %{n}, ptr {slot}, i32 0, i32 {idx}")
                    if fv["k"] == "struct":            # nested struct field: store the aggregate
                        g.emit(f"  store %{sub} {g.load_struct(fv)}, ptr {fp}")
                    elif fv["k"] == "buffer":          # buffer field: pack {ptr, i64} pair
                        t0 = g.tmp(); g.emit(f"  {t0} = insertvalue {{ptr, i64}} undef, ptr {fv['ptr']}, 0")
                        t1 = g.tmp(); g.emit(f"  {t1} = insertvalue {{ptr, i64}} {t0}, i64 {fv['len']}, 1")
                        g.emit(f"  store {{ptr, i64}} {t1}, ptr {fp}")
                    else:
                        g.emit(f"  store {fll} {fv['v']}, ptr {fp}")
                return {"k": "struct", "v": slot, "st": n, "slot": True}
            return {"k": "i32", "v": str(g.vtag(n)), "signed": True}
        if k == "ucall":
            # Region arguments are checker-only substitutions and never runtime ABI operands.
            args = [g.expr(a) for a in e["args"]]
            def _passarg(x):                           # pass aggregates (struct/enum) by value
                if x["k"] == "struct": return {"k": "struct", "st": x["st"], "v": g.load_struct(x)}
                if x["k"] == "enum": return {"k": "enum", "en": x["en"], "v": g.load_enum(x)}
                return x
            args = [_passarg(x) for x in args]
            ret = g.fnret.get(e["n"], "i32")
            rstr = g.fnrty.get(e["n"], "")
            packed = []
            for x in args:
                if x.get("k") == "buffer":             # pack {ptr, i64} at the callsite
                    t0 = g.tmp(); g.emit(f"  {t0} = insertvalue {{ptr, i64}} undef, ptr {x['ptr']}, 0")
                    t1 = g.tmp(); g.emit(f"  {t1} = insertvalue {{ptr, i64}} {t0}, i64 {x['len']}, 1")
                    packed.append(f"{{ptr, i64}} {t1}")
                else:
                    packed.append(f"{g.argty(x)} {x['v']}")
            argll = ', '.join(packed)
            if ret == "void":
                g.emit(f"  call void @{e['n']}({argll})"); return {"k": "unit"}
            t = g.tmp(); g.emit(f"  {t} = call {ret} @{e['n']}({argll})")
            el = _buf_elem(rstr)
            if el:                                     # unpack buffer return
                bp = g.tmp(); g.emit(f"  {bp} = extractvalue {{ptr, i64}} {t}, 0")
                bl = g.tmp(); g.emit(f"  {bl} = extractvalue {{ptr, i64}} {t}, 1")
                return {"k": "buffer", "ptr": bp, "len": bl,
                        "ell": _llty(el), "esigned": _is_signed(el)}
            if ret.startswith("%"):                    # aggregate return: spill to a slot
                nm = ret[1:]
                if nm in g.payenums or nm in ("Result", "Option"):
                    sl = g.stack_slot(f"%{nm}")
                    g.emit(f"  store %{nm} {t}, ptr {sl}")
                    type_args = _named_args(rstr, nm) if nm in ("Result", "Option") else None
                    return {"k": "enum", "v": sl, "en": nm, "slot": True,
                            "type_args": type_args}
                return {"k": "struct", "v": t, "st": nm, "slot": False}
            return {"k": ret, "v": t, "signed": True}
        return g.op(e)
    def op(g, e):
        a = [g.expr(x) for x in e["args"]]
        op = e["op"]; base, _, mode = op.partition(".")
        ty = e.get("tyargs") or ["i32"]
        w = _llty(ty[0]); signed = _is_signed(ty[0])
        if base == "buffer_new":                       # [OP-9] size-overflow trap, alloc, fill
            n_, fill = a[0], a[1]
            esz = _tybytes(ty[0])
            g.traps = True
            g.decls.add("declare {i64, i1} @llvm.umul.with.overflow.i64(i64, i64)")
            g.decls.add("declare ptr @malloc(i64)")
            pr = g.tmp(); g.emit(f"  {pr} = call {{i64, i1}} @llvm.umul.with.overflow.i64(i64 {n_['v']}, i64 {esz})")
            by = g.tmp(); g.emit(f"  {by} = extractvalue {{i64, i1}} {pr}, 0")
            ov = g.tmp(); g.emit(f"  {ov} = extractvalue {{i64, i1}} {pr}, 1")
            l = g.lbl(); g.emit(f"  br i1 {ov}, label %trap, label %{l}"); g.emit(f"{l}:")
            buf = g.tmp(); g.emit(f"  {buf} = call ptr @malloc(i64 {by})")
            # fill loop (entry-block counter slot; mem2reg cleans)
            cs = g.stack_slot("i64"); g.emit(f"  store i64 0, ptr {cs}")
            lc, lb, ld = g.lbl(), g.lbl(), g.lbl()
            g.emit(f"  br label %{lc}"); g.emit(f"{lc}:")
            cur = g.tmp(); g.emit(f"  {cur} = load i64, ptr {cs}")
            cc = g.tmp(); g.emit(f"  {cc} = icmp ult i64 {cur}, {n_['v']}")
            g.emit(f"  br i1 {cc}, label %{lb}, label %{ld}"); g.emit(f"{lb}:")
            ep = g.tmp(); g.emit(f"  {ep} = getelementptr {w}, ptr {buf}, i64 {cur}")
            g.emit(f"  store {w} {fill['v']}, ptr {ep}")
            nx = g.tmp(); g.emit(f"  {nx} = add i64 {cur}, 1")
            g.emit(f"  store i64 {nx}, ptr {cs}"); g.emit(f"  br label %{lc}")
            g.emit(f"{ld}:")
            return {"k": "buffer", "ptr": buf, "len": n_["v"], "ell": w, "esigned": signed}
        if base == "len":                              # len<T>(b) -> own u64 [OP-1]
            src = a[0]
            if src.get("k") == "constarr":             # [CONST-2] len of a const array is its static N
                return {"k": "i64", "v": str(src["n"]), "signed": False}
            assert src.get("k") == "buffer", "len over a non-buffer value (democ subset)"
            return {"k": "i64", "v": src["len"], "signed": False}
        VERB = {"iadd": "add", "isub": "sub", "imul": "mul"}
        if base in VERB:                               # add/sub/mul: result-overflow axis
            if mode == "wrap":
                t = g.tmp(); g.emit(f"  {t} = {VERB[base]} {w} {a[0]['v']}, {a[1]['v']}")
                return {"k": w, "v": t, "signed": signed}
            if mode == "sat":                          # clamp to T's range [OP-8]
                if base == "imul": raise SystemExit("demo: imul.sat (widen+clamp) not in subset")
                iv = ("s" if signed else "u") + VERB[base]
                g.decls.add(f"declare {w} @llvm.{iv}.sat.{w}({w}, {w})")
                t = g.tmp(); g.emit(f"  {t} = call {w} @llvm.{iv}.sat.{w}({w} {a[0]['v']}, {w} {a[1]['v']})")
                return {"k": w, "v": t, "signed": signed}
            g.traps = g.traps or mode == "trap"
            iv = g.ovf_decl(VERB[base], signed, w)
            p_ = g.tmp(); g.emit(f"  {p_} = call {{{w}, i1}} @llvm.{iv}.with.overflow.{w}({w} {a[0]['v']}, {w} {a[1]['v']})")
            v = g.tmp(); g.emit(f"  {v} = extractvalue {{{w}, i1}} {p_}, 0")
            o = g.tmp(); g.emit(f"  {o} = extractvalue {{{w}, i1}} {p_}, 1")
            if mode == "checked":                      # Result<T, Overflow> as an SSA enum: tag i1 (Ok=0/Err=1)
                return {"k": "enumv", "tag": o, "tty": "i1", "pay": v, "pty": w,
                        "psigned": signed, "payidx": 0, "en": "Result"}
            l = g.lbl(); g.emit(f"  br i1 {o}, label %trap, label %{l}"); g.emit(f"{l}:")
            return {"k": w, "v": v, "signed": signed}
        if base in ("idiv", "irem"):                   # trap on zero divisor + signed MIN/-1 [OP-2]
            verb = (("sdiv" if signed else "udiv") if base == "idiv"
                    else ("srem" if signed else "urem"))
            dz = g.tmp(); g.emit(f"  {dz} = icmp eq {w} {a[1]['v']}, 0")
            if mode == "trap":
                g.traps = True
                l = g.lbl(); g.emit(f"  br i1 {dz}, label %trap, label %{l}"); g.emit(f"{l}:")
                if signed:
                    mn = g.tmp(); g.emit(f"  {mn} = icmp eq {w} {a[0]['v']}, {INT_MIN[ty[0]]}")
                    m1 = g.tmp(); g.emit(f"  {m1} = icmp eq {w} {a[1]['v']}, -1")
                    ov = g.tmp(); g.emit(f"  {ov} = and i1 {mn}, {m1}")
                    l2 = g.lbl(); g.emit(f"  br i1 {ov}, label %trap, label %{l2}"); g.emit(f"{l2}:")
                q = g.tmp(); g.emit(f"  {q} = {verb} {w} {a[0]['v']}, {a[1]['v']}")
                return {"k": w, "v": q, "signed": signed}
            err = dz                                   # checked: divert to Err; safe divisor avoids poison
            if signed:
                mn = g.tmp(); g.emit(f"  {mn} = icmp eq {w} {a[0]['v']}, {INT_MIN[ty[0]]}")
                m1 = g.tmp(); g.emit(f"  {m1} = icmp eq {w} {a[1]['v']}, -1")
                ov = g.tmp(); g.emit(f"  {ov} = and i1 {mn}, {m1}")
                err = g.tmp(); g.emit(f"  {err} = or i1 {dz}, {ov}")
            sb = g.tmp(); g.emit(f"  {sb} = select i1 {err}, {w} 1, {w} {a[1]['v']}")
            q = g.tmp(); g.emit(f"  {q} = {verb} {w} {a[0]['v']}, {sb}")
            return {"k": "enumv", "tag": err, "tty": "i1", "pay": q, "pty": w,
                    "psigned": signed, "payidx": 0, "en": "Result"}
        BIT = {"iand": "and", "ior": "or", "ixor": "xor"}
        if base in BIT:                                # bitwise: total [OP-8]
            t = g.tmp(); g.emit(f"  {t} = {BIT[base]} {w} {a[0]['v']}, {a[1]['v']}")
            return {"k": w, "v": t, "signed": signed}
        if base in ("ishl", "ishr"):                   # logical/arith shift; amount out-of-range axis
            amt32 = a[1]["v"]
            if mode == "trap":
                g.traps = True
                oor = g.tmp(); g.emit(f"  {oor} = icmp uge i32 {amt32}, {INT_WIDTH[ty[0]]}")
                l = g.lbl(); g.emit(f"  br i1 {oor}, label %trap, label %{l}"); g.emit(f"{l}:")
                rawamt = amt32
            else:                                      # wrap: mask amount to width-1 [OP-8]
                mk = g.tmp(); g.emit(f"  {mk} = and i32 {amt32}, {INT_WIDTH[ty[0]] - 1}")
                rawamt = mk
            if w == "i32":
                amt = rawamt
            elif INT_WIDTH[ty[0]] > 32:
                amt = g.tmp(); g.emit(f"  {amt} = zext i32 {rawamt} to {w}")
            else:
                amt = g.tmp(); g.emit(f"  {amt} = trunc i32 {rawamt} to {w}")
            instr = "shl" if base == "ishl" else ("ashr" if signed else "lshr")
            t = g.tmp(); g.emit(f"  {t} = {instr} {w} {a[0]['v']}, {amt}")
            return {"k": w, "v": t, "signed": signed}
        if base in ("irotl", "irotr"):                 # rotates: dotless-total via fshl/fshr [OP-8]
            fn = "fshl" if base == "irotl" else "fshr"
            g.decls.add(f"declare {w} @llvm.{fn}.{w}({w}, {w}, {w})")
            if w == "i32":
                amt = a[1]["v"]
            elif INT_WIDTH[ty[0]] > 32:
                amt = g.tmp(); g.emit(f"  {amt} = zext i32 {a[1]['v']} to {w}")
            else:
                amt = g.tmp(); g.emit(f"  {amt} = trunc i32 {a[1]['v']} to {w}")
            t = g.tmp(); g.emit(f"  {t} = call {w} @llvm.{fn}.{w}({w} {a[0]['v']}, {w} {a[0]['v']}, {w} {amt})")
            return {"k": w, "v": t, "signed": signed}
        if base in ("imin", "imax"):                   # signedness-parametric min/max [OP-8]
            iv = ("s" if signed else "u") + base[1:]
            g.decls.add(f"declare {w} @llvm.{iv}.{w}({w}, {w})")
            t = g.tmp(); g.emit(f"  {t} = call {w} @llvm.{iv}.{w}({w} {a[0]['v']}, {w} {a[1]['v']})")
            return {"k": w, "v": t, "signed": signed}
        if base in ("band", "bor", "bxor"):            # Bool ops stay i1 [PRE-1/OWN-1 amendment]
            instr = {"band": "and", "bor": "or", "bxor": "xor"}[base]
            t = g.tmp(); g.emit(f"  {t} = {instr} i1 {a[0]['v']}, {a[1]['v']}")
            return {"k": "i1", "v": t}
        if base == "bnot":
            t = g.tmp(); g.emit(f"  {t} = xor i1 {a[0]['v']}, true")
            return {"k": "i1", "v": t}
        if base in ("eeq", "ene"):                   # nominal tag-only equality [OP-1/7/8]
            if len(ty) != 1 or not _is_tag_only_enum_spelling(ty[0], g.enums):
                raise CheckError(
                    "OP-1",
                    f"{base} requires one exact nominal tag-only enum type, including Bool")
            pred = "eq" if base == "eeq" else "ne"
            t = g.tmp(); g.emit(f"  {t} = icmp {pred} {w} {a[0]['v']}, {a[1]['v']}")
            return {"k": "i1", "v": t}
        CMP_S = {"ieq": "eq", "ine": "ne", "ilt": "slt", "ile": "sle", "igt": "sgt", "ige": "sge"}
        CMP_U = {"ieq": "eq", "ine": "ne", "ilt": "ult", "ile": "ule", "igt": "ugt", "ige": "uge"}
        if base in CMP_S:                              # sign-correct integer comparison
            pred = (CMP_S if signed else CMP_U)[base]
            t = g.tmp(); g.emit(f"  {t} = icmp {pred} {w} {a[0]['v']}, {a[1]['v']}")
            return {"k": "i1", "v": t}
        if base == "cvt":                              # exact-or-Result [OP-6]
            return g.cvt(ty[0], ty[1], a[0])
        if base == "reinterpret":                      # same-width int<->int resign = no-op [OP-8]
            return {"k": _llty(ty[1]), "v": a[0]["v"], "signed": _is_signed(ty[1])}
        raise SystemExit(f"demo: op {op} not in subset")
    def cvt(g, src, dst, a):
        ws, wd = INT_WIDTH[src], INT_WIDTH[dst]
        ls, ld = _llty(src), _llty(dst)
        ss, sd = _is_signed(src), _is_signed(dst)
        x = a["v"]
        if wd > ws and (not ss or sd):                 # total (value-preserving) widening -> Dst, no Result
            t = g.tmp(); g.emit(f"  {t} = {'sext' if ss else 'zext'} {ls} {x} to {ld}")
            return {"k": ld, "v": t, "signed": sd}
        if wd < ws:                                    # narrowing candidate
            y = g.tmp(); g.emit(f"  {y} = trunc {ls} {x} to {ld}")
        elif wd == ws:                                 # same-width sign change: bit-identical candidate
            y = x
        else:                                          # widening non-total (iN->uM): sign-extend candidate
            y = g.tmp(); g.emit(f"  {y} = {'sext' if ss else 'zext'} {ls} {x} to {ld}")
        W = f"i{2 * max(ws, wd)}"                       # exact round-trip test in a headroom-safe width
        xe = g.tmp(); g.emit(f"  {xe} = {'sext' if ss else 'zext'} {ls} {x} to {W}")
        ye = g.tmp(); g.emit(f"  {ye} = {'sext' if sd else 'zext'} {ld} {y} to {W}")
        err = g.tmp(); g.emit(f"  {err} = icmp ne {W} {xe}, {ye}")
        return {"k": "enumv", "tag": err, "tty": "i1", "pay": y, "pty": ld,
                "psigned": sd, "payidx": 0, "en": "Result"}
    def stmts(g, body):
        for s in body:
            if g.term: break
            k = s["s"]
            if k == "doc": continue
            if k == "let":
                if "match" in s:                       # value-match with give [GIVE-1]
                    decl_base = s["ty"].split("<", 1)[0]
                    gty = g.llty(s["ty"]); gsigned = _is_signed(s["ty"])
                    slot = None
                    if s["ty"] != "unit":
                        slot = g.stack_slot(gty)
                    prev, prevt = g.give_slot, g.give_ty
                    g.give_slot = slot; g.give_ty = "unit" if s["ty"] == "unit" else gty
                    g.gen_match(s["match"])
                    g.give_slot, g.give_ty = prev, prevt
                    if g.term:                         # every arm transferred control; no merge value exists
                        continue
                    if s["ty"] == "unit":
                        g.env[s["n"]] = {"k": "unit"}
                    elif decl_base in g.payenums or decl_base in ("Result", "Option"):
                        type_args = (_named_args(s["ty"], decl_base)
                                     if decl_base in ("Result", "Option") else None)
                        g.env[s["n"]] = {"k": "enum", "v": slot, "en": decl_base,
                                         "slot": True, "type_args": type_args}
                    elif decl_base in g.structs:
                        g.env[s["n"]] = {"k": "struct", "v": slot, "st": decl_base,
                                         "slot": True}
                    elif _buf_elem(s["ty"]):
                        packed = g.tmp(); g.emit(f"  {packed} = load {{ptr, i64}}, ptr {slot}")
                        bp = g.tmp(); g.emit(f"  {bp} = extractvalue {{ptr, i64}} {packed}, 0")
                        bl = g.tmp(); g.emit(f"  {bl} = extractvalue {{ptr, i64}} {packed}, 1")
                        elem = _buf_elem(s["ty"])
                        g.env[s["n"]] = {"k": "buffer", "ptr": bp, "len": bl,
                                         "ell": _llty(elem), "esigned": _is_signed(elem)}
                    else:
                        g.env[s["n"]] = {"k": "slot", "v": slot, "ty": gty,
                                         "signed": gsigned}
                    continue
                v = g.expr(s["e"])
                decl_base = s["ty"].split("<", 1)[0]
                if v.get("k") in ("enum", "enumv") and decl_base in ("Result", "Option"):
                    v = dict(v)
                    v["type_args"] = _named_args(s["ty"], decl_base)
                if v["k"] in INT_LL or v["k"] == "i1":   # i1: mutable Bool locals [OWN-1 amendment]
                    slot = g.stack_slot(v["k"])
                    g.emit(f"  store {v['k']} {v['v']}, ptr {slot}")
                    g.env[s["n"]] = {"k": "slot", "v": slot, "ty": v["k"],
                                     "signed": v.get("signed", True)}
                elif v["k"] == "struct":               # a struct binding is an addressable own local
                    if v.get("slot"):                  # construct already made the slot -> reuse it
                        g.env[s["n"]] = v
                    else:                              # call result (aggregate) -> spill to a slot
                        slot = g.stack_slot(f"%{v['st']}")
                        g.emit(f"  store %{v['st']} {v['v']}, ptr {slot}")
                        g.env[s["n"]] = {"k": "struct", "v": slot, "st": v["st"], "slot": True}
                elif v["k"] == "enum":                 # an enum binding is an addressable own local
                    if v.get("slot"):                  # construct already made the slot -> reuse it
                        g.env[s["n"]] = v
                    else:                              # call result (aggregate) -> spill to a slot
                        slot = g.stack_slot(f"%{v['en']}")
                        g.emit(f"  store %{v['en']} {v['v']}, ptr {slot}")
                        g.env[s["n"]] = {"k": "enum", "v": slot, "en": v["en"], "slot": True,
                                         "type_args": v.get("type_args")}
                else: g.env[s["n"]] = v
            elif k == "give":
                v = g.expr(s["e"])
                if v["k"] == "unit":
                    continue
                if v["k"] == "enumv":
                    value = g.pack_enumv(v)
                elif v["k"] == "enum":
                    value = g.load_enum(v)
                elif v["k"] == "struct":
                    value = g.load_struct(v)
                elif v["k"] == "buffer":
                    p0 = g.tmp(); g.emit(f"  {p0} = insertvalue {{ptr, i64}} undef, ptr {v['ptr']}, 0")
                    value = g.tmp(); g.emit(f"  {value} = insertvalue {{ptr, i64}} {p0}, i64 {v['len']}, 1")
                else:
                    value = v["v"]
                g.emit(f"  store {g.give_ty} {value}, ptr {g.give_slot}")
            elif k == "try":                           # [ERR-3] -> match{Ok bind; Err re-return}
                g.stmts([{"s": "match", "scrut": s["e"], "arms": [
                    {"v": "Ok", "b": [{"field": "value", "name": s["n"]}], "body": []},
                    {"v": "Err", "b": [{"field": "error", "name": f"__terr_{g.n}"}],
                     "body": [{"s": "return", "e": {"e": "construct", "n": "Err",
                        "fields": [{"name": "error", "atom": {"e": "place",
                        "p": {"base": f"__terr_{g.n}", "path": [], "deref": 0}}}]}}]}]}])
            elif k == "set":
                pl = s["p"]
                if pl.get("index"):                    # bounds-checked element write [OP-4]
                    v = g.expr(s["e"])
                    ptr, ell, _sg, md = g.index_addr(pl)
                    g.emit(f"  store {ell} {v['v']}, ptr {ptr}{md}")
                elif pl.get("path") or pl.get("post"): # set struct field: GEP + store [TYPE-5]
                    v = g.expr(s["e"])
                    ptr, fll, _fs, sub, _tyn = g.place_addr(pl)
                    if v["k"] == "struct":
                        g.emit(f"  store %{sub} {g.load_struct(v)}, ptr {ptr}")
                    elif v["k"] == "buffer":           # replace a buffer field: pack {ptr, i64}
                        t0 = g.tmp(); g.emit(f"  {t0} = insertvalue {{ptr, i64}} undef, ptr {v['ptr']}, 0")
                        t1 = g.tmp(); g.emit(f"  {t1} = insertvalue {{ptr, i64}} {t0}, i64 {v['len']}, 1")
                        g.emit(f"  store {{ptr, i64}} {t1}, ptr {ptr}{g.scope_suffix(pl, hdr=True)}")
                    else:
                        g.emit(f"  store {fll} {v['v']}, ptr {ptr}{g.scope_suffix(pl, hdr=True)}")
                else:
                    v = g.expr(s["e"]); tgt = g.env[pl["base"]]
                    assert tgt["k"] in ("ptr", "slot"), "set target must be param ptr or own local"
                    g.emit(f"  store {tgt.get('ty', 'i32')} {v['v']}, ptr {tgt['v']}")
            elif k == "return":
                v = g.expr(s["e"])
                if v.get("k") == "buffer":             # pack {ptr, i64} aggregate return
                    t0 = g.tmp(); g.emit(f"  {t0} = insertvalue {{ptr, i64}} undef, ptr {v['ptr']}, 0")
                    t1 = g.tmp(); g.emit(f"  {t1} = insertvalue {{ptr, i64}} {t0}, i64 {v['len']}, 1")
                    g.emit(f"  ret {{ptr, i64}} {t1}"); g.term = True; continue
                if g.f["name"] == "main": g.emit("  ret i32 0")
                elif v["k"] == "unit": g.emit("  ret void")
                elif v["k"] == "struct": g.emit(f"  ret {g.rllty} {g.load_struct(v)}")
                elif v["k"] == "enum": g.emit(f"  ret {g.rllty} {g.load_enum(v)}")
                elif v["k"] == "enumv": g.emit(f"  ret {g.rllty} {g.pack_enumv(v)}")
                else: g.emit(f"  ret {g.rllty} {v['v']}")
                g.term = True
            elif k == "region": g.stmts(s["body"])
            elif k == "loop":
                hd, end = g.lbl(), g.lbl()
                g.emit(f"  br label %{hd}"); g.emit(f"{hd}:")
                g.loopstk.append((s["l"], hd, end))
                g.stmts(s["body"])
                if not g.term: g.emit(f"  br label %{hd}")
                g.loopstk.pop()
                g.emit(f"{end}:"); g.term = False
            elif k == "break":
                for (lb, hd, end) in reversed(g.loopstk):
                    if lb == s["l"]:
                        g.emit(f"  br label %{end}"); g.term = True; break
            elif k == "check":
                g.traps = True
                c = g.expr(s["e"]); l = g.lbl()
                # [OP-5] An explicit source check executes in every build; it
                # is not an optimizer hint.  The volatile round-trip keeps the
                # observation and failure edge present even when a backend can
                # prove the source predicate tautological.  Implicit OP-4
                # checks remain eligible for deterministic proof elision.
                cs = g.stack_slot("i1")
                g.emit(f"  store volatile i1 {c['v']}, ptr {cs}")
                observed = g.tmp(); g.emit(f"  {observed} = load volatile i1, ptr {cs}")
                g.emit(f"  br i1 {observed}, label %{l}, label %trap")
                g.emit(f"{l}:")
            elif k == "match":
                g.gen_match(s)
            else: g.expr(s["e"])
    def gen_match(g, s):
        # One dispatcher over the unified {tag, payload} enum model. Scrutinee forms:
        #   {"k":"i1"}   Bool  -- degenerate 2-variant tag-only enum (special i1 fast path)
        #   {"k":"i32"}  tag-only user enum       (tag is the bare i32)
        #   {"k":"enum"} in-memory payload enum    (aggregate %E = { i32, iN } in a slot)
        #   {"k":"enumv"} SSA enum (prelude Ok/Err, Some/None, checked-op Result)
        sc = g.expr(s["scrut"])
        have = {a["v"] for a in s["arms"]}
        if sc["k"] == "i1":
            first = s["arms"][0]["v"]
            pair = ("False", "True") if first in ("True", "False") \
                else _TAGONLY2[g.venum[first][0]]
            need = set(pair)
        else:                                          # every other form dispatches on the registry
            need = set(g.evariants[g.venum[s["arms"][0]["v"]][0]])
        if have != need:
            raise CheckError("ERR-2",
                f"non-exhaustive match: have {sorted(have)}, need {sorted(need)}")
        done = g.lbl(); any_open = False
        if sc["k"] == "i1":                            # Bool: 2-way branch on the value itself
            lt, lf = g.lbl(), g.lbl()
            g.emit(f"  br i1 {sc['v']}, label %{lt}, label %{lf}")
            for a in s["arms"]:
                l = lt if a["v"] == pair[1] else lf
                g.emit(f"{l}:"); g.term = False; g.stmts(a["body"])
                if not g.term: g.emit(f"  br label %{done}"); any_open = True
        else:
            kind = sc["k"]
            if kind == "enum":                         # load the tag word out of the aggregate
                tp = g.tmp(); g.emit(f"  {tp} = getelementptr %{sc['en']}, ptr {sc['v']}, i32 0, i32 0")
                tag_ssa = g.tmp(); g.emit(f"  {tag_ssa} = load i32, ptr {tp}"); tty = "i32"
            else:                                      # "i32" tag-only, or "enumv" SSA
                tag_ssa = sc["v"] if kind == "i32" else sc["tag"]
                tty = "i32" if kind == "i32" else sc["tty"]
            nxt = None
            for a in s["arms"]:
                if nxt: g.emit(f"{nxt}:")
                idx = g.venum[a["v"]][1]; la = g.lbl(); nxt = g.lbl()
                c = g.tmp(); g.emit(f"  {c} = icmp eq {tty} {tag_ssa}, {idx}")
                g.emit(f"  br i1 {c}, label %{la}, label %{nxt}")
                g.emit(f"{la}:"); g.term = False
                if a["b"]:                             # bind the variant's payload (copy) [GRAM-10]
                    nm = a["b"][0]["name"]
                    if kind == "enum":                 # extract from the aggregate at the field's type
                        fll, fsigned = g.variant_field_ll(
                            sc["en"], a["v"], sc.get("type_args"))
                        payll = g.enum_payll(sc["en"])
                        pp = g.tmp(); g.emit(f"  {pp} = getelementptr %{sc['en']}, ptr {sc['v']}, i32 0, i32 1")
                        raw = g.tmp(); g.emit(f"  {raw} = load {payll}, ptr {pp}")
                        g.env[nm] = {"k": fll, "v": g.coerce_int(raw, payll, fll), "signed": fsigned}
                    elif idx == sc["payidx"]:          # SSA enum: the carried payload belongs to this variant
                        g.env[nm] = {"k": sc["pty"], "v": sc["pay"], "signed": sc["psigned"]}
                    else:                              # other variant (e.g. Err's nullary error): no payload
                        g.env[nm] = {"k": "i32", "v": "0", "signed": True}
                g.stmts(a["body"])
                if not g.term: g.emit(f"  br label %{done}"); any_open = True
            g.emit(f"{nxt}:"); g.emit("  unreachable")   # exhaustive [ERR-2]
        g.term = not any_open
        if any_open: g.emit(f"{done}:")
    def md(g, mk):                                 # allocate one module metadata node; mk: id -> text
        i = g.mdctr[0]; g.mdctr[0] += 1
        g.mdefs.append(f"!{i} = {mk(i)}"); return i
    def build_scopes(g):
        """[F003 channel] one alias scope per provenance class visible at the boundary:
        each &uniq-struct param root (struct memory) and each of its buffer fields
        (element memory), each own-buffer param, and ONE class for everything reached
        through shared borrows. Facts the checker proved (OWN-1 affine disjointness,
        OWN-2/5 exclusivity, T-A singleton rooted lineages) — rustc has no source channel for these
        on loaded pointers, and LLVM cannot recover them from an opaque boundary."""
        eligible = []
        for q in g.f["params"]:
            ty0 = q["ty"].split("<")[0]
            if q["mode"]["kind"] == "ref":
                st = q["ty"] if q["ty"] in g.structs else None
                if not q["mode"]["uniq"]:
                    g.pmode[q["name"]] = "shared"
                    if st and "__shared" not in eligible: eligible.append("__shared")
                else:
                    g.pmode[q["name"]] = "uniq"
                    if st:
                        eligible.append((q["name"], ()))
                        for fld in g.structs[st]:
                            if _buf_elem(fld["ty"]): eligible.append((q["name"], (fld["name"],)))
            elif q["mode"]["kind"] == "own" and _buf_elem(q["ty"]):
                g.pmode[q["name"]] = "own"
                eligible.append((q["name"], ()))
        if not g.alias or g.mdefs is None or len(eligible) < 2: return
        fname = g.f["name"]
        dom = g.md(lambda i: f'distinct !{{!{i}, !"{fname}"}}')
        sid = {}
        for key in eligible:
            nm = key if key == "__shared" else ".".join((key[0],) + key[1])
            sid[key] = g.md(lambda i, nm=nm: f'distinct !{{!{i}, !{dom}, !"{fname}.{nm}"}}')
        for key in eligible:
            own = g.md(lambda i, k=key: "!{" + f"!{sid[k]}" + "}")
            others = ", ".join(f"!{sid[k]}" for k in eligible if k != key)
            noal = g.md(lambda i, o=others: "!{" + o + "}")
            g.scopes[key] = (own, noal)
    def run(g):
        ps = []
        g.build_scopes()
        for q in g.f["params"]:
            if q["mode"]["kind"] == "own" and (q["ty"].split("<")[0] in g.structs
                                               or q["ty"].split("<")[0] in g.payenums
                                               or q["ty"].split("<")[0] in ("Result", "Option")):
                s = q["ty"].split("<")[0]; ps.append(f"%{s} %{q['name']}")   # own aggregate param by value
                slot = g.stack_slot(f"%{s}")                  # spill for field/tag access
                g.prologue.append(f"  store %{s} %{q['name']}, ptr {slot}")
                if s in g.structs:
                    g.env[q["name"]] = {"k": "struct", "v": slot, "st": s, "slot": True}
                else:
                    type_args = _named_args(q["ty"], s) if s in ("Result", "Option") else None
                    g.env[q["name"]] = {"k": "enum", "v": slot, "en": s, "slot": True,
                                         "type_args": type_args}
                continue
            pll = g.llty(q["ty"]); psigned = _is_signed(q["ty"])
            if q["mode"]["kind"] == "own":
                el = _buf_elem(q["ty"])
                if el:                                 # buffer param: {ptr, i64} unpacked
                    ps.append(f"{{ptr, i64}} %{q['name']}")
                    bp = g.tmp(); g.prologue.append(f"  {bp} = extractvalue {{ptr, i64}} %{q['name']}, 0")
                    bl = g.tmp(); g.prologue.append(f"  {bl} = extractvalue {{ptr, i64}} %{q['name']}, 1")
                    g.env[q["name"]] = {"k": "buffer", "ptr": bp, "len": bl,
                                        "ell": _llty(el), "esigned": _is_signed(el)}
                    continue
                ps.append(f"{pll} %{q['name']}")
                g.env[q["name"]] = {"k": pll, "v": f"%{q['name']}", "signed": psigned}
            else:
                el = _buf_elem(q["ty"])
                if el:                                 # &/&uniq buffer param: {ptr,i64} by value (element
                    ps.append(f"{{ptr, i64}} %{q['name']}")   # writes go through the shared data pointer,
                    bp = g.tmp(); g.prologue.append(f"  {bp} = extractvalue {{ptr, i64}} %{q['name']}, 0")  # caller-visible; exclusivity is the checker's job
                    bl = g.tmp(); g.prologue.append(f"  {bl} = extractvalue {{ptr, i64}} %{q['name']}, 1")
                    g.env[q["name"]] = {"k": "buffer", "ptr": bp, "len": bl,
                                        "ell": _llty(el), "esigned": _is_signed(el)}
                    continue
                at = (" noalias" + ("" if q["mode"]["uniq"] else " readonly")) if g.alias else ""
                if g.alias:
                    sa = _size_align(q["ty"], g.structs)   # borrows are always valid+aligned (checker fact)
                    if sa: at += f" dereferenceable({sa[0]}) align {sa[1]}"
                ps.append(f"ptr{at} %{q['name']}")
                base = q["ty"].split("<", 1)[0]
                st = q["ty"] if q["ty"] in g.structs else None
                en = base if base in g.payenums or base in ("Result", "Option") else None
                type_args = _named_args(q["ty"], base) if base in ("Result", "Option") else None
                g.env[q["name"]] = {"k": "ptr", "v": f"%{q['name']}", "ty": pll,
                                    "signed": psigned, "st": st, "en": en,
                                    "type_args": type_args}
        rt = ("i32" if g.f["name"] == "main"
              else ("void" if g.f["rty"] == "unit" else g.llty(g.f["rty"])))
        g.rllty = rt
        # [EFF->attrs channel] declared+checked effect rows lower to LLVM
        # function attributes — facts rustc has no source channel for.
        attrs = ""
        if g.alias:
            eff = g.f.get("effects", [])
            has = lambda w: w in eff
            refp = any(q["mode"]["kind"] == "ref" for q in g.f["params"])
            mem = None
            if eff == ["pure"]:
                mem = "memory(argmem: read)" if refp else "memory(none)"
            elif not has("allocates"):
                parts = []
                if has("writes"): parts.append("argmem: readwrite")
                elif has("reads") or refp: parts.append("argmem: read")
                if has("traps"): parts.append("inaccessiblemem: write")
                if parts: mem = f"memory({', '.join(parts)})"
            attrs = " nounwind" + (" willreturn" if g.f["name"] in g.total else "") + ((" " + mem) if mem else "")
        g.emit(f"define {rt} @{g.f['name']}({', '.join(ps)}){attrs} {{")
        g.emit("entry:")
        entry_alloca_index = len(g.lines)
        for line in g.prologue: g.emit(line)
        # [FN-8] Checked preconditions are enforced at the callee boundary,
        # after parameter/header unpack and before any body effect.  Clause
        # locals are signature-local, so restore the body environment after
        # emitting the guard.  The check itself always remains (OP-5); only
        # downstream operations may be discharged by proof.
        body_env = dict(g.env)
        g.stmts(g.f.get("requires") or [])
        g.env = body_env
        g.stmts(g.f["body"])
        if not g.term:
            g.emit("  ret i32 0" if g.f["name"] == "main" else ("  ret void" if rt == "void" else "  unreachable"))
        if g.traps: g.emit("trap:\n  call void @llvm.trap()\n  unreachable")
        g.emit("}")
        g.lines[entry_alloca_index:entry_alloca_index] = g.entry_allocas
        return "\n".join(g.lines) + "\n"

# ---- [FN-4] checked-law channel: static discharge + reduction reassociation ----
# A stated law becomes an optimizer-usable fact ONLY when discharged. The demo's
# static prover accepts exactly one shape: the bound fn's body is a single table
# op whose law is OP-8 table data. Anything else is a hard reject (OWN-8 posture).
_LAW_TABLE = {
    # op -> (associative-for, commutative-for, identity value); "u" = unsigned T only
    "iadd.wrap": ("all", "all", 0), "imul.wrap": ("all", "all", 1),
    "iand": ("all", "all", None), "ior": ("all", "all", 0), "ixor": ("all", "all", 0),
    "imin": ("all", "all", None), "imax": ("u", "u", 0),
    "iadd.sat": ("u", "all", 0),   # signed sat-add is NOT associative: (MAX+1)+(-1) != MAX+(1+(-1))
}

def _single_op_body(f):
    """(opname, T) if f's body is exactly `return op<T>(param0, param1);`, else None."""
    if f.get("requires"):
        return None                                    # a partial-domain fn cannot witness a universal law [FN-4]
    body = [s for s in f["body"] if s.get("s") != "doc"]
    if len(body) != 1 or body[0].get("s") != "return": return None
    e = body[0]["e"]
    if e.get("e") != "op" or len(e.get("args", [])) != 2: return None
    ps = [q["name"] for q in f["params"]]
    if len(ps) != 2: return None
    for a, want in zip(e["args"], ps):
        if a.get("e") != "place" or a["p"]["base"] != want or a["p"].get("path") or a["p"].get("deref"):
            return None
    return e["op"], (e.get("tyargs") or ["i32"])[0]

def discharge_laws(contracts, conforms, fns):
    """Validate FN-3 bindings and statically discharge FN-4 laws.
    Returns {fn_name: {"op", "T", "laws": set, "identity": lit-expr}}."""
    fbyn = {f["name"]: f for f in fns}
    proved = {}
    seen_conformance = set()

    def mode_shape(mode):
        return (mode.get("kind"), mode.get("region"), bool(mode.get("uniq")))

    def signature_shape(sig):
        """FN-1/FN-3 exact public signature identity (no refinement tier)."""
        return (
            tuple(sig.get("regions", [])),
            tuple((q["name"], mode_shape(q["mode"]), q["ty"])
                  for q in sig.get("params", [])),
            mode_shape(sig.get("rmode", {})), sig.get("rty"),
            tuple(sig.get("effects", [])),
        )

    for cf in conforms:
        ckey = (cf["ty"], cf["contract"])
        if ckey in seen_conformance:
            raise CheckError("FN-3",
                f"duplicate conformance for ({cf['ty']}, {cf['contract']}); at most one is permitted")
        seen_conformance.add(ckey)
        c = contracts.get(cf["contract"])
        if c is None:
            raise CheckError("FN-3", f"conform names unknown contract {cf['contract']}")
        for member, sig in c["fns"].items():
            if member not in cf["binds"]:
                raise CheckError("FN-3", f"conform {cf['ty']}: {cf['contract']} does not bind member {member}")
            target = cf["binds"][member]
            f = fbyn.get(target)
            if f is None:
                raise CheckError("FN-3", f"conform binds {member} = {target}, but no fn {target} exists")
            if f.get("requires"):
                raise CheckError("FN-3",
                    f"conform member {member} = {target}: concrete requires is not part of fn_sig; "
                    "contract/refinement binding is deferred [FN-8]")
            if signature_shape(f) != signature_shape(sig):
                raise CheckError("FN-3",
                    f"conform member {member} = {target}: complete signature "
                    "(regions, parameter names/modes/types, return mode/type, effects) "
                    "does not exactly match the contract")
        for law in c["laws"]:
            target = cf["binds"][law["fn"]]
            f = fbyn[target]
            ot = _single_op_body(f)
            if ot is None:
                raise CheckError("FN-4", f"law {law['law']}({law['fn']}) is stated but not discharged: "
                                 f"{target}'s body is not a single table op (the demo's only static-proof shape)")
            opname, T = ot
            row = _LAW_TABLE.get(opname)
            unsigned = not _is_signed(T)
            def _holds(scope): return scope == "all" or (scope == "u" and unsigned)
            if law["law"] == "associative":
                if row is None or not _holds(row[0]):
                    raise CheckError("FN-4", f"law associative({law['fn']}) REFUTED for {opname}<{T}>: "
                                     "not in the checked associativity table for this signedness")
            elif law["law"] == "commutative":
                if row is None or not _holds(row[1]):
                    raise CheckError("FN-4", f"law commutative({law['fn']}) REFUTED for {opname}<{T}>")
            else:                                      # identity(f, e)
                e = law["e"]
                if row is None or row[2] is None or e.get("e") != "lit" \
                        or e["ty"] != T or e["v"] != row[2]:
                    raise CheckError("FN-4", f"law identity({law['fn']}, ...) is stated but not discharged "
                                     f"for {opname}<{T}>")
            d = proved.setdefault(target, {"op": opname, "T": T, "laws": set(), "identity": None})
            d["laws"].add(law["law"])
            if law["law"] == "identity": d["identity"] = law["e"]
    return proved

def _mkop(op, T, args): return {"e": "op", "op": op, "args": args, "tyargs": [T]}
def _pl(name): return {"e": "place", "p": {"base": name, "path": [], "deref": 0}}
def _lit(v, T): return {"e": "lit", "v": v, "ty": T}
def _let(n, e): return {"s": "let", "n": n, "m": None, "ty": "u64", "e": e}
def _set(n, e): return {"s": "set", "p": {"base": n, "path": [], "deref": 0}, "e": e}

def _reduction_shape(st, proved):
    """Match democ's canonical reduction loop; return (i, m, acc, elem_let, f) or None.
    loop @l { match ige<u64>(i, m) {True=>{break}, False=>{}};
              let x: T = index<T>(buf, i); set acc = f(acc, x); set i = i + 1 }"""
    if st.get("s") != "loop": return None
    b = [s for s in st["body"] if s.get("s") != "doc"]
    if len(b) != 4 or b[0].get("s") != "match": return None
    sc = b[0]["scrut"]
    if sc.get("e") != "op" or sc["op"] != "ige" or (sc.get("tyargs") or [None])[0] != "u64": return None
    if any(a.get("e") != "place" or a["p"].get("path") or a["p"].get("deref") for a in sc["args"]): return None
    iv, mv = sc["args"][0]["p"]["base"], sc["args"][1]["p"]["base"]
    arms = {a["v"]: a for a in b[0]["arms"]}
    if set(arms) != {"True", "False"} or arms["False"]["body"]: return None
    tb = [s for s in arms["True"]["body"] if s.get("s") != "doc"]
    if len(tb) != 1 or tb[0].get("s") != "break": return None
    if b[1].get("s") != "let" or "e" not in b[1]: return None
    le = b[1]["e"]
    if le.get("e") != "place" or not le["p"].get("index"): return None
    ix = le["p"]["index"]
    if ix["atom"].get("e") != "place" or ix["atom"]["p"]["base"] != iv: return None
    xn = b[1]["n"]
    if b[2].get("s") != "set" or b[2]["p"]["base"] is None or b[2]["p"].get("path"): return None
    accn = b[2]["p"]["base"]
    ce = b[2]["e"]
    if ce.get("e") != "ucall" or len(ce["args"]) != 2: return None
    a0, a1 = ce["args"]
    if not (a0.get("e") == "place" and a0["p"]["base"] == accn
            and a1.get("e") == "place" and a1["p"]["base"] == xn): return None
    fn = ce["n"]
    d = proved.get(fn)
    if d is None or not {"associative", "commutative", "identity"} <= d["laws"]: return None
    if b[3].get("s") != "set" or b[3]["p"]["base"] != iv: return None
    inc = b[3]["e"]
    if inc.get("e") != "op" or inc["op"].split(".")[0] != "iadd": return None
    if not (inc["args"][0].get("e") == "place" and inc["args"][0]["p"]["base"] == iv
            and inc["args"][1].get("e") == "lit" and inc["args"][1]["v"] == 1): return None
    return iv, mv, accn, b[1], d


# ---- [OP-4] PROOF-1: structural in-bounds prover ---------------------------
# Marks index places whose bounds check is discharged by a dominating guard
# (pattern A) or a power-of-two mask (pattern C). Deliberately conservative:
# only the exact blessed idioms qualify; anything else keeps its check.
# Soundness rests on three facts: buffer lengths are immutable after
# allocation; there is no shadowing (TYPE-6), so a name means one binding per
# fn; and elision is only applied where the guard's fall-through DOMINATES the
# access with no intervening write to the index, bound, or buffer name.

def _mutates_name(st, name):
    """does this statement (recursively) mutate an existing binding?

    Unlike _writes_name this deliberately ignores the binding's establishing
    let/try.  TYPE-6 forbids a second binding with the same name, so subsequent
    changes can only be direct sets (or moves, handled separately).
    """
    if st.get("s") == "set" and not st["p"].get("path") and not st["p"].get("post") \
            and not st["p"].get("index") and st["p"]["base"] == name:
        return True
    if "body" in st and any(_mutates_name(x, name) for x in st["body"]):
        return True
    if st.get("s") == "match" and any(
            _mutates_name(x, name) for a in st["arms"] for x in a["body"]):
        return True
    if "match" in st and any(
            _mutates_name(x, name) for a in st["match"]["arms"] for x in a["body"]):
        return True
    return False

def _moved_or_reset(body, name):
    """Conservative: any move, mutation, or uniq alias of `name` anywhere.

    A write through a uniq-borrow holder does not syntactically name its
    referent.  Treating creation of that alias as invalidation keeps proof
    facts sound without duplicating the ownership checker's holder-resolution
    machinery here.
    """
    # Passing an existing reference/buffer dependency to a user function can
    # mutate or replace the referent under that callee's declared authority.
    # The current proof tier is deliberately intraprocedural: do not try to
    # summarize callees, even when today's codegen lacks one replacement form.
    if _uniq_borrows_name(body, name) or _user_call_mentions_name(body, name):
        return True
    for st in body:
        if _mutates_name(st, name):
            return True
        for e in _exprs_of(st):
            if _expr_moves(e, name):
                return True
    return False

def _exprs_of(st):
    out = []
    for k in ("e", "expr", "init", "scrut"):
        if k in st and isinstance(st[k], dict):
            out.append(st[k])
    if "match" in st:
        out.append(st["match"]["scrut"])
        for a in st["match"]["arms"]:
            for x in a["body"]: out.extend(_exprs_of(x))
    if st.get("s") == "match":
        for a in st["arms"]:
            for x in a["body"]: out.extend(_exprs_of(x))
    if "body" in st:
        for x in st["body"]: out.extend(_exprs_of(x))
    return out

def _direct_exprs_of(st):
    """Expressions evaluated by `st` itself, excluding child statement lists."""
    out = []
    for k in ("e", "expr", "init", "scrut"):
        if k in st and isinstance(st[k], dict):
            out.append(st[k])
    if "match" in st:
        out.append(st["match"]["scrut"])
    return out

def _expr_moves(e, name):
    if isinstance(e, list):
        return any(_expr_moves(item, name) for item in e)
    if not isinstance(e, dict): return False
    if e.get("e") == "move" and e["p"]["base"] == name and not e["p"].get("path") \
            and not e["p"].get("post"):
        return True
    return any(_expr_moves(value, name) for value in e.values())

def _expr_mentions_name(e, name):
    if isinstance(e, list):
        return any(_expr_mentions_name(item, name) for item in e)
    if not isinstance(e, dict): return False
    if e.get("e") in ("place", "move") and e.get("p", {}).get("base") == name:
        return True
    if e.get("e") == "borrow" and e.get("place", {}).get("base") == name:
        return True
    return any(_expr_mentions_name(value, name) for value in e.values())

def _user_call_mentions_name(body, name):
    """Conservative interprocedural invalidation for proof dependencies."""
    for st in body:
        for e in _exprs_of(st):
            if e.get("e") == "ucall" and any(
                    _expr_mentions_name(arg, name) for arg in e.get("args", [])):
                return True
    return False

def _uniq_borrows_name(node, name):
    """Whether any uniq-borrow expression is rooted at `name`.

    The originating borrow is always present even when its holder is later
    dereferenced, passed, or reborrowed, so this deliberately conservative
    provenance test covers all writes through that alias chain.
    """
    if isinstance(node, list):
        return any(_uniq_borrows_name(item, name) for item in node)
    if not isinstance(node, dict): return False
    if node.get("e") == "borrow" and node.get("uniq"):
        place = node.get("place", {})
        if place.get("base") == name:
            return True
    return any(_uniq_borrows_name(value, name) for value in node.values())

def _borrows_name(node, name):
    """Whether any shared or uniq borrow expression is rooted at `name`."""
    if isinstance(node, list):
        return any(_borrows_name(item, name) for item in node)
    if not isinstance(node, dict): return False
    if node.get("e") == "borrow" and node.get("place", {}).get("base") == name:
        return True
    return any(_borrows_name(value, name) for value in node.values())

def _mark_indexes(stmts, bvar, ivar, stop_names):
    """mark index<T>(bvar, ivar) as proven in `stmts` until any stop-name is
    written; returns False when marking must stop for the caller too."""
    for st in stmts:
        # Mark only expressions evaluated before this statement's mutation or
        # control transfer.  Child bodies are traversed below in execution
        # order; recursively harvesting them here would prove post-mutation
        # accesses before seeing the mutation.
        for e in _direct_exprs_of(st):
            _mark_in_expr(e, bvar, ivar)
        if st.get("s") == "set" and st["p"].get("index"):
            _mark_place(st["p"], bvar, ivar)
        if st.get("s") == "match":
            if not all(_mark_indexes(a["body"], bvar, ivar, stop_names) for a in st["arms"]):
                return False
        if "match" in st:
            if not all(_mark_indexes(a["body"], bvar, ivar, stop_names)
                       for a in st["match"]["arms"]):
                return False
        if st.get("s") in ("loop", "region"):
            return False                       # nested control: stop (conservative)
        # At this point child paths have been handled.  A mutation in the
        # current statement ends the fact for all following statements.
        if any(st.get("s") == "set" and not st["p"].get("path")
               and not st["p"].get("post") and not st["p"].get("index")
               and st["p"]["base"] == nm for nm in stop_names):
            return False
    return True

def _mark_in_expr(e, bvar, ivar):
    if not isinstance(e, dict): return
    if e.get("e") in ("place", "move") and e.get("p", {}).get("index"):
        _mark_place(e["p"], bvar, ivar)
    for a in e.get("args", []):
        _mark_in_expr(a, bvar, ivar)

def _mark_place(pl, bvar, ivar):
    ix = pl["index"]; ipl = ix["place"]
    if ipl.get("path") or ipl.get("post") or ipl.get("deref"):
        return                                  # v1: direct bindings only
    at = ix["atom"]
    if ipl["base"] == bvar and at.get("e") == "place" and not at["p"].get("path") \
            and not at["p"].get("post") and at["p"]["base"] == ivar:
        ix["proof"] = "dominating-guard"

def _collect_direct_sets(stmts, name):
    out = []
    for st in stmts:
        if st.get("s") == "set" and not st["p"].get("path") and not st["p"].get("post") \
                and not st["p"].get("index") and st["p"]["base"] == name:
            out.append(st)
        if "body" in st:
            out.extend(_collect_direct_sets(st["body"], name))
        if st.get("s") == "match":
            for a in st["arms"]: out.extend(_collect_direct_sets(a["body"], name))
        if "match" in st:
            for a in st["match"]["arms"]: out.extend(_collect_direct_sets(a["body"], name))
    return out

def _direct_place_name(e):
    if not isinstance(e, dict) or e.get("e") != "place": return None
    p = e["p"]
    if p.get("path") or p.get("post") or p.get("deref") or p.get("index"): return None
    return p["base"]

def _bounded_offset_vars(stmts, ivar, limit, whole_body):
    """Direct i+d temporaries with 0 <= d < limit and no later mutation."""
    offsets = {ivar: 0}
    for st in stmts:
        e = st.get("e")
        if st.get("s") != "let" or not isinstance(e, dict) or e.get("e") != "op" \
                or e.get("op") != "iadd.wrap" or len(e.get("args", [])) != 2:
            continue
        base, amount = e["args"]
        if e.get("tyargs") != ["u64"]: continue         # WG-1: pin width — no narrow-wrap laundering
        if _direct_place_name(base) != ivar or amount.get("e") != "lit": continue
        if amount.get("ty") != "u64": continue
        d = amount["v"]
        if 0 <= d < limit and not _moved_or_reset(whole_body, st["n"]):
            offsets[st["n"]] = d
    return offsets

def _mark_offset_place(pl, bvar, offsets, reason):
    ix = pl["index"]; ipl = ix["place"]
    if ipl.get("path") or ipl.get("post") or ipl.get("deref") or ipl["base"] != bvar:
        return
    at = ix["atom"]
    name = _direct_place_name(at)
    if name in offsets:
        ix["proof"] = reason

def _mark_offset_expr(e, bvar, offsets, reason):
    if not isinstance(e, dict): return
    if e.get("e") in ("place", "move") and e.get("p", {}).get("index"):
        _mark_offset_place(e["p"], bvar, offsets, reason)
    for a in e.get("args", []): _mark_offset_expr(a, bvar, offsets, reason)

def _mark_offset_indexes(stmts, bvar, ivar, limit, whole_body, reason):
    offsets = _bounded_offset_vars(stmts, ivar, limit, whole_body)
    for st in stmts:
        for e in _direct_exprs_of(st):
            _mark_offset_expr(e, bvar, offsets, reason)
        if st.get("s") == "set" and st["p"].get("index"):
            _mark_offset_place(st["p"], bvar, offsets, reason)

def _plain_direct_place(e, deref=None):
    """Return the root name for a direct place atom, preserving deref identity."""
    if not isinstance(e, dict) or e.get("e") != "place": return None
    p = e.get("p", {})
    if p.get("path") or p.get("post") or p.get("index"): return None
    if deref is not None and p.get("deref", 0) != deref: return None
    return p.get("base")

def _proof_premise(reason, path, expected=None, observed=None):
    """Stable, machine-readable first-failure record for proof diagnostics."""
    out = {"reason": reason, "path": path}
    if expected is not None: out["expected"] = str(expected)
    if observed is not None: out["observed"] = str(observed)
    return out

def _requirement_mismatch(reason, path, expected=None, observed=None):
    return {"state": "mismatch", "fact": None,
            "first_failed_premise": _proof_premise(
                reason, path, expected=expected, observed=observed)}

def _normalize_capacity_requirement(f):
    """Normalize the one admitted PROOF-2 v1 checked predicate.

    Unlike the old dict-or-None recognizer, this function preserves why a
    present predicate failed.  It deliberately does not inspect the body.
    """
    req = f.get("requires") or []
    if not req:
        return {"state": "missing", "fact": None,
                "first_failed_premise": None}
    if len(req) != 5 or any(st.get("s") == "doc" for st in req):
        return _requirement_mismatch(
            "requirement.statement-shape", "requires",
            "four u64 lets followed by one check", f"{len(req)} statements")
    lets, final = req[:-1], req[-1]
    if any(st.get("s") != "let" for st in lets):
        return _requirement_mismatch(
            "requirement.statement-shape", "requires[0:4]", "let statements")
    if final.get("s") != "check":
        return _requirement_mismatch(
            "requirement.final-check", "requires[4]", "check", final.get("s"))
    if any(st.get("m", {}).get("kind") != "own" or st.get("ty") != "u64"
           for st in lets):
        return _requirement_mismatch(
            "requirement.let-types", "requires[0:4]", "own u64 bindings")
    defs = {st["n"]: st.get("e") for st in lets}
    if len(defs) != 4:
        return _requirement_mismatch(
            "requirement.statement-shape", "requires[0:4]", "four distinct bindings")

    cond = final.get("e")
    if not isinstance(cond, dict) or cond.get("e") != "op" \
            or cond.get("op") != "ile" or cond.get("tyargs") != ["u64"] \
            or len(cond.get("args", [])) != 2:
        actual = cond.get("op") if isinstance(cond, dict) else None
        return _requirement_mismatch(
            "requirement.comparator-ile", "requires[4].condition", "ile<u64>", actual)
    src_len = _direct_place_name(cond["args"][0])
    covered = _direct_place_name(cond["args"][1])
    src_expr, covered_expr = defs.get(src_len), defs.get(covered)
    if not isinstance(src_expr, dict) or src_expr.get("e") != "op" \
            or src_expr.get("op") != "len" or len(src_expr.get("args", [])) != 1:
        return _requirement_mismatch(
            "requirement.source-len", "requires.condition.left", "len(source)")
    src = _plain_direct_place(src_expr["args"][0], deref=0)
    if not src:
        return _requirement_mismatch(
            "requirement.source-len", "requires.condition.left", "direct owned buffer")

    if not isinstance(covered_expr, dict) or covered_expr.get("e") != "op" \
            or covered_expr.get("op") != "imul.wrap" \
            or covered_expr.get("tyargs") != ["u64"] \
            or len(covered_expr.get("args", [])) != 2:
        return _requirement_mismatch(
            "requirement.multiplier-three", "requires.covered", "groups * 3_u64")
    groups = _direct_place_name(covered_expr["args"][0])
    three = covered_expr["args"][1]
    if not isinstance(three, dict) or three.get("e") != "lit" \
            or three.get("ty") != "u64" or three.get("v") != 3:
        actual = three.get("v") if isinstance(three, dict) else None
        return _requirement_mismatch(
            "requirement.multiplier-three", "requires.covered.multiplier", 3, actual)

    group_expr = defs.get(groups)
    if not isinstance(group_expr, dict) or group_expr.get("e") != "op" \
            or group_expr.get("op") != "ishr.wrap" \
            or group_expr.get("tyargs") != ["u64"] \
            or len(group_expr.get("args", [])) != 2:
        return _requirement_mismatch(
            "requirement.output-shift-two", "requires.groups", "out_len >> 2_u32")
    out_len = _direct_place_name(group_expr["args"][0])
    shift = group_expr["args"][1]
    if not isinstance(shift, dict) or shift.get("e") != "lit" \
            or shift.get("ty") != "u32" or shift.get("v") != 2:
        actual = shift.get("v") if isinstance(shift, dict) else None
        return _requirement_mismatch(
            "requirement.output-shift-two", "requires.groups.shift", 2, actual)

    out_expr = defs.get(out_len)
    if not isinstance(out_expr, dict) or out_expr.get("e") != "op" \
            or out_expr.get("op") != "len" or len(out_expr.get("args", [])) != 1:
        return _requirement_mismatch(
            "requirement.output-len", "requires.groups.input", "len(deref(output))")
    out = _plain_direct_place(out_expr["args"][0], deref=1)
    if not out or src == out:
        return _requirement_mismatch(
            "requirement.output-len", "requires.groups.input", "distinct unique output")

    params = {p["name"]: p for p in f.get("params", [])}
    sp, op = params.get(src), params.get(out)
    if not sp or sp.get("mode", {}).get("kind") != "own" or not _buf_elem(sp.get("ty")):
        return _requirement_mismatch(
            "requirement.parameter-modes", f"parameter.{src}", "owned buffer")
    if not op or op.get("mode", {}).get("kind") != "ref" \
            or not op.get("mode", {}).get("uniq") or not _buf_elem(op.get("ty")):
        return _requirement_mismatch(
            "requirement.parameter-modes", f"parameter.{out}", "unique buffer reference")
    return {"state": "normalized", "fact": {"src": src, "out": out},
            "first_failed_premise": None}

def _capacity_requirement(f):
    """Compatibility wrapper for callers that need only an admitted fact."""
    normalized = _normalize_capacity_requirement(f)
    return normalized["fact"] if normalized["state"] == "normalized" else None

def _exact_direct_inc(st, name, amount):
    if st.get("s") != "set": return False
    p, e = st.get("p", {}), st.get("e", {})
    return (p.get("base") == name and not p.get("path") and not p.get("post")
            and not p.get("index") and not p.get("deref")
            and isinstance(e, dict) and e.get("e") == "op"
            and e.get("op") == "iadd.wrap" and e.get("tyargs") == ["u64"]
            and len(e.get("args", [])) == 2
            and _direct_place_name(e["args"][0]) == name
            and e["args"][1].get("e") == "lit"
            and e["args"][1].get("ty") == "u64"
            and e["args"][1].get("v") == amount)

def _linear_statements(stmts):
    """The v1 lockstep proof admits no nested control in a group body.
    WHITELIST: only statement kinds proven straight-line may appear (check and
    expr can abort but never skip forward); any new statement kind fails
    closed until it is audited for dominance."""
    return all(st.get("s") in ("doc", "let", "set", "check", "expr")
               and "match" not in st for st in stmts)

def _mark_capacity_place(pl, out, offsets, covered, apply_proof):
    ix = pl.get("index")
    if not ix: return
    target = ix.get("place", {})
    if target.get("base") != out or target.get("path") or target.get("post") \
            or target.get("deref", 0) != 1:
        return
    name = _direct_place_name(ix.get("atom"))
    if name in offsets:
        covered[id(ix)] = offsets[name]
        if apply_proof:
            ix["proof"] = "output-capacity-lockstep"

def _mark_capacity_indexes(stmts, out, ovar, apply_proof=True):
    # Scope the stability scan to this linear group/arm.  Bindings with the
    # same spelling in sibling match arms are distinct TYPE-6 environments.
    offsets = _bounded_offset_vars(stmts, ovar, 4, stmts)
    covered = {}
    for st in stmts:
        for e in _direct_exprs_of(st):
            if e.get("e") in ("place", "move") and e.get("p", {}).get("index"):
                _mark_capacity_place(e["p"], out, offsets, covered, apply_proof)
            for a in e.get("args", []):
                _mark_capacity_expr(a, out, offsets, covered, apply_proof)
        if st.get("s") == "set" and st.get("p", {}).get("index"):
            _mark_capacity_place(st["p"], out, offsets, covered, apply_proof)
    return covered

def _mark_capacity_expr(e, out, offsets, covered, apply_proof):
    if not isinstance(e, dict): return
    if e.get("e") in ("place", "move") and e.get("p", {}).get("index"):
        _mark_capacity_place(e["p"], out, offsets, covered, apply_proof)
    for value in e.values():
        if isinstance(value, dict):
            _mark_capacity_expr(value, out, offsets, covered, apply_proof)
        elif isinstance(value, list):
            for item in value:
                _mark_capacity_expr(item, out, offsets, covered, apply_proof)

def _indexed_access_nodes(node):
    """Return each indexed read or write once, in source order."""
    out, seen = [], set()
    def walk(value):
        if isinstance(value, list):
            for item in value: walk(item)
            return
        if not isinstance(value, dict): return
        # Context-independent place enumeration: every grammar form eventually
        # contains the same place dictionary, including reads, moves, writes,
        # borrows, calls, and future expression containers.
        ix = value.get("index")
        if isinstance(ix, dict) and "place" in ix and "atom" in ix:
            if id(ix) not in seen:
                seen.add(id(ix)); out.append(ix)
        for child in value.values(): walk(child)
    walk(node)
    return out

def _lowered_indexed_access_nodes(stmts):
    """Enumerate exactly the index origins Gen.stmts will attempt to lower.

    The parser/checker permits syntactically unreachable statements after a
    terminator, while Gen.stmts deliberately stops lowering them.  Coverage
    accounting must retain that distinction: every reachable final-AST origin
    emits once, and an unreachable origin emits zero records without making
    report collection reject an otherwise accepted program.
    """
    out, seen = [], set()

    def add(node):
        for ix in _indexed_access_nodes(node):
            if id(ix) not in seen:
                seen.add(id(ix)); out.append(ix)

    def walk_match(match):
        add(match.get("scrut"))
        terms = [walk_stmts(arm.get("body", [])) for arm in match.get("arms", [])]
        return bool(terms) and all(terms)

    def walk_stmts(body):
        terminated = False
        for stmt in body:
            if terminated: break
            kind = stmt.get("s")
            if kind == "doc":
                continue
            if kind == "let" and "match" in stmt:
                terminated = walk_match(stmt["match"])
            elif kind in {"let", "give", "try", "return", "check"}:
                add(stmt.get("e"))
                terminated = kind == "return"
            elif kind == "set":
                # Gen evaluates the value before computing an indexed address.
                add(stmt.get("e")); add(stmt.get("p"))
            elif kind == "region":
                terminated = walk_stmts(stmt.get("body", []))
            elif kind == "loop":
                walk_stmts(stmt.get("body", []))
                # Gen always emits the loop end block and resumes lowering,
                # even when it has no predecessor.
                terminated = False
            elif kind == "break":
                terminated = True
            elif kind == "match":
                terminated = walk_match(stmt)
            else:
                add(stmt.get("e"))
        return terminated

    walk_stmts(stmts)
    return out

def _direct_increment_name(st):
    """Name of a direct `x = x + literal` u64 increment, at any amount."""
    if st.get("s") != "set": return None
    p, e = st.get("p", {}), st.get("e", {})
    if p.get("path") or p.get("post") or p.get("index") or p.get("deref"):
        return None
    name = p.get("base")
    if not name or not isinstance(e, dict) or e.get("e") != "op" \
            or e.get("op") != "iadd.wrap" or e.get("tyargs") != ["u64"] \
            or len(e.get("args", [])) != 2 \
            or _direct_place_name(e["args"][0]) != name \
            or e["args"][1].get("e") != "lit" \
            or e["args"][1].get("ty") != "u64":
        return None
    return name

def _increment_names(node):
    names = set()
    def walk(value):
        if isinstance(value, list):
            for item in value: walk(item)
            return
        if not isinstance(value, dict): return
        name = _direct_increment_name(value)
        if name: names.add(name)
        for child in value.values(): walk(child)
    walk(node)
    return names

def _capacity_cursor_sites(stmts, ovar, params):
    """Lexically find direct unique-buffer accesses indexed by `o` or `o+d`.

    This deliberately ignores other cursors and other targets: they neither
    acquire this obligation nor poison a canonical lockstep region.
    """
    out, seen = [], set()
    def eligible(ix, names):
        name = _direct_place_name(ix.get("atom"))
        target = ix.get("place", {})
        root = target.get("base")
        p = params.get(root)
        return name in names and target.get("deref", 0) == 1 \
            and not target.get("path") and not target.get("post") and p \
            and p.get("mode", {}).get("kind") == "ref" \
            and p.get("mode", {}).get("uniq") and _buf_elem(p.get("ty"))
    def add_expr(e, names):
        if isinstance(e, list):
            for item in e: add_expr(item, names)
            return
        if not isinstance(e, dict): return
        if e.get("e") in ("place", "move") and e.get("p", {}).get("index"):
            ix = e["p"]["index"]
            if eligible(ix, names) and id(ix) not in seen:
                seen.add(id(ix)); out.append(ix)
        for child in e.values(): add_expr(child, names)
    def walk(seq, inherited):
        names = set(inherited)
        for st in seq:
            for e in _direct_exprs_of(st): add_expr(e, names)
            if st.get("s") == "set" and st.get("p", {}).get("index"):
                ix = st["p"]["index"]
                if eligible(ix, names) and id(ix) not in seen:
                    seen.add(id(ix)); out.append(ix)
            e = st.get("e")
            if st.get("s") == "let" and isinstance(e, dict) \
                    and e.get("e") == "op" and e.get("op") == "iadd.wrap" \
                    and e.get("tyargs") == ["u64"] and len(e.get("args", [])) == 2 \
                    and _direct_place_name(e["args"][0]) == ovar \
                    and e["args"][1].get("e") == "lit" \
                    and e["args"][1].get("ty") == "u64":
                names.add(st["n"])
            if st.get("s") in ("loop", "region"):
                walk(st.get("body", []), names)
            if st.get("s") == "match":
                for arm in st.get("arms", []): walk(arm.get("body", []), names)
            if "match" in st:
                for arm in st["match"].get("arms", []): walk(arm.get("body", []), names)
    walk(stmts, {ovar})
    return out

def _capacity_failure(writes, reason, path, expected=None, observed=None,
                      src=None, out=None):
    return {
        "state": "failed-premise", "sites": writes, "src": src, "out": out,
        "first_failed_premise": _proof_premise(
            reason, path, expected=expected, observed=observed),
    }

def _capacity_candidate_frontier(f):
    """Conservative per-site frontier for the registered capacity family.

    A non-literal index through any unique reference, or any non-literal indexed
    write at all—including cursor fields, aggregate paths, and moved owned
    buffers—may be a cursor-funded output site.
    Failure of the exact recognizer is therefore indeterminate, not affirmative
    not-applicability. Fixed-literal accesses and non-indexed control are
    explicitly outside this v1 family. False detention is intentional until a
    narrower negative proof is implemented.
    """
    roots = {
        p["name"] for p in f.get("params", [])
        if p.get("mode", {}).get("kind") == "ref"
        and p.get("mode", {}).get("uniq")
    }

    def discover(value):
        if isinstance(value, list):
            for item in value: discover(item)
            return
        if not isinstance(value, dict): return
        if value.get("s") == "let" and isinstance(value.get("m"), dict) \
                and value["m"].get("kind") == "ref" \
                and value["m"].get("uniq"):
            roots.add(value["n"])
        for child in value.values(): discover(child)

    discover(f["body"])

    write_sites = set()
    def collect_writes(value):
        if isinstance(value, list):
            for item in value: collect_writes(item)
            return
        if not isinstance(value, dict): return
        if value.get("s") == "set" and value.get("p", {}).get("index"):
            write_sites.add(id(value["p"]["index"]))
        if value.get("e") == "borrow" and value.get("uniq") \
                and value.get("place", {}).get("index"):
            write_sites.add(id(value["place"]["index"]))
        for child in value.values(): collect_writes(child)
    collect_writes(f["body"])

    frontier = set()
    for ix in _indexed_access_nodes(f["body"]):
        target = ix.get("place", {})
        base = target.get("base")
        through_unique = base in roots and target.get("deref", 0) == 1
        dynamic_write = id(ix) in write_sites
        if (through_unique or dynamic_write) \
                and ix.get("atom", {}).get("e") != "lit":
            frontier.add(id(ix))
    return frontier

def _analyze_capacity_lockstep_body(f):
    """Derive PROOF-2's candidate obligation from the body, never `requires`.

    This is intentionally the old proof's exact conjunction, made observable.
    A failed premise can only retain checks; it never licenses codegen.
    """
    body = f["body"]
    all_accesses = _indexed_access_nodes(body)
    if not all_accesses:
        return {"state": "not-applicable", "sites": [],
                "first_failed_premise": None}

    loops = [(pos, st) for pos, st in enumerate(body) if st.get("s") == "loop"]
    anchors = []
    for pos, loop in loops:
        b = [st for st in loop.get("body", []) if st.get("s") != "doc"]
        if len(b) >= 2 and b[0].get("s") == "let" and b[1].get("s") == "match":
            sub = b[0].get("e", {})
            if isinstance(sub, dict) and sub.get("e") == "op" \
                    and sub.get("op") == "isub.wrap":
                anchors.append((pos, loop, b))
    if not anchors:
        # A near-family must not become a B2 "no family applies" pass merely
        # by inserting one harmless statement before the canonical anchor.
        if len(loops) == 1:
            _pos, _loop = loops[0]
            candidate_body = [st for st in _loop.get("body", []) if st.get("s") != "doc"]
            rems = [st for st in candidate_body if st.get("s") == "let"
                    and isinstance(st.get("e"), dict)
                    and st["e"].get("e") == "op"
                    and st["e"].get("op") == "isub.wrap"
                    and len(st["e"].get("args", [])) == 2]
            if len(rems) == 1:
                _nvar, _ivar = map(_direct_place_name, rems[0]["e"]["args"])
                params = {p["name"]: p for p in f.get("params", [])}
                _ovar = _direct_increment_name(candidate_body[-1]) \
                    if candidate_body else None
                if _ivar and (not _ovar or _ovar == _ivar):
                    cursor_candidates = [
                        name for name in sorted(_increment_names(candidate_body) - {_ivar})
                        if _capacity_cursor_sites(body, name, params)
                    ]
                    _ovar = cursor_candidates[0] if len(cursor_candidates) == 1 else None
                sites = _capacity_cursor_sites(body, _ovar, params) if _ovar else []
                if sites:
                    return _capacity_failure(
                        sites, "body.supported-control", "loop.anchor-position",
                        "remainder then guard at loop entry")
        return {"state": "not-applicable", "sites": [],
                "first_failed_premise": None}
    if len(anchors) != 1 or len(loops) != 1:
        params = {p["name"]: p for p in f.get("params", [])}
        sites, seen = [], set()
        for _pos, _loop, candidate_body in anchors:
            args = candidate_body[0].get("e", {}).get("args", [])
            _ivar = _direct_place_name(args[1]) if len(args) == 2 else None
            _ovar = _direct_increment_name(candidate_body[-1]) \
                if candidate_body else None
            if _ivar and (not _ovar or _ovar == _ivar):
                cursor_candidates = [
                    name for name in sorted(_increment_names(candidate_body[2:]) - {_ivar})
                    if _capacity_cursor_sites(body, name, params)
                ]
                _ovar = cursor_candidates[0] if len(cursor_candidates) == 1 else None
            for ix in _capacity_cursor_sites(body, _ovar, params) if _ovar else []:
                if id(ix) not in seen:
                    seen.add(id(ix)); sites.append(ix)
        if sites:
            return _capacity_failure(
                sites, "body.supported-control", "body.loops", "one lockstep loop",
                f"{len(loops)} loops")
        return {"state": "not-applicable", "sites": [],
                "first_failed_premise": None}
    pos, loop, b = anchors[0]

    rem, sub = b[0]["n"], b[0].get("e")
    if sub.get("tyargs") != ["u64"] or len(sub.get("args", [])) != 2:
        return {"state": "not-applicable", "sites": [],
                "first_failed_premise": None}
    nvar, ivar = map(_direct_place_name, sub["args"])
    if not nvar or not ivar:
        return {"state": "not-applicable", "sites": [],
                "first_failed_premise": None}

    params = {p["name"]: p for p in f.get("params", [])}
    ovar = _direct_increment_name(b[-1]) if b else None
    if not ovar or ovar == ivar:
        candidates = [
            name for name in sorted(_increment_names(b[2:]) - {ivar})
            if _capacity_cursor_sites(body, name, params)
        ]
        ovar = candidates[0] if len(candidates) == 1 else None
    if not ovar:
        return {"state": "not-applicable", "sites": [],
                "first_failed_premise": None}
    writes = _capacity_cursor_sites(body, ovar, params)
    if not writes:
        return {"state": "not-applicable", "sites": [],
                "first_failed_premise": None}
    outputs = {ix.get("place", {}).get("base") for ix in writes}
    out = next(iter(outputs)) if len(outputs) == 1 else None

    ndefs = [st for st in body[:pos] if st.get("s") == "let" and st.get("n") == nvar]
    if len(ndefs) != 1:
        return _capacity_failure(
            writes, "body.source-length", f"body.binding.{nvar}", "one len(source) binding")
    ne = ndefs[0].get("e", {})
    src = None
    if ne.get("e") == "op" and ne.get("op") == "len" and len(ne.get("args", [])) == 1:
        src = _plain_direct_place(ne["args"][0], deref=0)
    if not src:
        return _capacity_failure(
            writes, "body.source-length", f"body.binding.{nvar}", "len(direct owned buffer)")

    sp = params.get(src)
    if not sp or sp.get("mode", {}).get("kind") != "own" or not _buf_elem(sp.get("ty")):
        return _capacity_failure(
            writes, "body.source-root", f"parameter.{src}", "owned buffer", src=src)

    guard, sc = b[1], b[1].get("scrut")
    if not isinstance(sc, dict) or sc.get("e") != "op" or sc.get("op") != "ilt" \
            or sc.get("tyargs") != ["u64"] or len(sc.get("args", [])) != 2 \
            or _direct_place_name(sc["args"][0]) != rem:
        return _capacity_failure(
            writes, "body.group-guard", "loop.guard", "ilt<u64>(remainder, 3)",
            src=src, out=out)
    stride_lit = sc["args"][1]
    if stride_lit.get("e") != "lit" or stride_lit.get("ty") != "u64" \
            or stride_lit.get("v") != 3:
        observed = stride_lit.get("v") if isinstance(stride_lit, dict) else None
        return _capacity_failure(
            writes, "body.guard-input-stride", "loop.guard.literal", 3, observed,
            src=src, out=out)
    arms = {a["v"]: a for a in guard.get("arms", [])}
    tguard = [x for x in arms.get("True", {}).get("body", []) if x.get("s") != "doc"]
    if set(arms) != {"True", "False"} or arms["False"]["body"] \
            or len(tguard) != 1 or tguard[0].get("s") != "break" \
            or tguard[0].get("l") != loop.get("l"):
        return _capacity_failure(
            writes, "body.group-guard", "loop.guard.arms", "true breaks; false falls through",
            src=src, out=out)

    def exact_base(name, value):
        return [st for st in body[:pos] if st.get("s") == "let" and st.get("n") == name
                and st.get("e", {}).get("e") == "lit" and st["e"].get("ty") == "u64"
                and st["e"].get("v") == value]
    if len(exact_base(ivar, 0)) != 1:
        return _capacity_failure(
            writes, "body.input-base", f"body.binding.{ivar}", "0_u64", src=src, out=out)
    if len(exact_base(ovar, 0)) != 1:
        return _capacity_failure(
            writes, "body.output-base", f"body.binding.{ovar}", "0_u64", src=src, out=out)

    group_i = _collect_direct_sets(b[2:], ivar)
    if len(group_i) != 1 or not _exact_direct_inc(group_i[0], ivar, 3):
        return _capacity_failure(
            writes, "body.input-stride", "loop.input-increment", "i += 3",
            src=src, out=out)
    group_o = _collect_direct_sets(b[2:], ovar)
    if len(group_o) != 1:
        return _capacity_failure(
            writes, "body.output-mutation", "loop.output-cursor", "one output increment",
            f"{len(group_o)} mutations", src=src, out=out)
    if not _exact_direct_inc(group_o[0], ovar, 4):
        return _capacity_failure(
            writes, "body.output-stride", "loop.output-increment", "o += 4",
            src=src, out=out)
    if len(b) < 4 or b[-2] is not group_i[0] or b[-1] is not group_o[0]:
        return _capacity_failure(
            writes, "body.increment-order", "loop.synchronization-point",
            "final i += 3 then o += 4", src=src, out=out)
    if not _linear_statements(b[2:]):
        unstable = [
            _direct_place_name(ix.get("atom")) for ix in writes
            if _direct_place_name(ix.get("atom")) not in (None, ovar)
            and _moved_or_reset(b[2:], _direct_place_name(ix.get("atom")))
        ]
        if unstable:
            return _capacity_failure(
                writes, "body.output-offset-stability",
                f"loop.binding.{sorted(set(unstable))[0]}", "stable funded offset",
                src=src, out=out)
        return _capacity_failure(
            writes, "body.supported-control", "loop.group-body", "straight-line group body",
            src=src, out=out)

    tail_seq = [st for st in body[pos + 1:] if st.get("s") != "doc"]
    if len(tail_seq) < 3:
        return _capacity_failure(
            writes, "body.tail-shape", "body.after-loop", "tail binding and cases 1,2",
            src=src, out=out)
    tail_let, tail_one, tail_two = tail_seq[:3]
    te = tail_let.get("e", {})
    if tail_let.get("s") != "let" or te.get("e") != "op" \
            or te.get("op") != "isub.wrap" or te.get("tyargs") != ["u64"] \
            or len(te.get("args", [])) != 2 \
            or _direct_place_name(te["args"][0]) != nvar \
            or _direct_place_name(te["args"][1]) != ivar:
        return _capacity_failure(
            writes, "body.tail-expression", "body.tail", "n - i", src=src, out=out)
    tail = tail_let["n"]
    if _collect_direct_sets(body[pos + 1:], tail):
        return _capacity_failure(
            writes, "body.tail-stability", f"body.binding.{tail}", "no mutation",
            src=src, out=out)

    def tail_arm(match, value):
        if match.get("s") != "match":
            return None, _proof_premise(
                "body.tail-case", f"tail[{value}]", f"tail == {value}")
        eq = match.get("scrut", {})
        lit = eq.get("args", [{}, {}])[1] if isinstance(eq, dict) \
            and len(eq.get("args", [])) == 2 else {}
        if eq.get("e") != "op" or eq.get("op") != "ieq" \
                or eq.get("tyargs") != ["u64"] or len(eq.get("args", [])) != 2 \
                or _direct_place_name(eq["args"][0]) != tail \
                or lit.get("e") != "lit" or lit.get("ty") != "u64" \
                or lit.get("v") != value:
            observed = lit.get("v") if isinstance(lit, dict) else None
            return None, _proof_premise(
                "body.tail-case", f"tail[{value}].condition", value, observed)
        aa = {a["v"]: a for a in match.get("arms", [])}
        false_body = [x for x in aa.get("False", {}).get("body", []) if x.get("s") != "doc"]
        true_body = [x for x in aa.get("True", {}).get("body", []) if x.get("s") != "doc"]
        if set(aa) != {"True", "False"} or not true_body:
            return None, _proof_premise(
                "body.tail-shape", f"tail[{value}].arms", "nonempty true; empty false")
        if false_body:
            reason = "body.output-mutation" if _collect_direct_sets(false_body, ovar) \
                else "body.tail-shape"
            return None, _proof_premise(
                reason, f"tail[{value}].false", "empty false arm")
        if not _linear_statements(true_body):
            return None, _proof_premise(
                "body.supported-control", f"tail[{value}].true", "straight-line body")
        if not _exact_direct_inc(true_body[-1], ovar, 4):
            return None, _proof_premise(
                "body.output-stride", f"tail[{value}].output-increment", "o += 4")
        return true_body, None

    one_body, failure = tail_arm(tail_one, 1)
    if failure is not None:
        return {"state": "failed-premise", "sites": writes, "src": src, "out": out,
                "first_failed_premise": failure}
    two_body, failure = tail_arm(tail_two, 2)
    if failure is not None:
        return {"state": "failed-premise", "sites": writes, "src": src, "out": out,
                "first_failed_premise": failure}

    approved_i = [group_i[0]]
    approved_o = [group_o[0], one_body[-1], two_body[-1]]
    if _collect_direct_sets(body, ivar) != approved_i:
        return _capacity_failure(
            writes, "body.input-mutation", "body.input-cursor", "one approved increment",
            src=src, out=out)
    if _collect_direct_sets(body, ovar) != approved_o:
        return _capacity_failure(
            writes, "body.output-mutation", "body.output-cursor", "three approved increments",
            src=src, out=out)
    if _collect_direct_sets(body, tail):
        return _capacity_failure(
            writes, "body.tail-stability", "body.tail", "no mutation", src=src, out=out)
    if _borrows_name(body, ovar) or _expr_moves(body, ovar):
        return _capacity_failure(
            writes, "body.output-cursor-stability", f"body.binding.{ovar}", "stable direct cursor",
            src=src, out=out)
    for name, reason in ((nvar, "body.source-length-stability"),
                         (ivar, "body.input-cursor-stability"),
                         (rem, "body.remainder-stability"),
                         (tail, "body.tail-stability")):
        reset_invalidates = name != ivar and _moved_or_reset(body, name)
        if _borrows_name(body, name) or _expr_moves(body, name) or reset_invalidates:
            return _capacity_failure(
                writes, reason, f"body.binding.{name}", "stable direct value",
                src=src, out=out)
    if _moved_or_reset(body, src):
        return _capacity_failure(
            writes, "body.source-stability", f"parameter.{src}", "stable source",
            src=src, out=out)
    output_failures = {
        root: _proof_premise(
            "body.output-stability", f"parameter.{root}", "stable output")
        for root in outputs if _moved_or_reset(body, root)
    }

    return {
        "state": "derived", "sites": writes, "src": src, "outputs": outputs,
        "output_failures": output_failures,
        "ivar": ivar, "ovar": ovar, "nvar": nvar, "rem": rem, "tail": tail,
        "regions": [b[2:-2], one_body[:-1], two_body[:-1]],
        "first_failed_premise": None,
    }

def _capacity_expr_total_for_exact(e, covered):
    """Closed whitelist for expressions that cannot preempt the modeled sites."""
    if not isinstance(e, dict): return False
    kind = e.get("e")
    if kind == "lit": return True
    if kind in ("place", "move"):
        p = e.get("p", {})
        if p.get("index"):
            return id(p["index"]) in covered \
                and _capacity_expr_total_for_exact(p["index"].get("atom"), covered)
        return not p.get("path") and not p.get("post")
    if kind == "op" and e.get("op") in {
            "len", "isub.wrap", "iadd.wrap", "ilt", "ieq"}:
        return all(_capacity_expr_total_for_exact(arg, covered)
                   for arg in e.get("args", []))
    return False

def _capacity_region_is_exact(region, coverage):
    """Closed-fragment coverage part of exactness for one emitted group."""
    return len(coverage) == 4 and set(coverage.values()) == {0, 1, 2, 3}

def _capacity_body_is_exact(body, covered):
    """Reject exact/minimal claims when any path may preempt a funded site."""
    if any(id(ix) not in covered for ix in _indexed_access_nodes(body)):
        return False
    def explicit_preemption(node):
        if isinstance(node, list):
            return any(explicit_preemption(item) for item in node)
        if not isinstance(node, dict): return False
        if node.get("s") in ("check", "expr"): return True
        return any(explicit_preemption(value) for value in node.values())
    if explicit_preemption(body): return False
    return all(_capacity_expr_total_for_exact(e, covered)
               for st in body for e in _exprs_of(st))

def _capacity_missing_fact(plan, out):
    return {
        "family": "output-capacity-lockstep",
        "source": plan["src"], "output": out,
        "input_stride": 3, "output_stride": 4,
        "condition": "len(source) <= 3 * floor(len(output) / 4)",
    }

def _obligation_analysis(completed=()):
    expected = [name for name, _analyzer in _OBLIGATION_ANALYZER_REGISTRY]
    completed = list(completed)
    complete = completed == expected
    return {
        "scope": _OBLIGATION_ANALYSIS_SCOPE if complete else None,
        "complete": complete,
        "analyzers": completed,
    }

def _not_applicable_obligation_info(completed):
    """Affirmative result after the complete v1 analyzer set examined a site."""
    return {
        "obligation_analysis": _obligation_analysis(completed),
        "obligation": None,
        "obligation_status": "not-applicable",
        "obligation_exactness": None,
        "requirement_relation": "not-applicable",
        "first_missing_fact": None,
        "first_failed_premise": None,
    }

def _indeterminate_obligation_info():
    """Fail-closed fallback for a lowered site absent from analyzer input."""
    return {
        "obligation_analysis": _obligation_analysis(),
        "obligation": None,
        "obligation_status": "unknown",
        "obligation_exactness": "unknown",
        "requirement_relation": "unknown",
        "first_missing_fact": None,
        "first_failed_premise": None,
    }

def _capacity_info(status, exactness=None, relation="unknown", missing=None, failed=None):
    return {
        # The orchestration layer stamps completion only after every analyzer
        # in the registry returns successfully.
        "obligation_analysis": _obligation_analysis(),
        "obligation": "output-capacity-lockstep",
        "obligation_status": status,
        "obligation_exactness": exactness,
        "requirement_relation": relation,
        "first_missing_fact": missing,
        "first_failed_premise": failed,
    }

def _analyze_capacity_obligation(f, apply_proofs, annotate=True):
    """Attach diagnostics and, only for the legacy exact match, proof markers."""
    plan = _analyze_capacity_lockstep_body(f)
    if plan["state"] == "not-applicable": return
    if plan["state"] == "failed-premise":
        info = _capacity_info(
            "failed-premise", exactness="unknown", relation="unknown",
            failed=plan["first_failed_premise"])
        if annotate:
            for ix in plan["sites"]: ix["obligation_info"] = info.copy()
        return

    requirement = _normalize_capacity_requirement(f)
    region_site_sets = [
        {id(ix) for ix in _indexed_access_nodes(region)} for region in plan["regions"]
    ]
    region_sites = set().union(*region_site_sets)
    for out in sorted(plan["outputs"]):
        sites = [ix for ix in plan["sites"] if ix.get("place", {}).get("base") == out]
        output_failure = plan["output_failures"].get(out)
        if output_failure is not None:
            if annotate:
                info = _capacity_info(
                    "failed-premise", exactness="unknown", relation="unknown",
                    failed=output_failure)
                for ix in sites: ix["obligation_info"] = info.copy()
            continue

        missing = None
        failed = requirement["first_failed_premise"]
        if requirement["state"] == "missing":
            relation = "missing"; missing = _capacity_missing_fact(plan, out)
        elif requirement["state"] == "mismatch":
            relation = "mismatch"; missing = _capacity_missing_fact(plan, out)
        else:
            fact = requirement["fact"]
            if fact["src"] != plan["src"]:
                relation = "mismatch"; missing = _capacity_missing_fact(plan, out)
                failed = _proof_premise(
                    "requirement.matches-body-source", "requires.source",
                    plan["src"], fact["src"])
            elif fact["out"] != out:
                relation = "mismatch"; missing = _capacity_missing_fact(plan, out)
                failed = _proof_premise(
                    "requirement.matches-body-output", "requires.output", out, fact["out"])
            else:
                relation = "equivalent"

        region_coverage = [
            _mark_capacity_indexes(region, out, plan["ovar"], apply_proof=False)
            for region in plan["regions"]
        ]
        covered = {}
        for region in region_coverage: covered.update(region)
        exact = all(_capacity_region_is_exact(region, coverage)
                    for region, coverage in zip(plan["regions"], region_coverage)) \
            and _capacity_body_is_exact(f["body"], covered)

        if relation == "equivalent" and apply_proofs:
            for region in plan["regions"]:
                _mark_capacity_indexes(region, out, plan["ovar"], apply_proof=True)

        if not annotate: continue
        for ix in sites:
            if id(ix) in covered:
                ix["obligation_info"] = _capacity_info(
                    "derived", exactness="exact" if exact else "sufficient",
                    relation=relation, missing=missing, failed=failed)
            else:
                name = _direct_place_name(ix.get("atom"))
                unstable = name not in (None, plan["ovar"]) and any(
                    id(ix) in site_ids and _moved_or_reset(region, name)
                    for region, site_ids in zip(plan["regions"], region_site_sets)
                )
                if unstable:
                    reason = "body.output-offset-stability"
                elif id(ix) not in region_sites:
                    reason = "body.output-site-unfunded"
                else:
                    reason = "body.output-offset-funded"
                ix["obligation_info"] = _capacity_info(
                    "failed-premise", exactness="unknown", relation="unknown",
                    failed=_proof_premise(
                        reason, "body.indexed-access",
                        "direct output offset in [0, 3]"))

def _prove_capacity_lockstep(f, fact=None):
    """Compatibility entry: body-first analysis owns legacy marker application."""
    _analyze_capacity_obligation(f, apply_proofs=True, annotate=False)

# The names used in proof reports are derived from the callables actually
# iterated here.  Adding an analyzer therefore cannot silently claim complete
# coverage without also registering and executing it.
_OBLIGATION_ANALYZER_REGISTRY = (
    ("output-capacity-lockstep-v1", _analyze_capacity_obligation),
)

def _prove_remainder_loops(stmts, lens, whole_body):
    """Pattern B: prove an exact fixed-stride remainder loop.

    Base i=0 and the sole mutation i=i+K establish i<=len(buf) inductively.
    On the fall-through of (len-i)<K, offsets 0..K-1 are in bounds.  The same
    invariant licenses exact tail==T arms after the loop.  Every syntactic
    premise is required because unsigned wrapping makes the guard alone
    insufficient when i may already exceed len.
    """
    for pos, loop in enumerate(stmts):
        if loop.get("s") != "loop": continue
        b = [st for st in loop["body"] if st.get("s") != "doc"]
        if len(b) < 3 or b[0].get("s") != "let" or b[1].get("s") != "match": continue
        rem, sub = b[0]["n"], b[0].get("e")
        if not isinstance(sub, dict) or sub.get("e") != "op" or sub.get("op") != "isub.wrap" \
                or len(sub.get("args", [])) != 2:
            continue
        nvar, ivar = map(_direct_place_name, sub["args"])
        bvar = lens.get(nvar)
        if not bvar or not ivar: continue
        guard = b[1]; sc = guard["scrut"]
        if not isinstance(sc, dict) or sc.get("e") != "op" or sc.get("op") != "ilt" \
                or len(sc.get("args", [])) != 2 \
                or _direct_place_name(sc["args"][0]) != rem \
                or sc["args"][1].get("e") != "lit":
            continue
        stride = sc["args"][1]["v"]
        arms = {a["v"]: a for a in guard["arms"]}
        tstmts = [x for x in arms.get("True", {}).get("body", []) if x.get("s") != "doc"]
        if stride <= 0 or set(arms) != {"True", "False"} or arms["False"]["body"] \
                or len(tstmts) != 1 or tstmts[0].get("s") != "break" \
                or tstmts[0].get("l") != loop["l"]:
            continue
        initial = [st for st in stmts[:pos] if st.get("s") == "let" and st.get("n") == ivar \
                   and st.get("e", {}).get("e") == "lit" and st["e"]["v"] == 0]
        increments = [st for st in b[2:] if st.get("s") == "set"
                      and not st["p"].get("path") and not st["p"].get("post")
                      and not st["p"].get("index") and st["p"]["base"] == ivar
                      and st.get("e", {}).get("e") == "op"
                      and st["e"].get("op") == "iadd.wrap"
                      and _direct_place_name(st["e"]["args"][0]) == ivar
                      and st["e"]["args"][1].get("e") == "lit"
                      and st["e"]["args"][1]["v"] == stride]
        if len(initial) != 1 or len(increments) != 1 \
                or _collect_direct_sets(whole_body, ivar) != increments \
                or _uniq_borrows_name(whole_body, ivar) or _expr_moves(whole_body, ivar) \
                or _moved_or_reset(whole_body, nvar) or _moved_or_reset(whole_body, bvar) \
                or _moved_or_reset(whole_body, rem):
            continue
        inc_pos = b.index(increments[0])
        _mark_offset_indexes(b[2:inc_pos], bvar, ivar, stride, whole_body,
                             "remainder-guard")

        # Exact post-loop tail arms: tail=len-i and tail==T prove offsets <T.
        tails = {}
        for st in stmts[pos + 1:]:
            e = st.get("e")
            if st.get("s") == "let" and isinstance(e, dict) and e.get("e") == "op" \
                    and e.get("op") == "isub.wrap" and len(e.get("args", [])) == 2 \
                    and _direct_place_name(e["args"][0]) == nvar \
                    and _direct_place_name(e["args"][1]) == ivar:
                tails[st["n"]] = True
                continue
            if st.get("s") != "match": continue
            eq = st["scrut"]
            if not isinstance(eq, dict) or eq.get("e") != "op" or eq.get("op") != "ieq" \
                    or len(eq.get("args", [])) != 2:
                continue
            tail = _direct_place_name(eq["args"][0]); lit = eq["args"][1]
            if tail not in tails or _moved_or_reset(whole_body, tail) \
                    or lit.get("e") != "lit" or not (0 < lit["v"] < stride):
                continue
            marms = {a["v"]: a for a in st["arms"]}
            if set(marms) == {"True", "False"}:
                _mark_offset_indexes(marms["True"]["body"], bvar, ivar, lit["v"],
                                     whole_body, "remainder-tail")

def prove_inbounds(fns, consts):
    for f in fns:
        body = f["body"]
        lens = {}                               # mvar -> bvar for `let m = len<T>(b);`
        for st in body:
            if st.get("s") == "let" and isinstance(st.get("e"), dict) \
                    and st["e"].get("e") == "op" and st["e"]["op"] == "len":
                a0 = st["e"]["args"][0]
                if a0.get("e") == "place" and not a0["p"].get("path") \
                        and not a0["p"].get("post"):
                    lens[st["n"]] = a0["p"]["base"]
        def walk(stmts):
            for st in stmts:
                if st.get("s") == "loop":
                    b = [x for x in st["body"] if x.get("s") != "doc"]
                    if b and b[0].get("s") == "match":
                        g = b[0]
                        sc = g["scrut"]
                        arms = {a["v"]: a for a in g["arms"]}
                        if (isinstance(sc, dict) and sc.get("e") == "op"
                                and sc["op"] == "ige"
                                and sc["args"][0].get("e") == "place"
                                and sc["args"][1].get("e") == "place"
                                and not sc["args"][0]["p"].get("path")
                                and not sc["args"][1]["p"].get("path")
                                and set(arms) == {"True", "False"}
                                and not arms["False"]["body"]
                                and len([x for x in arms["True"]["body"]
                                         if x.get("s") != "doc"]) == 1
                                and arms["True"]["body"][0].get("s") == "break"):
                            ivar = sc["args"][0]["p"]["base"]
                            mvar = sc["args"][1]["p"]["base"]
                            bvar = lens.get(mvar)
                            if bvar and not _moved_or_reset(body, mvar) \
                                    and not _moved_or_reset(body, bvar):
                                _mark_indexes(b[1:], bvar, ivar, {ivar, mvar, bvar})
                    walk(st["body"])
                if st.get("s") == "match":
                    for a in st["arms"]: walk(a["body"])
                if "match" in st:
                    for a in st["match"]["arms"]: walk(a["body"])
                if st.get("s") == "region":
                    walk(st["body"])
        walk(body)
        _prove_remainder_loops(body, lens, body)
        _analyze_capacity_obligation(f, apply_proofs=True, annotate=False)
        # pattern C: const-table index masked to the table size (power of two)
        def _markc_expr(e, masks):
            if isinstance(e, list):
                for item in e: _markc_expr(item, masks)
                return
            if not isinstance(e, dict): return
            if e.get("e") in ("place", "move") and e.get("p", {}).get("index"):
                _markc_place(e["p"], masks)
            for value in e.values(): _markc_expr(value, masks)
        def _markc_place(pl, masks):
            ix = pl["index"]; ipl = ix["place"]
            if ipl.get("path") or ipl.get("post"): return
            cinfo = consts.get(ipl["base"])
            at = ix["atom"]
            if cinfo and cinfo[0] != "scalar" and at.get("e") == "place" \
                    and not at["p"].get("path") and not at["p"].get("post"):
                info = masks.get(at["p"]["base"])
                n = cinfo[1]
                if info is not None:
                    k, deps = info
                if info is not None and k + 1 == n and (n & (n - 1)) == 0 \
                        and all(not _moved_or_reset(body, dep) for dep in deps):
                    ix["proof"] = "masked-index"
        def _mask_binding(st, masks):
            if st.get("s") != "let" or not isinstance(st.get("e"), dict) \
                    or st["e"].get("e") != "op":
                return None
            e = st["e"]
            if e["op"] == "iand":
                aa = e["args"]
                if aa[1].get("e") == "lit":
                    return aa[1]["v"], (st["n"],)
            if e["op"] == "cvt" and len(e["args"]) == 1 \
                    and e["args"][0].get("e") == "place" \
                    and not e["args"][0]["p"].get("path") \
                    and not e["args"][0]["p"].get("post"):
                src, dst = e["tyargs"]
                origin = e["args"][0]["p"]["base"]
                # Unsigned widening is value-preserving, so the exact mask
                # range survives (including through a chain).
                if origin in masks and not _is_signed(src) and not _is_signed(dst) \
                        and src in INT_WIDTH and dst in INT_WIDTH \
                        and INT_WIDTH[dst] >= INT_WIDTH[src]:
                    k, deps = masks[origin]
                    return k, deps + (st["n"],)
            return None
        def markc(stmts, inherited):
            # Facts flow into child lexical scopes; child declarations never
            # escape or leak into sibling arms. TYPE-6 permits sibling arms to
            # reuse a dead spelling, so a function-global name table is unsound.
            masks = dict(inherited)
            for st in stmts:
                for e in _direct_exprs_of(st):
                    _markc_expr(e, masks)
                if st.get("s") == "set" and st["p"].get("index"):
                    _markc_place(st["p"], masks)
                if "body" in st:
                    markc(st["body"], masks)
                if st.get("s") == "match":
                    for arm in st["arms"]:
                        arm_masks = dict(masks)
                        for binder in arm["b"]: arm_masks.pop(binder["name"], None)
                        markc(arm["body"], arm_masks)
                if "match" in st:
                    for arm in st["match"]["arms"]:
                        arm_masks = dict(masks)
                        for binder in arm["b"]: arm_masks.pop(binder["name"], None)
                        markc(arm["body"], arm_masks)
                if st.get("s") in ("let", "try"):
                    info = _mask_binding(st, masks)
                    masks.pop(st["n"], None)
                    if info is not None: masks[st["n"]] = info
        markc(body, {})

def reassociate_reductions(fns, proved):
    """[FN-4 consumer] rewrite proved-law reduction loops into 4 independent
    accumulators (block-interleaved: licensed by associative+commutative) seeded
    with the proved identity, then combine; the original loop remains as the
    scalar tail. Purely a codegen-level transform: runs after check_program."""
    if not proved: return
    n = [0]
    def walk(body):
        out = []
        for st in body:
            for key in ("body",):
                if key in st and isinstance(st[key], list): st[key] = walk(st[key])
            if st.get("s") == "match":
                for a in st["arms"]: a["body"] = walk(a["body"])
            hit = _reduction_shape(st, proved)
            if hit is None:
                out.append(st); continue
            iv, mv, accn, elem_let, d = hit
            op, T, ident = d["op"], d["T"], d["identity"]
            u = f"ra{n[0]}"; n[0] += 1
            xs, accs = [f"{u}x{k}" for k in range(4)], [f"{u}a{k}" for k in range(4)]
            iks = [f"{u}i{k}" for k in range(1, 4)]
            def elem_at(iname, xname):                 # let x_k = index<T>(buf, i_k)
                e2 = {"e": "place", "p": json.loads(json.dumps(elem_let["e"]["p"]))}
                e2["p"]["index"]["atom"] = _pl(iname)
                # This is a new access origin.  It must not inherit proof or
                # obligation-analysis credit from the source index it cloned.
                e2["p"]["index"].pop("proof", None)
                e2["p"]["index"].pop("obligation_info", None)
                return {"s": "let", "n": xname, "m": None, "ty": T, "e": e2}
            wide = {"s": "loop", "l": f"@{u}", "body": [
                {"s": "match", "scrut": _mkop("ige", "u64", [_pl(iv), _pl(f"{u}lim")]),
                 "arms": [{"v": "True", "b": [], "body": [{"s": "break", "l": f"@{u}"}]},
                          {"v": "False", "b": [], "body": []}]},
                elem_at(iv, xs[0]), _set(accs[0], _mkop(op, T, [_pl(accs[0]), _pl(xs[0])])),
                _let(iks[0], _mkop("iadd.wrap", "u64", [_pl(iv), _lit(1, "u64")])),
                elem_at(iks[0], xs[1]), _set(accs[1], _mkop(op, T, [_pl(accs[1]), _pl(xs[1])])),
                _let(iks[1], _mkop("iadd.wrap", "u64", [_pl(iv), _lit(2, "u64")])),
                elem_at(iks[1], xs[2]), _set(accs[2], _mkop(op, T, [_pl(accs[2]), _pl(xs[2])])),
                _let(iks[2], _mkop("iadd.wrap", "u64", [_pl(iv), _lit(3, "u64")])),
                elem_at(iks[2], xs[3]), _set(accs[3], _mkop(op, T, [_pl(accs[3]), _pl(xs[3])])),
                _set(iv, _mkop("iadd.wrap", "u64", [_pl(iv), _lit(4, "u64")])),
            ]}
            guard = {"s": "match", "scrut": _mkop("ilt", "u64", [_pl(iv), _pl(mv)]), "arms": [
                {"v": "True", "b": [], "body": [
                    _let(f"{u}rem", _mkop("isub.wrap", "u64", [_pl(mv), _pl(iv)])),
                    {"s": "match", "scrut": _mkop("ige", "u64", [_pl(f"{u}rem"), _lit(4, "u64")]),
                     "arms": [{"v": "True", "b": [], "body": [
                         _let(f"{u}lim", _mkop("isub.wrap", "u64", [_pl(mv), _lit(3, "u64")])),
                         *[{"s": "let", "n": a, "m": None, "ty": T, "e": dict(ident)} for a in accs],
                         wide,
                         _let(f"{u}c0", _mkop(op, T, [_pl(accs[0]), _pl(accs[1])])),
                         _let(f"{u}c1", _mkop(op, T, [_pl(accs[2]), _pl(accs[3])])),
                         _let(f"{u}c", _mkop(op, T, [_pl(f"{u}c0"), _pl(f"{u}c1")])),
                         _set(accn, _mkop(op, T, [_pl(accn), _pl(f"{u}c")])),
                     ]}, {"v": "False", "b": [], "body": []}]},
                ]},
                {"v": "False", "b": [], "body": []}]}
            out.append(guard)
            out.append(st)                             # original loop = scalar tail
        return out
    for f in fns:
        f["body"] = walk(f["body"])

def compile_program(src, alias=True, elide_bounds=False, proof_report=None):
    # elide_bounds is an EXPERIMENT-ONLY ceiling probe (perfect-prover upper
    # bound): never a shipping mode; real elision arrives via the OP-4 proof
    # tier (THE-PLAN.md bet 1).
    if proof_report is not None and proof_report:
        raise ValueError("proof_report must be a fresh empty list")
    structs, enums, fns, contracts, conforms, consts = parse_program(src)
    check_program(build_prog(structs, enums, fns, consts))
    type_spellings = _democ_type_spellings(structs, enums, fns, consts)
    _validate_erased_prelude_payloads(type_spellings, enums)
    proved = discharge_laws(contracts, conforms, fns)
    if proof_report is not None:
        # Snapshot source-derived obligation diagnostics before any facts-only
        # AST rewrite.  Later proof application may set markers but must not
        # reinterpret this diagnostic control oracle.
        for f in fns:
            # Begin fail-closed, run every registered analyzer, and only then
            # finalize untouched sites as affirmatively not applicable.
            # Any later-introduced index retains Gen's indeterminate fallback.
            sites = _indexed_access_nodes((f.get("requires") or []) + f["body"])
            for ix in sites:
                ix["obligation_info"] = _indeterminate_obligation_info()
            completed = []
            for analyzer_name, analyzer in _OBLIGATION_ANALYZER_REGISTRY:
                analyzer(f, apply_proofs=False)
                completed.append(analyzer_name)
            candidate_frontier = _capacity_candidate_frontier(f)
            for ix in sites:
                info = ix["obligation_info"]
                if info["obligation"] is None \
                        and info["obligation_status"] == "unknown":
                    if id(ix) in candidate_frontier:
                        info = _capacity_info(
                            "failed-premise", exactness="unknown", relation="unknown",
                            failed=_proof_premise(
                                "body.obligation-candidate", "body.indexed-access",
                                "recognized capacity shape or explicit exclusion"))
                        info["obligation_analysis"] = _obligation_analysis(completed)
                        ix["obligation_info"] = info
                    else:
                        ix["obligation_info"] = (
                            _not_applicable_obligation_info(completed)
                        )
                else:
                    info["obligation_analysis"] = _obligation_analysis(completed)
    if alias: reassociate_reductions(fns, proved)      # [FN-4] facts channel; --no-facts is the control
    payenums = {en for en, vs in enums.items() if _enum_payw(vs) > 0}
    _TAGONLY2.clear()
    _TAGONLY2.update({en: (vs[0][0], vs[1][0]) for en, vs in enums.items()
                      if len(vs) == 2 and all(not flds for _vn, flds in vs)})
    def _ret(f):
        if f["name"] == "main": return "i32"
        if f["rty"] == "unit": return "void"
        if _buf_elem(f["rty"]): return "{ptr, i64}"
        base = f["rty"].split("<")[0]
        if base in structs or base in payenums or base in ("Result", "Option"):
            return "%" + base
        return _llty(f["rty"])
    fnret = {f["name"]: _ret(f) for f in fns}
    Gen._fnrty = {f["name"]: f["rty"] for f in fns}
    decls = set()
    total = compute_total(fns)
    mdefs, mdctr = [], [0]                             # scoped-alias metadata, one numbering per module
    cmap = {}                                          # [CONST-2] const name -> (llvm elem type, N)
    cglobals = []
    for c in consts:
        parts = _arr_parts(c["ty"])
        if parts:
            elem, n = parts; ell = _llty(elem)
            vals = ", ".join(f"{ell} {_litval(v)}" for v in c["vals"])
            cglobals.append(f"@__const_{c['name']} = private unnamed_addr constant [{n} x {ell}] [{vals}]")
            cmap[c["name"]] = (ell, n, _is_signed(elem))
        else:                                          # scalar const: fold to its literal at use sites
            cmap[c["name"]] = (
                "scalar", _litval(c["vals"][0]), _is_signed(c["ty"]), _llty(c["ty"])
            )
    Gen._consts = cmap
    if alias:
        prove_inbounds(fns, cmap)                      # [OP-4 PROOF-1/2] facts channel
    final_report_sites = None
    if proof_report is not None:
        final_report_sites = {
            f["name"]: _lowered_indexed_access_nodes(
                (f.get("requires") or []) + f["body"]
            )
            for f in fns
        }
        for sites in final_report_sites.values():
            for ix in sites: ix["_proof_report_emissions"] = 0
    bodies = [Gen(f, enums, structs, alias, fnret, decls, total, mdefs, mdctr,
                  elide=elide_bounds, proof_report=proof_report).run() for f in fns]
    if final_report_sites is not None:
        expected_report_sites = sum(len(sites) for sites in final_report_sites.values())
        missed = [
            f"{function}:{ordinal}={ix.get('_proof_report_emissions', 0)}"
            for function, sites in final_report_sites.items()
            for ordinal, ix in enumerate(sites)
            if ix.get("_proof_report_emissions") != 1
        ]
        if missed:
            raise ValueError(
                "proof-report origin coverage invariant failed: " + ", ".join(missed)
            )
        if len(proof_report) != expected_report_sites:
            raise ValueError(
                "proof-report cardinality invariant failed: "
                f"expected {expected_report_sites}, observed {len(proof_report)}"
            )
    if cglobals: bodies.insert(0, "\n".join(cglobals) + "\n")
    if mdefs: bodies.append("\n".join(mdefs) + "\n")
    # named aggregate type per used struct, emitted once (only when structs exist -> byte-stable
    # i32/enum output). fields lower to their per-width int / i1 / nested-%T / i32-tag types [TYPE-2].
    styp = [f"%{n} = type {{ " + ", ".join(_field_ll(fl["ty"], structs) for fl in flds) + " }"
            if flds else f"%{n} = type {{}}" for n, flds in structs.items()]
    # named aggregate per payload-carrying enum: { i32 tag, i<payw> payload } (tag-only enums
    # stay a bare i32 tag, so this list is empty for tag-only-enum / scalar / struct programs
    # -> their output is unchanged) [TYPE-2, payload-carrying enums].
    etyp = [f"%{en} = type {{ i32, i{_enum_payw(vs)} }}"
            for en, vs in enums.items() if _enum_payw(vs) > 0]
    for pen in ("Result", "Option"):                   # prelude payload enums: word-erased i64
        used = any(spelling.split("<", 1)[0] == pen for spelling in type_spellings)
        if used:
            etyp.append(f"%{pen} = type {{ i32, i64 }}")
    # fixed header (sadd/ssub/smul.i32 + trap emitted unconditionally for byte-stable i32 output);
    # any other intrinsic used by width/sign-generic codegen is appended, sorted for determinism.
    hdr = (styp + etyp
           + [f"declare {{i32, i1}} @llvm.{n}.with.overflow.i32(i32, i32)" for n in ("sadd", "ssub", "smul")]
           + ["declare void @llvm.trap()"] + sorted(decls) + [""])
    return "\n".join(hdr + bodies)

if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith('-')]
    flags = {a for a in sys.argv[1:] if a.startswith('-')}
    if not args:
        print("usage: democ.py FILE.wf [--no-facts] [--asm] [--run] [--totality]"); sys.exit(0)
    src_path = Path(args[0])
    try:
        ir = compile_program(src_path.read_text(), alias='--no-facts' not in flags,
                             elide_bounds='--elide-bounds-experiment' in flags)
    except CheckError as e:
        print(f"{src_path.name}: REJECTED {e}"); sys.exit(1)
    out = src_path.with_suffix('.ll'); out.write_text(ir)
    print(f"{src_path.name}: OK -> {out}")
    if '--totality' in flags:
        _s, _e, _f = parse_program(src_path.read_text())[:3]
        print(totality_report(_f))
    cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"
    if '--asm' in flags:
        s = src_path.with_suffix('.s')
        r = subprocess.run([cc, "-O2", "-S", str(out), "-o", str(s)], capture_output=True, text=True)
        print(f"clang -O2 -> {s}" if r.returncode == 0 else f"clang failed: {r.stderr[:300]}")
    if '--run' in flags:
        exe = src_path.with_suffix('')
        r = subprocess.run([cc, "-O2", str(out), "-o", str(exe)], capture_output=True, text=True)
        if r.returncode: print("link failed:", r.stderr[:300]); sys.exit(1)
        rr = subprocess.run([str(exe)])
        print(f"ran {exe.name}: exit={rr.returncode}")
