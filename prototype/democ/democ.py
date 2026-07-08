#!/usr/bin/env python3
"""democ: demo compiler for a growing subset of kernel-spec v0.6.
source -> parse -> program check (type + ownership) -> LLVM IR (-> native).
Subset: fns, own/&/&uniq i32 params, let/set/return, deref places, iadd.wrap/
trap/checked, ieq/ilt comparisons, enums + builtin Bool/Result, named
construction K(field: atom), named match binders K(field: binder), named
user-fn call args f(param: atom), give value-match, check-else-trap, region
stmts, doc fields, cross-fn calls, runnable main.
Temporary tool (owner ruling): endgame is a self-hosted compiler.
"""
import re, sys, subprocess
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'checker'))
from checker import check_program, CheckError

TOK = re.compile(r'"[^"]*"|\'[a-z][a-z0-9_]*|[0-9]+_i32|[0-9]+'
                 r'|[a-z][a-z0-9_]*(?:\.[a-z]+)?|[A-Z][A-Za-z0-9]*'
                 r'|->|=>|&uniq|&|[(){}<>:;,=\[\]]')

def toks(src):
    if re.search(r'//|/\*', re.sub(r'"[^"]*"', '', src)):   # FORM-4: comments do not exist
        raise CheckError("FORM-4", "comments do not exist; documentation rides the doc field of a declaration")
    return TOK.findall(src)

class P:
    def __init__(s, t): s.t = t; s.i = 0
    def peek(s, k=0): return s.t[s.i+k] if s.i+k < len(s.t) else None
    def eat(s, x=None):
        v = s.t[s.i]; s.i += 1
        assert x is None or v == x, f"expected {x}, got {v} at {s.t[max(0,s.i-4):s.i+3]}"
        return v

def is_typeid(t): return bool(t) and t[0].isupper()

def parse_mode(p):
    if p.peek() == '&uniq': p.eat(); return {"kind": "ref", "region": p.eat()[1:], "uniq": True}
    if p.peek() == '&': p.eat(); return {"kind": "ref", "region": p.eat()[1:], "uniq": False}
    p.eat('own'); return {"kind": "own"}

def parse_type(p):
    base = p.eat()
    if p.peek() == '<':
        p.eat()
        while p.peek() != '>':
            parse_type(p)
            if p.peek() == ',': p.eat()
        p.eat('>')
    return base

def parse_place(p):
    if p.peek() == 'deref':
        p.eat(); p.eat('('); inner = parse_place(p); p.eat(')')
        inner["deref"] = inner.get("deref", 0) + 1
        return inner
    return {"base": p.eat(), "path": [], "deref": 0}

def parse_expr(p):
    t = p.peek()
    if t in ('&', '&uniq'):
        uniq = p.eat() == '&uniq'; r = p.eat()[1:]
        return {"e": "borrow", "uniq": uniq, "region": r, "place": parse_place(p)}
    if t == 'unit': p.eat(); return {"e": "unit"}
    if re.fullmatch(r'[0-9]+_i32', t):                 # suffixed integer literal
        p.eat(); digits = t.split('_')[0]
        if len(digits) > 1 and digits[0] == '0':       # FORM-7: leading-zero form is illegal (0 is its own form)
            raise CheckError("FORM-7", f"leading-zero integer literal '{t}' is illegal; the single digit 0 is its own form")
        v = int(digits)
        if v > 2147483647:                             # FORM-7: literal exceeds its suffix type's range
            raise CheckError("FORM-7", f"integer literal '{t}' is out of range for i32 (max 2147483647)")
        return {"e": "lit", "v": v}
    if re.fullmatch(r'[0-9]+', t):                     # FORM-5: a bare integer lacks its mandatory type suffix
        raise CheckError("FORM-5", f"integer literal '{t}' must carry its mandatory type suffix (e.g. {t}_i32)")
    if is_typeid(t):                                   # construct K(field: atom, ...) [GRAM-8]
        n = p.eat(); p.eat('('); fields = []
        while p.peek() != ')':
            fname = p.eat(); p.eat(':'); atom = parse_atom(p)
            fields.append({"name": fname, "atom": atom})
            if p.peek() == ',': p.eat()
        p.eat(')'); return {"e": "construct", "n": n, "fields": fields}
    if '.' in t or p.peek(1) == '<':                   # OPNAME<ty>(atoms)
        op = p.eat()
        if '.' in op and op.split('.', 1)[1] not in ("wrap", "trap", "checked", "strict"):
            raise CheckError("FORM-3", f"'{op}' is not a legal OPNAME; the mode suffix is a closed word set {{wrap,trap,checked,strict}}")
        p.eat('<'); tyargs = [parse_type(p)]
        while p.peek() == ',': p.eat(); tyargs.append(parse_type(p))
        p.eat('>'); p.eat('(')
        args = [parse_atom(p)]                          # [GRAM-9] operands are atoms, not nested calls
        while p.peek() == ',': p.eat(); args.append(parse_atom(p))
        p.eat(')'); return {"e": "op", "op": op, "args": args, "tyargs": tyargs}
    if p.peek(1) == '(' and t not in ('deref', 'index'):   # user call f(param: atom, ...) [GRAM-11]
        n = p.eat(); p.eat('('); args = []; argnames = []
        while p.peek() != ')':
            pname = p.eat()
            if p.peek() != ':':                        # [GRAM-11] call args are named (param: atom), never positional
                raise CheckError("GRAM-11", "a user-fn call must name its arguments (param: atom) in declared order; positional args are illegal")
            p.eat(':'); atom = parse_atom(p)
            argnames.append(pname); args.append(atom)
            if p.peek() == ',': p.eat()
        p.eat(')'); return {"e": "ucall", "n": n, "args": args, "argnames": argnames}
    return {"e": "place", "p": parse_place(p)}

def parse_atom(p):                                     # [GRAM-9] atom := literal | place | borrow | deref | construct
    e = parse_expr(p)
    if e["e"] in ("op", "ucall"):                      # a call in an atom position does not derive (three-address/ANF)
        raise CheckError("GRAM-9", "a call in an atom position does not derive (three-address form); bind it with a preceding let")
    return e

def parse_match(p):
    p.eat('match'); scrut = parse_expr(p); p.eat('{'); arms = []
    while p.peek() != '}':
        vn = p.eat(); p.eat('('); binders = []
        while p.peek() != ')':                         # K(field: binder, ...) [GRAM-10]
            field = p.eat(); p.eat(':'); binder = p.eat()
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
        p.eat(); n = p.eat(); p.eat(':'); m = parse_mode(p); ty = parse_type(p); p.eat('=')
        if p.peek() == 'match':                        # value-match initializer [GIVE-1]
            return {"s": "let", "n": n, "m": m, "ty": ty, "match": parse_match(p)}
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
        p.eat(); r = p.eat()[1:]; p.eat('{'); b = []
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

def parse_program(src):
    p = P(toks(src)); enums = {}; fns = []
    while p.peek():
        if p.peek() == 'enum':
            p.eat(); name = p.eat()
            if not is_typeid(name):                    # [FORM-3] an enum name is a TYPEID ([A-Z]...)
                raise CheckError("FORM-3", f"enum name must be a TYPEID ([A-Z]...), got '{name}'")
            p.eat('{'); vs = []
            while p.peek() != '}':
                vn = p.eat(); p.eat('('); fields = []      # vfield := IDENT ":" type
                while p.peek() != ')':
                    fname = p.eat(); p.eat(':'); fty = parse_type(p)
                    fields.append({"name": fname, "ty": fty})
                    if p.peek() == ',': p.eat()
                p.eat(')'); p.eat(';'); vs.append((vn, fields))
            p.eat('}'); enums[name] = vs
        else:
            p.eat('fn'); name = p.eat()
            if not re.fullmatch(r'[a-z][a-z0-9_]*', name):   # [FORM-3] a fn name is an IDENT ([a-z][a-z0-9_]*)
                raise CheckError("FORM-3", f"fn name must be an IDENT ([a-z][a-z0-9_]*), got '{name}'")
            regions = []
            if p.peek() == '[':
                p.eat()
                while p.peek() != ']':
                    regions.append(p.eat()[1:])
                    if p.peek() == ',': p.eat()
                p.eat(']')
            p.eat('('); params = []
            while p.peek() != ')':
                pn = p.eat(); p.eat(':'); m = parse_mode(p); ty = parse_type(p)
                params.append({"name": pn, "mode": m, "ty": ty})
                if p.peek() == ',': p.eat()
            p.eat(')'); p.eat('->'); rmode = parse_mode(p); rty = parse_type(p)
            eff = []
            while p.peek() != '{': eff.append(p.eat())   # effect row [EFF-1/EFF-2]
            p.eat('{'); body = []
            while p.peek() != '}': body.append(parse_stmt(p))
            p.eat('}')
            fns.append({"name": name, "regions": regions, "params": params,
                        "rmode": rmode, "rty": rty, "effects": eff, "body": body})
    return enums, fns

# ---- v0.6 type-layer mapping: democ parse tree -> check_program `prog` dict ----
PRIM_SET = {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64"}
def ttype(base):
    if base in PRIM_SET: return {"kind": "prim", "name": base}
    if base == "unit": return {"kind": "unit"}
    if base in ("box", "buffer"): return {"kind": base, "elem": {"kind": "any"}}
    if base in ("slice", "arena"): return {"kind": base, "region": None, "elem": {"kind": "any"}}
    if base == "array": return {"kind": "array", "elem": {"kind": "any"}, "n": None}
    return {"kind": "named", "name": base}
def tplace(pl):
    node = {"kind": "var", "name": pl["base"]}
    for f in pl.get("path", []):
        node = {"kind": "field", "place": node, "name": f}
    for _ in range(pl.get("deref", 0)):
        node = {"kind": "deref", "place": node}
    return node
def texpr(e):
    k = e["e"]
    if k == "lit": return {"kind": "lit", "ty": {"kind": "prim", "name": "i32"}}
    if k == "unit": return {"kind": "lit", "ty": {"kind": "unit"}}
    if k == "place": return {"kind": "use", "place": tplace(e["p"])}
    if k == "borrow": return {"kind": "borrow", "region": e["region"], "uniq": e["uniq"],
                              "place": tplace(e["place"])}
    if k == "construct": return {"kind": "construct", "name": e["n"],
        "fields": [{"name": f["name"], "atom": texpr(f["atom"])} for f in e["fields"]]}
    if k == "ucall": return {"kind": "call", "callee": e["n"],
        "args": [texpr(a) for a in e["args"]], "argnames": e["argnames"]}
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
    if k == "match": return tmatch(s)
    if k == "give": return {"kind": "give", "expr": texpr(s["e"])}
    if k == "expr": return {"kind": "expr", "expr": texpr(s["e"])}
    raise SystemExit(f"democ: cannot map stmt {k}")
def tstmts(body):
    return [x for x in (tstmt(s) for s in body) if x is not None]
def build_prog(enums, fns):
    if len([f for f in fns if f["name"] == "main"]) > 1:   # FN-7: at most one main
        raise CheckError("FN-7", "at most one fn main")
    prog = {"structs": {}, "enums": {}, "fns": {}}
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
            "body": tstmts(f["body"]),
        }
    return prog

# ---- LLVM IR ----
class Gen:
    def __init__(g, f, enums, alias=True):
        g.f = f; g.enums = enums; g.alias = alias
        g.n = 0; g.lines = []; g.env = {}; g.traps = False; g.term = False
        g.loopstk = []; g.give_slot = None
    def tmp(g): g.n += 1; return f"%t{g.n}"
    def lbl(g): g.n += 1; return f"L{g.n}"
    def emit(g, s): g.lines.append(s)
    def vtag(g, name):
        for en, vs in g.enums.items():
            for i, (vn, _) in enumerate(vs):
                if vn == name: return i
        return None
    def expr(g, e):
        k = e["e"]
        if k == "lit": return {"k": "i32", "v": str(e["v"])}
        if k == "unit": return {"k": "unit"}
        if k == "place":
            v = g.env[e["p"]["base"]]
            if v["k"] in ("ptr", "slot"):
                t = g.tmp(); g.emit(f"  {t} = load i32, ptr {v['v']}"); return {"k": "i32", "v": t}
            return v
        if k == "borrow":                              # &'r p / &uniq 'r p -> pointer to place
            src = g.env[e["place"]["base"]]
            if src["k"] in ("slot", "ptr"):
                return {"k": "ptr", "v": src["v"]}
            slot = g.tmp(); g.emit(f"  {slot} = alloca i32")   # spill own SSA to make addressable
            g.emit(f"  store i32 {src['v']}, ptr {slot}")
            return {"k": "ptr", "v": slot}
        if k == "construct":
            n = e["n"]; flds = e["fields"]
            if n == "True": return {"k": "i1", "v": "true"}
            if n == "False": return {"k": "i1", "v": "false"}
            if n == "Ok": a = g.expr(flds[0]["atom"]); return {"k": "pair", "tag": "false", "val": a["v"]}
            if n == "Err": return {"k": "pair", "tag": "true", "val": "0"}
            return {"k": "i32", "v": str(g.vtag(n))}
        if k == "ucall":
            args = [g.expr(a) for a in e["args"]]
            t = g.tmp()
            g.emit(f"  {t} = call i32 @{e['n']}({', '.join('i32 ' + a['v'] for a in args)})")
            return {"k": "i32", "v": t}
        a = [g.expr(x) for x in e["args"]]
        op = e["op"]
        ARITH = {"iadd": ("add", "sadd"), "isub": ("sub", "ssub"), "imul": ("mul", "smul")}
        base, mode = (op.split('.') + [None])[:2]
        if base in ARITH and mode == "wrap":
            t = g.tmp(); g.emit(f"  {t} = {ARITH[base][0]} i32 {a[0]['v']}, {a[1]['v']}"); return {"k": "i32", "v": t}
        if base in ARITH and mode in ("trap", "checked"):
            g.traps = g.traps or mode == "trap"
            iv = ARITH[base][1]
            p_ = g.tmp(); g.emit(f"  {p_} = call {{i32, i1}} @llvm.{iv}.with.overflow.i32(i32 {a[0]['v']}, i32 {a[1]['v']})")
            v = g.tmp(); g.emit(f"  {v} = extractvalue {{i32, i1}} {p_}, 0")
            o = g.tmp(); g.emit(f"  {o} = extractvalue {{i32, i1}} {p_}, 1")
            if mode == "checked": return {"k": "pair", "tag": o, "val": v}
            l = g.lbl(); g.emit(f"  br i1 {o}, label %trap, label %{l}"); g.emit(f"{l}:")
            return {"k": "i32", "v": v}
        if op == "__dead_iadd.checked":
            p_ = g.tmp(); g.emit(f"  {p_} = call {{i32, i1}} @llvm.sadd.with.overflow.i32(i32 {a[0]['v']}, i32 {a[1]['v']})")
            v = g.tmp(); g.emit(f"  {v} = extractvalue {{i32, i1}} {p_}, 0")
            o = g.tmp(); g.emit(f"  {o} = extractvalue {{i32, i1}} {p_}, 1")
            return {"k": "pair", "tag": o, "val": v}
        if op == "__dead_iadd.trap":
            g.traps = True
            p_ = g.tmp(); g.emit(f"  {p_} = call {{i32, i1}} @llvm.sadd.with.overflow.i32(i32 {a[0]['v']}, i32 {a[1]['v']})")
            v = g.tmp(); g.emit(f"  {v} = extractvalue {{i32, i1}} {p_}, 0")
            o = g.tmp(); g.emit(f"  {o} = extractvalue {{i32, i1}} {p_}, 1")
            l = g.lbl(); g.emit(f"  br i1 {o}, label %trap, label %{l}"); g.emit(f"{l}:")
            return {"k": "i32", "v": v}
        cmps = {"ieq": "eq", "ine": "ne", "ilt": "slt", "ile": "sle", "igt": "sgt", "ige": "sge"}
        if op in cmps:
            t = g.tmp(); g.emit(f"  {t} = icmp {cmps[op]} i32 {a[0]['v']}, {a[1]['v']}"); return {"k": "i1", "v": t}
        raise SystemExit(f"demo: op {op} not in subset")
    def stmts(g, body):
        for s in body:
            if g.term: break
            k = s["s"]
            if k == "doc": continue
            if k == "let":
                if "match" in s:                       # value-match with give [GIVE-1]
                    slot = g.tmp(); g.emit(f"  {slot} = alloca i32")
                    prev = g.give_slot; g.give_slot = slot
                    g.gen_match(s["match"])
                    g.give_slot = prev
                    g.env[s["n"]] = {"k": "slot", "v": slot}
                    continue
                v = g.expr(s["e"])
                if v["k"] == "i32":
                    slot = g.tmp(); g.emit(f"  {slot} = alloca i32")
                    g.emit(f"  store i32 {v['v']}, ptr {slot}")
                    g.env[s["n"]] = {"k": "slot", "v": slot}
                else: g.env[s["n"]] = v
            elif k == "give":
                v = g.expr(s["e"])
                g.emit(f"  store i32 {v['v']}, ptr {g.give_slot}")
            elif k == "set":
                v = g.expr(s["e"]); tgt = g.env[s["p"]["base"]]
                assert tgt["k"] in ("ptr", "slot"), "set target must be param ptr or own local"
                g.emit(f"  store i32 {v['v']}, ptr {tgt['v']}")
            elif k == "return":
                v = g.expr(s["e"])
                if g.f["name"] == "main": g.emit("  ret i32 0")
                elif v["k"] == "unit": g.emit("  ret void")
                else: g.emit(f"  ret i32 {v['v']}")
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
                g.emit(f"  br i1 {c['v']}, label %{l}, label %trap")
                g.emit(f"{l}:")
            elif k == "match":
                g.gen_match(s)
            else: g.expr(s["e"])
    def gen_match(g, s):
        sc = g.expr(s["scrut"])
        have = {a["v"] for a in s["arms"]}
        need = ({"True", "False"} if sc["k"] == "i1" else
                {"Ok", "Err"} if sc["k"] == "pair" else
                {vn for en, vs in g.enums.items() for (vn, _) in vs
                 if g.vtag(s["arms"][0]["v"]) is not None
                 and any(v2[0] == s["arms"][0]["v"] for v2 in vs)})
        if have != need:
            raise CheckError("ERR-2",
                f"non-exhaustive match: have {sorted(have)}, need {sorted(need)}")
        done = g.lbl(); any_open = False
        if sc["k"] == "i1":
            lt, lf = g.lbl(), g.lbl()
            g.emit(f"  br i1 {sc['v']}, label %{lt}, label %{lf}")
            for a in s["arms"]:
                l = lt if a["v"] == "True" else lf
                g.emit(f"{l}:"); g.term = False; g.stmts(a["body"])
                if not g.term: g.emit(f"  br label %{done}"); any_open = True
        elif sc["k"] == "pair":
            lo, le = g.lbl(), g.lbl()
            g.emit(f"  br i1 {sc['tag']}, label %{le}, label %{lo}")
            for a in s["arms"]:
                l = lo if a["v"] == "Ok" else le
                g.emit(f"{l}:"); g.term = False
                if a["b"]:
                    g.env[a["b"][0]["name"]] = {"k": "i32", "v": sc["val"] if a["v"] == "Ok" else "0"}
                g.stmts(a["body"])
                if not g.term: g.emit(f"  br label %{done}"); any_open = True
        else:
            nxt = None
            for a in s["arms"]:
                if nxt: g.emit(f"{nxt}:")
                tag = g.vtag(a["v"]); la = g.lbl(); nxt = g.lbl()
                c = g.tmp(); g.emit(f"  {c} = icmp eq i32 {sc['v']}, {tag}")
                g.emit(f"  br i1 {c}, label %{la}, label %{nxt}")
                g.emit(f"{la}:"); g.term = False; g.stmts(a["body"])
                if not g.term: g.emit(f"  br label %{done}"); any_open = True
            g.emit(f"{nxt}:"); g.emit("  unreachable")   # exhaustive [ERR-2]
        g.term = not any_open
        if any_open: g.emit(f"{done}:")
    def run(g):
        ps = []
        for q in g.f["params"]:
            if q["mode"]["kind"] == "own":
                ps.append(f"i32 %{q['name']}"); g.env[q["name"]] = {"k": "i32", "v": f"%{q['name']}"}
            else:
                at = (" noalias" + ("" if q["mode"]["uniq"] else " readonly")) if g.alias else ""
                ps.append(f"ptr{at} %{q['name']}"); g.env[q["name"]] = {"k": "ptr", "v": f"%{q['name']}"}
        rt = "i32" if (g.f["rty"] != "unit" or g.f["name"] == "main") else "void"
        g.emit(f"define {rt} @{g.f['name']}({', '.join(ps)}) {{")
        g.emit("entry:")
        g.stmts(g.f["body"])
        if not g.term:
            g.emit("  ret i32 0" if g.f["name"] == "main" else ("  ret void" if rt == "void" else "  unreachable"))
        if g.traps: g.emit("trap:\n  call void @llvm.trap()\n  unreachable")
        g.emit("}")
        return "\n".join(g.lines) + "\n"

def compile_program(src, alias=True):
    enums, fns = parse_program(src)
    check_program(build_prog(enums, fns))
    ir = [f"declare {{i32, i1}} @llvm.{n}.with.overflow.i32(i32, i32)" for n in ("sadd","ssub","smul")] + ["declare void @llvm.trap()", ""]
    for f in fns: ir.append(Gen(f, enums, alias).run())
    return "\n".join(ir)

if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith('-')]
    flags = {a for a in sys.argv[1:] if a.startswith('-')}
    if not args:
        print("usage: democ.py FILE.xl [--no-facts] [--asm] [--run]"); sys.exit(0)
    src_path = Path(args[0])
    try:
        ir = compile_program(src_path.read_text(), alias='--no-facts' not in flags)
    except CheckError as e:
        print(f"{src_path.name}: REJECTED {e}"); sys.exit(1)
    out = src_path.with_suffix('.ll'); out.write_text(ir)
    print(f"{src_path.name}: OK -> {out}")
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
