#!/usr/bin/env python3
"""democ: demo compiler for a micro-subset of kernel-spec v0.3.
source -> parse -> ownership check (prototype checker) -> LLVM IR.
Demonstrates end-to-end: &uniq/& borrow modes lower to noalias/readonly,
iadd.wrap lowers to unflagged add (N002), iadd.trap to sadd.with.overflow+trap.
Temporary tool (owner ruling): to be replaced by a self-hosted compiler.
"""
import re, sys, subprocess
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent.parent / 'checker'))
from checker import check_fn, CheckError

TOK = re.compile(r"'[a-z][a-z0-9_]*|[0-9]+_i32|[a-z][a-z0-9_]*(?:\.[a-z]+)?|->|&uniq|&|[(){}<>:;,=\[\]]")

def toks(src):
    out = TOK.findall(src)
    return out

class P:
    def __init__(s, t): s.t = t; s.i = 0
    def peek(s): return s.t[s.i] if s.i < len(s.t) else None
    def eat(s, x=None):
        v = s.t[s.i]; s.i += 1
        assert x is None or v == x, f"expected {x}, got {v}"
        return v

def parse_mode(p):
    if p.peek() == '&uniq': p.eat(); return {"kind": "ref", "region": p.eat()[1:], "uniq": True}
    if p.peek() == '&': p.eat(); return {"kind": "ref", "region": p.eat()[1:], "uniq": False}
    p.eat('own'); return {"kind": "own"}

def parse_place(p):
    if p.peek() == 'deref':
        p.eat(); p.eat('('); inner = parse_place(p); p.eat(')')
        return {"base": inner["base"], "path": inner["path"], "deref": True}
    return {"base": p.eat(), "path": []}

def parse_expr(p):
    t = p.peek()
    if t in ('&', '&uniq'):
        uniq = p.eat() == '&uniq'; r = p.eat()[1:]
        return {"e": "borrow", "uniq": uniq, "region": r, "place": parse_place(p)}
    if re.fullmatch(r'[0-9]+_i32', t): p.eat(); return {"e": "lit", "v": int(t.split('_')[0])}
    if '.' in t:  # OPNAME
        op = p.eat(); p.eat('<'); ty = p.eat(); p.eat('>'); p.eat('(')
        args = [parse_expr(p)]
        while p.peek() == ',': p.eat(); args.append(parse_expr(p))
        p.eat(')'); return {"e": "op", "op": op, "ty": ty, "args": args}
    return {"e": "place", "p": parse_place(p)}

def parse_fn(p):
    p.eat('fn'); name = p.eat(); regions = []
    if p.peek() == '[':
        p.eat()
        while p.peek() != ']':
            regions.append(p.eat()[1:])
            if p.peek() == ',': p.eat()
        p.eat(']')
    p.eat('('); params = []
    while p.peek() != ')':
        pn = p.eat(); p.eat(':'); m = parse_mode(p); ty = p.eat()
        params.append({"name": pn, "mode": m, "ty": ty})
        if p.peek() == ',': p.eat()
    p.eat(')'); p.eat('->'); rmode = parse_mode(p); rty = p.eat()
    effs = []
    while p.peek() != '{': effs.append(p.eat())
    p.eat('{'); body = []
    while p.peek() != '}':
        body.append(parse_stmt(p))
    p.eat('}')
    return {"name": name, "regions": regions, "params": params,
            "rmode": rmode, "rty": rty, "effects": effs, "body": body}

def parse_stmt(p):
    t = p.peek()
    if t == 'let':
        p.eat(); n = p.eat(); p.eat(':'); m = parse_mode(p); ty = p.eat()
        p.eat('='); e = parse_expr(p); p.eat(';')
        return {"s": "let", "n": n, "m": m, "ty": ty, "e": e}
    if t == 'set':
        p.eat(); pl = parse_place(p); p.eat('='); e = parse_expr(p); p.eat(';')
        return {"s": "set", "p": pl, "e": e}
    if t == 'return':
        p.eat(); e = parse_expr(p); p.eat(';')
        return {"s": "return", "e": e}
    e = parse_expr(p); p.eat(';'); return {"s": "expr", "e": e}

# ---- map to prototype-checker AST ----
def cplace(pl): return {"base": pl["base"], "path": pl.get("path", [])}

def cexpr(e):
    k = e["e"]
    if k == "lit": return {"kind": "lit"}
    if k == "place": return {"kind": "use", "place": cplace(e["p"])}
    if k == "borrow": return {"kind": "borrow", "region": e["region"], "uniq": e["uniq"], "place": cplace(e["place"])}
    return {"kind": "call", "args": [cexpr(a) for a in e["args"]]}

def cstmt(s):
    if s["s"] == "let": return {"kind": "let", "name": s["n"], "mode": s["m"], "init": cexpr(s["e"])}
    if s["s"] == "set": return {"kind": "set", "place": cplace(s["p"]), "expr": cexpr(s["e"])}
    if s["s"] == "return": return {"kind": "return", "expr": cexpr(s["e"])}
    return {"kind": "expr", "expr": cexpr(s["e"])}

def spec_check(f):
    check_fn({"kind": "fn", "name": f["name"], "regions": f["regions"],
              "params": [{"name": q["name"], "mode": q["mode"]} for q in f["params"]],
              "body": [cstmt(s) for s in f["body"]]})

# ---- LLVM IR emission ----
class Gen:
    def __init__(g, f, alias_facts=True):
        g.f = f; g.n = 0; g.lines = []; g.env = {}; g.alias = alias_facts; g.traps = False
    def tmp(g): g.n += 1; return f"%t{g.n}"
    def load_place(g, pl):
        v = g.env[pl["base"]]
        if v["kind"] == "ptr":
            t = g.tmp(); g.lines.append(f"  {t} = load i32, ptr {v['v']}")
            return t
        return v["v"]
    def expr(g, e):
        if e["e"] == "lit": return str(e["v"])
        if e["e"] == "place": return g.load_place(e["p"])
        if e["e"] == "op":
            a, b = (g.expr(x) for x in e["args"]) if len(e["args"]) == 2 else (g.expr(e["args"][0]), None)
            if e["op"] == "iadd.wrap":
                t = g.tmp(); g.lines.append(f"  {t} = add i32 {a}, {b}"); return t   # unflagged = modulo [N002]
            if e["op"] == "iadd.trap":
                g.traps = True
                p_ = g.tmp(); g.lines.append(f"  {p_} = call {{i32, i1}} @llvm.sadd.with.overflow.i32(i32 {a}, i32 {b})")
                v = g.tmp(); g.lines.append(f"  {v} = extractvalue {{i32, i1}} {p_}, 0")
                o = g.tmp(); g.lines.append(f"  {o} = extractvalue {{i32, i1}} {p_}, 1")
                lbl = g.tmp()[1:]
                g.lines.append(f"  br i1 {o}, label %trap, label %{lbl}")
                g.lines.append(f"{lbl}:")
                return v
            raise SystemExit(f"demo: op {e['op']} not in micro-subset")
        raise SystemExit("demo: expr kind not lowerable")
    def run(g):
        ps = []
        for q in g.f["params"]:
            if q["mode"]["kind"] == "own":
                ps.append(f"i32 %{q['name']}"); g.env[q["name"]] = {"kind": "val", "v": f"%{q['name']}"}
            else:
                at = ""
                if g.alias:
                    at = " noalias" + ("" if q["mode"]["uniq"] else " readonly")
                ps.append(f"ptr{at} %{q['name']}"); g.env[q["name"]] = {"kind": "ptr", "v": f"%{q['name']}"}
        for s in g.f["body"]:
            if s["s"] == "let":
                g.env[s["n"]] = {"kind": "val", "v": g.expr(s["e"])}
            elif s["s"] == "set":
                v = g.expr(s["e"]); tgt = g.env[s["p"]["base"]]
                assert tgt["kind"] == "ptr"
                g.lines.append(f"  store i32 {v}, ptr {tgt['v']}")
            elif s["s"] == "return":
                g.lines.append(f"  ret i32 {g.expr(s['e'])}" if g.f["rty"] == "i32" else "  ret void")
        rt = "i32" if g.f["rty"] == "i32" else "void"
        hdr = f"define {rt} @{g.f['name']}({', '.join(ps)}) {{\nentry:"
        tail = "\ntrap:\n  call void @llvm.trap()\n  unreachable\n" if g.traps else "\n"
        decl = "declare {i32, i1} @llvm.sadd.with.overflow.i32(i32, i32)\ndeclare void @llvm.trap()\n" if g.traps else ""
        return decl + hdr + "\n" + "\n".join(g.lines) + tail + "}\n"

def compile_src(src, alias_facts=True):
    f = parse_fn(P(toks(src)))
    spec_check(f)
    return f["name"], Gen(f, alias_facts).run()

if __name__ == "__main__":
    args = [a for a in sys.argv[1:] if not a.startswith('-')]
    flags = {a for a in sys.argv[1:] if a.startswith('-')}
    if not args:
        print("usage: democ.py FILE.xl [--no-facts] [--asm]\n"
              "  compiles kernel-subset source to FILE.ll (checker-gated);\n"
              "  --no-facts  omit ownership facts (noalias/readonly) for A/B\n"
              "  --asm       also run clang -O2 -S -> FILE.s\n"
              "examples: examples/twice_read.xl examples/dangle.xl")
        sys.exit(0)
    src_path = Path(args[0])
    try:
        name, ir = compile_src(src_path.read_text(), alias_facts='--no-facts' not in flags)
    except CheckError as e:
        print(f"{src_path.name}: REJECTED {e}"); sys.exit(1)
    out = src_path.with_suffix('.ll')
    out.write_text(ir)
    print(f"{src_path.name}: OK -> {out}")
    if '--asm' in flags:
        s = src_path.with_suffix('.s')
        cc = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"   # native target; PATH clang may default to wasm
        r = subprocess.run([cc, "-O2", "-S", str(out), "-o", str(s)], capture_output=True, text=True)
        print(f"clang -O2 -> {s}" if r.returncode == 0 else f"clang failed: {r.stderr[:200]}")
