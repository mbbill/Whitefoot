"""Retained checker-core reference model for a kernel-spec-v0.8 subset.

This focused historical model is not the active v0.9 compiler or a complete
v0.9 semantic oracle. Its still-relevant judgments remain independent evidence.

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


# [OP-1/FORM-3] These spellings name table operations or operation modes, so
# they cannot bind user identifiers.  Keeping the inventory here gives both
# the source parser and direct checker clients one closed source of truth.
DOTLESS_OPERATION_IDENTS = frozenset({
    "ieq", "ine", "ilt", "ile", "igt", "ige",
    "eeq", "ene",
    "feq", "flt", "fle", "fgt", "fge", "fne",
    "band", "bor", "bxor", "bnot",
    "cvt", "len", "slice_of", "box_new", "arena_new", "array_new", "buffer_new",
    "iand", "ior", "ixor", "inot", "irotl", "irotr", "ipopcount", "iclz", "ictz",
    "ibswap", "imulhi", "imin", "imax", "reinterpret",
    "fneg", "fabs", "fcopysign", "fmin", "fmax", "ffloor", "fceil", "ftrunc",
    "froundeven", "frem", "finf", "fnan",
})
OPERATION_MODE_WORDS = frozenset({"wrap", "trap", "checked", "sat", "strict"})
RESERVED_BINDING_IDENTS = (
    DOTLESS_OPERATION_IDENTS | OPERATION_MODE_WORDS | frozenset({"requires"})
)


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
    derived_from: Optional[str] = None  # parent holder for match-derived borrows


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


def _referenced_regions(value) -> set:
    """Every REGIONID named in a mode subtree (params, return mode, effects)."""
    result = set()
    if isinstance(value, dict):
        if value.get("kind") == "ref" and value.get("region"):
            result.add(value["region"])
        for v in value.values():
            result |= _referenced_regions(v)
    elif isinstance(value, list):
        for item in value:
            result |= _referenced_regions(item)
    return result


class Checker:
    def __init__(self, fn: dict):
        self.fn = fn
        self.ctx = Ctx()
        self.loops = []   # (region depth at loop entry, binding names at entry)
        self.call_frames = []   # ctx.borrows length at call-argument start
        self.deliver = []   # stack of let-init modes for `give` [GIVE-1] (v0.6)
        self.copy_enums = frozenset()   # tag-only enum names: copy per OWN-1 amendment
        self.structs = {}   # declaration table used to classify projected places
        self.match_binder_types = {}  # id(match-binder AST) -> concrete payload type

    # -- entry ---------------------------------------------------------------
    def check(self):
        self._check_signature_regions()
        requirements = (self.fn.get("requires") or [])
        if requirements:
            # [FN-8] A requires clause is a checked, parameter-only prologue.
            # Its helper bindings are clause-local: they neither collide with
            # nor become visible to the function body.
            self._seed_entry(include_consts=False)
            self.check_block(requirements)
        self._seed_entry(include_consts=True)
        self.check_block(self.fn["body"])

    def _check_signature_regions(self):
        # [OWN-3] every region named in the signature (param modes, return mode)
        # must be a declared region parameter; regions are lexical and are never
        # introduced by mention.
        declared = set(self.fn.get("regions", []))
        referenced = (_referenced_regions(self.fn.get("params", []))
                      | _referenced_regions(self.fn.get("rmode", {}) or {}))
        for r in sorted(referenced - declared):
            raise CheckError("OWN-3",
                f"signature region {r} is not a declared region parameter")

    def _seed_entry(self, include_consts):
        self.ctx = Ctx()
        self.loops = []
        self.call_frames = []
        self.deliver = []
        for r in self.fn.get("regions", []):
            self.ctx.regions.append(r)         # caller regions: outermost
        if include_consts:
            for name, ty in getattr(self, "consts", {}).items():  # [CONST-2] body-only statics
                cb = Binding(name, {"kind": "own"}, 0)
                cb.is_const = True
                cb.ty = ty
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
                self._require_explicit_deref(ex["place"], "try operand")
                ex = {"kind": "move", "place": ex["place"]}
            self.check_expr(ex)
            b = Binding(s["name"], {"kind": "own"}, self.ctx.depth())
            b.ty = s.get("ty")
            self.ctx.bindings[s["name"]] = b
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
            self._check_place_atoms(s["place"])
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
            ex = s["expr"]
            if (self.fn.get("rmode", {"kind": "own"})["kind"] == "own"
                    and isinstance(ex, dict) and ex.get("kind") == "use"):
                if not self._copy_place(ex["place"]):
                    ex = {"kind": "move", "place": ex["place"]}
            self.check_expr(ex, returning=True)
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

    def _field_ty(self, ty, name):
        if not isinstance(ty, dict) or ty.get("kind") != "named":
            return None
        for field in self.structs.get(ty.get("name"), []):
            if field.get("name") == name:
                return field.get("ty")
        return None

    def _place_desc(self, place):
        """Resolve a flow place's holder category and projected value type.

        Copy classification belongs to the projected value, not its affine root:
        a tag-only field or indexed element remains Copy even inside a unique owner.
        Dereference depth is mode-sensitive: dereferencing a reference exposes its
        referent, while dereferencing an owned box/arena exposes its element. This
        helper is deliberately side-effect free; index atoms are evaluated exactly
        once by `_check_place_atoms` at the enclosing access.
        """
        if "kind" not in place:
            binding = self.ctx.bindings.get(place.get("base"))
            ty = binding.ty if binding is not None else None
            for name in place.get("path", []):
                ty = self._field_ty(ty, name)
            cat = ("ref" if binding is not None
                   and binding.mode.get("kind") == "ref"
                   and not place.get("path") else "val")
            return {"cat": cat, "ty": ty}
        kind = place.get("kind")
        if kind == "var":
            binding = self.ctx.bindings.get(place.get("name"))
            return {"cat": ("ref" if binding is not None
                             and binding.mode.get("kind") == "ref" else "val"),
                    "ty": binding.ty if binding is not None else None}
        if kind == "deref":
            desc = self._place_desc(place["place"])
            if desc["cat"] == "ref":
                return {"cat": "val", "ty": desc["ty"]}
            ty = desc["ty"]
            if isinstance(ty, dict) and ty.get("kind") in ("box", "arena"):
                return {"cat": "val", "ty": ty.get("elem")}
            return {"cat": "val", "ty": None}
        if kind == "field":
            desc = self._place_desc(place["place"])
            return {"cat": "val",
                    "ty": self._field_ty(desc["ty"], place.get("name"))}
        if kind == "index":
            ty = self._place_desc(place["place"])["ty"]
            if isinstance(ty, dict) and ty.get("kind") in ("array", "buffer", "slice"):
                return {"cat": "val", "ty": ty.get("elem")}
            return {"cat": "val", "ty": None}
        return {"cat": "val", "ty": None}

    def _place_ty(self, place):
        return self._place_desc(place)["ty"]

    def _copy_place(self, place):
        return self._copy_ty(self._place_ty(place))

    def _require_explicit_deref(self, place, context):
        """Reject a holder that remains after the place's written dereferences."""
        desc = self._place_desc(place)
        ty = desc.get("ty")
        if desc.get("cat") == "ref" or (isinstance(ty, dict)
                                         and ty.get("kind") in ("box", "arena")):
            raise CheckError("TYPE-7",
                f"{context} is a reference/box/arena holder; write deref(.)")

    def _check_place_atoms(self, place):
        """Apply ordinary expression ownership to every index atom once."""
        if not isinstance(place, dict) or "kind" not in place:
            return
        inner = place.get("place")
        if isinstance(inner, dict):
            self._check_place_atoms(inner)
        if place.get("kind") == "index":
            self.check_expr(place["atom"])

    # -- match core (shared by statement- and let-initializer matches) --------
    def _check_match(self, s):
        # [OWN-13]/[T-B]: binder modes DERIVED from scrutinee; own moves,
        # borrow-mode scrutinee stays live and binders alias its content
        scrut = s["scrut"]; binder_mode = {"kind": "own"}; alias_of = None
        parent_holder = None
        if scrut.get("kind") == "borrow":
            raise CheckError("TYPE-7",
                "match scrutinee is a reference value; bind it and write deref(.)")
        if scrut.get("kind") == "use":
            self._require_explicit_deref(scrut["place"], "match scrutinee")
            base = self.place_key(scrut["place"])[0]
            sb = self.require_live(base)
            if sb.mode["kind"] == "own":
                if self._copy_place(scrut["place"]):
                    self.check_expr(scrut)
                else:
                    self.check_expr({"kind": "move", "place": scrut["place"]})
            else:
                self.check_expr(scrut)
                binder_mode = {"kind": "ref", "region": sb.region,
                               "uniq": sb.mode.get("uniq", False)}
                alias_of = self.resolve(self.place_key(scrut["place"]))
                parent_holder = base
        else:
            self.check_expr(scrut)      # owned temporary [OWN-13]
        snap_b = {n: copy.copy(b) for n, b in self.ctx.bindings.items()}
        snap_br = [copy.copy(x) for x in self.ctx.borrows]
        moved_any = set(); joined_borrows = []
        for arm in s["arms"]:
            self.ctx.bindings = {n: copy.copy(b) for n, b in snap_b.items()}
            self.ctx.borrows = [copy.copy(x) for x in snap_br]
            for binder in arm.get("binders", []):
                bn = self._bname(binder)
                b_ = Binding(bn, binder_mode, self.ctx.depth())
                b_.ty = self.match_binder_types.get(id(binder))
                if binder_mode["kind"] == "ref":
                    b_.region = binder_mode["region"]
                    field_name = (binder.get("field")
                                  if isinstance(binder, dict) else None)
                    binder_place = ((*alias_of, field_name)
                                    if field_name is not None else alias_of)
                    self.ctx.borrows.append(Borrow(
                        binder_place, binder_mode["uniq"], binder_mode["region"],
                        holder=bn, derived_from=parent_holder))
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
            self._check_place_atoms(e["place"])
            raw = self.place_key(e["place"])
            self.require_live(raw[0])
            self.require_readable(self.resolve(raw), holder=raw[0])
            return
        if k == "move":
            self._check_place_atoms(e["place"])
            key = self.place_key(e["place"])
            b = self.require_live(key[0])
            if getattr(b, "is_const", False):
                raise CheckError("CONST-2", f"a const item ({key[0]}) is never moved")
            if self._copy_place(e["place"]):
                raise CheckError("OWN-1",
                    f"move of copy binding {key[0]}; copy values are used bare")
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
            self._check_place_atoms(e["place"])
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
        # the ownership layer (resolve() models read-through-borrow). This is
        # a pure structural helper: callers evaluate index atoms exactly once
        # through `_check_place_atoms` before resolving the resulting place.
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

    def _borrow_descends_from(self, holder: str, ancestor: str) -> bool:
        """Whether holder is a match-derived projection of ancestor."""
        seen = set()
        current = holder
        while current and current not in seen:
            seen.add(current)
            br = next((candidate for candidate in self.ctx.borrows
                       if candidate.holder == current), None)
            if br is None or br.derived_from is None:
                return False
            if br.derived_from == ancestor:
                return True
            current = br.derived_from
        return False

    def require_readable(self, key: tuple, holder: str = ""):
        # [OWN-5] reading overlapping places conflicts with a live mutable
        # borrow unless the read is through that borrow's holder or through a
        # match-derived projection of that holder. A parent holder is not a
        # descendant of its payload binders, so it remains frozen in the arm.
        for br in self.ctx.borrows:
            if (br.uniq and places_overlap(br.place, key)
                    and br.holder != holder
                    and not self._borrow_descends_from(holder, br.holder)):
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
            if (places_overlap(br.place, key) and br.holder != holder
                    and not self._borrow_descends_from(holder, br.holder)):
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

INT_PRIMS = {"i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64"}
FLOAT_PRIMS = {"f32", "f64"}
PRIMS = INT_PRIMS | FLOAT_PRIMS
NAMED_BOOL = {"kind": "named", "name": "Bool"}
NAMED_RESULT = {"kind": "named", "name": "Result"}
U64_TY = {"kind": "prim", "name": "u64"}

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


def _cvt_total(src: str, dst: str) -> bool:
    """The exact 29 total conversion pairs listed by OP-6."""
    int_bits = {f"{prefix}{bits}": bits
                for prefix in ("i", "u") for bits in (8, 16, 32, 64)}
    if src in int_bits and dst in int_bits:
        src_prefix, dst_prefix = src[0], dst[0]
        return ((src_prefix == dst_prefix and int_bits[src] < int_bits[dst])
                or (src_prefix == "u" and dst_prefix == "i"
                    and int_bits[src] < int_bits[dst]))
    if dst == "f32":
        return src in {"i8", "i16", "u8", "u16"}
    if dst == "f64":
        return src in {"i8", "i16", "i32", "u8", "u16", "u32", "f32"}
    return False


def _single_explicit_tyarg(callee, tyargs):
    """Return the sole explicit type argument for an operation-table call."""
    if not tyargs:
        raise CheckError("FN-2", f"{callee} requires one explicit type argument")
    if len(tyargs) != 1:
        raise CheckError("OP-1", f"{callee} requires exactly one type argument")
    return tyargs[0]


def _tag_only_enum_ty(ty, enums) -> bool:
    """Exact v0.8 eeq/ene domain: one nominal enum with only nullary variants."""
    if (not isinstance(ty, dict) or ty.get("kind") != "named"
            or ty.get("args") is not None):
        return False
    variants = enums.get(ty.get("name")) if isinstance(enums, dict) else None
    return (isinstance(variants, list)
            and all(isinstance(variant, dict)
                    and isinstance(variant.get("fields"), list)
                    and not variant["fields"]
                    for variant in variants))


def op_type(callee: str, tyargs, enums=None):
    """Return (param_specs, result_spec) for a table op, or None if unknown
    (unknown ops are typed leniently). Each spec is (cat, TY) with cat 'val'."""
    T = tyargs[0] if tyargs else ANY
    parts = callee.split(".")
    base, mode = parts[0], (parts[1] if len(parts) > 1 else None)
    int_cmps = {"ieq", "ine", "ilt", "ile", "igt", "ige"}
    float_cmps = {"feq", "flt", "fle", "fgt", "fge", "fne"}
    if callee in int_cmps:
        T = _single_explicit_tyarg(callee, tyargs)
        if (not isinstance(T, dict) or T.get("kind") != "prim"
                or T.get("name") not in INT_PRIMS):
            raise CheckError("OP-1", f"{callee} is defined only for integer types")
        return ([("val", T), ("val", T)], ("val", NAMED_BOOL))
    if callee in float_cmps:
        T = _single_explicit_tyarg(callee, tyargs)
        if (not isinstance(T, dict) or T.get("kind") != "prim"
                or T.get("name") not in FLOAT_PRIMS):
            raise CheckError("OP-1", f"{callee} is defined only for float types")
        return ([("val", T), ("val", T)], ("val", NAMED_BOOL))
    if callee in {"eeq", "ene"}:
        T = _single_explicit_tyarg(callee, tyargs)
        if not _tag_only_enum_ty(T, enums):
            raise CheckError(
                "OP-1",
                f"{callee} requires one exact nominal tag-only enum type, including Bool")
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
    if base == "cvt":
        if len(tyargs) != 2:
            raise CheckError("OP-6", "cvt requires exactly <Src, Dst>")
        src, dst = tyargs
        src_name = src.get("name") if src.get("kind") == "prim" else None
        dst_name = dst.get("name") if dst.get("kind") == "prim" else None
        if src_name not in PRIMS or dst_name not in PRIMS:
            raise CheckError("OP-6", "cvt Src and Dst must be numeric primitives")
        if src_name == dst_name:
            raise CheckError("OP-6", "cvt<T, T> is not an operation")
        result = dst if _cvt_total(src_name, dst_name) else _res_of(dst, "NarrowError")
        return ([("val", src)], ("val", result))
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
    "eeq", "ene",
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


def _fn8_expr(expr, available, params, enums, fnname, require_call=False):
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
    sig = op_type(callee, expr.get("tyargs", []), enums)
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


def check_requires(fn, enums=None):
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
        _fn8_expr(
            stmt.get("init"), available, params, enums, fnname, require_call=True)
        available.add(name)
    _fn8_expr(requirements[-1].get("expr"), available, params, enums, fnname)


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
        self.match_binder_types = {}  # id(match-binder AST) -> specialized payload TY

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
                    and ex.get("kind") == "use"):
                # Return position copies or consumes its place by OWN-1. Resolve
                # the place once so an embedded index atom is not rechecked.
                d = self.place_desc(ex["place"])
            else:
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
            if isinstance(ex, dict) and ex.get("kind") == "use":
                # `try` consumes its Result place. Resolve it once so an index
                # atom receives one ordinary type judgment, not a preflight and
                # a second expression judgment.
                d = self.place_desc(ex["place"])
            else:
                d = self.expr_desc(ex)
            if (d["cat"] == "ref" or d["ty"].get("kind") in ("box", "arena")):
                raise CheckError("TYPE-7",
                    "try operand is a reference/box/arena holder; write deref(.)")
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
        if not (scrut_ty.get("kind") == "named"
                and scrut_ty.get("name") in self.P.enums):
            raise CheckError("TYPE-5",
                f"match scrutinee must be an enum, got {_ty_str(scrut_ty)}")
        variants = {v["variant"]: v["fields"]
                    for v in self.P.enums[scrut_ty["name"]]}
        binder_is_ref = (sd["cat"] == "ref")
        for arm in s["arms"]:
            saved = dict(self.env)
            vname = arm["variant"]
            if vname not in variants:
                raise CheckError("TYPE-6",
                    f"variant {vname} not in enum {scrut_ty['name']}")
            decl = variants[vname]
            args = scrut_ty.get("args") if scrut_ty.get("kind") == "named" else None
            if scrut_ty.get("name") == "Result" and args is not None:
                payload = args[0] if vname == "Ok" else args[1]
                decl = ([{"name": "value", "ty": payload}] if vname == "Ok"
                        else [{"name": "error", "ty": payload}])
            elif scrut_ty.get("name") == "Option" and args is not None:
                decl = ([{"name": "value", "ty": args[0]}] if vname == "Some" else [])
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
                self.match_binder_types[id(b)] = df["ty"]
            self.check_block(arm["body"])
            self.env = saved

    def _scrut_desc(self, e):
        """A match scrutinee that is a place is MOVED by the match [OWN-13], so a bare
        place here is not an OWN-1 affine-use error; other scrutinees type normally."""
        if e["kind"] == "use":
            d = self.place_desc(e["place"])
            if d["cat"] == "ref":
                raise CheckError("TYPE-7",
                    "match scrutinee is a reference holder; write deref(.)")
            if d["ty"].get("kind") in ("box", "arena"):
                raise CheckError("TYPE-7",
                    "match scrutinee is a box/arena holder; write deref(.)")
            if d.get("through_ref") is not None:
                provenance = d["through_ref"]
                return {**d, "cat": "ref",
                        "region": provenance.get("region"),
                        "uniq": provenance.get("uniq", False)}
            return d
        if e["kind"] == "move":
            d = self.expr_desc(e)
            if d.get("through_ref") is not None:
                raise CheckError("OWN-1",
                    "match cannot move a borrowed referent; match deref(holder) "
                    "to derive borrowed payload binders")
        else:
            d = self.expr_desc(e)
        if d["cat"] == "ref":
            raise CheckError("TYPE-7",
                "match scrutinee is a reference value; bind it and write deref(.)")
        if d["ty"].get("kind") in ("box", "arena"):
            raise CheckError("TYPE-7",
                "match scrutinee is a box/arena holder; write deref(.)")
        ty = d.get("ty", {})
        if (ty.get("kind") == "named"
                and ty.get("name") in ("Result", "Option")
                and ty.get("args") is None):
            raise CheckError("TYPE-5",
                f"match scrutinee {ty['name']} lacks instantiated payload types; "
                "bind the constructor to an explicitly typed value before matching")
        return d

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
                return {"cat": "val", "ty": d["ty"],
                        "through_ref": {"region": d.get("region"),
                                        "uniq": d.get("uniq", False)}}
            if d["ty"].get("kind") in ("box", "arena"):
                result = {"cat": "val", "ty": d["ty"]["elem"]}
                if d.get("through_ref") is not None:
                    result["through_ref"] = d["through_ref"]
                return result                                   # box/arena content
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
                    result = {"cat": "val", "ty": f["ty"]}
                    if d.get("through_ref") is not None:
                        result["through_ref"] = d["through_ref"]
                    return result
            raise CheckError("TYPE-5",
                f"struct {d['ty']['name']} has no field {place['name']}")
        if k == "index":
            d = self.place_desc(place["place"])
            if d["cat"] == "ref":
                raise CheckError("TYPE-7",
                    "index through a reference holder requires deref(.)")
            if d["ty"].get("kind") in ("box", "arena"):
                raise CheckError("TYPE-7",
                    "index through a box/arena holder requires deref(.)")
            if d["ty"].get("kind") in ("array", "buffer", "slice"):
                elem = d["ty"]["elem"]
                stated_elem = place.get("elem")
                if stated_elem is not None and not _ty_eq(stated_elem, elem):
                    raise CheckError("TYPE-5",
                        f"index element type {_ty_str(stated_elem)} != "
                        f"container element type {_ty_str(elem)}")
                offset = self.expr_desc(place["atom"])
                self.expect_value(offset, U64_TY, "index offset")
                result = {"cat": "val", "ty": elem}
                if d.get("through_ref") is not None:
                    result["through_ref"] = d["through_ref"]
                return result
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
        field_descs = []
        for fld, df in zip(e["fields"], decl):
            d = self.expr_desc(fld["atom"])
            self.expect_value(d, df["ty"], f"field {name}.{df['name']}")
            field_descs.append(d)
        return {"cat": "val", "ty": result_ty,
                "constructor": name, "field_descs": field_descs}

    def check_call(self, e):
        callee = e["callee"]
        if callee in self.P.fns:                       # user fn: GRAM-11 named args
            sig = self.P.fns[callee]
            params = sig["params"]
            # [TYPE-5] explicit call-site region arguments, when stated, must
            # match the callee's declared region parameters in count. Omission
            # is the staged region-retention debt and is not checked here.
            regions = sig.get("regions", [])
            actuals = e.get("region_args")
            if actuals is not None and len(actuals) != len(regions):
                raise CheckError("TYPE-5",
                    f"call to {callee} states {len(actuals)} region argument(s), "
                    f"but {callee} declares {len(regions)} region parameter(s) {regions}")
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
            sig = op_type(callee, e.get("tyargs", []), self.P.enums)
            if sig is None:                            # unknown op: lenient
                for arg in e["args"]:
                    self.expr_desc(arg)
                return {"cat": "val", "ty": ANY}
            params, (rcat, rty) = sig
            if len(e["args"]) != len(params):
                rule = "OP-1" if callee in ("eeq", "ene") else "GRAM-11"
                raise CheckError(rule,
                    f"op {callee} expects {len(params)} operand(s), got {len(e['args'])}")
            for arg, (_cat, ty) in zip(e["args"], params):
                d = self.expr_desc(arg)
                self.expect_value(d, ty, f"operand of {callee}")
                if callee in ("eeq", "ene") and d.get("ty") != ty:
                    raise CheckError(
                        "TYPE-5",
                        f"operand of {callee}: exact nominal type {_ty_str(ty)} required, "
                        f"got {_ty_str(d.get('ty'))}")
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
        constructor = desc.get("constructor")
        args = ty.get("args") if ty.get("kind") == "named" else None
        expected_payload = None
        if args is not None and ty.get("name") == "Result":
            if constructor == "Ok":
                expected_payload = args[0]
            elif constructor == "Err":
                expected_payload = args[1]
        elif args is not None and ty.get("name") == "Option" and constructor == "Some":
            expected_payload = args[0]
        if expected_payload is not None and desc.get("field_descs"):
            self.expect_value(desc["field_descs"][0], expected_payload,
                              f"{ctx} payload")
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


def _check_binding_ident(name, context):
    if name in RESERVED_BINDING_IDENTS:
        source_rule = "FN-8" if name == "requires" else "OP-1"
        raise CheckError("FORM-3",
            f"'{name}' is reserved by {source_rule} and cannot bind a {context}")


def _check_reserved_match(match):
    for arm in match.get("arms", []):
        for binder in arm.get("binders", []):
            name = binder.get("name") if isinstance(binder, dict) else binder
            _check_binding_ident(name, "match binder")
        _check_reserved_block(arm.get("body", []))


def _check_reserved_block(body):
    for stmt in body:
        kind = stmt.get("kind")
        if kind in ("let", "try"):
            _check_binding_ident(stmt.get("name"), f"{kind} binder")
        if kind == "let" and stmt.get("init", {}).get("kind") == "match":
            _check_reserved_match(stmt["init"])
        elif kind == "match":
            _check_reserved_match(stmt)
        elif kind == "region":
            _check_binding_ident(stmt.get("name"), "region")
            _check_reserved_block(stmt.get("body", []))
        elif kind == "loop":
            _check_reserved_block(stmt.get("body", []))


def _check_reserved_bindings(prog):
    """Enforce OP-1's closed identifier reservation for every binding site."""
    for fields in prog.get("structs", {}).values():
        for field in fields:
            _check_binding_ident(field.get("name"), "struct field")
    for variants in prog.get("enums", {}).values():
        for variant in variants:
            for field in variant.get("fields", []):
                _check_binding_ident(field.get("name"), "variant field")
    for name in prog.get("consts", {}):
        _check_binding_ident(name, "const")
    for name, fn in prog.get("fns", {}).items():
        _check_binding_ident(name, "function")
        for region in fn.get("regions", []):
            _check_binding_ident(region, "region parameter")
        for param in fn.get("params", []):
            _check_binding_ident(param.get("name"), "parameter")
        _check_reserved_block(fn.get("requires") or [])
        _check_reserved_block(fn.get("body", []))


def _check_type_declaration_namespace(prog):
    """TYPE-6: user structs/enums share one namespace with prelude types."""
    structs = set(prog.get("structs", {}))
    enums = set(prog.get("enums", {}))
    user_collision = sorted(structs & enums)
    if user_collision:
        name = user_collision[0]
        raise CheckError(
            "TYPE-6",
            f"type name '{name}' is declared as both a struct and an enum")
    prelude_collision = sorted((structs | enums) & set(PRELUDE_ENUMS))
    if prelude_collision:
        name = prelude_collision[0]
        raise CheckError(
            "TYPE-6",
            f"type name '{name}' redeclares a prelude enum")


def check_program(prog: dict):
    """Type layer + ownership over a whole program.

    prog = {"structs": {Name: [{"name","ty"}]},
            "enums":   {Enum: [{"variant","fields":[{"name","ty"}]}]},
            "fns":     {Fn: {"regions","params":[{"name","mode","ty"}],
                             "rmode","rty","body":[STMT]}}}
    Returns None on acceptance; raises CheckError(rule_id, msg) on rejection.
    """
    _check_type_declaration_namespace(prog)
    _check_reserved_bindings(prog)
    P = Program(prog)
    for name, fn in prog["fns"].items():
        check_requires({**fn, "name": name}, P.enums)  # FN-8 checked parameter-only prologue
        tc = TypeChecker(fn, P)
        tc.check()                             # GRAM-8/10/11, TYPE-5/6/7, GIVE-1
        ch = Checker(fn)
        ch.copy_enums = frozenset(en for en, vs in P.enums.items()
                                  if all(not v["fields"] for v in vs))
        ch.structs = P.structs
        ch.match_binder_types = tc.match_binder_types
        ch.consts = P.consts                   # [CONST-2] seed read-only static bindings
        ch.check()                             # OWN-1..13 (reused machinery)
    check_effects(prog)                        # EFF-1 row order, EFF-2 traps exhibits
