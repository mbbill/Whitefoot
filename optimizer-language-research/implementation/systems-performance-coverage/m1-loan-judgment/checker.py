"""M1 reference checker for xlang's loan/freeze judgment.

Implements EXACTLY the 15 merged rules and the machine spec in
../m1extract/machine_spec.md. One syntax-directed pass over an explicit AST
(no parser, no fixpoint). See programs_ast.py for the transcribed programs and
run.py for the harness.

Structure:
  * Data model: Place, TypeDecl / Field / Variant, Sig / ParamSpec, TypeInfo,
    and the AST statement/expr nodes (dataclasses; hand-built constructors).
  * GLOBAL_FORMS: the sealed-form op table (entry/range/lock/... ) with their
    declared loan clauses (R15 fail-closed).
  * Checker: per-function state (bindings, loan table, block stack, snapshots)
    and the per-statement transitions.

A rejection is signalled by raising Reject; ACCEPT means the whole program
(all declarations + all checked bodies) passed with no Reject.
"""

from __future__ import annotations
from dataclasses import dataclass, field, replace
from typing import Optional
import copy

SELF = "SELF"  # tie sentinel


class Reject(Exception):
    """A checked error: the program is REJECT for the carried reason."""
    def __init__(self, rule, msg):
        super().__init__(f"[{rule}] {msg}")
        self.rule = rule


# --------------------------------------------------------------------------
# Places (OWN-5) and loan entries (R4)
# --------------------------------------------------------------------------

@dataclass(frozen=True)
class Place:
    root: str
    path: tuple = ()

    def __str__(self):
        return self.root + "".join("." + p for p in self.path)


def overlap(p: Place, q: Place) -> bool:
    """overlap(P,Q): same root and one path is a prefix of the other."""
    if p.root != q.root:
        return False
    n = min(len(p.path), len(q.path))
    return p.path[:n] == q.path[:n]


@dataclass(frozen=True)
class Entry:
    place: Place
    kind: str      # 'uniq' | 'shr'
    holder: str    # binding name currently holding the token


# --------------------------------------------------------------------------
# Types
# --------------------------------------------------------------------------

@dataclass
class Field:
    name: str
    # exactly one of these classifications holds:
    borrow_mode: Optional[str] = None   # '&' | '&uniq' for a borrow field
    borrow_region: Optional[str] = None
    confined: bool = False              # a confined-typed field
    kind: Optional[str] = None          # kind of a confined field
    type_name: Optional[str] = None
    region_args: tuple = ()             # regions used by a borrow/confined field


@dataclass
class Variant:
    name: str
    fields: list  # list[Field]


@dataclass
class TypeDecl:
    name: str
    is_confined: bool
    kind: Optional[str] = None          # 'uniq'|'shr' for confined types
    region_params: tuple = ()
    fields: list = field(default_factory=list)   # struct fields
    is_enum: bool = False
    variants: list = field(default_factory=list)  # list[Variant]


@dataclass
class TypeInfo:
    """A use-site reference to a (possibly confined) type."""
    type_name: str
    confined: bool = False
    kind: Optional[str] = None
    region_args: tuple = ()   # region names in application order


# --------------------------------------------------------------------------
# Signatures (R9/R10) — form ops and helper fns share this shape
# --------------------------------------------------------------------------

@dataclass
class ParamSpec:
    name: str
    mode: str                 # '&' | '&uniq' | 'own'
    region: Optional[str] = None   # declared region for a NON-confined borrow
    confined: bool = False
    type_name: Optional[str] = None
    kind: Optional[str] = None
    region_args: tuple = ()


@dataclass
class Sig:
    name: str
    region_params: tuple = ()
    params: list = field(default_factory=list)
    result: Optional[TypeInfo] = None
    # form op only: an explicit loan clause overrides derived effects.
    form_clause: Optional[str] = None   # none|issues|issues_on_source|consumes|reissues
    clause_kind: Optional[str] = None   # kind for issues clauses
    is_form: bool = False
    loanable_form: bool = False         # receiver is a loanable sealed form
    address_stable: bool = True         # R15 & -receiver obligation
    concurrent_safe: bool = True
    # cache of derived ties (region -> param name | SELF | None), filled by prepass
    ties: dict = field(default_factory=dict)


# --------------------------------------------------------------------------
# AST nodes (statements / call args). Built by hand in programs_ast.py.
# --------------------------------------------------------------------------

@dataclass
class ArgOwn:
    place: Place

@dataclass
class ArgBorrow:
    mode: str        # '&' | '&uniq'
    place: Place

@dataclass
class ArgValue:      # a plain owned non-confined value (k, v, literals)
    pass


# Statements ----------------------------------------------------------------

@dataclass
class SLetNew:
    """let x = <owned non-confined>  (table::new(), seq::new(), literals, ...)."""
    name: str
    type_name: str = "owned"

@dataclass
class SLetCall:
    """let x = op(args)  or  x = recv.op(args). result may be confined."""
    name: Optional[str]      # None for a bare expression-statement call
    op: str
    args: list               # list[ArgOwn|ArgBorrow|ArgValue]

@dataclass
class SLetConstruct:
    """let x = ConfinedType { field: arg, ... }  (struct or enum variant)."""
    name: str
    type_name: str
    variant: Optional[str]           # None for struct, else variant name
    field_args: list                 # list[(field_name, ArgOwn|ArgBorrow)]

@dataclass
class SLetMove:
    """let x = move h."""
    name: str
    src: str

@dataclass
class SRebind:
    """rebind x = h  (x previously consumed, identical confined type)."""
    name: str
    src: str

@dataclass
class SBlock:
    """A nested lexical block { ... }."""
    body: list

@dataclass
class SMatch:
    scrutinee: object    # Place (a binding) matched, or None for value match
    scrut_type: Optional[TypeInfo]
    arms: list           # list[MatchArm]

@dataclass
class MatchArm:
    variant: Optional[str]     # variant name (confined destructure) or None
    binder: Optional[str]      # bound token name for confined destructure
    struct_pattern: bool       # True if this arm uses a struct pattern (R11 reject)
    body: list

@dataclass
class SLoop:
    body: list

@dataclass
class SBreak:
    pass

@dataclass
class SReturn:
    value: Optional[str]    # returned binding name, or None
    value_place: Optional[Place] = None

@dataclass
class SPar:
    """par::for_chunks(slot1, slot2, body_fn) — slots per form marks."""
    op: str
    slots: list      # list[(mode_mark, ArgBorrow)]  mode_mark in split|replicate|body
    body_fn: str


# --------------------------------------------------------------------------
# Program bundle
# --------------------------------------------------------------------------

@dataclass
class Func:
    sig: Sig
    body: Optional[list]      # None => trusted (signature only, "...")
    is_main: bool = False

@dataclass
class Program:
    pid: str
    types: list = field(default_factory=list)     # list[TypeDecl]
    local_forms: list = field(default_factory=list)  # program-local form ops
    funcs: list = field(default_factory=list)     # list[Func]


# --------------------------------------------------------------------------
# Binding state (checker state (1)) and Frame (block stack, checker state (3))
# --------------------------------------------------------------------------

@dataclass
class Binding:
    name: str
    type_name: str
    mode: str                 # 'owned' | 'reference'
    ref_region: Optional[str]
    status: str               # 'live' | 'consumed'
    decl_index: int
    confined: bool = False
    kind: Optional[str] = None
    region_params: tuple = ()
    src: dict = field(default_factory=dict)
    is_param: bool = False
    borrow_cap: Optional[str] = None   # 'shr'|'uniq' for a reference binding's
                                       # declared borrow mode; None for owned


@dataclass
class Frame:
    kind: str                 # 'func'|'block'|'loop'|'arm'
    names: list
    cutoff: int


# --------------------------------------------------------------------------
# R9 tie derivation (DECLARATIONS PRE-PASS (b))
# --------------------------------------------------------------------------

def derive_ties(sig: Sig) -> dict:
    ties = {}
    for rp in sig.region_params:
        conf_using = [p for p in sig.params if p.confined and rp in p.region_args]
        result_uses = bool(sig.result and sig.result.confined and rp in sig.result.region_args)
        if not conf_using and not result_uses:
            ties[rp] = None
            continue
        D = [p for p in sig.params
             if (not p.confined) and p.mode in ('&', '&uniq') and p.region == rp]
        if len(D) == 1:
            ties[rp] = D[0].name
        elif len(D) == 0:
            borrow_conf = [p for p in conf_using if p.mode in ('&', '&uniq')]
            own_conf = [p for p in conf_using if p.mode == 'own']
            if sig.is_form:
                # form-table token self-ties (its source place is recorded, not
                # passed) even by own; the clause carries the real effect.
                ties[rp] = SELF
            elif borrow_conf and not own_conf and not result_uses:
                ties[rp] = SELF
            else:
                raise Reject('R9', f"{sig.name}: region {rp} ties to zero borrow "
                             f"params (own/result needs a distinct tie)")
        else:
            raise Reject('R9', f"{sig.name}: region {rp} ties to {len(D)} borrow params")
    return ties


def check_form_op(s: Sig):
    """R15: an & -receiver op on a loanable form must be address-stable and
    concurrent-safe; otherwise the declaration is rejected (fail-closed)."""
    if s.loanable_form and s.params and s.params[0].mode == '&':
        if not (s.address_stable and s.concurrent_safe):
            raise Reject('R15', f"& -receiver op {s.name} fails address-stability / "
                         f"concurrent-safety obligation")
    # Pre-pass (a): a loanable confined type must carry >=1 region parameter. An
    # issuing op whose result is a zero-region confined type is undeclarable --
    # enforced HERE at declaration, not only at a call site (CE5).
    if s.result is not None and s.result.confined and len(s.result.region_args) == 0 \
            and s.form_clause in ('issues', 'issues_on_source', 'reissues'):
        raise Reject('R1', f"issuing form op {s.name} produces zero-region loanable "
                     f"{s.result.type_name}")


def check_type_decl(t: TypeDecl, types: dict):
    """DECLARATIONS PRE-PASS (a): confined formation and kind coherence (R1)."""
    allf = list(t.fields)
    for v in t.variants:
        allf += v.fields
    has_bc = any(f.borrow_mode or f.confined for f in allf)
    if not t.is_confined:
        if has_bc:
            raise Reject('R1', f"non-confined {t.name} has a borrow/confined field")
        return
    if has_bc and len(t.region_params) == 0:
        raise Reject('R1', f"loanable confined {t.name} has zero region params")
    if t.kind == 'shr':
        for f in allf:
            if f.borrow_mode == '&uniq':
                raise Reject('R1', f"confined(shr) {t.name} has a &uniq field")
            if f.confined and f.kind == 'uniq':
                raise Reject('R1', f"confined(shr) {t.name} has a confined(uniq) field")
            if f.confined and f.type_name in types and types[f.type_name].kind == 'uniq':
                raise Reject('R1', f"confined(shr) {t.name} transitively holds uniq")
    for f in allf:
        for r in f.region_args:
            if r not in t.region_params:
                raise Reject('R1', f"field region {r} not a param of {t.name}")


# --------------------------------------------------------------------------
# Checker (one syntax-directed pass, no fixpoint)
# --------------------------------------------------------------------------

def always_exits(stmts) -> bool:
    """Syntactic test (JOIN-1): does every path end in break/return?"""
    if not stmts:
        return False
    last = stmts[-1]
    if isinstance(last, (SBreak, SReturn)):
        return True
    if isinstance(last, SMatch):
        return all(always_exits(a.body) for a in last.arms)
    if isinstance(last, SBlock):
        return always_exits(last.body)
    return False


class Checker:
    def __init__(self, prog, types, forms):
        self.prog = prog
        self.types = types
        self.forms = forms
        self._visited = set()   # structural single-pass guarantee

    # --- state helpers ----------------------------------------------------

    def snapshot(self):
        return (copy.deepcopy(self.bindings), copy.deepcopy(self.loans), self.next_decl)

    def restore(self, snap):
        self.bindings = copy.deepcopy(snap[0])
        self.loans = copy.deepcopy(snap[1])
        self.next_decl = snap[2]

    def projection(self, cutoff):
        ents = frozenset((e.place, e.kind, e.holder) for e in self.loans
                         if self.bindings[e.holder].decl_index < cutoff)
        lv = frozenset((n, b.status) for n, b in self.bindings.items()
                       if b.decl_index < cutoff)
        return (ents, lv)

    def _new_binding(self, name, type_name, mode='owned', confined=False,
                     kind=None, region_params=(), ref_region=None):
        b = Binding(name, type_name, mode, ref_region, 'live', self.next_decl,
                    confined, kind, tuple(region_params), {})
        self.bindings[name] = b
        self.next_decl += 1
        if self.frames:
            self.frames[-1].names.append(name)
        return b

    def _remove_holder(self, holder):
        self.loans = [e for e in self.loans if e.holder != holder]

    def _rename_holder(self, old, new):
        self.loans = [Entry(e.place, e.kind, new) if e.holder == old else e
                      for e in self.loans]

    def lookup_op(self, name):
        return self.forms.get(name)

    # --- mint legality (R6) with the consume-exception --------------------

    def _mint_ok(self, place, mode, consumed_holders):
        overlapping = [e for e in self.loans if overlap(e.place, place)]
        if mode == '&':
            conflicting = [e for e in overlapping if e.kind == 'uniq']
        else:  # '&uniq' or move/own/assign/drop
            conflicting = list(overlapping)
        if not conflicting:
            return True
        holders = set(e.holder for e in overlapping)
        return len(holders) == 1 and holders <= consumed_holders

    def _place_of_param(self, sig, args, tie_name):
        for p, a in zip(sig.params, args):
            if p.name == tie_name:
                if isinstance(a, (ArgOwn, ArgBorrow)):
                    return a.place
        return None

    def _mode_ok(self, place):
        """Mint-mode capability (machine spec [Mint legality], mode clause): a
        &uniq mint or own-pass of a place is legal only if the place's root can
        be uniquely accessed -- an owned local or a &uniq-borrowed parameter.
        A &-shared-borrowed parameter cannot be upgraded to unique. This removes
        M1's silent dependency on the base ownership layer (BREAK-1)."""
        b = self.bindings.get(place.root)
        if b is None:
            return True
        return not (b.mode == 'reference' and b.borrow_cap == 'shr')

    def _check_stmt_mints(self, mints, rule):
        """Statement-local disjointness (machine spec [Mint legality], statement
        clause; the R14 authority for par): every mint of ONE statement is
        accumulated as a pseudo-entry (&uniq -> uniq, & -> shr) and any two
        overlapping mints where at least one is uniq are rejected. Two shared
        readers of the same place stay legal. Closes same-statement aliased
        &uniq mints for ordinary calls (E2d/S1a) and par cross-slot aliasing
        (BREAK-2/2b/2c/2d)."""
        for i in range(len(mints)):
            pi, ki = mints[i]
            for j in range(i + 1, len(mints)):
                pj, kj = mints[j]
                if overlap(pi, pj) and (ki == 'uniq' or kj == 'uniq'):
                    raise Reject(rule, f"aliased mints in one statement: "
                                 f"{ki} {pi} and {kj} {pj} overlap")

    # --- statement dispatch (one visit per node: single-pass guarantee) ----

    def visit(self, s, frame):
        assert id(s) not in self._visited, "statement visited twice (no fixpoint allowed)"
        self._visited.add(id(s))
        m = getattr(self, 'v_' + type(s).__name__)
        return m(s, frame)

    def v_SLetNew(self, s, frame):
        self._new_binding(s.name, s.type_name, mode='owned', confined=False)
        return 'fall'

    def v_SLetMove(self, s, frame):
        hb = self.bindings[s.src]
        if hb.status != 'live':
            raise Reject('R8', f"move of consumed {s.src}")
        # R6 freeze: a place overlapped by a live entry cannot be moved. (A holder
        # transfer renames its own entries, which sit on the SOURCE place, not on
        # the holder's place, so a legal let-move of a holder is unaffected.)
        if any(overlap(e.place, Place(s.src)) for e in self.loans):
            raise Reject('R6', f"move of {s.src} while a live loan overlaps it")
        b = self._new_binding(s.name, hb.type_name, mode='owned', confined=hb.confined,
                              kind=hb.kind, region_params=hb.region_params)
        b.src = dict(hb.src)
        self._rename_holder(s.src, s.name)
        hb.status = 'consumed'
        return 'fall'

    def v_SRebind(self, s, frame):
        tgt = self.bindings.get(s.name)
        if tgt is None or tgt.status != 'consumed':
            raise Reject('R8', f"rebind over live/undeclared {s.name}")
        hb = self.bindings[s.src]
        if hb.status != 'live':
            raise Reject('R8', f"rebind from consumed {s.src}")
        if tgt.type_name != hb.type_name:
            raise Reject('R8', f"rebind type mismatch {tgt.type_name} != {hb.type_name}")
        self._rename_holder(s.src, s.name)
        tgt.src = dict(hb.src)
        tgt.status = 'live'
        hb.status = 'consumed'
        return 'fall'

    def v_SBlock(self, s, frame):
        return self._run_block(s.body, 'block')

    def v_SBreak(self, s, frame):
        li = max(i for i, f in enumerate(self.frames) if f.kind == 'loop')
        for i in range(len(self.frames) - 1, li - 1, -1):
            self.process_scope_end(self.frames[i].names)
        cutoff = self.frames[li].cutoff
        self.break_stack[-1].append((self.snapshot(), self.projection(cutoff)))
        return 'break'

    def v_SReturn(self, s, frame):
        self._check_region_escape(s.value)                 # R13
        for i in range(len(self.frames) - 1, -1, -1):
            self.process_scope_end(self.frames[i].names, exclude=s.value)
        self.check_exit_effect(s.value, s.value_place)
        return 'return'

    def _check_region_escape(self, ret_name):
        """R13 (ESC-1): a confined value may be returned only if every region
        parameter of its type is a region parameter of the enclosing function --
        i.e. each region's source place roots at a reference PARAMETER, never a
        locally introduced (owned-local) source."""
        if ret_name is None:
            return
        rb = self.bindings.get(ret_name)
        if rb is None or not rb.confined:
            return
        for region in rb.region_params:
            S = rb.src.get(region)
            if S is None:
                raise Reject('R13', f"returned {ret_name} carries an unbound region {region}")
            root_b = self.bindings.get(S.root)
            if root_b is None or not (root_b.mode == 'reference' and root_b.is_param):
                raise Reject('R13', f"returned {ret_name} carries a locally introduced "
                             f"region {region} (source {S} is not a function parameter)")

    def v_SPar(self, s, frame):
        slot_mints = []
        for mark, arg in s.slots:
            if mark == 'body':
                continue
            b = self.bindings.get(arg.place.root)
            # R3: a slot argument that is a confined holder must be LIVE (CE4).
            if b is not None and b.confined and b.status != 'live':
                raise Reject('R3', f"consumed confined {arg.place.root} in a par slot")
            if mark == 'split':
                if arg.mode != '&uniq' or not self._mode_ok(arg.place) \
                        or not self._mint_ok(arg.place, '&uniq', set()):
                    raise Reject('R14', f"split-unique slot needs a legal &uniq mint of {arg.place}")
                slot_mints.append((arg.place, 'uniq'))
            elif mark == 'replicate':
                if b is not None and b.confined:
                    held = [e for e in self.loans if e.holder == arg.place.root]
                    if any(e.kind == 'uniq' for e in held):
                        raise Reject('R14', "holder with a uniq entry into a replicate-shared slot")
                else:
                    if any(overlap(e.place, arg.place) and e.kind == 'uniq' for e in self.loans):
                        raise Reject('R14', f"uniq entry overlaps replicate-shared {arg.place}")
                slot_mints.append((arg.place, 'shr'))
        # cross-slot disjointness (R14): no two par slots may take overlapping
        # views where one is unique (BREAK-2/2b/2c/2d); replicate+replicate on the
        # same place stays legal.
        self._check_stmt_mints(slot_mints, 'R14')
        return 'fall'

    # --- call (op / helper fn) : R10 caller+callee, R5 issue, R6 freeze ----

    def v_SLetCall(self, s, frame):
        sig = self.lookup_op(s.op)
        if sig is None:
            raise Reject('R15', f"op {s.op} not covered by the form table (fail-closed)")
        params, args = sig.params, s.args

        consumed = set()
        for p, a in zip(params, args):
            if p.confined and p.mode == 'own' and isinstance(a, ArgOwn):
                if sig.is_form:
                    if sig.form_clause in ('consumes', 'reissues'):
                        consumed.add(a.place.root)
                else:
                    consumed.add(a.place.root)

        # (i) brand check R10(a)
        for p, a in zip(params, args):
            if not p.confined or not isinstance(a, (ArgOwn, ArgBorrow)):
                continue
            root = a.place.root
            hb = self.bindings.get(root)
            if hb is None or not hb.confined:
                continue
            if hb.status != 'live':
                raise Reject('R3', f"use of consumed confined {root}")
            for i, creg in enumerate(p.region_args):
                tie = sig.ties.get(creg)
                hreg = hb.region_params[i] if i < len(hb.region_params) else None
                if tie is None:
                    continue
                if tie is SELF:
                    # A form-table op's confined receiver whose region has no
                    # distinct borrow candidate ties to the receiver holder
                    # itself (receiver-holder tie, R9 amendment). It skips the
                    # distinct-place comparison but STILL brand-checks the token
                    # identity -- the R10a coverage the frozen is_form carve-out
                    # silently dropped:
                    #   * a token with a recorded source must actually hold a live
                    #     entry there (a genuine issued token);
                    #   * a LOCAL confined value with no recorded source reaching a
                    #     form op is forged/foreign -> reject (defense behind R5);
                    #   * a self-tied PARAMETER has no recorded source and is
                    #     trusted from the signature (R9 self-tie) -> skip.
                    if sig.is_form:
                        S = hb.src.get(hreg)
                        if S is not None:
                            if Entry(S, hb.kind, root) not in self.loans:
                                raise Reject('R10a', f"form token {root} is not a live "
                                             f"holder of its recorded loan")
                        elif not hb.is_param:
                            raise Reject('R10a', f"form token {root} has no recorded "
                                         f"source (forged/foreign)")
                    continue
                # distinct tie: instance-brand comparison
                R = self._place_of_param(sig, args, tie)
                if hb.src.get(hreg) is None:
                    raise Reject('R10a', f"self-tied-origin {root} into distinct-tie slot")
                if hb.src[hreg] != R:
                    raise Reject('R10a', f"brand mismatch {root}: src {hb.src[hreg]} != tied {R}")
                if Entry(R, hb.kind, root) not in self.loans:
                    raise Reject('R10a', f"no live entry ({R},{hb.kind},{root})")

        # (ii) mint legality for borrow args
        mints = []
        for p, a in zip(params, args):
            if isinstance(a, ArgBorrow):
                if a.mode == '&uniq' and not self._mode_ok(a.place):
                    raise Reject('R6', f"&uniq mint of a &-shared-borrowed source {a.place}")
                if not self._mint_ok(a.place, a.mode, consumed):
                    raise Reject('R6', f"illegal {a.mode} mint of {a.place}")
                mints.append((a.place, 'uniq' if a.mode == '&uniq' else 'shr'))
        self._check_stmt_mints(mints, 'R6')

        # (iii) consumes
        for p, a in zip(params, args):
            if not isinstance(a, ArgOwn):
                continue
            hb = self.bindings.get(a.place.root)
            if p.confined and a.place.root in consumed:
                self._remove_holder(a.place.root)
                hb.status = 'consumed'
            elif p.confined and hb is not None:
                # clause-none own-mode confined argument: affine consume (machine
                # spec (iii), "other own arguments ... mark consumed"). Its entries
                # linger and trip R7 at scope end if never resolved. (CE2.)
                if any(overlap(e.place, a.place) for e in self.loans):
                    raise Reject('R6', f"own-pass of loaned place {a.place}")
                hb.status = 'consumed'
            elif not p.confined and hb is not None:
                if not self._mode_ok(a.place):
                    raise Reject('R6', f"own-pass of a &-shared-borrowed source {a.place}")
                if any(overlap(e.place, a.place) for e in self.loans):
                    raise Reject('R6', f"own-pass of loaned {a.place}")
                hb.status = 'consumed'

        # (iv) issues (result)
        if s.name is not None:
            if sig.result is not None and sig.result.confined:
                self._issue_result(s.name, sig, args)
            else:
                tn = sig.result.type_name if sig.result else 'value'
                self._new_binding(s.name, tn, mode='owned', confined=False)
        return 'fall'

    def _issue_result(self, name, sig, args):
        res = sig.result
        if len(res.region_args) == 0:
            raise Reject('R1', f"issue into loanable confined {res.type_name} with zero regions")
        b = self._new_binding(name, res.type_name, mode='owned', confined=True,
                              kind=res.kind, region_params=res.region_args)
        # receiver holder (first confined arg) for issues_on_source
        recv_root = None
        for p, a in zip(sig.params, args):
            if p.confined and isinstance(a, (ArgOwn, ArgBorrow)):
                recv_root = a.place.root
                break
        for i, creg in enumerate(res.region_args):
            if sig.form_clause == 'issues_on_source':
                rb = self.bindings[recv_root]
                S = rb.src[rb.region_params[i]]
            else:
                tie = sig.ties.get(creg)
                S = self._place_of_param(sig, args, tie)
            if S is None:
                raise Reject('R5', f"cannot resolve issue source for {creg}")
            if res.kind == 'uniq':
                if any(overlap(e.place, S) for e in self.loans):
                    raise Reject('R5', f"uniq issue on {S} with an overlapping entry")
            else:
                if any(overlap(e.place, S) and e.kind == 'uniq' for e in self.loans):
                    raise Reject('R5', f"shr issue on {S} with an overlapping uniq entry")
            self.loans.append(Entry(S, res.kind, name))
            b.src[creg] = S

    def v_SLetConstruct(self, s, frame):
        td = self.types[s.type_name]
        fields = td.fields
        if s.variant is not None:
            var = next(v for v in td.variants if v.name == s.variant)
            fields = var.fields
        fmap = {f.name: f for f in fields}
        if td.is_confined and any(f.borrow_mode or f.confined for f in fields) \
                and len(td.region_params) == 0:
            raise Reject('R1', f"loanable confined {td.name} with zero regions")
        b = self._new_binding(s.name, td.name, mode='owned', confined=td.is_confined,
                              kind=td.kind, region_params=td.region_params)
        bound = set()
        cap_mints = []
        for fname, arg in s.field_args:
            fld = fmap[fname]
            if fld.confined:                       # moved confined field (R8)
                hb = self.bindings[arg.place.root]
                if hb.status != 'live':
                    raise Reject('R8', f"move of consumed {arg.place.root}")
                self._rename_holder(arg.place.root, s.name)
                for j, r in enumerate(fld.region_args):
                    b.src[r] = hb.src[hb.region_params[j]]
                    bound.add(r)
                hb.status = 'consumed'
            else:                                  # captured borrow (R5)
                S = arg.place
                if fld.borrow_mode == '&uniq' and not self._mode_ok(S):
                    raise Reject('R6', f"&uniq capture of a &-shared-borrowed source {S}")
                if not self._mint_ok(S, fld.borrow_mode, set()):
                    raise Reject('R6', f"illegal {fld.borrow_mode} capture of {S}")
                cap_mints.append((S, 'uniq' if fld.borrow_mode == '&uniq' else 'shr'))
                if td.kind == 'uniq':
                    if any(overlap(e.place, S) for e in self.loans):
                        raise Reject('R5', f"uniq capture on {S} overlaps an entry")
                else:
                    if any(overlap(e.place, S) and e.kind == 'uniq' for e in self.loans):
                        raise Reject('R5', f"shr capture on {S} overlaps a uniq entry")
                self.loans.append(Entry(S, td.kind, s.name))
                b.src[fld.borrow_region] = S
                bound.add(fld.borrow_region)
        self._check_stmt_mints(cap_mints, 'R6')
        missing = set(td.region_params) - bound
        if missing:
            raise Reject('R5', f"construction of {td.name} leaves region(s) {missing} unbound")
        return 'fall'

    # --- match / loop / blocks (JOIN-1, R11, R12) -------------------------

    def _run_block(self, stmts, kind, injects=None):
        frame = Frame(kind, [], self.next_decl)
        self.frames.append(frame)
        if injects:
            for name, b, entries in injects:
                b.decl_index = self.next_decl
                self.next_decl += 1
                self.bindings[name] = b
                frame.names.append(name)
                self.loans.extend(entries)
        flow = 'fall'
        for st in stmts:
            f = self.visit(st, frame)
            if f in ('break', 'return', 'exit'):
                flow = f
                break
        if flow == 'fall':
            self.process_scope_end(frame.names)
        self.frames.pop()
        return flow

    def v_SMatch(self, s, frame):
        cutoff = self.next_decl
        # The R11 gate keys on the BINDING's confinedness (machine spec [match]:
        # "if the scrutinee is a live confined binding"), NOT the AST annotation
        # s.scrut_type -- a confined scrutinee annotated as a value cannot bypass
        # the destructure checks or the consume (CE3).
        sb = None
        if isinstance(s.scrutinee, Place):
            sb = self.bindings.get(s.scrutinee.root)
        scrut_confined = sb is not None and sb.status == 'live' and sb.confined
        scrut_entries, scrut_type_name, scrut_kind, scrut_regparams, scrut_src = [], None, None, (), {}
        if scrut_confined:
            root = s.scrutinee.root
            td = self.types.get(sb.type_name)
            if td is None or (not td.is_enum) or any(a.struct_pattern for a in s.arms):
                raise Reject('R11', "struct / non-enum destructure of a confined value")
            for v in td.variants:
                cf = [f for f in v.fields if f.confined]
                bf = [f for f in v.fields if f.borrow_mode]
                if len(cf) != 1 or bf:
                    raise Reject('R11', f"variant {v.name} not a single confined field")
                if cf[0].kind != sb.kind or len(cf[0].region_args) != len(td.region_params):
                    raise Reject('R11', f"variant {v.name} kind/region mismatch")
            scrut_entries = [e for e in self.loans if e.holder == root]
            self.loans = [e for e in self.loans if e.holder != root]
            scrut_type_name, scrut_kind = sb.type_name, sb.kind
            scrut_regparams = sb.region_params
            scrut_src = dict(sb.src)
            sb.status = 'consumed'

        base = self.snapshot()
        fall_states = []
        for arm in s.arms:
            self.restore(base)
            injects = None
            if scrut_confined and arm.binder:
                nb = Binding(arm.binder, scrut_type_name, 'owned', None, 'live',
                             0, True, scrut_kind, scrut_regparams, dict(scrut_src))
                ents = [Entry(e.place, e.kind, arm.binder) for e in scrut_entries]
                injects = [(arm.binder, nb, ents)]
            flow = self._run_block(arm.body, 'arm', injects=injects)
            if flow == 'fall':
                fall_states.append((self.snapshot(), self.projection(cutoff)))
        if fall_states:
            base_proj = fall_states[0][1]
            for _, p in fall_states[1:]:
                if p != base_proj:
                    raise Reject('R12', "match fall-through arms disagree")
            self.restore(fall_states[0][0])
            return 'fall'
        return 'exit'

    def v_SLoop(self, s, frame):
        cutoff = self.next_decl
        entry_proj = self.projection(cutoff)
        self.break_stack.append([])
        flow = self._run_block(s.body, 'loop')
        if flow == 'fall' and not always_exits(s.body):
            if self.projection(cutoff) != entry_proj:
                raise Reject('R12', "loop back-edge state != entry state")
        breaks = self.break_stack.pop()
        if breaks:
            p0 = breaks[0][1]
            for _, p in breaks[1:]:
                if p != p0:
                    raise Reject('R12', "loop break states disagree")
            self.restore(breaks[0][0])
            return 'fall'
        return 'exit'

    # --- scope-end processing (R7) ----------------------------------------

    def process_scope_end(self, names, exclude=None):
        for name in reversed(names):
            if name == exclude:
                continue
            b = self.bindings[name]
            if b.status == 'consumed':
                continue
            if b.confined:
                self._remove_holder(name)     # auto-consume: release + drop its entries
                b.status = 'consumed'
                # A confined binding is an owned binding being dropped; if it is
                # itself a loan SOURCE, a live entry still overlaps its place after
                # its own entries are released -> dangling source, REJECT (CE1).
                if any(overlap(e.place, Place(name)) for e in self.loans):
                    raise Reject('R7', f"drop of confined source {name} while a live loan overlaps it")
            else:
                if b.mode == 'owned':
                    pl = Place(name)
                    if any(overlap(e.place, pl) for e in self.loans):
                        raise Reject('R7', f"drop of {name} while a live loan overlaps it")
                b.status = 'consumed'

    # --- exit-effect check at return (R10c) -------------------------------

    def check_exit_effect(self, ret_name, ret_place):
        sig = self.cur_func.sig
        if self.cur_func.is_main:
            return
        expected = set()
        res = sig.result
        if ret_name is not None and res is not None and res.confined:
            for creg in res.region_args:
                tie = sig.ties.get(creg)
                if tie in (None, SELF):
                    continue
                expected.add((Place(tie), res.kind, ret_name))
        param_roots = {p.name for p in sig.params}
        actual = {(e.place, e.kind, e.holder) for e in self.loans if e.place.root in param_roots}
        if actual != expected:
            raise Reject('R10c', f"exit effect {actual} != signature effect {expected}")

    # --- function driver --------------------------------------------------

    def check_func(self, func):
        if func.body is None:
            return
        self.bindings, self.loans, self.next_decl = {}, [], 0
        self.frames, self.break_stack = [], []
        self.cur_func = func
        frame = Frame('func', [], 0)
        self.frames.append(frame)
        sig = func.sig
        cap = {'&': 'shr', '&uniq': 'uniq', 'own': None}
        for p in sig.params:
            if p.confined:
                b = Binding(p.name, p.type_name, 'owned' if p.mode == 'own' else 'reference',
                            p.region, 'live', self.next_decl, True, p.kind, tuple(p.region_args), {},
                            is_param=True, borrow_cap=cap.get(p.mode))
            else:
                b = Binding(p.name, p.type_name or 'container', 'reference', p.region,
                            'live', self.next_decl, False, None, (), {},
                            is_param=True, borrow_cap=cap.get(p.mode))
            self.bindings[p.name] = b
            self.next_decl += 1
            frame.names.append(p.name)
        for p in sig.params:                       # function-entry seeding (R10c)
            if p.confined:
                for i, creg in enumerate(p.region_args):
                    tie = sig.ties.get(creg)
                    if tie and tie != SELF:
                        B = Place(tie)
                        self.loans.append(Entry(B, p.kind, p.name))
                        self.bindings[p.name].src[p.region_args[i]] = B
        flow = 'fall'
        for st in func.body:
            f = self.visit(st, frame)
            if f in ('break', 'return', 'exit'):
                flow = f
                break
        if flow == 'fall':
            self.process_scope_end(frame.names)
        self.frames.pop()


# --------------------------------------------------------------------------
# Program driver
# --------------------------------------------------------------------------

def check_program(prog: Program):
    """Returns 'ACCEPT' or ('REJECT', reason)."""
    try:
        types = {t.name: t for t in prog.types}
        for t in prog.types:                       # R1
            check_type_decl(t, types)
        forms = dict(GLOBAL_FORMS)
        for s in prog.local_forms:                 # R15 + fail-closed
            check_form_op(s)
            s.ties = derive_ties(s)
            forms[s.name] = s
        for s in prog.funcs:                       # R9 (helper signatures)
            s.sig.ties = derive_ties(s.sig)
            if not s.is_main:                      # helper calls resolve here
                forms[s.sig.name] = s.sig
        chk = Checker(prog, types, forms)
        for f in prog.funcs:
            chk.check_func(f)
        return 'ACCEPT'
    except Reject as r:
        return ('REJECT', str(r))


# --------------------------------------------------------------------------
# GLOBAL_FORMS — sealed-form op table (R15 fail-closed). Each op is a Sig with
# the receiver as params[0]; loan behaviour is the declared form_clause.
# --------------------------------------------------------------------------

def _cont(name, mode, region, tyname):      # non-confined container-borrow param
    return ParamSpec(name, mode, region=region, confined=False, type_name=tyname)

def _tok(name, mode, tyname, kind, regs):   # confined token param
    return ParamSpec(name, mode, confined=True, type_name=tyname, kind=kind, region_args=regs)

def _val(name):                             # plain owned value
    return ParamSpec(name, 'own')

def _op(name, params, result=None, clause='none', ckind=None, regs=(),
        loanable=True, addr=True, conc=True):
    return Sig(name, region_params=regs, params=params, result=result,
               form_clause=clause, clause_kind=ckind, is_form=True,
               loanable_form=loanable, address_stable=addr, concurrent_safe=conc)


GLOBAL_FORMS = {}
for _s in [
    # --- Hashbrown / SQLite b-tree table -------------------------------
    _op('entry',  [_cont('t', '&uniq', 't', 'table'), _val('k')],
        TypeInfo('Entry', True, 'uniq', ('t',)), 'issues', 'uniq', ('t',)),
    _op('insert', [_cont('t', '&uniq', 't', 'table'), _val('k'), _val('v')]),
    _op('remove', [_cont('t', '&uniq', 't', 'table'), _val('k')]),
    _op('range',  [_cont('t', '&', 't', 'table'), _val('lo'), _val('hi')],
        TypeInfo('Cursor', True, 'shr', ('t',)), 'issues', 'shr', ('t',)),
    _op('get',    [_cont('t', '&', 't', 'table'), _val('k')], TypeInfo('value'), 'none'),
    _op('delete_at', [_cont('t', '&uniq', 't', 'table'),
                      _tok('c', 'own', 'Cursor', 'shr', ('t',))], None, 'consumes', regs=('t',)),
    _op('next',   [_tok('c', '&uniq', 'Cursor', 'shr', ('t',))], TypeInfo('option'), 'none', regs=('t',)),
    _op('next_view', [_tok('c', '&', 'Cursor', 'shr', ('t',))],
        TypeInfo('View', True, 'shr', ('t',)), 'issues_on_source', 'shr', ('t',)),
    _op('set',    [_cont('d', '&uniq', 'd', 'buffer'), _val('i'), _val('v')]),
    # --- seq -----------------------------------------------------------
    _op('push',   [_cont('s', '&uniq', 's', 'seq'), _val('v')]),
    _op('pop',    [_cont('s', '&uniq', 's', 'seq')], TypeInfo('option'), 'none'),
    # --- mutex / condvar ----------------------------------------------
    _op('lock',   [_cont('m', '&', 'm', 'mutex')],
        TypeInfo('Guard', True, 'uniq', ('m',)), 'issues', 'uniq', ('m',)),
    _op('try_lock', [_cont('m', '&', 'm', 'mutex')],
        TypeInfo('Guard', True, 'uniq', ('m',)), 'issues', 'uniq', ('m',)),
    _op('wait',   [_cont('cv', '&', None, 'condvar'), _cont('m', '&', 'm', 'mutex'),
                   _tok('g', 'own', 'Guard', 'uniq', ('m',))],
        TypeInfo('Guard', True, 'uniq', ('m',)), 'reissues', 'uniq', ('m',), loanable=False),
    _op('predicate', [_tok('g', '&', 'Guard', 'uniq', ('m',))], TypeInfo('Poll'), 'none', regs=('m',)),
    _op('acquired', [_tok('g', '&', 'Guard', 'uniq', ('m',))], TypeInfo('bool'), 'none', regs=('m',)),
    _op('dismiss', [_tok('g', 'own', 'Guard', 'uniq', ('m',))], None, 'consumes', regs=('m',)),
    _op('unlock_via', [_cont('m', '&', 'm', 'mutex'),
                       _tok('g', 'own', 'Guard', 'uniq', ('m',))], None, 'consumes', regs=('m',)),
    # --- token finishers ----------------------------------------------
    _op('fill',    [_tok('e', 'own', 'Entry', 'uniq', ('t',)), _val('v')], None, 'consumes', regs=('t',)),
    _op('replace', [_tok('o', 'own', 'OccTok', 'uniq', ('t',)), _val('v')], None, 'consumes', regs=('t',)),
    # --- ring / conc_queue endpoints ----------------------------------
    _op('producer', [_cont('r', '&', 'r', 'ring')],
        TypeInfo('Prod', True, 'shr', ('r',)), 'issues', 'shr', ('r',)),
    _op('consumer', [_cont('r', '&', 'r', 'ring')],
        TypeInfo('Cons', True, 'shr', ('r',)), 'issues', 'shr', ('r',)),
    _op('retire',   [_cont('r', '&', 'r', 'ring'), _tok('p', 'own', 'Prod', 'shr', ('r',))],
        None, 'consumes', regs=('r',)),
    _op('endpoint_reset', [_cont('r', '&', 'r', 'ring')], TypeInfo('value'), 'none'),
    _op('close_one', [_cont('t', '&uniq', 't', 'table'),
                      _tok('c', 'own', 'Cursor', 'shr', ('t',))], None, 'consumes', regs=('t',)),
    _op('open',    [_cont('t', '&uniq', 't', 'table'),
                    _tok('tok', 'own', 'MaybeTok', 'uniq', ('r',))],
        TypeInfo('Entry', True, 'uniq', ('t',)), 'issues', 'uniq', ('t', 'r')),
    # --- pure queries returning plain values --------------------------
    _op('next_k', [], TypeInfo('value'), 'none', loanable=False),
    _op('done',   [], TypeInfo('bool'), 'none', loanable=False),
    _op('noop',   [], None, 'none', loanable=False),
]:
    GLOBAL_FORMS[_s.name] = _s

for _s in GLOBAL_FORMS.values():
    check_form_op(_s)
    _s.ties = derive_ties(_s)
