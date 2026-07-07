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
            # [OWN-13]/[T-B]: binder modes DERIVED from scrutinee; own moves,
            # borrow-mode scrutinee stays live and binders alias its content
            scrut = s["scrut"]; binder_mode = {"kind": "own"}; alias_of = None
            if scrut["kind"] == "use":
                base = scrut["place"]["base"]
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
                for bn in arm.get("binders", []):
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
        elif k == "return":
            self.check_expr(s["expr"], returning=True)
        elif k == "expr":
            self.check_expr(s["expr"])
        else:
            raise CheckError("GRAM-4", f"unknown stmt kind {k}")

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
            before = len(self.ctx.borrows)
            self.call_frames.append(before)
            for a in e["args"]:
                self.check_expr(a)
            self.call_frames.pop()
            new = self.ctx.borrows[before:]
            for i in range(len(new)):
                for j in range(i + 1, len(new)):
                    if places_overlap(new[i].place, new[j].place) and (new[i].uniq or new[j].uniq):
                        raise CheckError("OWN-12",
                            f"call arguments alias: {new[i].place} vs {new[j].place} "
                            f"with a uniq borrow among them")
            return
        raise CheckError("GRAM-4", f"unknown expr kind {k}")

    # -- helpers -------------------------------------------------------------
    def place_key(self, place) -> tuple:
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
