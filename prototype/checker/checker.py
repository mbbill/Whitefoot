"""Checker-core prototype (kernel-spec-v0.6).

Implements the v0.6 ownership calculus (OWN-1..13 incl. the 2026-07-10 tag-only-copy
amendment, CONST-2, GIVE-1, ERR-3 flow, EFF-1/2 rows-and-exhibits)
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
    is_const: bool = False          # [CONST-2] read-only static: no move/set/&uniq
    ty: Optional[dict] = None       # declared type, for copy classification [OWN-1]


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
        self.copy_enums = frozenset()   # tag-only enum names: copy per OWN-1 amendment

    # -- entry ---------------------------------------------------------------
    def check(self):
        requirements = (self.fn.get("requires") or [])
        if requirements:
            # [FN-8] A requires clause is a checked, parameter-only prologue.
            # Its helper bindings are clause-local: they neither collide with
            # nor become visible to the function body.
            self._seed_entry(include_consts=False)
            self.check_block(requirements)
        self._seed_entry(include_consts=True)
        self.check_block(self.fn["body"])

    def _seed_entry(self, include_consts):
        self.ctx = Ctx()
        self.loops = []
        self.call_frames = []
        self.deliver = []
        for r in self.fn.get("regions", []):
            self.ctx.regions.append(r)         # caller regions: outermost
        if include_consts:
            for name in getattr(self, "consts", ()):    # [CONST-2] body-only static bindings
                cb = Binding(name, {"kind": "own"}, 0)
                cb.is_const = True
                self.ctx.bindings[name] = cb
        for p in self.fn["params"]:
            b = Binding(p["name"], p["mode"], self.ctx.depth())
            b.ty = p.get("ty")
            if p["mode"]["kind"] == "ref":
                b.region = p["mode"]["region"]
            self.ctx.bindings[p["name"]] = b

    # -- statements ----------------------------------------------------------
    def check_block(self, stmts):
        for s in stmts:
            self.check_stmt(s)
            # [OWN-6] call-scoped temporaries die at end of the enclosing statement
            self.ctx.borrows = [b for b in self.ctx.borrows if b.holder is not None]

    def check_stmt(self, s):
        k = s["kind"]
        if k == "try":                                 # [ERR-3] flow: consumes operand
            ex = s["expr"]
            if isinstance(ex, dict) and ex.get("kind") == "use":
                ex = {"kind": "move", "place": ex["place"]}
            self.check_expr(ex)
            self.ctx.bindings[s["name"]] = Binding(s["name"], {"kind": "own"},
                                                   self.ctx.depth())
            return
        if k == "let":
            # v0.6: a `let` may be initialized by a value-match [GIVE-1]
            if s["init"].get("kind") == "match":
                self.deliver.append(s["mode"])
                self._check_match(s["init"])
                self.deliver.pop()
            else:
                self.check_expr(s["init"], expect_mode=s["mode"])
            b = Binding(s["name"], s["mode"], self.ctx.depth())
            b.ty = s.get("ty")
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

    def _copy_ty(self, ty):
        """copy classification for the flow layer [OWN-1 amendment]: primitives,
        unit, and tag-only enums never consume on use."""
        if not isinstance(ty, dict): return False
        k = ty.get("kind")
        if k in ("prim", "unit", "any"): return True
        return k == "named" and ty.get("name") in self.copy_enums

    # -- match core (shared by statement- and let-initializer matches) --------
    def _check_match(self, s):
        # [OWN-13]/[T-B]: binder modes DERIVED from scrutinee; own moves,
        # borrow-mode scrutinee stays live and binders alias its content
        scrut = s["scrut"]; binder_mode = {"kind": "own"}; alias_of = None
        if scrut.get("kind") == "use":
            base = self.place_key(scrut["place"])[0]
            sb = self.require_live(base)
            if sb.mode["kind"] == "own" and not self._copy_ty(sb.ty):
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
            if getattr(b, "is_const", False):
                raise CheckError("CONST-2", f"a const item ({key[0]}) is never moved")
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
            if getattr(b, "is_const", False) and e.get("uniq"):
                raise CheckError("CONST-2", f"a const item ({key[0]}) cannot be &uniq-borrowed")
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
        if getattr(b, "is_const", False):
            raise CheckError("CONST-2", f"a const item ({key[0]}) is immutable; no set")
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

def _res_of(T, err):
    return {"kind": "named", "name": "Result",
            "args": [T, {"kind": "named", "name": err}]}
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
        if a["name"] != b["name"]:
            return False
        aa, ba = a.get("args"), b.get("args")
        if aa is None or ba is None:                   # erased side: name-compat [ERR-3 leniency]
            return True
        return len(aa) == len(ba) and all(_ty_eq(x, y) for x, y in zip(aa, ba))
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
               "imin", "imax", "imulhi",
               "fadd", "fsub", "fmul", "fdiv"}
    if base in bin_int:
        if mode == "checked":
            err = "DivError" if base in ("idiv", "irem") else "Overflow"
            return ([("val", T), ("val", T)], ("val", _res_of(T, err)))
        return ([("val", T), ("val", T)], ("val", T))
    if base in {"ishl", "ishr", "irotl", "irotr"}:   # [OP-1/8] amount is always u32
        U32 = {"kind": "prim", "name": "u32"}
        return ([("val", T), ("val", U32)], ("val", T))
    un_int = {"ineg", "iabs", "inot"}
    if base in un_int:
        if mode == "checked":
            return ([("val", T)], ("val", _res_of(T, "Overflow")))
        return ([("val", T)], ("val", T))
    if base in {"band", "bor", "bxor"}:
        return ([("val", NAMED_BOOL), ("val", NAMED_BOOL)], ("val", NAMED_BOOL))
    if base == "bnot":
        return ([("val", NAMED_BOOL)], ("val", NAMED_BOOL))
    if base == "buffer_new":                           # [OP-9] (u64, T) -> buffer<T>
        return ([("val", {"kind": "prim", "name": "u64"}), ("val", T)],
                ("val", {"kind": "buffer", "elem": T}))
    if base == "len":                                  # [OP-1] -> u64 (operand typed leniently)
        return ([("val", ANY)], ("val", {"kind": "prim", "name": "u64"}))
    return None


# [FN-8] `requires` is deliberately not a second expression language.  This
# closed prototype set is the intersection of OP-1's pure, total,
# primitive/Bool-returning rows and operations whose signatures + lowerings
# this compiler implements exactly.
# Result-returning `.checked` operations are omitted even though they are total:
# unpacking a Result would require control/construct forms forbidden in a clause.
FN8_TOTAL_VALUE_OPS = frozenset({
    "iadd.wrap", "isub.wrap", "imul.wrap",
    "iadd.sat", "isub.sat",
    "ieq", "ine", "ilt", "ile", "igt", "ige",
    "band", "bor", "bxor", "bnot",
    "len",
    "iand", "ior", "ixor",
    "ishl.wrap", "ishr.wrap", "irotl", "irotr",
    "imin", "imax",
})


def _fn8_copy_result_ty(ty):
    """The clause may bind only resource-free values understood exactly here."""
    return (isinstance(ty, dict)
            and (ty.get("kind") == "prim"
                 or (ty.get("kind") == "named" and ty.get("name") == "Bool")))


def _fn8_len_place(place, params, fnname):
    """Accept exactly len(buffer-param) or len(deref(direct-ref-param))."""
    if not isinstance(place, dict):
        raise CheckError("FN-8", f"{fnname}: requires len operand is not a place")
    if place.get("kind") == "var":
        pname = place.get("name")
        p = params.get(pname)
        if p is None or p.get("ty", {}).get("kind") not in ("buffer", "array", "slice"):
            raise CheckError("FN-8",
                f"{fnname}: requires len operand must be a direct buffer/array/slice parameter")
        return
    if place.get("kind") == "deref":
        inner = place.get("place")
        if not isinstance(inner, dict) or inner.get("kind") != "var":
            raise CheckError("FN-8",
                f"{fnname}: requires len deref must name one direct reference parameter")
        p = params.get(inner.get("name"))
        if (p is None or p.get("mode", {}).get("kind") != "ref"
                or p.get("ty", {}).get("kind") not in ("buffer", "array", "slice")):
            raise CheckError("FN-8",
                f"{fnname}: requires len deref must name a buffer/array/slice reference parameter")
        return
    raise CheckError("FN-8",
        f"{fnname}: requires len operand must be a direct parameter (optionally one deref)")


def _fn8_atom(expr, available, fnname):
    if not isinstance(expr, dict):
        raise CheckError("FN-8", f"{fnname}: malformed requires atom")
    if expr.get("kind") == "lit":
        return
    if expr.get("kind") != "use":
        raise CheckError("FN-8",
            f"{fnname}: requires operands are literals or direct names; "
            "moves, borrows, constructs, indexes, and nested calls are forbidden")
    place = expr.get("place")
    if not isinstance(place, dict) or place.get("kind") != "var":
        raise CheckError("FN-8",
            f"{fnname}: requires operand must be a direct parameter or earlier clause local")
    name = place.get("name")
    if name not in available:
        raise CheckError("FN-8",
            f"{fnname}: requires operand '{name}' is not a parameter or earlier clause local")


def _fn8_expr(expr, available, params, fnname, require_call=False):
    if not isinstance(expr, dict):
        raise CheckError("FN-8", f"{fnname}: malformed requires expression")
    if expr.get("kind") != "call":
        if require_call:
            raise CheckError("FN-8",
                f"{fnname}: every requires let must bind one pure total table operation")
        _fn8_atom(expr, available, fnname)
        return

    callee = expr.get("callee")
    if callee not in FN8_TOTAL_VALUE_OPS:
        raise CheckError("FN-8",
            f"{fnname}: requires operation '{callee}' is not a permitted pure total "
            "primitive/Bool table operation")
    if expr.get("argnames") is not None:
        raise CheckError("FN-8",
            f"{fnname}: requires permits table operations only, never user-function calls")
    sig = op_type(callee, expr.get("tyargs", []))
    if sig is None or not _fn8_copy_result_ty(sig[1][1]):
        raise CheckError("FN-8",
            f"{fnname}: requires operation '{callee}' lacks an exact copy-result signature")
    args = expr.get("args", [])
    if callee == "len":
        if len(args) != 1 or not isinstance(args[0], dict) or args[0].get("kind") != "use":
            raise CheckError("FN-8",
                f"{fnname}: requires len takes one direct place operand")
        _fn8_len_place(args[0].get("place"), params, fnname)
        return
    for arg in args:
        _fn8_atom(arg, available, fnname)


def check_requires(fn):
    """[FN-8] Validate the closed, checked precondition-prologue fragment.

    Type agreement is intentionally left to TypeChecker so ordinary TYPE/OWN
    diagnostics remain stable; this pass rejects only the clause's structure and
    authority surface.
    """
    requirements = fn.get("requires")
    if requirements is None:
        return
    if not requirements:                       # block present but no effective statements
        raise CheckError("FN-8",
            "requires must contain zero or more lets followed by one final check "
            "(doc statements alone do not form a clause)")
    fnname = fn.get("name", "<anonymous>")
    params = {p["name"]: p for p in fn.get("params", [])}
    available = set(params)
    if requirements[-1].get("kind") != "check":
        raise CheckError("FN-8",
            f"{fnname}: nonempty requires clause must end in exactly one check")
    for stmt in requirements[:-1]:
        if stmt.get("kind") != "let":
            raise CheckError("FN-8",
                f"{fnname}: requires contains only let statements followed by one final check")
        name = stmt.get("name")
        if name in available:
            raise CheckError("FN-8",
                f"{fnname}: requires local '{name}' is not fresh within its parameter-only scope")
        if stmt.get("mode", {}).get("kind") != "own" or not _fn8_copy_result_ty(stmt.get("ty")):
            raise CheckError("FN-8",
                f"{fnname}: requires let '{name}' must bind an own primitive or Bool")
        _fn8_expr(stmt.get("init"), available, params, fnname, require_call=True)
        available.add(name)
    _fn8_expr(requirements[-1].get("expr"), available, params, fnname)


class Program:
    """Declaration tables built once per program (structs, enums, fns, prelude)."""

    def __init__(self, prog: dict):
        self.structs = {n: list(fs) for n, fs in prog.get("structs", {}).items()}
        self.enums = {n: list(vs) for n, vs in PRELUDE_ENUMS.items()}
        for en, variants in prog.get("enums", {}).items():
            self.enums[en] = list(variants)
        self.fns = prog.get("fns", {})
        self.consts = prog.get("consts", {})   # const name -> TY  [CONST-2]
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
        params = {p["name"]: (p["mode"], p["ty"]) for p in self.fn["params"]}
        requirements = (self.fn.get("requires") or [])
        if requirements:
            # [FN-8] Constants and body locals are outside the clause.  Discard
            # every clause-local binding before checking the ordinary body.
            self.env = dict(params)
            self.check_block(requirements)
        self.env = {name: ({"kind": "own"}, ty)
                    for name, ty in self.P.consts.items()}  # [CONST-2] body-only statics
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
            if d["ty"].get("kind") == "buffer":
                raise CheckError("STOR-1",
                    "whole-buffer replacement is outside v0 pending the "
                    "take/replace + old-storage release rule; mutate indexed elements instead")
            de = self.expr_desc(s["expr"])
            self.expect_value(de, d["ty"], "set")
        elif k == "return":
            ex = s["expr"]
            if (self.fn["rmode"]["kind"] == "own" and isinstance(ex, dict)
                    and ex.get("kind") == "use"
                    and not self._is_copy(self.place_desc(ex["place"]))):
                ex = {"kind": "move", "place": ex["place"]}   # return consumes affine [OWN-1/FR T-Move]
            d = self.expr_desc(ex)
            self.expect_mode_ty(d, self.fn["rmode"], self.fn["rty"], "return")
        elif k == "give":
            if not self.deliver:
                raise CheckError("GIVE-1", "give outside a let-initializer match")
            mode, ty = self.deliver[-1]
            d = self.expr_desc(s["expr"])
            self.expect_mode_ty(d, mode, ty, "give")
        elif k == "try":                               # [ERR-3] let x: own T = try e;
            if s["name"] in self.env:
                raise CheckError("TYPE-6",
                    f"redeclaration of live name '{s['name']}' (no shadowing)")
            ex = s["expr"]
            if (isinstance(ex, dict) and ex.get("kind") == "use"
                    and not self._is_copy(self.place_desc(ex["place"]))):
                ex = {"kind": "move", "place": ex["place"]}   # try consumes its Result
            d = self.expr_desc(ex)
            dt = d["ty"]
            if not (dt.get("kind") == "named" and dt.get("name") == "Result"):
                raise CheckError("ERR-3",
                    f"try operand must be a Result, got {_ty_str(dt)}")
            args = dt.get("args")
            if args is None:
                raise CheckError("ERR-3",
                    "try operand's Result payload types are unknown (erased); "
                    "same-error-type propagation cannot be verified")
            okT, errT = args
            self.expect_value({"cat": "val", "ty": okT}, s["ty"], f"try {s['name']}")
            rt = self.fn["rty"]
            if not (isinstance(rt, dict) and rt.get("kind") == "named"
                    and rt.get("name") == "Result" and rt.get("args") is not None):
                raise CheckError("ERR-3",
                    "try requires the enclosing fn to return Result<U, E> with "
                    "known payload types")
            if not _ty_eq(errT, rt["args"][1]) or rt["args"][1].get("kind") == "any":
                raise CheckError("ERR-3",
                    f"try error type {_ty_str(errT)} != enclosing fn error type "
                    f"{_ty_str(rt['args'][1])} (same E required; no conversions)")
            self.env[s["name"]] = (s["mode"], okT)
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

    def _is_copy(self, desc):                               # OWN-1 copy classification
        # prim/unit copy; borrows copy; tag-only enums copy (resource-free; OWN-1
        # amendment 2026-07-10); `any` is the generic-payload wildcard of unknown
        # affinity -> treat as copy (lenient), matching the type layer's generic handling.
        if desc["cat"] == "ref":
            return not desc.get("uniq", False)             # OWN-1: shared copy, uniq affine
        if desc["ty"].get("kind") in ("prim", "unit", "any"):
            return True
        ty = desc["ty"]
        if ty.get("kind") == "named":
            vs = self.P.enums.get(ty.get("name"))
            return vs is not None and all(not v["fields"] for v in vs)
        return False

    # -- expression typing ---------------------------------------------------
    def expr_desc(self, e):
        k = e["kind"]
        if k == "lit":
            return {"cat": "val", "ty": e.get("ty", {"kind": "unit"})}
        if k == "move":
            d = self.place_desc(e["place"])
            if self._is_copy(d) and d["cat"] != "ref":     # OWN-1: one spelling — copies are used bare
                raise CheckError("OWN-1",
                    f"move of copy value of type {_ty_str(d['ty'])}; copy values are used bare")
            return d
        if k == "use":                                     # OWN-1: a bare use must be copy
            d = self.place_desc(e["place"])
            if not self._is_copy(d):
                if getattr(self, "_place_read_operand", False):   # len/slice_of read their
                    return d                                       # place operand, never consume
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
                # OWN-1 has one spelling for affine transfer. An own affine
                # argument must be written `move place`; silently interpreting
                # a bare use as a move here would bypass the flow-layer kill
                # and permit use-after-transfer.
                d = self.expr_desc(arg)
                self.expect_mode_ty(d, p["mode"], p["ty"],
                                    f"arg {p['name']} of {callee}")
            return self._mode_desc(sig["rmode"], sig["rty"])
        # table operation: positional operands [GRAM-11]
        if e.get("argnames") is not None:
            raise CheckError("GRAM-11",
                f"table op {callee} takes positional operands, not named")
        self._place_read_operand = callee in ("len", "slice_of")
        try:
            sig = op_type(callee, e.get("tyargs", []))
            if sig is None:                            # unknown op: lenient
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
        finally:
            self._place_read_operand = False

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
            if bool(desc.get("uniq")) != bool(mode.get("uniq")):
                actual = "&uniq" if desc.get("uniq") else "&"
                expected = "&uniq" if mode.get("uniq") else "&"
                raise CheckError("TYPE-5",
                    f"{ctx}: borrow mode {actual} != expected {expected}; "
                    "there is no implicit shared/exclusive conversion [OWN-2]")
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


EFF_RANK = {"reads": 0, "writes": 1, "allocates": 2, "traps": 3}


def _parse_effect_row(toks, fnname):
    """Validate an effect row [EFF-1]; return (kinds set, declares_traps)."""
    if toks == ["pure"]:
        return set(), False
    if not toks:
        raise CheckError("EFF-1", f"{fnname}: empty effect row; use 'pure'")
    if "pure" in toks:
        raise CheckError("EFF-1", f"{fnname}: 'pure' is the empty row; it cannot combine with effects")
    kinds, i, n = [], 0, len(toks)
    while i < n:
        k = toks[i]
        if k not in EFF_RANK:
            raise CheckError("EFF-1", f"{fnname}: unknown effect '{k}' (reads/writes/allocates/traps)")
        kinds.append(k); i += 1
        if k in ("reads", "writes"):
            if i >= n or toks[i] != "(":
                raise CheckError("EFF-1", f"{fnname}: {k} needs (REGIONID+)")
            i += 1; cnt = 0
            while i < n and toks[i] != ")":
                if not toks[i].startswith("'"):
                    raise CheckError("EFF-1", f"{fnname}: {k} takes regions")
                cnt += 1; i += 1
            if i >= n or cnt == 0:
                raise CheckError("EFF-1", f"{fnname}: {k} needs (REGIONID+)")
            i += 1
        elif k == "allocates":
            if i >= n or toks[i] != "(":
                raise CheckError("EFF-1", f"{fnname}: allocates needs (heap|arena REGIONID)+")
            i += 1; cnt = 0
            while i < n and toks[i] != ")":
                if toks[i] == "heap":
                    i += 1; cnt += 1
                elif toks[i] == "arena":
                    i += 1
                    if i >= n or not toks[i].startswith("'"):
                        raise CheckError("EFF-1", f"{fnname}: arena needs a REGIONID")
                    i += 1; cnt += 1
                else:
                    raise CheckError("EFF-1", f"{fnname}: allocates takes heap or arena REGIONID")
            if i >= n or cnt == 0:
                raise CheckError("EFF-1", f"{fnname}: allocates needs (heap|arena REGIONID)+")
            i += 1
        if i < n:
            if toks[i] != ",":
                raise CheckError("EFF-1", f"{fnname}: expected ',' between effects, got '{toks[i]}'")
            i += 1
            if i >= n:
                raise CheckError("EFF-1", f"{fnname}: trailing ',' in effect row")
    ranks = [EFF_RANK[k] for k in kinds]
    if ranks != sorted(ranks) or len(set(kinds)) != len(kinds):
        raise CheckError("EFF-1",
            f"{fnname}: effect row not in canonical order reads<writes<allocates<traps ({kinds})")
    return set(kinds), ("traps" in kinds)


def _exhibits_traps(fn, declared_traps):
    """EFF-2: do the checked prologue or body exhibit traps? (.trap op, check,
    bounds-checked index, or a call whose declared row includes traps)."""
    hit = [False]

    def place(p):
        if isinstance(p, dict) and not hit[0]:
            if p.get("kind") == "index":
                hit[0] = True
            if "place" in p:
                place(p["place"])

    def expr(e):
        if not isinstance(e, dict) or hit[0]:
            return
        k = e.get("kind")
        if k == "call":
            c = e.get("callee", "")
            if isinstance(c, str) and (c.endswith(".trap") or c == "buffer_new" or declared_traps.get(c)):
                hit[0] = True
            for a in e.get("args", []):
                expr(a)
        elif k == "construct":
            for f in e.get("fields", []):
                expr(f["atom"])
        elif k in ("use", "move", "borrow"):
            place(e.get("place"))

    def match(m):
        expr(m.get("scrut"))
        for arm in m.get("arms", []):
            for s in arm.get("body", []):
                stmt(s)

    def stmt(s):
        if hit[0]:
            return
        k = s.get("kind")
        if k == "check":
            hit[0] = True
        elif k == "let":
            init = s.get("init")
            if isinstance(init, dict) and init.get("kind") == "match":
                match(init)
            else:
                expr(init)
        elif k == "match":
            match(s)
        elif k in ("return", "give", "expr"):
            expr(s.get("expr"))
        elif k == "try":                       # [ERR-3] scrutinee can trap (e.g. index in a try)
            expr(s.get("expr"))
        elif k == "set":
            place(s.get("place")); expr(s.get("expr"))
        elif k in ("region", "loop"):
            for b in s.get("body", []):
                stmt(b)

    for s in (fn.get("requires") or []):
        stmt(s)
    for s in fn["body"]:
        stmt(s)
    return hit[0]


def check_effects(prog):
    """EFF-1 (row grammar/order) + EFF-2 (declared-vs-exhibited traps). No-op for a
    harness that omits effect rows (e.g. the ownership-only test AST)."""
    fns = prog["fns"]
    declared_traps = {}
    for name, fn in fns.items():
        declared_traps[name] = (_parse_effect_row(fn["effects"], name)[1]
                                if "effects" in fn else False)
    for name, fn in fns.items():
        if "effects" not in fn:
            continue
        ex = _exhibits_traps(fn, declared_traps)
        if ex and not declared_traps[name]:
            raise CheckError("EFF-2",
                f"{name}: body exhibits traps but the row omits it (undeclared-but-exhibited)")
        if declared_traps[name] and not ex:
            raise CheckError("EFF-2",
                f"{name}: row declares traps but the body does not exhibit it (declared-but-unexhibited)")


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
        check_requires({**fn, "name": name})         # FN-8 checked parameter-only prologue
        TypeChecker(fn, P).check()             # GRAM-8/10/11, TYPE-5/6/7, GIVE-1
        ch = Checker(fn)
        ch.copy_enums = frozenset(en for en, vs in P.enums.items()
                                  if all(not v["fields"] for v in vs))
        ch.consts = P.consts                   # [CONST-2] seed read-only static bindings
        ch.check()                             # OWN-1..13 (reused machinery)
    check_effects(prog)                        # EFF-1 row order, EFF-2 traps exhibits
