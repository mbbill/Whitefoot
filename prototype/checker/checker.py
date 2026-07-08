"""Checker-core prototype: D1a blocking-gate experiment.

Implements the kernel-spec-v0 ownership calculus (rules OWN-1..OWN-8, STOR-4)
over a toy canonical AST: affine ownership, lexical regions, borrow
exclusivity, escape checking. Deliberately simplified per D1a: no inference,
lexical liveness (OWN-6), reject-when-unsure (OWN-8).

AST (Python dicts):
  fn:      {"kind":"fn","name":str,"params":[param],"regions":[str],"body":[stmt]}
  param:   {"name":str,"mode":mode}
  mode:    {"kind":"own"} | {"kind":"ref","region":str,"uniq":bool}
  stmt:
    {"kind":"let","name":str,"mode":mode,"init":expr}
    {"kind":"set","place":place,"expr":expr}
    {"kind":"region","name":str,"body":[stmt]}
    {"kind":"return","expr":expr}
    {"kind":"expr","expr":expr}
  expr:
    {"kind":"use","place":place}                  # read (copy for prims)
    {"kind":"move","place":place}                 # move out of place
    {"kind":"borrow","region":str,"uniq":bool,"place":place}
    {"kind":"call","args":[expr]}
    {"kind":"lit"}
  place:   {"base":str,"path":[str]}              # x.f.g -> base "x", path ["f","g"]

Diagnostics cite kernel-spec-v0 rule IDs.
"""

import copy
from dataclasses import dataclass, field
from typing import Optional


class CheckError(Exception):
    def __init__(self, rule: str, msg: str):
        self.rule = rule
        super().__init__(f"[{rule}] {msg}")


@dataclass
class Binding:
    name: str
    mode: dict
    depth: int                      # lexical region depth where bound
    state: str = "live"             # live | moved
    region: Optional[str] = None    # for ref modes


@dataclass
class Borrow:
    place: tuple                    # (base, *path)
    uniq: bool
    region: str
    holder: Optional[str] = None    # binding name holding this borrow


@dataclass
class Ctx:
    bindings: dict = field(default_factory=dict)   # name -> Binding
    borrows: list = field(default_factory=list)    # live Borrows
    regions: list = field(default_factory=list)    # region name stack (outer..inner)

    def depth(self) -> int:
        return len(self.regions)

    def region_depth(self, r: str) -> int:
        # caller-supplied regions (depth 0) outlive all locals [OWN-3]
        return self.regions.index(r) if r in self.regions else -1

    def outlives(self, a: str, b: str) -> bool:
        # a outlives b iff a introduced at outer-or-equal depth [OWN-3]
        return self.region_depth(a) <= self.region_depth(b)


def places_overlap(p: tuple, q: tuple) -> bool:
    # [OWN-7] prefix overlap
    n = min(len(p), len(q))
    return p[:n] == q[:n]


class Checker:
    def __init__(self, fn: dict):
        self.fn = fn
        self.ctx = Ctx()
        self.loops = []   # (region depth at loop entry, binding names at entry)
        self.call_frames = []   # ctx.borrows length at call-argument start
        self.deliver = []   # stack of let-init modes for `give` [GIVE-1] (v0.6)

    # -- entry ---------------------------------------------------------------
    def check(self):
        for r in self.fn.get("regions", []):
            self.ctx.regions.append(r)         # caller regions: outermost
        for p in self.fn["params"]:
            b = Binding(p["name"], p["mode"], self.ctx.depth())
            if p["mode"]["kind"] == "ref":
                b.region = p["mode"]["region"]
            self.ctx.bindings[p["name"]] = b
        self.check_block(self.fn["body"])

    # -- statements ----------------------------------------------------------
    def check_block(self, stmts):
        for s in stmts:
            self.check_stmt(s)
            # [OWN-6] call-scoped temporaries die at end of the enclosing statement
            self.ctx.borrows = [b for b in self.ctx.borrows if b.holder is not None]

    def check_stmt(self, s):
        k = s["kind"]
        if k == "let":
            # v0.6: a `let` may be initialized by a value-match [GIVE-1]
            if s["init"].get("kind") == "match":
                self.deliver.append(s["mode"])
                self._check_match(s["init"])
                self.deliver.pop()
            else:
                self.check_expr(s["init"], expect_mode=s["mode"])
            b = Binding(s["name"], s["mode"], self.ctx.depth())
            if s["mode"]["kind"] == "ref":
                b.region = s["mode"]["region"]
                # record that this binding holds the borrow just created
                for br in self.ctx.borrows:
                    if br.holder is None:
                        br.holder = s["name"]
            self.ctx.bindings[s["name"]] = b
        elif k == "set":
            raw = self.place_key(s["place"])
            self.require_writable(self.resolve(raw), holder=raw[0])
            self.check_expr(s["expr"])
        elif k == "region":
            self.ctx.regions.append(s["name"])
            depth = self.ctx.depth()
            saved = dict(self.ctx.bindings)
            self.check_block(s["body"])
            # region end: kill borrows in the exiting region (index depth-1)
            # or deeper, then bindings at this depth
            self.ctx.borrows = [b for b in self.ctx.borrows
                                if self.ctx.region_depth(b.region) < depth - 1]
            self.ctx.bindings = {n: b for n, b in self.ctx.bindings.items()
                                 if b.depth < depth or n in saved}
            self.ctx.regions.pop()
        elif k == "loop":
            self.loops.append((self.ctx.depth(), set(self.ctx.bindings)))
            self.check_block(s["body"])
            self.loops.pop()
        elif k == "match":
            self._check_match(s)
        elif k == "give":                             # v0.6 [GIVE-1]: deliver arm value
            expect = self.deliver[-1] if self.deliver else None
            self.check_expr(s["expr"], expect_mode=expect)
        elif k == "return":
            self.check_expr(s["expr"], returning=True)
        elif k == "expr":
            self.check_expr(s["expr"])
        elif k == "check":                            # v0.6: runtime check condition
            self.check_expr(s["expr"])
        elif k in ("break", "doc"):                   # v0.6: no ownership effect here
            pass
        else:
            raise CheckError("GRAM-4", f"unknown stmt kind {k}")

    # -- match core (shared by statement- and let-initializer matches) --------
    def _check_match(self, s):
        # [OWN-13]/[T-B]: binder modes DERIVED from scrutinee; own moves,
        # borrow-mode scrutinee stays live and binders alias its content
        scrut = s["scrut"]; binder_mode = {"kind": "own"}; alias_of = None
        if scrut.get("kind") == "use":
            base = self.place_key(scrut["place"])[0]
            sb = self.require_live(base)
            if sb.mode["kind"] == "own":
                self.check_expr({"kind": "move", "place": scrut["place"]})
            else:
                self.check_expr(scrut)
                binder_mode = {"kind": "ref", "region": sb.region,
                               "uniq": sb.mode.get("uniq", False)}
                alias_of = self.resolve(self.place_key(scrut["place"]))
        else:
            self.check_expr(scrut)      # owned temporary [OWN-13]
        snap_b = {n: copy.copy(b) for n, b in self.ctx.bindings.items()}
        snap_br = [copy.copy(x) for x in self.ctx.borrows]
        moved_any = set(); joined_borrows = []
        for arm in s["arms"]:
            self.ctx.bindings = {n: copy.copy(b) for n, b in snap_b.items()}
            self.ctx.borrows = [copy.copy(x) for x in snap_br]
            for bn in [self._bname(b) for b in arm.get("binders", [])]:
                b_ = Binding(bn, binder_mode, self.ctx.depth())
                if binder_mode["kind"] == "ref":
                    b_.region = binder_mode["region"]
                    self.ctx.borrows.append(Borrow(alias_of,
                        binder_mode["uniq"], binder_mode["region"], holder=bn))
                self.ctx.bindings[bn] = b_
            self.check_block(arm["body"])
            for n in snap_b:
                if self.ctx.bindings[n].state == "moved":
                    moved_any.add(n)
            arm_locals = set(self.ctx.bindings) - set(snap_b)
            joined_borrows += [x for x in self.ctx.borrows
                               if x.holder not in arm_locals
                               and not any(x is y or (x.place, x.region, x.uniq, x.holder)
                                           == (y.place, y.region, y.uniq, y.holder)
                                           for y in snap_br)]
        self.ctx.bindings = snap_b
        for n in moved_any:                       # moved-in-any-arm => moved after
            self.ctx.bindings[n].state = "moved"
        self.ctx.borrows = snap_br + joined_borrows

    @staticmethod
    def _bname(b):                                 # binder is a name (old) or {field,name} (v0.6)
        return b["name"] if isinstance(b, dict) else b

    # -- expressions ---------------------------------------------------------
    def check_expr(self, e, expect_mode=None, returning=False):
        k = e["kind"]
        if k == "lit":
            return
        if k == "use":
            raw = self.place_key(e["place"])
            self.require_live(raw[0])
            self.require_readable(self.resolve(raw), holder=raw[0])
            return
        if k == "move":
            key = self.place_key(e["place"])
            b = self.require_live(key[0])
            key = self.resolve(key)
            # [OWN-5] cannot move while any borrow of an overlapping place lives
            for br in self.ctx.borrows:
                if places_overlap(br.place, key):
                    raise CheckError("OWN-5",
                        f"move of {key} while borrow of {br.place} is live")
            if b.mode["kind"] != "own":
                raise CheckError("OWN-1", f"move through non-owning binding {key[0]}")
            if self.loops and self.place_key(e["place"])[0] in self.loops[-1][1]:
                raise CheckError("OWN-11",
                    "move of a binding declared outside the enclosing loop "
                    "(second iteration would use a dead binding)")
            b.state = "moved"                              # [OWN-1]
            return
        if k == "borrow":
            key = self.resolve(self.place_key(e["place"]))
            b = self.require_live(self.place_key(e["place"])[0])
            r = e["region"]
            if self.ctx.region_depth(r) == -1 and r not in self.fn.get("regions", []):
                raise CheckError("OWN-3", f"unknown region {r}")
            # [OWN-11] inside a loop, borrows may only name regions introduced
            # inside the loop body (kills cross-iteration double-uniq)
            if self.loops and self.ctx.region_depth(r) < self.loops[-1][0]:
                raise CheckError("OWN-11",
                    f"borrow region {r} was introduced outside the enclosing loop")
            # [OWN-10] borrow-storage duration: the borrowed place's storage
            # must outlive the borrow's region.
            if b.mode["kind"] == "own":
                # [OWN-10] the borrow's region must end no later than the
                # binding's storage. Caller regions outlive the frame: never
                # legal for own bindings. A local region is legal iff its
                # block ends within the binding's enclosing block: for a
                # binding bound at depth d (> n caller regions), the region's
                # index must be >= d-1; function-level bindings (d == n)
                # accept any local region.
                n = len(self.fn.get("regions", []))
                idx = self.ctx.region_depth(r)
                if idx < n:
                    raise CheckError("OWN-10",
                        f"caller region {r} outlives storage of own binding {key[0]}")
                if b.depth > n and idx < b.depth - 1:
                    raise CheckError("OWN-10",
                        f"region {r} outlives storage of own binding {key[0]}")
            else:
                # through-borrow: source borrow's region must outlive 'r
                if not self.ctx.outlives(b.region, r):
                    raise CheckError("OWN-10",
                        f"borrow region {r} outlives source borrow region {b.region}")
            # [OWN-5] exclusivity against existing borrows (resolved places);
            # conflicts among arguments of the SAME call are cited as OWN-12
            for i, br in enumerate(self.ctx.borrows):
                if self.call_frames and i >= self.call_frames[-1]:
                    continue
                if places_overlap(br.place, key) and (br.uniq or e["uniq"]):
                    raise CheckError("OWN-5",
                        f"{'uniq ' if e['uniq'] else ''}borrow of {key} conflicts "
                        f"with live {'uniq ' if br.uniq else ''}borrow of {br.place}")
            # [OWN-4] escape: if this borrow initializes a binding whose declared
            # region outlives r, reject (checked at let via expect_mode)
            if expect_mode is not None and expect_mode["kind"] == "ref":
                dest_r = expect_mode["region"]
                if not self.ctx.outlives(r, dest_r) and dest_r != r:
                    raise CheckError("OWN-4",
                        f"borrow in region {r} stored into region {dest_r} "
                        f"which it does not outlive")
            if returning:
                # returned borrows must be in a caller-supplied region [OWN-4]
                if r not in self.fn.get("regions", []):
                    raise CheckError("OWN-4",
                        f"borrow in local region {r} escapes via return")
            self.ctx.borrows.append(Borrow(key, e["uniq"], r))
            return
        if k == "call":
            self._check_call_args(e["args"])
            return
        if k == "construct":                          # v0.6 [GRAM-8]: like a call over fields
            self._check_call_args([f["atom"] for f in e["fields"]])
            return
        raise CheckError("GRAM-4", f"unknown expr kind {k}")

    def _check_call_args(self, arg_exprs):
        before = len(self.ctx.borrows)
        self.call_frames.append(before)
        for a in arg_exprs:
            self.check_expr(a)
        self.call_frames.pop()
        new = self.ctx.borrows[before:]
        for i in range(len(new)):
            for j in range(i + 1, len(new)):
                if places_overlap(new[i].place, new[j].place) and (new[i].uniq or new[j].uniq):
                    raise CheckError("OWN-12",
                        f"call arguments alias: {new[i].place} vs {new[j].place} "
                        f"with a uniq borrow among them")

    # -- helpers -------------------------------------------------------------
    def place_key(self, place) -> tuple:
        # accepts the flat form {base,path} (pre-v0.6 AST) and the nested v0.6
        # place tree {kind: var|deref|field|index}. `deref` is transparent to
        # the ownership layer (resolve() models read-through-borrow).
        if "kind" in place:
            k = place["kind"]
            if k == "var":
                return (place["name"],)
            if k == "deref":
                return self.place_key(place["place"])
            if k == "field":
                return (*self.place_key(place["place"]), place["name"])
            if k == "index":
                return self.place_key(place["place"])
            raise CheckError("GRAM-5", f"bad place kind {k}")
        return (place["base"], *place.get("path", []))

    def resolve(self, key: tuple) -> tuple:
        # [OWN-6] holder resolution: a place rooted at a binding that holds a
        # recorded borrow resolves to the borrowed place plus the suffix.
        seen = set()
        while key[0] not in seen:
            seen.add(key[0])
            held = next((br for br in self.ctx.borrows if br.holder == key[0]), None)
            if held is None:
                return key
            key = (*held.place, *key[1:])
        return key

    def require_live(self, base: str) -> Binding:
        b = self.ctx.bindings.get(base)
        if b is None:
            raise CheckError("TYPE-5", f"unknown binding {base}")
        if b.state == "moved":
            raise CheckError("OWN-1", f"use of moved value {base}")
        return b

    def require_readable(self, key: tuple, holder: str = ""):
        # [OWN-5] reading overlapping places conflicts with a live mutable
        # borrow unless the read is through that borrow's holder
        for br in self.ctx.borrows:
            if br.uniq and places_overlap(br.place, key) and br.holder != holder:
                raise CheckError("OWN-5",
                    f"read of {key} while uniq borrow of {br.place} is live")

    def require_writable(self, key: tuple, holder: str = ""):
        b = self.require_live(holder or key[0])
        writing_through_holder = any(
            br.holder == holder and br.uniq for br in self.ctx.borrows)
        if b.mode["kind"] == "ref" and not b.mode.get("uniq") and not writing_through_holder:
            raise CheckError("OWN-5", f"write through shared borrow {holder}")
        for br in self.ctx.borrows:
            if places_overlap(br.place, key) and br.holder != holder:
                raise CheckError("OWN-5",
                    f"write to {key} while borrow of {br.place} is live")


def check_fn(fn: dict):
    """Returns None on acceptance; raises CheckError with a rule ID on rejection."""
    Checker(fn).check()


# ===========================================================================
# v0.6 TYPE LAYER  (check_program): decl tables + a type on every binding.
# Enforces GRAM-8 (named construction), GRAM-10 (named match binders),
# GRAM-11 (named user-fn call args / positional table ops), TYPE-7 (explicit
# deref; borrow/box/arena-where-value is a type error), TYPE-6 (globally
# unique variant names), and basic type agreement (cited TYPE-5). Ownership
# (OWN-1..13) is checked by reusing the Checker above.
#
# TY   = {"kind":"prim","name":str} | {"kind":"named","name":str}
#      | {"kind":"box","elem":TY} | {"kind":"buffer","elem":TY}
#      | {"kind":"array","elem":TY,"n":int} | {"kind":"slice","region":str,"elem":TY}
#      | {"kind":"arena","region":str,"elem":TY} | {"kind":"unit"} | {"kind":"any"}
# PLACE (v0.6) = {"kind":"var","name":str} | {"kind":"deref","place":PLACE}
#      | {"kind":"field","place":PLACE,"name":str}
#      | {"kind":"index","place":PLACE,"elem":TY,"atom":EXPR}
# ===========================================================================

PRIMS = {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f32", "f64"}
NAMED_BOOL = {"kind": "named", "name": "Bool"}
NAMED_RESULT = {"kind": "named", "name": "Result"}
ANY = {"kind": "any"}

# Prelude enums [PRE-1]: Option/Result payloads are named; generic payloads are
# typed `any` (v0.6 democ is monomorphic; generics are out of the surface layer).
PRELUDE_ENUMS = {
    "Bool": [{"variant": "True", "fields": []}, {"variant": "False", "fields": []}],
    "Option": [{"variant": "None", "fields": []},
               {"variant": "Some", "fields": [{"name": "value", "ty": ANY}]}],
    "Result": [{"variant": "Ok", "fields": [{"name": "value", "ty": ANY}]},
               {"variant": "Err", "fields": [{"name": "error", "ty": ANY}]}],
    "Overflow": [{"variant": "Overflow", "fields": []}],
    "DivError": [{"variant": "DivideByZero", "fields": []},
                 {"variant": "DivOverflow", "fields": []}],
    "NarrowError": [{"variant": "NarrowError", "fields": []}],
}


def _ty_eq(a, b) -> bool:
    if a is None or b is None:
        return True
    if a.get("kind") == "any" or b.get("kind") == "any":
        return True
    if a["kind"] != b["kind"]:
        return False
    k = a["kind"]
    if k == "prim" or k == "named":
        return a["name"] == b["name"]
    if k == "unit":
        return True
    if k in ("box", "buffer"):
        return _ty_eq(a["elem"], b["elem"])
    if k == "array":
        return a.get("n") == b.get("n") and _ty_eq(a["elem"], b["elem"])
    if k in ("slice", "arena"):
        return a.get("region") == b.get("region") and _ty_eq(a["elem"], b["elem"])
    return False


def _ty_str(t) -> str:
    if t is None:
        return "?"
    k = t.get("kind")
    if k in ("prim", "named"):
        return t["name"]
    if k == "unit":
        return "unit"
    if k == "any":
        return "any"
    if k in ("box", "buffer"):
        return f"{k}<{_ty_str(t['elem'])}>"
    if k == "array":
        return f"array<{_ty_str(t['elem'])}, {t.get('n')}>"
    if k in ("slice", "arena"):
        return f"{k}<'{t.get('region')}, {_ty_str(t['elem'])}>"
    return str(t)


def op_type(callee: str, tyargs):
    """Return (param_specs, result_spec) for a table op, or None if unknown
    (unknown ops are typed leniently). Each spec is (cat, TY) with cat 'val'."""
    T = tyargs[0] if tyargs else ANY
    parts = callee.split(".")
    base, mode = parts[0], (parts[1] if len(parts) > 1 else None)
    cmps = {"ieq", "ine", "ilt", "ile", "igt", "ige",
            "feq", "flt", "fle", "fgt", "fge", "fne"}
    if callee in cmps:
        return ([("val", T), ("val", T)], ("val", NAMED_BOOL))
    bin_int = {"iadd", "isub", "imul", "idiv", "irem", "iand", "ior", "ixor",
               "imin", "imax", "imulhi", "ishl", "ishr", "irotl", "irotr",
               "fadd", "fsub", "fmul", "fdiv"}
    if base in bin_int:
        if mode == "checked":
            return ([("val", T), ("val", T)], ("val", NAMED_RESULT))
        return ([("val", T), ("val", T)], ("val", T))
    un_int = {"ineg", "iabs", "inot"}
    if base in un_int:
        if mode == "checked":
            return ([("val", T)], ("val", NAMED_RESULT))
        return ([("val", T)], ("val", T))
    if base in {"band", "bor", "bxor"}:
        return ([("val", NAMED_BOOL), ("val", NAMED_BOOL)], ("val", NAMED_BOOL))
    if base == "bnot":
        return ([("val", NAMED_BOOL)], ("val", NAMED_BOOL))
    return None


class Program:
    """Declaration tables built once per program (structs, enums, fns, prelude)."""

    def __init__(self, prog: dict):
        self.structs = {n: list(fs) for n, fs in prog.get("structs", {}).items()}
        self.enums = {n: list(vs) for n, vs in PRELUDE_ENUMS.items()}
        for en, variants in prog.get("enums", {}).items():
            self.enums[en] = list(variants)
        self.fns = prog.get("fns", {})
        self.variant_of = {}   # variant name -> (enum name, fields)  [TYPE-6 global]
        for en, variants in self.enums.items():
            for v in variants:
                name = v["variant"]
                if name in self.variant_of and self.variant_of[name][0] != en:
                    raise CheckError("TYPE-6",
                        f"enum variant {name} declared in both "
                        f"{self.variant_of[name][0]} and {en} (globally unique)")
                self.variant_of[name] = (en, v["fields"])


class TypeChecker:
    """Type layer for one function; ownership is checked separately by Checker."""

    def __init__(self, fn: dict, P: Program):
        self.fn = fn
        self.P = P
        self.env = {}        # name -> (mode, TY)
        self.deliver = []    # stack of (mode, TY) for give [GIVE-1]

    def check(self):
        for p in self.fn["params"]:
            self.env[p["name"]] = (p["mode"], p["ty"])
        self.check_block(self.fn["body"])

    # -- statements ----------------------------------------------------------
    def check_block(self, body):
        for s in body:
            self.check_stmt(s)

    def check_stmt(self, s):
        k = s["kind"]
        if k == "doc":
            return
        if k == "let":
            if s["name"] in self.env:                      # TYPE-6: no shadowing/redeclaration
                raise CheckError("TYPE-6",
                    f"redeclaration of live name '{s['name']}' (no shadowing)")
            if s["init"].get("kind") == "match":
                self.deliver.append((s["mode"], s["ty"]))
                self.check_match(s["init"])
                self.deliver.pop()
            else:
                d = self.expr_desc(s["init"])
                self.expect_mode_ty(d, s["mode"], s["ty"], f"let {s['name']}")
            self.env[s["name"]] = (s["mode"], s["ty"])
        elif k == "set":
            d = self.place_desc(s["place"])
            if d["cat"] != "val":
                raise CheckError("TYPE-7",
                    "set target is a borrow; write through deref(.)")
            de = self.expr_desc(s["expr"])
            self.expect_value(de, d["ty"], "set")
        elif k == "return":
            d = self.expr_desc(s["expr"])
            self.expect_mode_ty(d, self.fn["rmode"], self.fn["rty"], "return")
        elif k == "give":
            if not self.deliver:
                raise CheckError("GIVE-1", "give outside a let-initializer match")
            mode, ty = self.deliver[-1]
            d = self.expr_desc(s["expr"])
            self.expect_mode_ty(d, mode, ty, "give")
        elif k == "expr":
            self.expr_desc(s["expr"])
        elif k == "check":
            d = self.expr_desc(s["expr"])
            self.expect_value(d, NAMED_BOOL, "check condition")
        elif k == "region":
            self.check_block(s["body"])
        elif k == "loop":
            self.check_block(s["body"])
        elif k == "break":
            return
        elif k == "match":
            self.check_match(s)
        else:
            raise CheckError("GRAM-4", f"unknown stmt kind {k}")

    def check_match(self, s):
        sd = self._scrut_desc(s["scrut"])
        scrut_ty = sd["ty"]
        variants = None
        if scrut_ty.get("kind") == "named" and scrut_ty["name"] in self.P.enums:
            variants = {v["variant"]: v["fields"]
                        for v in self.P.enums[scrut_ty["name"]]}
        binder_is_ref = (sd["cat"] == "ref")
        for arm in s["arms"]:
            saved = dict(self.env)
            vname = arm["variant"]
            if variants is not None:
                if vname not in variants:
                    raise CheckError("TYPE-6",
                        f"variant {vname} not in enum {scrut_ty['name']}")
                decl = variants[vname]
            elif vname in self.P.variant_of:
                decl = self.P.variant_of[vname][1]
            else:
                raise CheckError("TYPE-6", f"unknown variant {vname}")
            # GRAM-10: binder field names equal declared field names in order
            self._check_field_names(
                [{"name": b["field"]} for b in arm["binders"]], decl,
                "GRAM-10", f"match arm {vname}")
            for b, df in zip(arm["binders"], decl):
                if b["name"] in self.env:
                    raise CheckError("GRAM-10",
                        f"match binder {b['name']} is not fresh [TYPE-6]")
                bmode = ({"kind": "ref", "region": sd.get("region"),
                          "uniq": sd.get("uniq", False)} if binder_is_ref
                         else {"kind": "own"})
                self.env[b["name"]] = (bmode, df["ty"])
            self.check_block(arm["body"])
            self.env = saved

    def _scrut_desc(self, e):
        """A match scrutinee that is a place is MOVED by the match [OWN-13], so a bare
        place here is not an OWN-1 affine-use error; other scrutinees type normally."""
        if e["kind"] in ("use", "move"):
            return self.place_desc(e["place"])
        return self.expr_desc(e)

    @staticmethod
    def _is_copy(desc):                                     # OWN-1 copy classification
        # prim/unit copy; borrows copy; `any` is the generic-payload wildcard of unknown
        # affinity -> treat as copy (lenient), matching the type layer's generic handling.
        return (desc["cat"] == "ref"
                or desc["ty"].get("kind") in ("prim", "unit", "any"))

    # -- expression typing ---------------------------------------------------
    def expr_desc(self, e):
        k = e["kind"]
        if k == "lit":
            return {"cat": "val", "ty": e.get("ty", {"kind": "unit"})}
        if k == "move":
            return self.place_desc(e["place"])
        if k == "use":                                     # OWN-1: a bare use must be copy
            d = self.place_desc(e["place"])
            if not self._is_copy(d):
                raise CheckError("OWN-1",
                    f"bare use of affine value of type {_ty_str(d['ty'])}; write move")
            return d
        if k == "borrow":
            inner = self.place_desc(e["place"])
            return {"cat": "ref", "ty": inner["ty"],
                    "uniq": e["uniq"], "region": e["region"]}
        if k == "construct":
            return self.check_construct(e)
        if k == "call":
            return self.check_call(e)
        raise CheckError("GRAM-5", f"unknown expr kind {k}")

    def place_desc(self, place):
        k = place["kind"]
        if k == "var":
            if place["name"] not in self.env:
                raise CheckError("TYPE-5", f"unknown binding {place['name']}")
            mode, ty = self.env[place["name"]]
            if mode["kind"] == "ref":
                return {"cat": "ref", "ty": ty,
                        "uniq": mode.get("uniq", False), "region": mode["region"]}
            return {"cat": "val", "ty": ty}
        if k == "deref":
            d = self.place_desc(place["place"])
            if d["cat"] == "ref":
                return {"cat": "val", "ty": d["ty"]}           # borrow referent
            if d["ty"].get("kind") in ("box", "arena"):
                return {"cat": "val", "ty": d["ty"]["elem"]}   # box/arena content
            raise CheckError("TYPE-7",
                f"deref of non-reference of type {_ty_str(d['ty'])}")
        if k == "field":
            d = self.place_desc(place["place"])
            if d["cat"] == "ref" or d["ty"].get("kind") in ("box", "arena"):
                raise CheckError("TYPE-7",
                    "field access through a reference requires deref(.)")
            if d["ty"].get("kind") != "named" or d["ty"]["name"] not in self.P.structs:
                raise CheckError("TYPE-5",
                    f"field access on non-struct type {_ty_str(d['ty'])}")
            for f in self.P.structs[d["ty"]["name"]]:
                if f["name"] == place["name"]:
                    return {"cat": "val", "ty": f["ty"]}
            raise CheckError("TYPE-5",
                f"struct {d['ty']['name']} has no field {place['name']}")
        if k == "index":
            d = self.place_desc(place["place"])
            if d["ty"].get("kind") in ("array", "buffer", "slice"):
                return {"cat": "val", "ty": d["ty"]["elem"]}
            raise CheckError("TYPE-5", f"index of non-indexable {_ty_str(d['ty'])}")
        raise CheckError("GRAM-5", f"bad place kind {k}")

    def check_construct(self, e):
        name = e["name"]
        if name in self.P.structs:
            decl = self.P.structs[name]
            result_ty = {"kind": "named", "name": name}
        elif name in self.P.variant_of:
            en, decl = self.P.variant_of[name]
            result_ty = {"kind": "named", "name": en}
        else:
            raise CheckError("TYPE-6", f"unknown constructor {name}")
        self._check_field_names(e["fields"], decl, "GRAM-8", name)
        for fld, df in zip(e["fields"], decl):
            d = self.expr_desc(fld["atom"])
            self.expect_value(d, df["ty"], f"field {name}.{df['name']}")
        return {"cat": "val", "ty": result_ty}

    def check_call(self, e):
        callee = e["callee"]
        if callee in self.P.fns:                       # user fn: GRAM-11 named args
            sig = self.P.fns[callee]
            params = sig["params"]
            argnames = e.get("argnames")
            if argnames is None:
                raise CheckError("GRAM-11",
                    f"call to user fn {callee} must name arguments "
                    f"{[p['name'] for p in params]}")
            self._check_field_names(
                [{"name": n} for n in argnames],
                [{"name": p["name"]} for p in params], "GRAM-11", f"call {callee}")
            for arg, p in zip(e["args"], params):
                d = self.expr_desc(arg)
                self.expect_mode_ty(d, p["mode"], p["ty"],
                                    f"arg {p['name']} of {callee}")
            return self._mode_desc(sig["rmode"], sig["rty"])
        # table operation: positional operands [GRAM-11]
        if e.get("argnames") is not None:
            raise CheckError("GRAM-11",
                f"table op {callee} takes positional operands, not named")
        sig = op_type(callee, e.get("tyargs", []))
        if sig is None:                                # unknown op: lenient
            for arg in e["args"]:
                self.expr_desc(arg)
            return {"cat": "val", "ty": ANY}
        params, (rcat, rty) = sig
        if len(e["args"]) != len(params):
            raise CheckError("GRAM-11",
                f"op {callee} expects {len(params)} operand(s), got {len(e['args'])}")
        for arg, (_cat, ty) in zip(e["args"], params):
            d = self.expr_desc(arg)
            self.expect_value(d, ty, f"operand of {callee}")
        return {"cat": rcat, "ty": rty}

    # -- agreement helpers ---------------------------------------------------
    def _mode_desc(self, mode, ty):
        if mode["kind"] == "ref":
            return {"cat": "ref", "ty": ty,
                    "uniq": mode.get("uniq", False), "region": mode.get("region")}
        return {"cat": "val", "ty": ty}

    def expect_value(self, desc, ty, ctx):
        if desc["cat"] == "ref":
            raise CheckError("TYPE-7",
                f"{ctx}: borrow used where value of type {_ty_str(ty)} "
                f"expected; use deref(.)")
        if not _ty_eq(desc["ty"], ty):
            dt = desc["ty"]
            if dt.get("kind") in ("box", "arena") and _ty_eq(dt["elem"], ty):
                raise CheckError("TYPE-7",
                    f"{ctx}: {dt['kind']} used where value of type "
                    f"{_ty_str(ty)} expected; use deref(.)")
            raise CheckError("TYPE-5",
                f"{ctx}: type {_ty_str(desc['ty'])} != expected {_ty_str(ty)}")

    def expect_mode_ty(self, desc, mode, ty, ctx):
        if mode["kind"] == "own":
            self.expect_value(desc, ty, ctx)
        else:                                          # ref: expect a borrow
            if desc["cat"] != "ref":
                raise CheckError("TYPE-5",
                    f"{ctx}: expected a borrow, got value of type "
                    f"{_ty_str(desc['ty'])}")
            if not _ty_eq(desc["ty"], ty):
                raise CheckError("TYPE-5",
                    f"{ctx}: borrow referent {_ty_str(desc['ty'])} "
                    f"!= {_ty_str(ty)}")

    @staticmethod
    def _check_field_names(actual, declared, rule, ctx):
        """GRAM-8/10/11: names present, correct, declared-order, right arity."""
        if len(actual) != len(declared):
            raise CheckError(rule,
                f"{ctx}: expected {len(declared)} field(s) "
                f"{[d['name'] for d in declared]}, got {len(actual)}")
        for a, d in zip(actual, declared):
            if a["name"] != d["name"]:
                raise CheckError(rule,
                    f"{ctx}: field name '{a['name']}' != declared '{d['name']}' "
                    f"(declared order {[x['name'] for x in declared]})")


def check_program(prog: dict):
    """Type layer + ownership over a whole program.

    prog = {"structs": {Name: [{"name","ty"}]},
            "enums":   {Enum: [{"variant","fields":[{"name","ty"}]}]},
            "fns":     {Fn: {"regions","params":[{"name","mode","ty"}],
                             "rmode","rty","body":[STMT]}}}
    Returns None on acceptance; raises CheckError(rule_id, msg) on rejection.
    """
    P = Program(prog)
    for name, fn in prog["fns"].items():
        TypeChecker(fn, P).check()             # GRAM-8/10/11, TYPE-5/6/7, GIVE-1
        Checker(fn).check()                    # OWN-1..13 (reused machinery)
