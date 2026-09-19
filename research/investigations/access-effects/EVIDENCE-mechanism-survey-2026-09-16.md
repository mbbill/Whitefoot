# Evidence: mechanism survey for the ownership redesign, 2026-09-16

Four research reports produced for [MECHANISM-MAP.md](MECHANISM-MAP.md) by
research agents under the primary agent's direction on 2026-09-16, reviewed
and spot-checked by the primary agent. They are evidence for that map, not
decisions. Quotes in part A were spot-checked against the specification
baseline v0.57 and the design tree; part B's LangRef quotes were verified by
its author against the live page; citations in parts C and D marked
"unverified" were not confirmed against a venue. Part B reads the parallel and
backend consumers; part D studies threads, locks and external resources.
Remove this file when the map it supports is superseded.

# Part A. Specification inventory of the ownership, reference and effect cluster


Source: worktree `/private/tmp/whitefoot-access-effects-research`, branch `research/access-effects`.
Spec read: `spec/kernel-spec.md` (3595 lines) — section 5 (671–1231), section 6 (1232–1331),
SET-1/SET-2/GIVE-1/CONST-2 (367–670), FN-1 (1572–1620), section 9 (1877–1937),
section 13 (2401–2472), ENT-5/CALL-6 (3089–3207).
Design tree read: `design/language/ownership.md` + `ownership/*`, `effects.md`,
`parallelism.md` + `parallelism/*`, `data-model.md`, `checks-and-proofs.md`.
Context skim: `research/investigations/{reborrow-investigation/MINIMAL-RULE.md,
take-replace/DESIGN.md, move-on-copy/REPORT.md, range-loans/DESIGN.md}`.

All quotes are from the rule text or the named design file, verbatim.
44 rows (M1–M44); the count exceeds the 20–35 estimate because the cluster's
sub-mechanisms are individually load-bearing and separately classifiable.

---

## A.1 Value class and transfer (OWN-1)

| # | mechanism | defined by | hazard/goal the text names | depends on | what the text says breaks without it |
|---|---|---|---|---|---|
| M1 | Copy/affine classification by type | OWN-1 ¶2; `design/language/ownership/copy-classification.md` | "primitives …, shared borrows, `Slice<'r, T>` …, and tag-only enums … copy on use; all other values … are affine" | TYPE-1/TYPE-2 type set; refined by PROV-6 | Design file: every-enum-affine "bought zero safety and forced integer-flag workarounds that cost a measured 1.6 to 1.8 times on scanner kernels" |
| M2 | Explicit `move`, consumed-exactly-once, dead-root kill | OWN-1 ¶3, ¶6 | "consumed exactly once by an explicit `move p`" … "After any consuming use, the whole binding rooting `p` is dead (partial moves kill the whole binding)" | M1; LIV-2 read-out exception; OWN-13, ERR-3, PROV-6 consume routes | Without the whole-binding kill the spec would need per-place flow state; `affine-replacement.md` calls "per-place flow-sensitive type states … exactly what the simplified ownership calculus excludes" |
| M3 | One spelling per meaning (bare-affine error, `move`-on-copy error, judged once per written body) | OWN-1 ¶5, ¶7; FORM-1; SET-2 ¶5; `copy-classification.md` | "copy values are used bare — one spelling per meaning, FORM-1"; "at a concrete instance of a generic template it is not re-made" | M1; FN-2 bounds (`T: copy/affine/linear`) | `copy-classification.md`: "rechecking the spelling per concrete type would contradict authoring a generic body once against its bound" |

## A.2 References, regions, exclusivity (OWN-2 … OWN-5)

| # | mechanism | defined by | hazard/goal the text names | depends on | what breaks without it |
|---|---|---|---|---|---|
| M4 | Two borrow modes `&` / `&uniq`, mode always written, each carrying one region | OWN-2; LEX-1 (l.173–174); `design/language/ownership.md` | "a borrow mode carries one region"; LEX-1: "exclusivity is the invariant; mutation is only its permission" | OWN-3 regions; FORM-8 elision | Nothing else distinguishes shared from exclusive access; OWN-5's whole judgment is keyed on the mode |
| M5 | Lexical regions, uniqueness of REGIONID, outlives-or-equals total reflexive relation | OWN-3; `ownership.md` ("lexical named or unnamed regions") | "Distinct caller-supplied regions are incomparable: any rule requiring an order between them fails closed (reject)" | OWN-11 (loop body region), FORM-8 | `ownership.md`: the alternative "requires a lifetime-solving model beyond the selected explicit regions" |
| M6 | Named-region borrow liveness + destination region check | OWN-4 | a borrow "is live exactly until the end of `'a`'s block (named-region liveness)"; storable/passable/returnable "only if `'a` outlives-or-equals `'b`" | M4, M5; OWN-6 for temporaries | A borrow could be stored into a longer-lived destination; OWN-10 alone constrains creation, not propagation |
| M7 | Resolved-place exclusivity + suspension + unconditional exclusivity invariant | OWN-5 ¶1–3, ¶6 | "no place overlapping resolved(`p`) may be read, written, moved, or borrowed"; "Exclusivity invariant, checked unconditionally: no two live usable `&uniq` borrows have overlapping resolved places" | OWN-6 resolution, OWN-7 overlap, OWN-13, OWN-14 | The one-usable-mutable-path-per-place guarantee (OWN-9) disappears; data-race impossibility (CAP-1 "D1 law") loses its basis |
| M8 | Content reached through a borrow may never be moved (SET-2 sole exception) | OWN-5 ¶5; SET-2 ¶12 | "sound because the exchange leaves no program point at which the referent place lacks exactly one valid owner" | M2, M7; SET-2 | A move through a borrow would leave the far-side owner's place empty with no owner and no reinitialization |
| M9 | View origin sets (finite static set of ultimate storage origins) | OWN-5 ¶7–10, ¶14; VIEW-1; FN-1 ll.1591–1618; `ownership/slice-result-provenance.md` | "Every view value … carries a finite set of possible ultimate storage origins"; "Each runtime slice still has exactly one actual storage origin, and that origin is always a member of the static set" | VIEW-2 formation, OWN-7 overlap, OWN-12 substitution, FN-1 ceiling | Design file: "a body-derived exact summary would make callable contracts depend on implementation bodies and need fixed points for recursive call groups" |
| M10 | No slice-valued control-flow join | OWN-5 ¶11–12 | "This specification defines no slice-valued control-flow join"; the error's restructuring is "use a match or if statement whose arms or branches return the slice directly" | M9; GIVE-1, TYPE-5 | Nothing defines how two arms' origin sets combine; a joined view would have an undefined origin set |
| M11 | View access judged through every origin; loan extent = holder liveness; shared child of an exclusive view | OWN-5 ¶13–20; VIEW-1 ¶3–5; VIEW-2 ¶12–15 | "An access through a view is judged as one access of that view's own loan strength through every resolved-place origin in its set"; "A loan's extent is its holding value's own liveness" | M7, M9, OWN-7, VIEW-1 copy/affine split | Shared views are copy, so without the last-use extent rule a shared loan would have no end; without origin projection a descriptor write would be checked against the descriptor, not the storage |
| M12 | `immutable-const` origin | OWN-5 ¶13, ¶19; OWN-7 ¶9; CONST-2; VIEW-2 ¶9 | "`immutable-const` creates no conflicting access because named const storage is permanently read-only [CONST-2]" | M9, CONST-2 | A const-backed slice would demand an overlap proof it can never need; FN-1's zero-candidate borrow result (l.1604) loses its only legal source |

## A.3 Holders, reborrows, overlap (OWN-6 … OWN-10)

| # | mechanism | defined by | hazard/goal the text names | depends on | what breaks without it |
|---|---|---|---|---|---|
| M13 | Holder + `resolved()` rewriting + call-scoped temporary loans + non-escaping control-header boundary | OWN-6 ¶1–2; `ownership/no-reborrow.md` | "All OWN-5/OWN-7 judgments use resolved places"; header boundary rejected alternative "blocked sequential typed acquisition through a retained unique provider after the header value had completed" | M4, M7, OWN-7 | Exclusivity would be judged on holder bindings, not the places they reach; `no-reborrow.md`: one immutable resolved root is "the load-bearing simplification of the whole frontend-scale checker" |
| M14 | Statement-scoped child reborrow (argument atom only) with parent suspension | OWN-6 ¶3–4; `no-reborrow.md` | "Creating a child suspends `h` until that child's temporary loan ends"; admitted only when "the receiving call's result mode is `own` or `unit`, never a borrow" | M13, OWN-7 sibling judgment, STOR-5 | `no-reborrow.md`: strict no-reborrow "forced the compiler's own source, at about a thousand sites that hand a borrow through to a callee, into threading owned values in place of borrows, a shape the language does not teach" |
| M15 | Candidate-position child reborrow, suspending its parent permanently | OWN-6 ¶3 (exceptions), ¶5; OWN-12 ¶2 | "suspends that holder for the remainder of its life; there is no statement-end resumption, because the child's loan may survive in the bound call result" | M14, M16, FN-1 provenance candidate | Statement-end resumption would give two usable `&uniq` paths to one place once the result holder is bound |
| M16 | Call-result borrow holder rooted at the signature's provenance candidate | OWN-6 ¶6; FN-1 ll.1602–1607; `no-reborrow.md` | "resolved(result holder) = the candidate actual's complete resolved place, even when the callee delivered a narrower suffix of it"; "Nothing here narrows FN-1" | M13, FN-1's one-candidate rule, OWN-4/5/10 | `no-reborrow.md`: "a summary derived from the callee's body would make callable contracts depend on implementations and need fixed points over recursive call groups" |
| M17 | Overlap relation: prefix + provably-different step (distinct fields, unequal literal subscripts) | OWN-7 ¶1–2, ¶6; `ownership.md` | "resolved `p` overlaps resolved `q` iff one is a prefix of the other"; "two places that agree there overlap however their later steps read" | M13 resolution | OWN-5, LIV-2 condition 2, PAR-1/PAR-2 footprints and ENT-5 kills all quantify over this relation; nothing else decides conflict |
| M18 | Range steps with proved disjointness (fixed four-ordering family) | OWN-7 ¶3–5, ¶8–9; VIEW-2; `ownership.md`; `research/investigations/range-loans/DESIGN.md` | "Two ranges under one identical containing path are disjoint when the current ProofContext proves either captured end no greater than the other's captured start, or either range empty"; "The captured endpoints are immutable mathematical values" | M17, ENT-6 goal family, VIEW-2 formation proofs | `ownership.md`: "runtime-width rows and uneven recursive output partitions need independent access to one contiguous allocation"; without it every view of one allocation is a whole-allocation conflict |
| M19 | Reject-when-unsure | OWN-8; `design/language/checks-and-proofs.md` | "the checker rejects any program it cannot prove conformant. Rejection of a sound-but-unprovable program is not a defect" | all of §5 | The fail-closed direction of OWN-3, OWN-7, PAR-1's unresolved-element rule and PROV-6's unconstrained-region rule has no stated ground |
| M20 | Optimizer noalias consequence (non-normative) | OWN-9 | "a live, usable `&uniq` borrow's resolved place is unaliased by any other usable access path"; "the guarantee is one usable mutable path per place" | M7, M14, M15, OWN-13, OWN-14 | The stated payoff of exclusivity — the no-alias fact for lowering — is unstated; LEX-1 separately forbids naming the mechanism `noalias` |
| M21 | Borrow-storage duration | OWN-10 | "`&'a p` is legal only if `p`'s storage outlives `'a`", by root class (own binding / borrow / arena content / named const) | M5 outlives, STOR-1 storage classes, STOR-4 | A borrow could name a region outliving its backing storage; FN-1 l.1604 ("OWN-10 admits no `'b`-region borrow rooted in callee-local storage") loses the premise that makes zero-candidate results legal |

## A.4 Loops, calls, match, return (OWN-11 … OWN-14)

| # | mechanism | defined by | hazard/goal the text names | depends on | what breaks without it |
|---|---|---|---|---|---|
| M22 | Loop body is a region; per-iteration liveness agreement; counted-binder restrictions | OWN-11; LIV-1 ¶4 | "every borrow it carries is dead before the next iteration starts"; a differing backedge status is an error "because one iteration would then start in a state the previous one did not leave" | M5, LIV-1 join rule, LIV-2 reinit, SET-1 | Loop-carried borrows would survive iterations; a body that consumes an outer binding and does not reinitialize it would be admitted |
| M23 | Call region substitution + argument loans + effect-path overlap check + suspended-ancestor exclusion | OWN-12; `effects.md` | "argument borrows are live accesses of their resolved places for the duration of the call"; "Region substitution controls loan liveness and type equality only; it never supplies effect identity" | M4–M7, M9, M14/M15, EFF-2 projection | Two overlapping `&uniq` arguments would be admitted; the callee's writes would not be checked against the caller's live borrows |
| M24 | Match scrutinee ownership (own-place match moves; non-place scrutinee is an owned temporary) | OWN-13 ¶1–2, ¶8; PROV-6 ¶ "An own-place `match` … is the enum form of the same destructuring" | "Matching a place of own mode moves it (the binding dies; binders receive `own` payloads)" | M2 consume routes, GIVE-1 for value forms | Payload extraction from an owned enum has no other route; PROV-6's destructuring-consume pair loses its enum half |
| M25 | Arm-scoped child reborrow (derived borrow-mode payload binder) with root suspension | OWN-13 ¶4–6 | "A borrow-mode payload binder is an arm-scoped child reborrow of the scrutinee place's root binding"; "creating the taken arm's binders from a `uniq`-mode root suspends that root binding" | M7, M13, M17 sibling judgment | Binders would be unrelated bindings and could alias the matched-through root; sibling `uniq` binders would be unchecked |
| M26 | Returned reborrow + closed disposition of every other reborrow form | OWN-14; `no-reborrow.md` | "control leaves the function before the enclosing statement ends, so `h` never resumes and no program point observes `h` and the returned reborrow both usable"; "return position is the sole non-argument position admitted because its creating statement is the function's last program point" | M13, M14, OWN-4, OWN-10, FN-1 | Every written reborrow outside argument position would be undispositioned; a returned child could coexist with a usable parent |

## A.5 Liveness, commits, replacement (LIV-1, LIV-2, SET-1, SET-2)

| # | mechanism | defined by | hazard/goal the text names | depends on | what breaks without it |
|---|---|---|---|---|---|
| M27 | Join-checked liveness → unconditional scope-exit release | LIV-1; STOR-3; `ownership/linearity.md` | "Liveness is join-checked, and that is what makes every scope-exit release unconditional"; "whether a compiler-derived release runs on an edge leaving a scope is not runtime state" | M2 kills, OWN-11 loop head, LIV-2 revival, PROV-6 linear refusal | Releases would need runtime drop flags; the spec's "every binding of that scope that is live on that edge takes its compiler-derived release there, unconditionally" is unavailable |
| M28 | One `set` commit: read-out, multi-target, three admission conditions | LIV-2; SET-1 ¶7–8; `ownership/multi-target-commit.md` | "No writer-observable program point lies between the read-out and the commit, so the statement exhibits no partial move, no dead root, and no uninitialized hole" | M2, M17 (condition 2), STOR-1, STOR-5, ENT-5 kill | `multi-target-commit.md`: a dedicated `swap` "changes a live affine binding's value without death or a new binding, a third mutation path that breaks initialization-keyed facts, loans, and liveness" |
| M29 | Closed writability relation for a commit target | SET-1 ¶2 (ll.586–596); SET-2 ¶2 | writable exactly when "rooted in a live own-mode value binding whose storage is frame-resident, box-owned, arena-owned, or buffer-owned … or reaches a referent through an explicit `deref` of a live usable `&uniq` holder"; "borrowing its descriptor uniquely does not grant unique access to the viewed storage" | M7 (suspension, loans), M11 (view strength), CONST-2, OWN-11 binder rule | Writing through a shared holder, a suspended `&uniq` holder, a `Slice`-rooted path, or a dead root would be admitted |
| M30 | Atomic affine replacement `let x = replace p = e;` | SET-2; STOR-1 ¶5–6; `ownership/affine-replacement.md` | "with no writer-observable program point between them: at every program point the place holds exactly one valid owner, and no temporary uninitialized hole, vacancy state, or move-from-target residue exists" | M8 (its sole exception), M29 writability, STOR-5 region-free target, VIEW-4 | Design file: a bare take "needs per-place vacancy flow, prohibition or repair of every scope-leaving edge in the window, and a meaning for an exclusive borrow over a vacant referent" |

## A.6 Stores, linearity, storage closure (PROV-1, PROV-6, STOR-3/4/5, BLK-4)

| # | mechanism | defined by | hazard/goal the text names | depends on | what breaks without it |
|---|---|---|---|---|---|
| M31 | Store identity is a region; brand travels in the type; one reserving occurrence per region; brand elision rule | PROV-1; STOR-1; BLK-2 | "A store's identity is a region, that region is a component of every type the store backs"; "a value taken from one store is never a value of another"; "A loan region does not become a store brand by elision" | M5 region uniqueness, TYPE-5 exact identity, OWN-12 substitution | A run's release could not resolve which store reclaims it; PROV-6's store-to-provider map has no key |
| M32 | Linearity by absent capability or `linear` modifier; `dispose`; the release graph and its one walk; destructuring consume | PROV-6; STOR-3 ¶ release table; `ownership/linearity.md` | "A scope holds that capability exactly when a binding of that store's provider type is live at that point in that function, reached directly or through a borrow"; "the release the scope exit would have run does not exist" | M1 (refines it), M27 (the edge it refuses), M31 (store→provider), EFF-2 (provider writes) | `linearity.md`: type-fixed linearity "forced dozens of explicit dispose statements per hosted function and pushed writers toward single-exit rewrites" |
| M33 | Linearity bounds on region and type parameters (`'s: affine/linear`, `T: copy/affine/linear`) | PROV-6 ll.939–949; FN-2 | "The three classes form the strict chain `copy < affine < linear`, ordered by what a body may do with a value of the class"; unbounded `'s` is "treated fail-closed as capability-released" | M1, M3, M32 | Generic bodies would be checked fail-closed at every unconstrained region; M3's once-per-body spelling judgment has no bound to read |
| M34 | Borrow-free, region-free storage + confinement closure (stored positions, generic arguments, arena confinement) | STOR-5; BLK-4; STOR-4 | region-bearing content is "a loan whose provenance the storage would hide, a value the region's own release reclaims, and a provider a move would strand"; "Consequently borrow and slice provenance cannot hide in a stored or generic payload" | M9 origins, M31 brands, FN-2, VIEW-1 loan-bearing | "An ordinary borrow can leave a callee only through its direct return value" fails; M9's finite origin sets could be hidden inside aggregates |

## A.7 View-specific restrictions (VIEW-4, VIEW-6)

| # | mechanism | defined by | hazard/goal the text names | depends on | what breaks without it |
|---|---|---|---|---|---|
| M35 | A commit may not displace a live loan | VIEW-4; VIEW-1 ¶5 | "[LIV-2]'s first condition would otherwise admit the commit at a copy target with nothing consumed, and the displaced view's loan would outlive the descriptor whose place it was held from" | M11 (copy view, last-use extent), M28 condition 1, SET-2 region-free class | A `Slice` place could be overwritten while its loan is live, since a copy target needs no read-out |
| M36 | View result ceiling + two results may not share one region | VIEW-6; FN-1 ll.1591–1599, 1612–1619; `slice-result-provenance.md` | "without it a demux written with one region returns views each of which the ceiling says may alias every input"; the shared-view extra ceiling "is what makes the fill-and-publish helper writable" | M9, M16, FN-1, OWN-6 child views | A returned view's origins could not be computed from the signature; recursive calls would need a body fixed point |

## A.8 Effects (EFF-1 … EFF-3)

| # | mechanism | defined by | hazard/goal the text names | depends on | what breaks without it |
|---|---|---|---|---|---|
| M37 | Three effect categories over formal-rooted paths; `allocates`; no capability category | EFF-1; `design/language/effects.md` | "Every `effect_path` is rooted at one formal value parameter of the same callable"; "A REGIONID never names effect identity: regions state loan liveness and outlives relations only" | FN-1 boundary, STOR-1 storage classes, PROV-1 providers | `effects.md`: separate external/blocks/traps categories "described mechanisms rather than state"; without formal rooting a row could not be projected at a call |
| M38 | Exhibited-row exactness both ways; call-boundary projection; reclamation contribution | EFF-2; `effects.md` | "Rows are checked both ways … undeclared-but-exhibited and declared-but-unexhibited are both EFF-2 errors"; "An effect path grants no permission, changes no loan extent"; "No effect root follows an owned value through moves, results, or replacements" | M9 origin projection, M13 holder resolution, M17 overlap, M32 provider writes | `effects.md`: "padding a row would be a place to smuggle effects"; ENT-5(b), PAR-1 footprints and PROV-6 provider accounting all read this projection |
| M39 | `pure` licensing | EFF-3; `effects.md` | "A call whose row is `pure` licenses deduplication and reordering with equal arguments"; "it does not promise termination" | M37, M38 | The only stated optimizer license from the row disappears; `ownership.md` notes "every proof that rests on purity, parallel permission first among them, would collapse" if globals existed |

## A.9 Execution overlap (CAP-1, PAR-1, PAR-2)

| # | mechanism | defined by | hazard/goal the text names | depends on | what breaks without it |
|---|---|---|---|---|---|
| M40 | One authority vocabulary for concurrency | CAP-1; `design/language/parallelism.md` | "`own`, `&`, `&uniq`, place overlap, and the ordinary effect row are the complete authority and interference vocabulary available to [PAR-1] and [PAR-2]"; "Data-race impossibility is D1 law" | M1, M4, M7, M17, M37 | A second sharing classification could be introduced; `permission-judgment.md` rejected "a separate parallel aliasing model" for duplicating or weakening the borrow checker |
| M41 | PAR-1 window permission: footprints, argument loans, intervening statements | PAR-1; `parallelism/permission-judgment.md` | "permission therefore requires of the resulting loan state exactly what [OWN-5] requires of one statement holding all of those loans at once"; "an unresolved element denies permission rather than granting it" | M7, M17, M23 argument loans, M38 projection | Design file: adjacent-pair enumeration made "permission turn on statement adjacency rather than semantics, measured as a 1.9 times wall-time difference" |
| M42 | PAR-2 counted-loop permission: one accumulator, affine element maps, adjacent-range assignments | PAR-2; `parallelism/loop-permission.md`; `range-loans/DESIGN.md` | "multiplication by the same nonzero integer a preserves distinctness, so their refined ranges do not overlap"; "nonnegative s gives `s*i+b+s <= s*j+b`"; "every other element injectivity argument deny permission rather than starting proof search" | M17/M18 overlap and ranges, M41 footprint forms, OP-4 retained bounds, ENT-6 domain | Design file: "reductions delivered the measured parallel win and runtime-width stencil rows and block-local workspaces need ordinary helper calls on ranges whose independence does not depend on worker count" |

## A.10 Fact maintenance across writes and calls (ENT-5, CALL-6)

| # | mechanism | defined by | hazard/goal the text names | depends on | what breaks without it |
|---|---|---|---|---|---|
| M43\* | ENT-5 support and the four kill events (a)–(d) | ENT-5 (ll.3135–3168); `effects.md` | support includes "every borrow or box/arena holder binding any of its places reads through by `deref`, a bound call-result holder included — its resolved place is the candidate actual's complete resolved place [OWN-6]"; a call kills where a projected write "projects onto a caller place or origin set containing a place that overlaps [OWN-7]" | M2 consumes, M16 result holders, M17 overlap, M28/M30 commits, M38 projection, MSR-2 descriptor boundary | `effects.md`: "preserving a changed inner run's former length is unsound and killing unrelated facts prevents useful modular proofs" |
| M44\* | CALL-6 publication: instantiation, support, establishment, routing, consistency | CALL-6 (ll.3089–3109) | "Those writes kill pre-call facts, not the exit relation that the verified callee establishes afterwards"; a routed relation "is not deferred to the arm, so an [ENT-5] event lying between the call and the arm kills a relation whose support it removes" | M43, M38 projection, MSR-3 denotations, FN-9 | Verified postconditions would be erased by the same call's own projected writes; a contradictory published set would make "every fact at every caller" derivable |

\* Numbered M43/M44 for continuity; the table's row count is 44, of which M43/M44 are the
proof-fact half. (Rows M1–M42 in §A.1–A.9 plus these two.)

---

# B. Classification

Buckets: **(1)** keeps "permission travels with the reference" sound; **(2)** storage-lifecycle
accounting; **(3)** interference reasoning (parallel overlap / optimizer facts);
**(4)** proof-fact maintenance; **(5)** value-class semantics; **(6)** other.

| # | mechanism | bucket | justification (secondary noted) |
|---|---|---|---|
| M1 | copy/affine classification | 5 | Fixes duplication vs. single-owner transfer per type; nothing about references. |
| M2 | explicit move / dead-root kill | 5 | Defines what transfer is and when a binding stops owning. Secondary (2): the kill is what makes M27's release decision static. |
| M3 | one spelling per meaning | 6 (surface-form/diagnostic discipline) | FORM-1 spelling rule; it selects a syntax, not a safety property. Secondary (5): the classes it spells are M1's. |
| M4 | `&` / `&uniq` modes | 1 | The mode *is* the permission the reference carries; with per-storage-identity tracking a pointer would need no mode. |
| M5 | lexical regions + outlives | 1 | Regions exist to bound how long a reference's permission lasts; PROV-1 reuses the syntax for brands but that is a separate mechanism (M31). |
| M6 | named-region borrow liveness + destination check | 1 | Constrains where a permission-carrying value may be stored or returned. |
| M7 | resolved-place exclusivity + suspension | 1 | The core "one usable permission per place" invariant. Secondary (3): OWN-9 and PAR-1 consume it directly. |
| M8 | no move through a borrow (SET-2 exception) | 1 | Keyed on how the place is *reached* (own-rooted vs. through a borrow). Secondary (2): the stated ground is the no-hole/one-owner property. |
| M9 | view origin sets | 1 | A view is a permission-carrying reference whose target must be recovered; per-storage-identity tracking would name the storage directly. Secondary (3)+(4): PAR-2 and EFF-2/ENT-5 project through it. |
| M10 | no slice-valued join | 6 (missing-mechanism restriction) | Exists because no join over M9's origin sets is defined; it is a refusal, not a positive rule. Secondary (1). |
| M11 | view access via origins, last-use extent, shared child of exclusive | 1 | Determines when the view's loan is in force and against what. Secondary (2): the copy-view last-use extent is an end-of-life question. |
| M12 | `immutable-const` origin | 6 (fail-open exemption) | A distinguished origin that exempts read-only static storage from conflict. Secondary (1). |
| M13 | holder resolution, temporaries, header boundary | 1 | Rewrites reference-rooted places back to the storage they reach; meaningless without reference-carried permission. |
| M14 | statement-scoped child reborrow | 1 | Transfers permission from a holder to a callee for one statement and suspends the parent. |
| M15 | candidate-position child (permanent suspension) | 1 | Same, for a loan that survives in a bound result. |
| M16 | call-result borrow holder / signature provenance | 1 | Determines which caller place a returned permission refers to. Secondary (4): ENT-5 support explicitly reads this resolved place. |
| M17 | overlap relation | 3 | The interference/alias primitive; a per-storage-identity model still needs it. Secondary (1) (OWN-5) and (4) (ENT-5 kills). |
| M18 | proved range disjointness | 3 | Exists so disjoint sub-ranges of one allocation may be accessed independently. Secondary (1) and (4). |
| M19 | reject-when-unsure | 6 (global acceptance stance) | Decidability/soundness policy over the whole checker, not a hazard-specific rule. |
| M20 | OWN-9 noalias consequence | 3 | Explicitly the optimizer fact exclusivity buys; non-normative. |
| M21 | borrow-storage duration | 1 | Constrains creating a reference whose region outlives its backing. Secondary (2): the hazard is storage ending first. |
| M22 | loop body region + liveness agreement + binder restrictions | 2 | The named hazard is a binding's live/dead state differing across the backedge, i.e. release accounting. Secondary (1): the region half kills per-iteration borrows. |
| M23 | call substitution + argument loans | 1 | Makes the callee's use of passed references a live access of caller places. Secondary (4): the effect-row overlap check. |
| M24 | match scrutinee ownership | 5 | Which consuming route a match is, and what mode binders receive. |
| M25 | arm-scoped child reborrow + root suspension | 1 | Derives a borrow-mode binder as a child of the scrutinee root's permission. |
| M26 | returned reborrow + closed disposition | 1 | The one non-argument position where a child permission may leave a body. |
| M27 | join-checked liveness + unconditional release | 2 | Directly "who releases, on which edge, without runtime state". Secondary (4): join agreement is also an ENT-5/join premise. |
| M28 | one `set` commit / read-out | 2 | No partial move, no dead root, no uninitialized hole. Secondary (5) transfer and (4) kill event. |
| M29 | closed writability relation | 1 | Decides whether a target path carries write permission, keyed on holder mode, suspension and view strength. |
| M30 | atomic `replace` | 2 | The no-hole affine exchange. Secondary (5): it is the affine transfer form for stored places. |
| M31 | store identity as region + brand resolution | 2 | Says which store backs a value and therefore who reclaims it. Secondary (6): it reuses region syntax for a non-loan purpose. |
| M32 | linearity, `dispose`, release graph, destructuring consume | 2 | Reclamation accounting: who holds the capability, what the walk visits, which edges refuse. Secondary (5). |
| M33 | linearity bounds on parameters | 5 | Names the class a generic body is written for. Secondary (2): it replaces PROV-6's fail-closed capability test. |
| M34 | borrow-free / region-free storage + confinement | 1 | Prevents a permission-carrying value from hiding inside storage. Secondary (2): the provider-stranding and arena-reclaim halves. |
| M35 | VIEW-4 commit may not displace a live loan | 1 | Stops a live reference's permission from being silently dropped by a copy-target commit. Secondary (2). |
| M36 | VIEW-6 result ceiling + distinct result regions | 1 | Signature-only provenance for a returned permission. Secondary (3): the demux aliasing case. |
| M37 | EFF-1 categories + formal-rooted paths | 4 | The row is the signature-level statement of what a call observes and changes, which is what caller facts are tested against. Secondary (3) (PAR-1 footprints) and (2) (`allocates`/provider writes). |
| M38 | EFF-2 exactness + projection + reclamation | 4 | Same, at the call boundary: "each projected write kills overlapping support under ENT-5". Secondary (3) and (2). |
| M39 | EFF-3 `pure` licensing | 3 | Purely an optimizer/reordering license. |
| M40 | CAP-1 | 6 (vocabulary closure) | Forbids a second permission or sharing classification; it adds no judgment of its own. Secondary (3). |
| M41 | PAR-1 window permission | 3 | Interference reasoning for statement overlap. |
| M42 | PAR-2 counted-loop permission | 3 | Interference reasoning for iteration overlap plus accumulator recombination. |
| M43 | ENT-5 support + kill events | 4 | Exactly "which facts survive a write or a call". |
| M44 | CALL-6 publication and routing | 4 | Where a declared relation becomes a fact and how kills order against it. |

Bucket totals (primary bucket only; several rows carry a secondary bucket as noted):

| bucket | count | rows |
|---|---|---|
| (1) permission travels with the reference | 19 | M4, M5, M6, M7, M8, M9, M11, M13, M14, M15, M16, M21, M23, M25, M26, M29, M34, M35, M36 |
| (2) storage-lifecycle accounting | 6 | M22, M27, M28, M30, M31, M32 |
| (3) interference reasoning | 6 | M17, M18, M20, M39, M41, M42 |
| (4) proof-fact maintenance | 4 | M37, M38, M43, M44 |
| (5) value-class semantics | 4 | M1, M2, M24, M33 |
| (6) other | 5 | M3, M10, M12, M19, M40 |

**Observation, not a recommendation:** bucket (1) is by far the largest group, and every
member of it is stated in terms of *how a place is reached* (holder, mode, region,
suspension, origin set) rather than in terms of the storage itself.

---

# C. Text that acknowledges a mechanism is provisional, deferred, or forced by another restriction

1. **Section 5 as a whole.** Heading, l.671: "## 5. Ownership, regions, borrows
   **(PROVISIONAL pending formal-calculus reconciliation)**".

2. **OWN-6's reborrow family is a fragment.** l.747: "Bound children, result-carrying
   children (reference-result provenance), `uniq`-to-`shared` downgrade, `match`-binder
   parents, and written grandchild chains through a bound direct reborrow are **DEFERRED
   with recorded delta**".

3. **OWN-14's returned reborrow is admitted for a structural accident, and the rest is
   deferred.** l.802: "Bound reborrows, `give`-position and stored reborrows,
   `uniq`-to-`shared` downgrade, and `match`-binder parents … remain **DEFERRED with
   recorded delta**; return position is the sole non-argument position admitted **because
   its creating statement is the function's last program point**."

4. **OWN-13 arm-end resumption is deferred, and the arm-result join is provisional.**
   l.790: "Arm-end resumption of a matched-through `uniq` root is **DEFERRED with recorded
   delta**." l.793: "This arm-result region join is an additive reuse of the return-of-borrow
   judgment and is **PROVISIONAL** pending confirmation against the formalized calculus
   before section-5 ratification (D1a)."

5. **The reborrow family exists because strict no-reborrow broke the compiler's own
   source.** `design/language/ownership/no-reborrow.md`: strict no-reborrow "forced the
   compiler's own source, **at about a thousand sites that hand a borrow through to a
   callee**, into threading owned values in place of borrows, **a shape the language does
   not teach**".

6. **LIV-1 exists to refuse drop flags.** l.804: "Liveness is join-checked, **and that is
   what makes every scope-exit release unconditional**"; l.809: "whether a compiler-derived
   release runs on an edge leaving a scope **is not runtime state**". `linearity.md` gives
   the paired motive for PROV-6's scope-relative linearity: type-fixed linearity "forced
   **dozens of explicit dispose statements per hosted function** and pushed writers toward
   single-exit rewrites".

7. **PROV-6's capability-by-binding is a syntactic proxy, with a fail-closed default.**
   l.886: "A scope holds that capability **exactly when a binding of that store's provider
   type is live at that point in that function**, reached directly or through a borrow";
   l.933: "a value branded by an unconstrained `'s` is treated **fail-closed** as
   capability-released."

8. **The no-slice-valued-join error exists because the mechanism is absent.** l.716: "This
   specification **defines no** slice-valued control-flow join" — the diagnostic then
   instructs the writer to restructure. The same shape appears at FN-1 l.1600: "This
   specification **has no signature summary** that carries both the returned descriptor's
   source-place provenance and the underlying slice value's complete origin set", which is
   why a borrow-mode slice result is rejected outright.

9. **Slice provenance stops at direct positions by deferral, not by design closure.**
   STOR-5 l.1299: "Per-leaf provenance inside stored values, `Result`, `Option`, user
   nominals, boxes, arenas, and other generic instances is a **DEFERRED specification
   addition**; a compiler limitation does not select that boundary."

10. **The `allocates(arena 'r)` spelling is explicitly transitional.** EFF-1 l.1905: "That
    alternative of the production **is transitional and retires with `arena<'r, T>`** and
    its `arena_new` row".

11. **The mode vocabulary itself is a reduced axis.** LEX-1 l.174: "**DEFERRED with
    recorded delta**: the two-axis mode vocabulary (exclusivity x write-permission, adding
    frozen/exclusive-read and an explicitly bounded shared-write form)."

12. **Fail-closed answers where the relation is undefined.** OWN-3 l.694: "Distinct
    caller-supplied regions are incomparable: **any rule requiring an order between them
    fails closed (reject)**." BLK-4 l.1170 repeats the reason: "the quantifier is the whole
    of it, because [OWN-3] makes two caller-supplied regions incomparable and **fail-closed
    is the answer there**."

---

# D. Ambiguities and places where the text does not settle the question

- **Overlap's owner.** OWN-7 defines one relation consumed by OWN-5, LIV-2 condition 2,
  PAR-1/PAR-2 footprints and ENT-5 kill (a)/(b). The text never says which consumer the
  relation is *for*, so M17's primary bucket is a judgment call, not a reading.
- **View loan strength vs. exclusivity.** OWN-5 l.722 says an exclusive view "admits a
  second **shared** one: that second formation is the shared child reborrow of a unique
  loan [OWN-6] applied to a view rather than to a place". Whether this is the same
  mechanism as OWN-6's child reborrow or an analogue is stated by analogy only.
- **Suspension endpoints.** OWN-6 gives three different endpoints (statement end,
  non-escaping header boundary, never-resumes for candidate-position and returned
  reborrows) and OWN-13 a fourth (end of the binder's derived region block). The text
  enumerates them but states no single principle from which they follow.
- **`resolved(result holder)`.** OWN-6 l.743 makes it "the candidate actual's **complete**
  resolved place, even when the callee delivered a narrower suffix of it", which is
  deliberately coarser than the loan the callee actually returned; the text gives no rule
  for recovering the narrower place, and ENT-5 l.3135 inherits the coarse place for kills.
- **EFF-1 vs. ENT-5 reach.** ENT-5(b) says how far a projected write reaches "is classified
  by [CALL-1] through [CALL-3] from the callee's declaration and by nothing else [CALL-5]";
  those rules were not in the assigned reading, so the descriptor-vs-element boundary for a
  projected write is stated here only as it appears in ENT-5 and MSR-2.


# Part B. Consumers of ownership information and what they read


Read-only survey of `research/access-effects` worktree at
`/private/tmp/whitefoot-access-effects-research`, spec `spec/kernel-spec.md`
(3595 lines), compiler `compiler/src`.

Purpose: enumerate exactly which source facts today's consumers read, so a
replacement model (static storage identity in the pointer type + per-identity
storage state + effect footprints over identities + proved index/range
disjointness) can be checked for information loss.

---

## 0. Orientation: the three layers that exist today

| Layer | Artifact | Where |
|---|---|---|
| Fact producer | resolved places, loans, effect rows, retained proof images | `compiler/src/semantic/places.rs`, `entailment.rs` |
| Fact consumers | PAR-1 window permission, PAR-2 loop permission, ENT-5 fact kill, EFF-2 call projection | `permission.rs`, `loop_permission.rs`, `entailment/flow.rs` |
| Backend | **nothing** — no alias or effect metadata is emitted at all | `compiler/src/backend/emitter.rs:4` |

The single most important empirical finding of this survey:

> **`compiler/src/backend/emitter.rs:4`**: *"Emission consumes only
> target-independent IR. It preserves every retained check, **emits no overflow
> or alias promises**, initializes complete aggregate representations, and keeps
> a defensive abort edge for enum discriminants."*

`grep -rn "noalias|alias.scope|readonly|readnone|nocapture|dereferenceable|invariant.load|tbaa|writeonly|captures("` over
`compiler/src`, `compiler/tests`, and the backend's `.c`/`.ll`/`.h` runtime
sources returns **zero matches**. `compiler/src/backend/abi.rs:3-4` states it
as a design property: *"This describes value representation only. **Source
access modes do not select aliasing permissions**, and a declaration and
definition use the same ABI."*

There is an explicit tripwire test forbidding the one attribute that could be
emitted wrongly today: `compiler/src/backend/tests/effect_attributes.rs`
asserts no emitted module contains `willreturn`, and its doc comment says the
test *"is a canary for the effect-attribute channel that a future change will
open"*.

So: the *only* thing the whole ownership/effect fact-base currently buys at the
backend is **PAR-1/PAR-2 overlap actualization**, carried as
`IrOverlap { members: Vec<IrValueId> }` (`compiler/src/lowering.rs:1587`) and
`IrOperation::LoopSplit { splitter, chunk, seed, lower, upper, captures, weight, work }`
(`compiler/src/lowering.rs:1357`). Neither carries a single alias or effect
fact into LLVM. The alias and effect *metadata* consumers described in
`docs/why-whitefoot.md` §4 and §5 are **retired prototype (`democ`)
experiments**, not current compiler behavior; §4 says so explicitly
(*"The current compiler emits no effect-derived attributes"*) and §5 likewise
(*"The current compiler checks the borrow relation but does not emit that alias
metadata"*).

---

## A. Consumer → facts read

Representation column says the granularity at which the consumer indexes the
fact.

### A.1 PAR-1 — sibling-call window permission

Spec: `spec/kernel-spec.md:2406-2434`. Impl: `compiler/src/semantic/permission.rs`.

| # | Fact read | Source rule | Representation | Where |
|---|---|---|---|---|
| 1 | **Statement form**: each member is one *declared-function* call written as `let_stmt` ordinary rhs, or the scrutinee of `match_stmt` / `value_match` / `value_if` | PAR-1 ¶3 | per statement, by checked-tree shape | `permission.rs:1531 candidate_of` |
| 2 | **Dataflow**: does an argument of s2 read a `BindingId` s1 defines (plus the two interposed clauses) | PAR-1 ¶4 | per statement, over `BindingId` sets | `permission.rs:1592 collect_used_bindings`, `1607 visit_read_bindings` |
| 3 | **Callee declared effect row**: `reads(path)`, `writes(path)`, `allocates(arena 'r)` — *formal-rooted static field paths only* | EFF-2, PAR-1 ¶5 | per call, per declared path | `PermissionSignature` `permission.rs:112-121` |
| 4 | **EFF-2 call-boundary projection**: each declared path's root formal selects its actual, the path's static field suffix is appended to the actual's *resolved place* | EFF-2 ¶"At a call" | per call × per declared path | `permission.rs:1036 user_call_footprint`; `place.extend_fields(&path.fields)` |
| 5 | **Consumed `own` actuals**: the caller place an `own` argument transfers away is a *written* footprint element | PAR-1 ¶5 | per call × per `own` argument | `permission.rs:1104 consumed_place` |
| 6 | **Region substitution for arenas**: `allocates(arena 'r)` becomes a written footprint element keyed by the *caller region `DeclarationId`*, not by a place | PAR-1 ¶5, EFF-2 `:1905` | per call × per allocates entry | `Access::Arena` `permission.rs:167-171` |
| 7 | **Argument borrow modes → loans**: `&uniq 'r` actual ⇒ `LoanStrength::Exclusive` on its resolved place; `&'r` ⇒ `Shared`; `own` ⇒ none. **Independent of the row** — a `pure` callee taking `&uniq` still holds an exclusive loan | OWN-12, PAR-1 ¶6 | per call × per borrow-mode parameter | `permission.rs:1084-1101`; the `Loan` doc at `:172-180` is explicit |
| 8 | **Argument-expression reads** (`O`): what the *caller thread* reads while evaluating operands, before the call. Address formation is **not** a read (`BorrowBuffer`/`BorrowAddressed`/`BorrowBox`/`ReborrowAddressed` contribute nothing) | PAR-1 ¶5 sentence 2 | per statement, per operand sub-expression, resolved to one place each | `permission.rs:1671 collect_operand_reads` |
| 9 | **View origin** for a slice actual or a slice read: `SliceIndex`/`SliceMeasure`/`SliceOf` resolve to the *origin place* the view was formed over, not the descriptor | VIEW-1, VIEW-2, OWN-5 origin-set paragraph | per slice-typed value, as **one** `Option<ResolvedPlace>` | `places.rs:469 view_origin`, `:484 view_origin_of` |
| 10 | **OWN-7 resolved-place overlap**: `root == root && !paths_diverge(path,path)`; a step separates only for two different `Field(u32)`, or two `Subscript` offsets that are **both literals with unequal values** | OWN-7 `:749-760` | pairwise over footprint elements | `places.rs:173 overlaps`, `:208 paths_diverge`, `:61 PlaceOffset::provably_distinct` |
| 11 | **Proved range separation** (VIEW-2 ranges): a `PlaceStep::Range(RangeId)` pair is separated only if the ProofContext discharged one of the four non-strict orderings and that outcome was retained | OWN-7 range paragraph `:752-755` | per `(RangeId, RangeId)` pair, a global set per function | `places.rs:308 separated_ranges`, `:220 paths_diverge_with_ranges`; installed from `outcome.formed_range_separation` at `places.rs:337` |
| 12 | **OWN-5 loan/access matrix**: exclusive loan excludes every overlapping access; shared loan excludes writes only; two loans conflict iff at least one is exclusive | OWN-5 `:699-708`, PAR-1 ¶7 | pairwise loan × (loan \| W \| R \| O) | `permission.rs:196-219 LoanStrength::excludes_use/excludes_loan` |
| 13 | **Exit edges** of every window statement | PAR-1 ¶12 | per statement | `ExitKind` `permission.rs:134` |
| 14 | **Fail-closed unresolved marker**: any footprint element, operand read, or loan whose caller place is unresolved denies | PAR-1 ¶8 | per statement, one `Option<NodePath>` flag each for row-projection and operand halves | `Footprint::unresolved` / `operand_unresolved` `permission.rs:1174-1211` |

Note on #7 vs #3 — these are **two independent channels**. The design records
it: `design/language/parallelism/permission-judgment.md` "The loan judgment
reuses the borrow checker's own overlap vocabulary lifted from one call's
arguments to the statements of one window". `permission.rs:172-180`: *"A loan
is not a use. The callee's declared row says what the callee* does *through the
borrow; the loan says what the borrow* forbids everyone else *while it is
live."*

Note on #9 — the compiler tracks **one** origin (`Option<ResolvedPlace>`), while
the spec (`OWN-5`, VIEW paragraphs at `:709-724`) defines a **finite set** of
possible origins. A callee-returned view yields `None` ⇒ deny
(`places.rs:295-298`: *"`None` means … a view whose origin this prepass does not
resolve, such as one a callee returned … Every consumer reads that as
unresolved and fails closed."*). A **formal** slice parameter anchors at
`ResolvedPlace::binding(parameter.binding)` — an opaque self-origin
(`places.rs:328-331`) — which is why intra-function checking never proves two
formal slices disjoint, exactly as OWN-7's last line requires (*"Formal-slice
origins are substituted before caller overlap checking; they never establish
that two actual sources are disjoint"*).

### A.2 PAR-2 — counted-loop permission

Spec: `spec/kernel-spec.md:2435-2465`. Impl: `compiler/src/semantic/loop_permission.rs`.

PAR-2 reads **everything in A.1** ("forming every written, read, and operand-read
footprint of a statement of B exactly as [PAR-1] forms one") plus:

| # | Fact read | Source rule | Representation | Where |
|---|---|---|---|---|
| 15 | **Iteration-own classification**: is the place's root a `BindingId` that block B itself introduces | PAR-2 ¶"Every place a footprint of B writes" | per place, over a `BindingId` set | `loop_permission.rs:917 is_iteration_own`; set built by `:1380 collect_introduced` |
| 16 | **Accumulator**: at most one whole-place write rooted outside L; every occurrence is one operand of `set a = a ⊕ e` / `set a = e ⊕ a`; ⊕ fixed across B and drawn from a **closed set of 10** exactly-associative total ops (`+wrap *wrap iand ior ixor imin imax band bor bxor`) | PAR-2 ¶3-4 | per loop: one `BindingId` + one `LoopCombine` | `loop_permission.rs:1309 combine_of`, `:1352` closed list |
| 17 | **Accumulator read count by binding**: the accumulator binding is read nowhere else in B | PAR-2 ¶3 | per loop, count of read occurrences of one `BindingId` | `loop_permission.rs:421 ReadOccurrence`; rationale at module doc "Why counting an accumulator's reads by binding is complete" |
| 18 | **Proved single-binder affine element map**: the *already discharged* `[OP-4]` bounds obligation at a direct subscript, with its **retained exact canonical value image** `a*i + b`, `a ≠ 0`, `i` = L's compiler-owned binder, `a`/`b` mathematical integer constants, no other symbolic term | PAR-2 ¶6-8 | per `set_stmt` target subscript / per element read; `ProvedAffineIndexMap { loop_id, coefficient: i128, constant: i128 }` | `entailment.rs:288`; consumed at `loop_permission.rs:693 proven_affine_map` |
| 19 | **Map agreement per root**: all writes to one mapped resolved root carry identical `(a,b)`; every operand read through that root is a direct subscript with the *same* `(a,b)`. Whole-root read, differing map, any overlapping loan, or unresolved place denies | PAR-2 ¶11 | per resolved root × per access | `loop_permission.rs:387 ProvenElementRange`, `:401 ProvenElementRead` |
| 20 | **Proved adjacent-range assignment**: an *exclusive* `[VIEW-2]` formation whose discharged endpoint domain retains exact images `[s*i+b, s*i+b+s)`, with retained proofs `0 <= s`, `0 <= b`, `s`/`b` fixed at L's preheader. Stride and base are **runtime values**, not constants | PAR-2 ¶"A proved adjacent-range assignment" | per formation; `ProvedRangePartition { loop_id, range: RangeId, stride: AffineForm, base: AffineForm, stride_nonnegative: DerivationId, base_nonnegative: DerivationId }` | `entailment.rs:299-306`; consumed at `loop_permission.rs:932 record_view_formation` |
| 21 | **Containment by resolved-place prefix**: a descriptor formed inside B inherits an assignment only when its complete origin path is a descendant; checked with `contains_path` (positive matching selections, *not* mere absence of divergence) | PAR-2 ¶"The source storage…" | per place, against the recorded `ProvenViewRange.place` | `loop_permission.rs:926 assigned_range`; `places.rs:186 contains_path` |
| 22 | **One partition per overlapping source**: two `ProvenViewRange`s whose *source* places overlap must name the same source place and identical `(stride, base)`; otherwise deny | PAR-2 ¶"All assigned ranges…" | pairwise over recorded partitions | `loop_permission.rs:975-985` |
| 23 | **Exclusive-loan containment**: every exclusive loan of a footprint of B — argument borrows *and* view formations — must be iteration-own **or** contained in an assigned range. Shared loans need no condition | PAR-2 ¶"Every place a footprint of B holds an exclusive loan on" | per loan | `loop_permission.rs:1003 record_loan` |
| 24 | **Borrow-form admission inside B**: a written borrow's shared-vs-uniq mode is **erased from the checked tree**, so any non-view borrow formation in B is admitted only when its place is iteration-own; otherwise refuse | PAR-2 ¶"An ordinary non-call borrow formation … still denies" | per expression | `loop_permission.rs:911 admits_borrow_forms`, `:1428 borrows_only_iteration_own` |
| 25 | **Exit edges**: no `return`, `give`, `break` to L or enclosing, no `propagate_let_rhs` | PAR-2 ¶last condition | per statement | `loop_permission.rs:1100 leaves` |
| 26 | **Exhaustive statement-form classification** — an unclassified form refuses ahead of the numbered conditions, because a missed statement would contribute an *empty* footprint and widen permission | design `parallelism/permission-judgment.md` "Unresolved overlap or an unsupported interposed statement form denies permission" | per statement | `loop_permission.rs:1104 refuse_form` |

### A.3 ENT-5 — fact kill

Spec: `spec/kernel-spec.md:3135-3175`. Impl: `compiler/src/semantic/entailment/flow.rs`.

| # | Fact read | Source rule | Representation | Where |
|---|---|---|---|---|
| 27 | **Support of an L0 fact** = every tracked place in its terms; every counted capture term; for a measure over P, **P's descriptor storage** and the support of every offset in P but **not** P's element storage; every borrow / box / arena holder any of its places `deref`s through, bound call-result holder included | ENT-5 ¶1 | per fact, a set of resolved places + `BindingId`s | `flow.rs` support computation |
| 28 | **Kill event (a)**: a SET-1 / SET-2 commit whose *resolved target* overlaps a support member under OWN-7; or the compiler-owned `for_stmt` binder update when the binder is a support member | ENT-5 ¶(a) | per statement edge | `KillEvent::Write { place, element, source }` `flow.rs:80-88` |
| 29 | **Element-vs-descriptor granularity**: `element: bool` — a write at an element position kills the written element's measures and **none of the collection's own**; a write to a sibling field kills neither | ENT-5 ¶(a), MSR-2 | per write event, one bool | `flow.rs:82-86` |
| 30 | **Kill event (b)**: a call one of whose **EFF-2 boundary-projected `writes`** occurrences projects onto a caller place or origin set overlapping a support member. *"a callee writing only through one `&uniq` actual kills exactly the facts whose support overlaps that actual's resolved place, and a call whose row carries no `writes` kills nothing"* | ENT-5 ¶(b) | per call × per declared write path | `KillEvent::Write`, `EntryImageHolderWrite` |
| 31 | **Reach classification CALL-1..CALL-3** — how far a projected write reaches (descriptor storage vs viewed range's element storage) is *"classified … from the callee's declaration and by nothing else [CALL-5], and neither the actual's spelling nor the callee's body is consulted"* | CALL-1 `:2925`, CALL-3 `:2939`, CALL-5 `:2952` | per parameter, from **declared mode + declared type** | `flow.rs` element flag selection |
| 32 | **Shared-borrow immunity (CALL-1)**: *"For an argument whose declared parameter mode is `&'r`, of any type, run and view included, the call is a kill event for no fact"*, grounded on OWN-5's shared-holder write prohibition | CALL-1 | per parameter mode | — |
| 33 | **Kill event (c)**: a consuming use of a support member's **root** | ENT-5 ¶(c) | per binding | `KillEvent::Consume { binding }` `flow.rs:90-93` |
| 34 | **Kill event (d)**: edges leaving a holder's region, a binding's lexical scope, or a capture term's counted construct | ENT-5 ¶(d) | per edge, over scope/region sets | `flow.rs` scope kill path |

ENT-5 therefore reads **exactly two structural notions**: (i) OWN-7
resolved-place overlap (spec `:2733`: *"Term identity thus under-approximates
aliasing, while kills use [OWN-7]'s resolved-place overlap relation and
over-approximate it"*), and (ii) a per-parameter *reach* classification from
declared mode+type. It does **not** read borrow *strength* except through
CALL-1's shared immunity, and it does not read loans at all.

### A.4 EFF-2 — call-boundary projection

Spec: `spec/kernel-spec.md:1908-1945`.

| # | Fact read | Representation |
|---|---|---|
| 35 | Declared `reads`/`writes`/`allocates` paths — **formal root + static struct field suffix**; *"An access rooted in a formal contributes the most precise static struct path EFF-1 admits for that resolved place; a dynamic element or range maps to its nearest statically nameable enclosing path"* | per declaration, list of `CheckedStatePath { root: DeclarationId, fields: Vec<u32> }` |
| 36 | Per-parameter *naming* rule: a borrow parameter's path names the **borrowed referent**; a direct `Slice<'r,T>` parameter's path names the **viewed backing state, not the descriptor**; an `own` parameter's path names that parameter's **current storage** | per parameter |
| 37 | At the call: root formal → actual, append field suffix to the actual's resolved place. Holder resolution reaches the borrowed referent; **a view actual projects through its complete origin set**; a multi-origin view contributes the *deduplicated union* of its formal-rooted origins | per call × per path |
| 38 | Region substitution for `allocates(arena 'r)`; *"A REGIONID never names effect identity: regions state loan liveness and outlives relations only"* | per call × per region parameter |
| 39 | **No permission**: *"An effect path grants no permission, changes no loan extent, and cannot narrow a borrow of a whole aggregate to one field"* | — |
| 40 | Both-ways check: undeclared-but-exhibited and declared-but-unexhibited are both errors; body contribution is **syntactic over the complete body**, unnarrowed by path conditions, constant evaluation, proofs, or optimizer results | per declaration |

The key granularity limit: **EFF-2 paths are static struct paths only.** There
is no element index, no range, and no per-span form in a declared row. Element
and range precision exists *only* inside PAR-2's permission refinement
(`ProvedAffineIndexMap`, `ProvedRangePartition`), and PAR-2 says explicitly
*"For permission only, this fixed form refines the ordinary whole-collection
write footprint to the single-element range"* — the refinement never enters the
declared row and never crosses a call boundary except as
*"the [EFF-2] projection of a helper's declared row counts as an access on its
actual range"* (PAR-2 ¶"All assigned ranges…"), i.e. the helper's whole-slice
row is read as covering the tile that its slice actual names.

### A.5 Backend alias / effect metadata emission

| Consumer | Status | Facts read | LLVM form |
|---|---|---|---|
| Overlap actualization | **live** | PAR-1 window verdicts, PAR-2 loop verdicts | none — outlined thunks + `wf__par_try_fork`; `IrOverlap`, `IrOperation::LoopSplit` |
| Alias metadata on loaded pointers | **retired prototype (`democ`)** | `buffer<T>` affine single-owner (OWN-1) ⇒ two live buffer values never overlap; `&uniq` exclusivity (OWN-5) + singleton loan provenance | one `!alias.scope` per uniq-rooted buffer field, one for struct memory, one shared class for all shared-borrow-rooted access; plus `dereferenceable`/`align` on borrow params |
| Effect attributes | **retired prototype (`democ`)** | `pure` row + a separate derived-totality tier | `nounwind willreturn memory(none)` on both `define` and `declare` |
| Anything in the current backend | — | — | only `align` on `alloca`, `getelementptr inbounds`, `mul nuw`, `noreturn` on the abort helper |

### A.6 Backend storage reuse — the one *live* backend consumer of ownership

`compiler/src/backend/storage.rs:1-12`: *"Source ownership is already checked
and is not inferred here. A value gets independent backing unless complete CFG
liveness proves that a selected update, edge transfer or alternative return can
reuse backing whose old contents are dead. … A consumed call input and its
consumed struct result field may occupy the same field of the complete result
allocation after a separate interference check."*

| # | Fact read | Representation | Where |
|---|---|---|---|
| 41 | **`IrSourceMode`** per parameter and per result: `Own` / `Shared` / `Unique` — the source mode, retained into IR *"independently of representation and erased regions"* | per function signature, one enum per parameter + one for the result | `lowering.rs:1638-1652 IrSourceMode`, `IrSourceSignature` |
| 42 | IR-level CFG liveness of the backing slot | per IR value, per block | `storage.rs:334 select_destinations`, `:740 call_reuse_operand` |
| 43 | `IrOverlap` non-emptiness — reuse is **disabled** whenever a function carries any actualized overlap group (`storage.rs:766, :786`) | per function | `storage.rs` |
| 44 | Exposure (`AddressOf` freezes a storage group) | per value | `storage.rs:57 exposed` |

This is the **only surviving backend consumer of ownership information**, and it
reads the source mode as a *three-valued tag*, not as a loan or a place. It does
not read resolved places, effect rows, or any proof. `emitter.rs:1320` and
`emitter/places.rs:43` record its one aliasing assumption: *"A result can alias
any consumed caller input"* — i.e. reuse is **permitted** aliasing, the opposite
polarity from `noalias`. A storage-identity model would express this directly
("the result's identity *is* the consumed argument's identity") rather than as a
liveness-plus-interference side condition.

Evidence for the retired channels: `research/experiments/scoped-alias-channel/RESULTS.md`,
`research/experiments/effect-attrs-channel/RESULTS.md` (neither has a
README/DESIGN; RESULTS.md is the whole record),
`docs/why-whitefoot.md:265-414`, `compiler/src/backend/emitter.rs:4`,
`compiler/src/backend/abi.rs:3-4`,
`compiler/src/backend/tests/effect_attributes.rs`.

What they bought (measured, on the retired compiler, restated by
`docs/ideas.md:24-56` as *"not current compiler results"*):

- alias channel: 8 vector adds / 0 runtime guards / 121 asm lines vs Rust's
  obvious shape at 65 / 29 / 2132. Time delta is real only at short trip counts
  (2.0× at n=8, 1.18× at n=16); at n ≥ 32 Rust's loop versioning amortizes and
  they tie. The durable win is **code size (17×) and guard count**, plus the
  16-column addendum: guards 29→111, asm 2132→2836, Whitefoot 183 lines / 0
  guards, still near parity in time.
- effect channel: O(n) → O(1) across an opaque object boundary (1.47 s → 0.00 s)
  — but **only** once `willreturn` was added from a separate totality
  derivation, because *"without `willreturn`, `memory(none)` alone hoists
  NOTHING (LICM requires non-divergence)"*. Rust with fat LTO ties.

---

## B. LLVM fact forms

*(filled from LangRef; see §B.1 verification note.)*

| Attribute / metadata | Granularity | What it promises | Minimal source fact that justifies emitting it |
|---|---|---|---|
| `noalias` **parameter** attr | one pointer parameter; the promise is over the *"based on"* provenance relation, for the **dynamic extent of one call**, and **across threads** | *"memory locations accessed via pointer values based on the argument … are not also accessed, during the execution of the function, via pointer values not based on the argument … This guarantee only holds for memory locations that are **modified**, by any means, during the execution of the function."* Plus *"noalias also applies to accesses from other threads, unless they happen-before function entry, or function exit happens-before them."* Violation ⇒ **UB**. `based on` is defined in Pointer Aliasing Rules over GEP / bitcast / inttoptr, transitively. | One argument whose resolved place is OWN-7-disjoint from every other argument's resolved place **and** from everything else the callee can reach. Today: a `&uniq 'r p` actual (OWN-9: *"a live, usable `&uniq` borrow's resolved place is unaliased by any other usable access path"*), disjointness checked at OWN-12 (*"two `&uniq` arguments whose resolved places overlap are an error"*). Note the read-only escape: WF may pass two overlapping `&'r` actuals and still emit `noalias` on both, because the guarantee covers only *modified* locations. |
| `noalias` **return** attr | the returned pointer | *"the function acts like a system memory allocation function, returning a pointer to allocated storage disjoint from the storage for any other object accessible to the caller."* Strictly stronger than C99 `restrict`, which has no return form. | A freshly-allocated affine owner returned by value — `buffer<T>` / `box<T>` construction, `arena_new`. Not currently emitted. |
| `!alias.scope` + `!noalias` **instruction metadata** | **one instruction** (`load`, `store`, memory-accessing `call`, and *also* `fence`), each carrying a *list* of scope MDNodes; scopes are partitioned into independent **domains** | *"if for some domain, the set of scopes with that domain in one instruction's `alias.scope` list is a subset of (or equal to) the set of scopes for that domain in another instruction's `noalias` list, then the two memory accesses are assumed not to alias."* Violation ⇒ **UB**. Domains matter: *"Because scopes in one domain don't affect scopes in other domains, separate domains can be used to compose multiple independent noalias sets."* | Per-*access* disjointness of two storages that are **not** function parameters — two `buffer<u64>` fields loaded out of one `&uniq Cols`. Needs (i) a static identity per loaded data pointer and (ii) a distinctness proof. WF has (ii) today (OWN-1 affine single-owner + OWN-7 field distinctness); it has no place to hang (i), because a loaded `buffer` data pointer is not a resolved-place root. |
| `llvm.experimental.noalias.scope.decl(metadata !scope.list)` | an **intrinsic call** marking where a scope becomes valid | *"identifies where a noalias scope is declared. When the intrinsic is duplicated, a decision must also be made about the scope … when the intrinsic is used inside a loop body, and that loop is unrolled, the associated noalias scope must also be duplicated. **Otherwise, the noalias property it signifies would spill across loop iterations, whereas it was only valid within a single iteration.**"* | **This is the construct PAR-2 tiles want.** A per-iteration disjointness fact (`ProvedRangePartition`'s `[s*i+b, s*i+b+s)`) is exactly a scope *valid within a single iteration*, and LangRef's own worked example is the local/loaded-`restrict` loop. |
| `memory(...)` **function** attr (also a call-site attr) | whole function or call site, as *location kind* × *access kind*. Location kinds: `argmem`, `inaccessiblemem`, `errnomem`, `target_mem#`, default. Access kinds: `none`, `read`, `write`, `readwrite`. Default when absent is `memory(readwrite)` | `read`: *"The location is only read. Writing to the location is immediate undefined behavior."* `write`: *"Only writes to the location are observable outside the function call … **Reading the location prior to writing it results in a poison value.**"* `none`: *"No reads or writes to the location are observed outside the function."* `argmem` = *"accesses that are based on pointer arguments to the function."* Call-site takes precedence: `CallSiteEffects & (FunctionEffects | OperandBundleEffects)`. | `pure` ⇒ `memory(none)`. `reads(p)` only ⇒ `memory(argmem: read)`, sound because EFF-2 makes every declared path formal-rooted (*"the paths take the same formal-rooted paths"*) so `argmem` covers them. `writes(p)` with matching `reads(p)` ⇒ `memory(argmem: readwrite)`. **Insufficient alone**: the measured hoist required `willreturn`, which EFF-3 does not license and `compiler/src/backend/tests/effect_attributes.rs` actively tripwires. |
| `readonly` / `writeonly` / `readnone` — **parameter attributes only** (they no longer exist as function attributes) | one pointer parameter, per **access path** through that pointer; other aliasing pointers are exempt | `readonly`: *"the function does not write through this pointer argument, even though it may write to the memory that the pointer points to."* `writeonly`: *"may write to, but does not read through this pointer argument … understood in the same way as the `memory(write)` attribute."* `readnone`: *"does not dereference that pointer argument, even though it may read or write the memory … if accessed through other pointers."* All three: violation ⇒ **UB**. | `&'r p` parameter ⇒ `readonly`, on CALL-1's ground: *"no write through a shared holder is admissible, so a body can exhibit none and [EFF-2]'s both-ways check admits none in the declared row."* A parameter absent from the declared `reads` row but present in `writes` ⇒ `writeonly`; absent from both ⇒ `readnone`. EFF-2's both-ways check is exactly the evidence these need. |
| `captures(...)` — **current spelling**; `nocapture` no longer appears in LangRef | one **argument copy**, by component (`address`, `address_is_null`, `provenance`, `read_provenance`) × location (`ret:` or default) | *"restricts the ways in which the callee may capture the pointer … applies only to the particular copy of the pointer passed in this argument."* `captures(none)` = not captured. UB rule: *"If an argument does not capture the provenance of the pointer, accesses that are based on the argument and are performed after the function returns (or unwinds) cause undefined behavior."* `captures(address, provenance)` ≡ omitting the attribute. | STOR-5 borrow-free storage (*"An owned enum's payload cannot store a borrow or a view"*) + VIEW-1 (*"Neither view is ever stored in a nominal field, an enum payload, or a run slot"*) + OWN-10 region containment. Every borrow parameter whose region is introduced locally at the call is `captures(none)` by construction. |
| `dereferenceable(<n>)` | parameter **or** return attr — **an instant-in-time promise, not a lasting one** | *"A pointer that is dereferenceable can be loaded from speculatively without a risk of trapping."* But: *"The dereferenceable attribute **only implies dereferenceability at the point of the attribute** (i.e. on function entry for arguments or at the point of the call for return values). The underlying object may still get freed after that point."* Implies `noundef`, and `nonnull` in addrspace 0. | OWN-10 borrow-storage duration + OP-9 layout ceiling: `&'r p` where `p`'s type has a known layout size `n`. The `democ` prototype emitted exactly this on borrow parameters. **Correction to a natural assumption**: WF's OWN-10 is *stronger* than what `dereferenceable` alone claims — pairing it with `nofree` is what LangRef says buys durability. |
| `dereferenceable_or_null(<n>)` | same positions | *"isn't both non-null and non-dereferenceable (up to `<n>` bytes) at the same time."* | Not needed: WF has no null borrow. |
| `!dereferenceable` / `!dereferenceable_or_null` metadata on a `load` | one load's **result**, at the *"current program point"* | *"tells the optimizer that the value loaded is known to be dereferenceable at the current program point, otherwise the behavior is undefined … can be combined with the `!nofree` metadata to indicate that the pointer will stay dereferenceable forever."* | A `buffer<T>` descriptor's data-pointer load where `len_of` is a live MSR-2 fact — a fact WF *has* today but has no IR root to attach to. |
| `!invariant.load` | one **load**, or a read-only intrinsic call | *"every memory location read by that operation must contain the same value **at all points in the program where that memory is dereferenceable**; otherwise, the behavior is undefined."* Program-wide, **not** region-scoped. | Only `immutable-const` origins qualify (CONST-2: *"named const storage is permanently read-only"*, and OWN-7: *"`immutable-const` needs no overlap proof because no accepted write or unique borrow of const storage exists"*). **A descriptor measure does NOT qualify** — an ENT-5 kill later in the program means the location does change, so a "quiescent window" is the wrong shape for this attribute. |
| `!invariant.group` + `llvm.launder.invariant.group` / `llvm.strip.invariant.group` | load/store, referencing *"a single metadata with no entries"* — and **tied to the SSA value identity of the pointer operand, not to the memory location** | *"every load and store to the same pointer operand can be assumed to load or store the same value."* And: *"The `invariant.group` metadata must be dropped when replacing one pointer by another based on aliasing information. This is because `invariant.group` is tied to the SSA value of the pointer operand."* `launder` returns an aliasing pointer *"considered different for the purposes of … `invariant.group`"*. Both the metadata and the intrinsics are marked **experimental**. | The SSA-value-identity keying is a near-exact match for a branded `ptr<'a, T>`: the brand *is* the identity the metadata wants. `launder` is the natural lowering of a storage-state transition that ends one identity's invariant window. Speculative and experimental on LLVM's side. |
| C `restrict` mapping | — | Parameter form: *"this definition of noalias is intentionally similar to the definition of restrict in C99 for function arguments."* `based on` is *"intentionally similar to the definition of 'based' in C99, though it is slightly weaker."* Inlining: *"As the noalias function parameters are turned into noalias scope metadata, **a new domain is used every time the function is inlined**."* Local/loaded `restrict` pointers use `!alias.scope`/`!noalias` + `llvm.experimental.noalias.scope.decl`. | The inliner's mint-a-fresh-domain behavior is the mechanism WF would reuse to attach a scope to an **origin range** or a **brand instantiation** rather than to a parameter position. |
| `align(<n>)` | parameter / return attr | *"If the pointer value does not have the specified alignment, **poison** value is returned or passed instead"* — poison, not UB. | STOR / OP-9 layout ceiling. |
| `!nonnull` | **`load` instruction only** (documented inside the `load` spec, not as its own metadata section) | *"tells the optimizer that the value loaded is known to never be null. If the value is null at runtime, a **poison** value is returned instead."* | Every WF borrow and descriptor data pointer is non-null by construction. |
| `!range` | `load`, `call`, `invoke` of integer or vector-of-integer type | *"expresses the possible ranges the loaded value or the value returned … is in. If the loaded or returned value is not in the specified range, a **poison** value is returned instead."* Per-lane for vectors. | Any discharged `[OP-2]`/`[OP-4]`/`[ENT-6]` bound on a loaded or returned integer — including every `len_of` whose MSR facts survive. |
| `initializes((Lo1,Hi1), …)` | **parameter attr**, a list of **byte ranges** relative to the pointer | *"the function initializes the ranges of the pointer parameter's memory `[%p+LoN, %p+HiN)` … all bytes in the specified range are written before the function returns, and not read prior to the initializing write."* Volatile/atomic write to a not-yet-initialized byte ⇒ UB; premature read ⇒ poison. | EFF-2's independence of `reads` and `writes`: *"an operation which observes prior state while changing it names the path in both categories, while a **complete overwrite need only write it**."* A declared `writes(p)` with no `reads(p)` is precisely `initializes` over `p`'s whole byte extent. **The cleanest unexploited fact in the current spec.** |
| `!tbaa` | load/store, one access tag `(BaseTy, AccessTy, Offset)` per instruction, within one TBAA root tree | Two tags alias iff one is reachable from the other via the Parent relation. *"If memory accesses alias even though they are noalias according to `!tbaa` metadata, the behavior is undefined."* Different roots ⇒ conservatively MayAlias. | WF nominal type identity + field offsets. Likely subsumed by per-identity alias scopes, which are strictly stronger for the struct-of-arrays case. |

### B.1 Verification note

All LangRef rows above are **verified** against the current
`https://llvm.org/docs/LangRef.html`; quoted text is verbatim. Three things that
a reader might expect were checked and are **not** on the page:

1. There is no `!llvm.alias.scope` named-metadata construct. Scopes and domains
   are ordinary (often self-referential, e.g. `!0 = !{!0}`) MDNodes referenced
   directly by an instruction's `!alias.scope` / `!noalias` operand.
2. `nocapture` does not appear anywhere; LangRef has fully migrated to
   `captures(...)`.
3. `readonly` / `readnone` / `writeonly` appear **only** as parameter
   attributes. There is no sentence saying they "were replaced by `memory(...)`";
   they have simply been dropped from the function-attribute list, and
   `memory(...)` covers the same ground. That replacement is release-note
   knowledge, not page text — **treat the historical framing as unverified**,
   though the current-state fact (parameter-only) is verified.

### B.2 The distinction that matters most for this decision

Parameter `noalias` and per-instruction `!alias.scope` are **not** two spellings
of one fact. They differ in *what can be named*:

| | parameter `noalias` | `!alias.scope` / `!noalias` |
|---|---|---|
| Names | an SSA **function argument** (or return value), via the `based on` provenance relation | an **arbitrary set of instructions**, via scope MDNodes |
| Scope of the promise | the dynamic extent of the call, **including other threads** | exactly the instructions carrying the tag |
| Can name a value **loaded from memory**? | **no** — provenance must trace back to an argument | **yes** — the tag is on the access, not on the value's definition |
| Number of independent facts per function | ≤ number of pointer parameters | unbounded; partitioned into **independent domains** so separately-derived facts do not interfere |
| Validity boundary | the call | wherever the scope is declared; `llvm.experimental.noalias.scope.decl` marks that point, and **must be duplicated when a loop is unrolled** or the fact *"would spill across loop iterations"* |
| What the compiler must have | a disjointness proof indexed by *parameter position* | a disjointness proof indexed by *some static identity carried alongside the pointer value* |

`docs/why-whitefoot.md:316-414` calls the second row "the half Rust cannot
reach": Rust's `&mut Cols` gets parameter `noalias`, but *"the `Vec` data
pointers loaded through it are fresh pointers the optimizer treats as possibly
overlapping each other."* `research/experiments/scoped-alias-channel/RESULTS.md`
says the same and names the fact that closed it: *"`buffer<T>` is affine
single-owner (OWN-1/T1): two distinct live buffer values never overlap. `&uniq`
is exclusive (OWN-2/5) and loans have singleton provenance (T-A). So inside a
function taking `s: &uniq 'r Cols`, every buffer field of `s` is
pairwise-disjoint element memory, disjoint also from the struct memory itself."*

The prototype derived that per-load identity from **the resolved place of the
field** (`deref(s).a`, `deref(s).b`, …) — i.e. it manufactured a scope from a
static path, on the fly, at emission. Nothing in the *type system* carried the
identity; it was a backend-side re-derivation from the checker's place
information. That re-derivation is exactly what a storage-identity-in-the-type
model would make a first-class, cross-procedural fact instead.

---

## C. Gap analysis

For each consumer: does **"storage identity distinctness + write/read footprints
over identities (fields, proved ranges, affine element maps) + proved index
disjointness"** carry at least the information of **"borrow-mode loans +
resolved-place overlap"**?

### C.0 The structural translation

Today's `ResolvedPlace` is already *almost* a storage identity:

```rust
// compiler/src/semantic/places.rs:161
pub(crate) struct ResolvedPlace { root: PlaceRoot, path: Vec<PlaceStep> }
// PlaceRoot = Binding(BindingId) | Constant(CheckedConstantId)
// PlaceStep = Field(u32) | Subscript(PlaceOffset) | Range(RangeId)
```

It is a **function-local** identity: the root is a binding of *this* function,
and a borrow holder is resolved *through* to the place it borrows (there is no
`Deref` step in the resolved path). It is therefore a static name for a storage,
already carrying fields, subscripts, and range frames. A `ptr<'a, T>` brand is
the same thing lifted out of "the current function's binding table" into "the
type". This makes most of the translation mechanical.

But the equivalence is not automatic, because today's resolved place is *only*
an identity relation, and today's judgment reads **two other things beside it**:
loan *strength* (an exclusion, not an access), and *liveness/suspension* state.

### C.1 Per-consumer verdict

| Consumer | Facts today | Identity-footprint equivalent | Verdict |
|---|---|---|---|
| **PAR-1 dataflow (#2)** | `BindingId` def/use | unchanged — this is SSA-level, not ownership | **equal** |
| **PAR-1 footprint disjointness (#3-#6, #10)** | EFF-2 row paths projected to resolved places; OWN-7 prefix overlap | identity distinctness + field footprints; `allocates(arena 'r)` becomes an identity for the region's allocation list | **equal**, provided the region-keyed arena element (`Access::Arena`, keyed by `DeclarationId`, *not* a place) survives as its own identity. Today it is a separate `Access` variant precisely because a region is not a place (`permission.rs:158-170`). |
| **PAR-1 argument loans (#7, #12)** | borrow **mode** of the *actual*, independent of the row | ✗ **information loss unless footprints are supplemented.** See C.2. | **loss** |
| **PAR-1 argument-expression reads (#8)** | per-operand-subexpression resolved places, with address formation excluded | read footprint over identities at the same granularity; the "address formation is not a read" rule is *about* identities (taking a `ptr<'a,T>` does not read `'a`'s storage) and is if anything cleaner | **equal or better** |
| **PAR-1 fail-closed unresolved (#14)** | one `Option<NodePath>` per half ⇒ deny | an identity-less pointer (unbranded, or branded with an existential the checker cannot instantiate) ⇒ deny | **equal**, and structurally more honest — today "unresolved" is an analysis failure; there it is a *typed* condition. |
| **PAR-2 accumulator (#16, #17)** | one `BindingId` + closed op set; read-count by binding | unchanged — not an aliasing fact | **equal** |
| **PAR-2 iteration-own (#15)** | root ∈ bindings B introduces | identity created inside the iteration; an identity whose introduction site is inside B | **equal**, and better if the brand is existential per iteration |
| **PAR-2 affine element map (#18, #19)** | `ProvedAffineIndexMap { loop_id, coefficient: i128, constant: i128 }` from a discharged OP-4 | same, hung off an identity instead of a resolved root | **equal** |
| **PAR-2 adjacent range (#20-#22)** | `ProvedRangePartition { loop_id, range: RangeId, stride: AffineForm, base: AffineForm, +2 DerivationIds }` + containment by `contains_path` | same, with containment as a subrange relation on the identity | **equal**; `RangeId` is already an identity token in all but name |
| **PAR-2 exclusive-loan containment (#23)** | every exclusive loan is iteration-own or inside an assignment | ✗ **loss**, same cause as #7. See C.4. | **loss** |
| **ENT-5 kill (#27-#34)** | OWN-7 overlap of a projected write against fact support; element/descriptor bit; CALL-1..3 reach from declared mode+type | identity overlap + a write footprint; the element/descriptor split is a *granularity of the footprint*, which identity footprints state directly | **equal or better** — see C.5 |
| **EFF-2 projection (#35-#40)** | static struct paths, formal-rooted; view actual projects through its **complete origin set**; multi-origin ⇒ deduplicated union | identity substitution at the call: the callee's `'a` is instantiated with the caller's identity | **better** — see C.6 |
| **Backend storage reuse (#41-#44)** | `IrSourceMode` 3-valued tag + IR CFG liveness + exposure | identity equality ("result's identity *is* the consumed argument's identity") replaces the liveness-plus-interference side condition | **equal or better**; the fact wanted here is *sameness*, which identities state and modes only imply |
| **Backend alias/effect emission** | nothing today; prototype re-derived scopes from resolved field paths at emission | per-identity scopes are directly available | **strictly better** — see D |

### C.2 The concrete loss: argument loans are an *exclusion*, not an access

This is the sharpest finding. PAR-1 ¶6:

> Each statement additionally holds, for the duration of its call, a loan on the
> resolved place of every argument written as a borrow: a `&uniq 'r` argument
> holds an exclusive loan and a `&'r` argument a shared loan, **whatever that
> argument's parameter region does or does not carry in the callee's row**.

And the compiler's own statement of it (`permission.rs:172-180`):

> A loan is not a use. The callee's declared row says what the callee *does*
> through the borrow; the loan says what the borrow *forbids everyone else*
> while it is live. The two are independent: `fn peek(c: &uniq 'c u64)
> reads(cell)` projects a read and holds an exclusive loan, and a `pure` callee
> projects nothing and still holds one.

So today there are **two separate channels**, and one of them (the loan) is
*not* an access footprint at all. A pure callee taking `&uniq x` contributes
`W = ∅, R = ∅` but still denies permission against *any* overlapping access of
the sibling statement.

**Concrete case where today's loan rule DENIES and identity footprints would GRANT:**

```
let a = peek(c: &uniq 'r cell);   // declared row: reads(cell)   -- or pure
let b = look(v: &'r cell);        // declared row: reads(cell)
```

Footprints today (`permission.rs:1036 user_call_footprint`):

| | s1 = `peek` | s2 = `look` |
|---|---|---|
| `W` (row `writes` + consumed `own`) | ∅ | ∅ |
| `R` (row `reads` projected) | `{cell}` (∅ if `pure`) | `{cell}` |
| `O` (operand reads) | ∅ — `&uniq 'r cell` is address formation, *"taking the address of a place is not reading it"* | ∅ — same |
| loans | **Exclusive** on `cell` | Shared on `cell` |

Today: `LoanStrength::Exclusive.excludes_loan(Shared) == true`
(`permission.rs:196-219`) and `excludes_use(Read) == true`
(`permission.rs:196-219`) — **denied**, on the loan channel alone, even though
every footprint pair is read-vs-read.

A pure identity-footprint model sees `W(s1)=W(s2)=∅`, `R(s1)=R(s2)={cell}`,
`O(s1)=O(s2)=∅` and **grants**.

Is the grant *wrong*? For execution, no — two reads do not race. But it changes
what the judgment means. PAR-1 states its ground explicitly:

> The reason is [OWN-5] itself: every borrow this rule judges is live and usable
> across the whole of its statement's call [OWN-12], so an implementation that
> overlaps two statements makes both statements' borrows simultaneously live and
> usable, and permission therefore requires of the resulting loan state exactly
> what [OWN-5] requires of one statement holding all of those loans at once.

The loan rule is not a race-avoidance rule. It is a **source-equivalence** rule:
overlapping the two statements would construct a program state that the source
checker refuses (two usable overlapping loans, one exclusive), and PAR-1 refuses
to construct a state the source language does not admit. Dropping it does not
obviously break execution, but it *does* break the design ground recorded in
`design/language/parallelism/permission-judgment.md`:

> The loan judgment reuses the borrow checker's own overlap vocabulary lifted
> from one call's arguments to the statements of one window, because four
> alternatives, reliance edges, schedule-parametric ownership, treating
> exclusive borrows as writes, and weakening the loan rule, **each either
> duplicated the borrow checker or weakened it**, instead of a separate parallel
> aliasing model.

Note that "**treating exclusive borrows as writes**" is listed among the four
*rejected* alternatives. An identity-footprint model that recovers the loan rule
by conservatively adding every `&uniq` argument's identity to the write
footprint is exactly that rejected alternative, and it **over-denies**: a
`&uniq` argument whose callee row is `reads(cell)` would then deny against a
sibling's read of `cell`, which today's rule *also* denies — so for PAR-1
specifically the two coincide. Where they diverge is ENT-5 (C.5) and anywhere a
`&uniq` is passed to a `pure` callee.

**Bottom line for C.2:** identity distinctness + footprints is *strictly less*
information than loans + overlap, because a loan carries a *mode on the actual*
that no footprint mentions. To carry the same information the replacement must
keep an explicit per-call **exclusivity claim on an identity**, distinct from
read/write footprints. That is not a footprint; it is a state transition on the
identity — which is what "storage state tracked per identity" is for, and is
where the proposed model should be checked hardest.

### C.3 Argument-expression reads in PAR-1

PAR-1 ¶5 sentence 2:

> Evaluating a statement's own argument expressions is part of that statement
> and therefore part of the overlap, so each call's written footprint also
> overlaps no place the other statement's argument expressions read; **taking
> the address of a place is not reading it**, and both directions are required
> because which statement's argument evaluation an overlap moves is the
> implementation's choice.

`collect_operand_reads` (`permission.rs:1671`) implements this by an exhaustive
match: `BorrowBuffer | BorrowAddressed | BorrowBox | ReborrowAddressed` ⇒ no
read; every place-reading form ⇒ one read of its resolved place; a `UserCall` or
`KernelCall` in argument position ⇒ `operand_unresolved` (fail-closed).

Under identity footprints this is **unchanged and arguably improved**: forming
`ptr<'a, T>` from a place is *manifestly* not an access of `'a`'s storage,
whereas today it is a special case in an exhaustive match that must be
maintained per expression form. The one thing that must survive is the
**asymmetry** derived at `permission.rs:1212-1240`: `W(T)` is judged against
`O(s2)` but *not* against `O(s1)`, because under either admitted schedule s1's
operands are evaluated before any interposed statement. That is a control-flow
fact, not an aliasing fact, and translates untouched.

### C.4 PAR-2's exclusive-loan condition

PAR-2:

> Every place a footprint of B holds an exclusive loan on — **including view
> formations and argument borrows holding loans exactly as [PAR-1]'s do** — is
> iteration-own storage or is contained in a proved adjacent-range assignment.
> Apart from the mapped-source prohibitions above, **shared loans need no
> additional condition, because they reach only enclosing storage that no
> iteration writes**. An ordinary non-call borrow formation of enclosing storage
> still denies permission; VIEW-2 formations use the range rule just stated.

Impl: `loop_permission.rs:1003 record_loan` — exclusive loan ⇒ must be
`is_iteration_own` or `assigned_range(place).is_some()`.

Two independent losses here:

1. **Same as C.2**: the condition is on *loans*, not on writes. An iteration
   passing `&uniq row` to a `pure` helper still holds an exclusive loan on
   `row` and must therefore have `row` inside its assigned tile. An
   identity-footprint model with no exclusivity claim would compute an empty
   write footprint and grant — a strict weakening.

2. **A second, subtler one**: `loop_permission.rs:1428
   borrows_only_iteration_own` exists because *"a written borrow's shared-or-uniq
   mode is erased from the checked tree"*, so for non-view borrow formations
   inside B the compiler **cannot tell shared from exclusive** and must refuse
   every borrow of non-iteration-own storage. A brand model that puts the
   identity *in the type* would make this distinction recoverable — this is an
   **information gain**, and one of the few places where the replacement is
   unambiguously better than the status quo.

### C.5 ENT-5 and the "unresolved element denies" fail-closed rule

ENT-5's kill is the one consumer that is **over-approximating by design**
(`spec:2733`: *"kills use [OWN-7]'s resolved-place overlap relation and
over-approximate it"*). Its inputs are:

- OWN-7 overlap of a written place against the support,
- an element/descriptor bit (`KillEvent::Write.element`, `flow.rs:86`),
- the CALL-1..CALL-3 reach classification, taken *from the declared parameter
  mode and type and nothing else* (CALL-5).

Identity footprints carry all three and can carry them **more precisely**:

| ENT-5 need | today | identity footprints |
|---|---|---|
| "a write at an element position kills the written element's measures and none of the collection's own" | one bool + MSR-2's descriptor-storage boundary | the footprint names the element identity vs the descriptor identity directly |
| "a callee writing only through one `&uniq` actual kills exactly the facts whose support overlaps that actual's resolved place" | EFF-2 projection onto one resolved place | identity substitution at the call; **strictly at least as precise** |
| "a call whose row carries no `writes` kills nothing" | row absence | footprint absence — **equal** |
| CALL-1: shared-borrow actual is never a kill event | mode of the *parameter* | needs the read/write split on the identity footprint, which the model has — **equal** |
| CALL-3: a write through a view reaches the range's storage and no measure of the origin place itself | reach classification per declared parameter type | a proved range is already an identity sub-extent — **better**, because today the classification is a coarse per-type table |

**The fail-closed rule.** Today "unresolved denies" appears in three places with
three different mechanisms: `Footprint::unresolved` / `operand_unresolved`
(PAR-1), the `None` of `view_origin` (views), and PAR-2's
`self.unresolved.get_or_insert(...)`. The spec states it once per rule (PAR-1
¶8, PAR-2 ¶"A footprint element whose caller place…"), and OWN-7's own last
clause is its type-level twin: *"Formal-slice origins are substituted before
caller overlap checking; they never establish that two actual sources are
disjoint."*

Under identity branding the fail-closed rule becomes a **typing** condition
rather than an analysis condition: an identity variable that has not been
instantiated is distinct from nothing. This is the same *direction* of
conservatism, but it is checked by the type system rather than by a
`get_or_insert` on an `Option<NodePath>`, so it cannot be *forgotten* by a
missing match arm. The recorded design ground for the current arrangement
(`design/language/parallelism/permission-judgment.md`) is precisely fear of that
failure mode:

> Unresolved overlap or an unsupported interposed statement form denies
> permission, and **a missing classification never contributes an empty
> footprint**, because a silent empty footprint would grant permission by
> omission.

**Verdict: identity footprints carry at least ENT-5's information, and make the
fail-closed rule structurally safer.**

### C.6 View origin sets after call substitution

This is where the replacement is clearly *better*, and where today's compiler is
provably weaker than today's spec.

Spec `OWN-5` (`:709-724`): every view value *"carries a finite set of possible
ultimate storage origins"*; EFF-2: *"a view actual projects through its complete
origin set … A multi-origin view contributes the deduplicated union of its
formal-rooted origins"*; OWN-7: *"Two slice values in a fully substituted caller
context overlap conservatively iff at least one pair of their resolved-place
origins overlaps."*

Compiler (`places.rs:299`): `view_origin: Option<ResolvedPlace>` — **one**
origin, or `None`. `None` ⇒ every consumer fails closed (`places.rs:295-298`). A
callee-returned view is always `None` (`places.rs:482-483`: *"A view a callee
returns leaves this `None`, and the judgments that read it deny rather than
guess."*). A formal slice parameter is anchored at its own binding
(`places.rs:328-331`), which is an opaque self-origin that overlaps nothing else
by construction and is never *substituted* inside the callee.

Consequences today:

- No call-boundary substitution of view origins happens **inside** the
  permission judgments at all. The check is entirely intra-procedural; the
  cross-procedural obligation lives in OWN-12's argument-overlap check at the
  caller, separately.
- A helper returning a view kills every downstream permission.
- The spec's "set of origins" is implemented as "at most one origin".

A `ptr<'a, T>` brand carries the origin **in the type**, so:

- a callee-returned view returns `Slice<'a, T>` for a caller-known `'a` and
  loses nothing;
- substitution at a call is type instantiation, already a thing the compiler
  does;
- the "set of origins" case (a view whose origin depends on a branch) becomes a
  *typing* problem — either the two arms unify on one `'a` or the program is
  rejected. Note that WF already forbids the hard case: `OWN-5` (`:715-717`) says
  *"This specification defines no slice-valued control-flow join"* and makes a
  view-typed `value_if`/`value_match` initializer a hard error. So the origin
  *set* is, today, only ever plural through **formal-slice substitution** — which
  is exactly what a brand parameter models.

**Verdict: strict information gain.** The one thing to preserve is OWN-7's
refusal to use formal-slice origins as evidence of *disjointness*: two distinct
brand variables `'a`, `'b` must default to *may-alias*, not to distinct, unless
the caller instantiated them with provably distinct identities. A brand system
whose default is "distinct variables name distinct storage" would be **unsound**
against OWN-7's stated rule.

### C.7 Cases where today's loan rule grants and identity footprints would not

I found one class.

**Shared loans on storage no iteration writes (PAR-2).** PAR-2: *"shared loans
need no additional condition, because they reach only enclosing storage that no
iteration writes."* The grant is justified by a *global* property of the loop
(nothing in B writes enclosing storage except via the accumulator/map/tile
rules), not by any property of the shared loan itself. An identity-footprint
model that judged each shared read footprint against each write footprint
pairwise would reach the same conclusion — but only if it retains PAR-2's
condition-2 global structure. A naive per-pair identity check would be **equal**;
the risk is only in losing the ¶-level structure, not the facts.

**A second, near-miss.** PAR-1 admits two overlapping *shared* loans and two
overlapping *read* footprints. Identity footprints do too. No gap.

**Not a gap, but worth recording**: OWN-9 is explicitly *non-normative*
(`spec:763`) and already states the exact backend consequence in
identity-flavored language: *"a live, usable `&uniq` borrow's resolved place is
unaliased by any other usable access path (a suspended holder is not usable; a
statement-scoped child, arm-scoped child, candidate-position child, bound
call-result holder, or returned reborrow and its suspended ancestor, though both
live, are never mutually noalias — the guarantee is one usable mutable path per
place); shared borrows are read-only for their duration; owned values are
unaliased except by their own live shared borrows."* The parenthesis is the
part an identity model must reproduce: **parent and child reborrows share an
identity and are not mutually distinct.** A brand model in which a reborrow
mints a *fresh* brand would assert exactly the `noalias` pair OWN-9 forbids. The
suspension/usability state — today a borrow-checker state — must become a
*storage state on the identity*, which is what the proposed "initialized /
uninitialized / ended" axis is for; it needs a fourth state, "suspended", or the
same information under another name.

---

## D. Facts the backend could exploit that borrows cannot express but per-identity footprints could

All rows are **speculative** unless marked otherwise; none is implemented today.
The "evidence" column cites the closest existing measurement.

| # | Fact | LLVM form | Why borrows cannot express it | Evidence / status |
|---|---|---|---|---|
| D1 | **Per-field alias scopes on loaded data pointers**: two `buffer<u64>` fields of one `&uniq Cols` name disjoint element storage | one `!alias.scope` per identity, all in one domain; each load/store tagged with its own scope in `!alias.scope` and every sibling scope in `!noalias` | A borrow mode attaches to a *parameter*; the loaded `buffer` data pointer is a fresh LLVM value with no parameter to carry `noalias`. Today the compiler would have to re-derive the identity from the resolved field path at emission (which is what `democ` did) | **Measured on the retired prototype**: 8 vector ops / 0 guards / 121 asm lines vs Rust obvious 65 / 29 / 2132; short-trip 2.0× at n=8; parity at n ≥ 32. `research/experiments/scoped-alias-channel/RESULTS.md`. Not current-compiler evidence (`docs/ideas.md:44-47`) |
| D2 | **Per-span alias scopes on PAR-2 tiles**: iteration `i`'s `[s*i+b, s*i+b+s)` view does not alias iteration `j`'s | `!alias.scope` per tile + `llvm.experimental.noalias.scope.decl` **inside the loop body**, which LangRef's own worked example is built for; or `noalias` on the outlined chunk thunk's slice parameter (simpler, since `LoopSplit` already outlines) | The proof exists today (`ProvedRangePartition`, `entailment.rs:299-306`, with its two retained `DerivationId` nonnegativity proofs) but dies at the permission judgment: `IrOperation::LoopSplit` carries `captures: Vec<IrValueId>` and no disjointness fact. The *language* fact is per-index; a borrow is per-lexical-region | **Speculative**, but the LLVM side is exactly designed for it: *"when the intrinsic is used inside a loop body, and that loop is unrolled, the associated noalias scope must also be duplicated. Otherwise, the noalias property … would spill across loop iterations, whereas it was only valid within a single iteration."* That sentence is a description of PAR-2's own argument |
| D3 | **`readonly` spans**: a shared view `Slice<'a,T>` of a *sub-range*, not the whole object | parameter `readonly` on the outlined chunk thunk's slice param (cheapest); or per-access `!alias.scope` tagging with no `!noalias` counterpart for the span | A `&'r p` borrow is `readonly` on the *whole* `p`. LLVM's `readonly` is likewise whole-pointer, per *access path*. A range is not a place a borrow can name; VIEW-2 ranges exist but stop at the checker | **Speculative.** VIEW-2's `RangeId` is already the identity token. Note LLVM has no byte-range `readonly` — only `initializes` is range-shaped, and it is a *write* fact (D5) |
| D4 | **`memory(argmem: read)` / `memory(argmem: readwrite)` from EFF-2 rows across an opaque boundary** | function attribute on both `define` and `declare` | Not a borrow gap — an *emission* gap. EFF-2 rows are formal-rooted and checked both ways, so the mapping is direct | **Measured on the retired prototype**: O(n)→O(1), 1.47 s → 0.00 s, across a no-LTO object boundary. `research/experiments/effect-attrs-channel/RESULTS.md`. **Blocked**: the measured win required `willreturn`, which EFF-3 does not license and which `compiler/src/backend/tests/effect_attributes.rs` actively tripwires |
| D5 | **`initializes(lo, hi)` from `writes(p)` without `reads(p)`** | parameter attribute | EFF-2 makes `reads` and `writes` *independent* facts (*"a complete overwrite need only write it"*), which is exactly `initializes`' precondition. No borrow mode distinguishes overwrite from read-modify-write | **Speculative**, and the cleanest unexploited fact in the current spec: the distinction is already declared and already checked both ways |
| D6 | **`captures(none)` / `nocapture` on every borrow parameter** | parameter attribute | Available from borrows too (STOR-5 + VIEW-1 + OWN-10) — listed for completeness, not as an identity-only gain | **Speculative**, low risk |
| D7 | **`dereferenceable(n)` + `align(n)` on borrow parameters** | parameter attributes | Available from borrows (OWN-10 + OP-9). `democ` emitted it | **Prototype-emitted** (`scoped-alias-channel/RESULTS.md`: *"plus `dereferenceable/align` on borrow params (borrow validity is a checker fact)"*) |
| D8 | **`!invariant.load` on reads of `immutable-const` origins** | load metadata | A borrow says "shared for this region"; `!invariant.load` says the location holds one value *"at all points in the program where that memory is dereferenceable"*. CONST-2's permanently-read-only storage is the only WF fact strong enough | **Speculative but sound.** **Correction to a natural assumption**: a *frozen descriptor measure* does **not** qualify — the ENT-5 kill graph proves a quiescent *window*, whereas `!invariant.load` demands program-wide invariance. A window is the wrong shape for this attribute |
| D9 | **`!invariant.group` on a branded descriptor between formation and its first ENT-5 kill**, with `llvm.launder.invariant.group` at the kill | load/store metadata + the launder intrinsic | The window shape D8 cannot use. LangRef keys `!invariant.group` to *"the SSA value of the pointer operand"*, not to the memory location — which is exactly what a `ptr<'a, T>` brand is. A lexical region cannot name an SSA pointer identity; a brand can | **Speculative**, and the most uncertain row: LangRef marks both the metadata and the intrinsics **experimental** (*"its semantics might change in the future"*), and the in-tree driver is vtables. But the SSA-identity keying is the single closest LLVM construct to the proposed model |
| D10 | **`!range` / `!nonnull` / `align` on descriptor loads from discharged bounds** | load metadata + param attrs | Not an identity gain per se, but today WF discharges `[OP-4]` / `[ENT-6]` bounds and then throws the numeric interval away at emission. Every `len_of` with a live MSR fact could carry `!range` | **Speculative**, cheap, and orthogonal to the ownership question. Poison (not UB) on violation makes it low risk |
| D11 | **`!tbaa` from nominal type identity** | any access | Available from types today; listed because a storage-identity model makes the *field-level* TBAA path cheap to derive | **Speculative**; likely subsumed by D1's alias scopes, which are strictly stronger for the SoA case and have no cross-root MayAlias fallback |
| D12 | **Cross-procedural alias facts: an identity instantiated at a call carries `noalias` into the callee's body without inlining** | parameter `noalias` on a `declare`, plus a matching `!alias.scope` domain if the callee is also compiled with the fact | This is the real structural gain. Today's origin substitution happens only at the caller (OWN-12) and the callee's formal-slice origin is opaque (`places.rs:328-331`), so the *callee* learns nothing. A brand in the signature makes the fact part of the callable boundary. LLVM's own inliner already does the reverse translation (parameter `noalias` → scope metadata + fresh domain per inline site), so both directions of the channel exist | **Speculative.** This is the alias analogue of D4's effect-row result, and the same *"per-file default = Rust's most expensive configuration"* argument would apply |

**Honest caveats on D, restated from the evidence itself:**

- `docs/ideas.md:44-47`: *"Their ABI, proof rules, and measured ratios are not
  current compiler results. The effect experiment used a separate totality
  derivation; the alias and law experiments include conditions where an expert
  baseline erased the advantage."*
- `scoped-alias-channel/RESULTS.md` claim 1 and the 16-column addendum: LLVM's
  loop versioning recovers most of the *time* at n ≥ 32 even at 16 columns. The
  durable deltas are **short trips, code size, and the obvious-shape-is-fast
  property**, not a large-n speedup.
- `docs/why-whitefoot.md:410`: *"In the historical Whitefoot experiment the
  obvious shape was the fast shape at every trip count; **reproducing that
  property in the current compiler remains open.**"*

---

## E. Summary of the information-loss question

| Question | Answer |
|---|---|
| Do identity footprints carry PAR-1's **footprint** information? | Yes, with the arena-region element kept as its own identity kind. |
| Do they carry PAR-1's **loan** information? | **No.** A loan is a per-actual *exclusivity claim*, independent of the callee's row. Recovering it needs an explicit exclusivity state on the identity, not a footprint. Modelling it as "a `&uniq` argument writes" is one of the four alternatives `design/language/parallelism/permission-judgment.md` records as rejected. |
| Do they carry PAR-2's **map / tile** information? | Yes; `ProvedAffineIndexMap` and `ProvedRangePartition` are already identity-shaped, and `RangeId` is already a brand in all but name. |
| Do they carry PAR-2's **exclusive-loan containment**? | No, for the same reason as PAR-1; but the model *gains* the shared-vs-uniq distinction that PAR-2 currently loses to checked-tree erasure (`loop_permission.rs:1428`). |
| Do they carry ENT-5's **kill** information? | Yes, and more precisely — the element/descriptor split and the CALL-1..3 reach table become footprint granularity rather than a per-type classification. |
| Do they carry EFF-2's **origin-set** information? | Yes, and strictly more: today's compiler implements the spec's origin *set* as `Option<ResolvedPlace>` and fails closed on callee-returned views. |
| Is there a case where today grants and identity footprints would not? | Not found, given PAR-2's paragraph structure is retained. |
| Is there a case where today denies and identity footprints would grant? | Yes — every `&uniq` actual to a callee whose row does not write it. This is the loan channel, and it is the whole of the loss. |
| The one soundness trap to watch | Two distinct brand *variables* must default to may-alias, not to distinct (OWN-7: formal-slice origins *"never establish that two actual sources are disjoint"*), and a reborrow must share its parent's identity, not mint a fresh one (OWN-9's parenthesis: parent and child *"are never mutually noalias"*). |


# Part C. Mechanism families per requirement, with constraint scoring


Purpose: separate **what must be guaranteed** (R1-R9) from **the mechanisms that
can guarantee it**, so that a change to one mechanism does not silently rewrite
five requirements at once (the Rust failure mode: one reference type serves R1,
R4, R5, R6, R9 together).

**Constraint legend** (used in the "fit" column; `+` good, `~` partial/conditional, `-` conflicts):

| | constraint |
|---|---|
| i | deterministic, budget-free checking: no SMT, no timeout, no heuristic search; fixed terminating derivation families + explicit writer steps |
| ii | no runtime safety check/trap in accepted programs; a falsifiable condition must be a typed outcome with real control flow |
| iii | no unsafe escape for writers |
| iv | AI writer: verbosity cheap; irregularity and non-local error feedback expensive; small regular spec |
| v | very high performance: static parallelism (statement/iteration overlap), optimizer-grade aliasing, no GC, no RC for ordinary data |

Citations are author / venue / year. Items marked **(unverified)** are real
artifacts whose formal venue I did not confirm, or which are not peer-reviewed.

---

## R1. Sequential memory safety

No uninitialized read; no use after free / realloc / scope exit / move-out; no double release.

| Mechanism family | Representative works | Information the checker needs | How it is checked | Fit i / ii / iii / iv / v | Cost or failure mode |
|---|---|---|---|---|---|
| Ownership + borrowing with inferred reference regions | Rust NLL (RFC 2094, Matsakis 2017, **unverified** as a paper); Polonius datalog reformulation (**unverified**, no paper); Oxide (Weiss, Gierczak, Patterson, Ahmed, arXiv 2019/2021); RustBelt (Jung, Jourdan, Krebbers, Dreyer, POPL 2018) | per-place initialization state, loan set per reference, region outlives constraints | dataflow + constraint solving over a fixed lattice; type rules for move/borrow | i `+` fixed-point, no search; ii `+` for borrows, `-` for drop flags (see R2); iii `-` as shipped, `+` if `unsafe` removed; iv `~` non-local lifetime errors are the classic complaint; v `+` | inference is non-local: an error surfaces far from its cause; region inference is the part hardest to explain to a writer |
| Explicit store/alias typing (pointer type is separate from the capability to use it) | Alias Types (Smith, Walker, Morrisett, ESOP 2000); Alias Types for Recursive Data Structures (Walker, Morrisett, TIC 2000); L3 (Ahmed, Fluet, Morrisett, TLCA 2005; Fundamenta Informaticae 2007) | a store typing `{ρ ↦ τ}` at each program point; explicit location-polymorphic signatures | syntax-directed type rules, no inference needed; capability is linear | i `+` fully syntax-directed; ii `+` nothing at runtime; iii `+`; iv `+` local errors, `~` very verbose store types; v `+` | store types must be written or elaborated everywhere; aggregate/ recursive structures need explicit pack/unpack, which grows the spec |
| Stateful views over a linear resource context | ATS stateful views (Zhu, Xi, PADL 2005); Cogent (O'Connor et al., ICFP 2016) | view assertions `T @ l`, `T? @ l` (uninitialized) in a linear context | linear type rules; view consumption/production per operation | i `+`; ii `+`; iii `+` (Cogent has no escape); iv `+` very regular, `~` view algebra is a second language; v `+` | expressing shared/recursive structure needs dependent views or a proof language; ATS leans on dependent types that can re-introduce solver load |
| Typestate / path-sensitive property automata | Strom, Yemini (IEEE TSE 1986); Bierhoff, Aldrich (OOPSLA 2007); Plaid (Aldrich, Sunshine, Saini, Sparks, Onward! 2009); ESP (Das, Lerner, Seigle, PLDI 2002) | a finite state per tracked object; aliasing-permission annotations (`unique`, `shared`, `full`) | finite-state dataflow, polynomial and terminating (ESP) | i `+` polynomial, no solver; ii `+`; iii `+`; iv `+` local per-variable errors; v `~` gives safety facts but weak aliasing facts | precision collapses under aliasing unless a permission system is bolted on; ESP is a *bug-finding* precision point, not a soundness one across aliases |
| Region / lifetime types with explicit region parameters | Tofte, Talpin (Information and Computation 1997); Cyclone (Grossman, Morrisett, Jim, Hicks, Wang, Cheney, PLDI 2002) | region of every pointer; region outlives ordering; effect (which regions a call touches) | type-and-effect rules; outlives is a partial order check | i `+`; ii `+` (Cyclone's *dynamic* regions were the exception); iii `-` in Cyclone (it kept unsafe casts); iv `+`; v `~` regions give lifetime, not exclusivity | region-only designs leak: a live region can hold dead objects; realloc and per-object freeing need a separate mechanism |
| Runtime-validated references | Vale generational references (Ovadia, verdagon.dev 2021/2023, **not peer-reviewed**); Mezzo's `adopt`/`give` dynamic ownership (Pottier, Protzenko, ICFP 2013) | a generation word per allocation; a remembered generation per pointer | runtime compare-and-trap on dereference | i `+`; ii `-` **disqualifying**; iii `+`; iv `+`; v `-` per-deref load + branch | fails constraint (ii) outright; useful only as a measured baseline for what static machinery must replace |

---

## R2. Resource lifecycle accounting

Every allocation/external resource released exactly once; linear obligations discharged; **no compiler-inserted drop flags**.

| Mechanism family | Representative works | Information the checker needs | How it is checked | Fit i / ii / iii / iv / v | Cost or failure mode |
|---|---|---|---|---|---|
| Strict linear types (obligation must be discharged syntactically) | Wadler, "Linear types can change the world!" (IFIP TC2, 1990); Austral (Borretti, language spec 2022+, **not peer-reviewed**); Cogent (ICFP 2016); Linear Haskell (Bernardy, Boespflug, Newton, Peyton Jones, Spiwack, POPL 2018) | multiplicity/linearity of each binding; join of linear contexts at control-flow merges | type rule: contexts must agree at merges; nothing left over at scope end | i `+`; ii `+` no flags exist; iii `+`; iv `+` errors are local and mechanical; v `+` | conditional release must be written as explicit control flow that consumes the resource on every path — verbose, but exactly the shape constraint (ii) wants |
| Affine types + compiler-inserted drop with runtime flags | Rust MIR drop elaboration / drop flags (rustc dev guide, **unverified** as a paper); Swift SIL Ownership SSA (Apple swift docs, **unverified**) | per-place maybe-init dataflow | dataflow, then code generation of a flag word and conditional destructor call | i `+`; ii `-` inserted branch on a hidden bit; iii `+`; iv `+` invisible to writer; v `~` flags are usually optimized out, not guaranteed | explicitly excluded by WF; also the hidden bit is exactly a fact the source could have stated |
| Ownership-based buffer deallocation as a compiler pass | MLIR `-ownership-based-buffer-deallocation` (LLVM/MLIR docs, **unverified** as a paper) | per-buffer ownership indicator threaded through the IR CFG | dataflow pass inserting `dealloc` guarded by a runtime ownership `i1` | i `+`; ii `-` runtime ownership bit; iii `+`; iv n/a (IR-level); v `~` | same objection as drop flags; instructive as the *lowering* WF wants to make statically unnecessary |
| Reference counting with ownership-directed reuse | Perceus (Reinking, Xie, de Moura, Leijen, PLDI 2021, Koka) | per-value ownership/borrow annotation inferred by the compiler | ownership inference + RC insertion, with drop-reuse specialization | i `+`; ii `-` RC traffic; iii `+`; iv `+`; v `-` excluded for ordinary data | excluded by (v); but its *reuse analysis* is a useful precedent for in-place update without a linear source discipline |
| Capability-indexed deallocation | Capability Calculus (Walker, Crary, Morrisett, TOPLAS 2000) | a linear capability per region; `free` consumes it | linear type rule; capability algebra (also a bounded fractional form) | i `+`; ii `+`; iii `+`; iv `+` one uniform rule; v `+` | region granularity: freeing one object inside a live region needs finer capabilities (which is what L3/Alias Types supply) |
| Typestate on resource protocols | Bierhoff, Aldrich (OOPSLA 2007); ESP applied to file-handle protocols (Das, Lerner, Seigle, PLDI 2002) | protocol automaton per resource type; permission kind per reference | finite-state dataflow with permission splitting | i `+`; ii `+`; iii `+`; iv `+` protocol states are nameable in diagnostics; v `~` | needs an aliasing story underneath; on its own it does not bound how many holders exist |

---

## R3. Value classes and transfer (copy / affine / linear; move; by-value vs by-handle; hole-and-refill)

| Mechanism family | Representative works | Information the checker needs | How it is checked | Fit i / ii / iii / iv / v | Cost or failure mode |
|---|---|---|---|---|---|
| Multiplicity-annotated arrows (linearity on the *use*, not the type) | Linear Haskell (POPL 2018) | arrow multiplicity `1`/`ω`; multiplicity of each binder | type rule with multiplicity semiring arithmetic | i `+`; ii `+`; iii `+`; iv `+` one annotation site, very regular; v `~` unrelated to layout | multiplicity polymorphism adds spec surface; says nothing about placement or handles |
| Uniqueness typing (a *reference* attribute, dual to linearity) | Clean (Barendsen, Smetsers, MSCS 1996); Futhark (Henriksen, Serup, Elsman, Henglein, Oancea, PLDI 2017) | uniqueness attribute per reference; sharing analysis | attribute propagation + sharing check, syntax-directed | i `+`; ii `+`; iii `+`; iv `+`; v `+` enables in-place array update | uniqueness is about the *last* reference, not about obligation; does not force release (see R2) |
| Explicit take/put on unboxed aggregates | Cogent (ICFP 2016) `take`/`put` for record fields; ATS `T?` views (PADL 2005) | which field currently holds a value vs a hole; type of the hole | linear type rule; field state is in the record's type | i `+`; ii `+`; iii `+`; iv `+` **directly models WF's hole-and-refill**; v `+` no hidden flag | the record type changes shape per field state, so types get large; needs good notation or the spec bloats |
| Mutable value semantics with projections/`inout` | Hylo/Val (Racordon, Shabalin, Zheng, Abrahams, Saeta, JOT 2022); Swift SE-0176 "Enforce Exclusive Access to Memory" (2017); Swift `_read`/`_modify` coroutine accessors (**unverified**) | which arguments are `let`/`inout`/`sink`/`set`; exclusivity of overlapping access paths | static exclusivity check on access paths; dynamic check only for class/global storage | i `+` static part; ii `~` Swift's *dynamic* exclusivity checks would violate (ii); iii `+`; iv `+` parameter conventions are a small regular vocabulary; v `+` by-handle passing with noalias semantics | pure MVS forbids aliased object graphs entirely; the "projection" mechanism is the clean answer to interior access without a general reference type |
| Linear ghost permission separated from the runtime value | Verus `Tracked<PointsTo<T>>` (Lattuada et al., OOPSLA 2023); L3 (TLCA 2005) | the runtime pointer, plus a separate linear permission naming the same location | linear/affine checking on the ghost term; erased before lowering | i `+` for the linear part (Verus's *functional* side uses SMT — WF must not); ii `+` ghost erased; iii `+`; iv `+` permissions are ordinary first-class values the writer moves around; v `+` | the writer must thread permissions manually through every call; this is the verbosity WF has decided it can afford |
| Affine move with implicit copy for `Copy` types | Rust (RFC 2094; Oxide arXiv 2019/2021) | per-type copy-ness; per-place move state | type-directed; move state via dataflow | i `+`; ii `-` drop flags for conditional moves; iii `-`; iv `+` familiar; v `+` | the `Copy`/`!Copy` split plus `Drop` plus partial moves is three interacting rules where one could do |

---

## R4. Aliasing facts for the optimizer (ideally per storage and per span, not per parameter)

| Mechanism family | Representative works | Information the checker needs | How it is checked | Fit i / ii / iii / iv / v | Cost or failure mode |
|---|---|---|---|---|---|
| Exclusive-reference types lowered to `noalias` | Rust `&mut` -> LLVM `noalias`; Stacked Borrows (Jung, Dang, Kang, Dreyer, POPL 2020); Tree Borrows (Villani, Hostert, Dreyer, Jung, PLDI 2025) | borrow kind per reference; retag points; the aliasing model itself | borrow checker statically; the *model* is validated dynamically by Miri | i `+`; ii `+`; iii `-` interior mutability and `unsafe` falsify the fact; iv `+`; v `~` facts attach to parameters, not to data loaded through them | the well-known gap: no fact survives a load of a pointer *out of* a struct; both Stacked and Tree Borrows exist because `unsafe` can break it |
| Per-location store typing (facts are per storage, not per parameter) | Alias Types (ESOP 2000); L3 (TLCA 2005); Capability Calculus (TOPLAS 2000) | a name `ρ` per abstract location; a linear capability per name; separation is by distinctness of names | syntax-directed; two distinct linear capabilities are trivially disjoint | i `+`; ii `+`; iii `+`; iv `~` verbose but regular; v `+` **strongest per-storage facts of any family here** | needs location polymorphism and explicit existential pack/unpack at data-structure boundaries |
| Region + effect annotations on code, not on pointers | DPJ (Bocchino et al., OOPSLA 2009); Lucassen, Gifford (POPL 1988); Legion (Bauer, Treichler, Slaughter, Aiken, SC 2012); Regent (Slaughter, Lee, Treichler, Bauer, Aiken, SC 2015) | region partition of the heap; read/write effect summary per method | effect subsumption + disjointness of region path expressions | i `+` for DPJ's index-based disjointness; ii `+` (Legion resolves some disjointness dynamically -> `-`); iii `+`; iv `+` effects are declarative and local; v `+` scales to loop nests and arrays | array index partitioning needs arithmetic disjointness proofs, which is exactly where solvers usually creep in — WF must fix a decidable fragment |
| Uniqueness / in-place update guarantees | Clean (MSCS 1996); Futhark (PLDI 2017); Cogent (ICFP 2016) | uniqueness attribute; array aliasing summary | attribute propagation; syntax-directed | i `+`; ii `+`; iii `+`; iv `+`; v `+` for arrays specifically | array-shaped only; weak for pointer graphs |
| Proof-derived separation exported to the backend | RefinedRust (Gäher, Sammler, Jung, Krebbers, Dreyer, PLDI 2024); Verus (OOPSLA 2023); Iris (Jung et al., POPL 2015; JFP 2018) | separation-logic assertions about disjoint footprints | proof obligations discharged by tactics/SMT | i `-` SMT/tactic search; ii `+`; iii `+`; iv `-` proof failures are non-local; v `+` in principle | the facts are the strongest available but the *derivation* is exactly what WF's constraint (i) forbids; usable only if the fact form is fixed and the steps are writer-supplied |
| Compiler-side recovery (no source fact at all) | Bondhugula, Hartono, Ramanujam, Sadayappan (PLDI 2008, polyhedral); Diwan, McKinley, Moss (PLDI 1998, type-based alias analysis) | nothing from the source | whole-function analysis, heuristically bounded | i `-` budgeted search; ii `+`; iii n/a; iv `+` zero writer cost; v `~` best-effort, defeated across calls | this is the archaeology WF exists to avoid; still the right fallback for facts no one wants to state |

---

## R5. Data-race freedom and static parallel permission (statement overlap, iteration overlap)

| Mechanism family | Representative works | Information the checker needs | How it is checked | Fit i / ii / iii / iv / v | Cost or failure mode |
|---|---|---|---|---|---|
| Region-and-effect deterministic parallelism | DPJ (Bocchino et al., OOPSLA 2009); Regent (SC 2015) | region partition; per-statement read/write effects; index-parameterized regions for loops | effect disjointness check; `foreach` legal iff per-iteration effects are provably disjoint | i `+` if the index disjointness fragment is fixed; ii `+`; iii `+`; iv `+` effects are declared where the writer already declares the loop; v `+` **built exactly for iteration overlap** | needs a decidable index algebra; DPJ used index-parameterized arrays + a small set of disjointness rules rather than general arithmetic |
| Ownership transfer to threads (`Send`/`Sync` style) | Rust; RustBelt (POPL 2018) | which types may cross a thread boundary; which may be shared | trait/marker check + borrow check | i `+`; ii `+`; iii `-` the marker traits are `unsafe` to implement; iv `+`; v `~` gives thread-level, not iteration-level, parallelism | says nothing about *which statements may overlap*; coarse-grained |
| Reference capabilities that deny operations | Pony (Clebsch, Drossopoulou, Blessing, McNeil, AGERE! 2015) `iso`/`val`/`ref`/`box`/`trn`/`tag` | a capability per reference; viewpoint adaptation rules | type rules (capability lattice + viewpoint adaptation) | i `+`; ii `+` for races; iii `+`; iv `~` six capabilities plus viewpoint adaptation is a lot of surface; v `~` actor-granular, and Pony uses a GC | the capability matrix is the standard complaint about regularity; and actor model != loop parallelism |
| Fractional / counting permissions | Boyland (SAS 2003); Bornat, Calcagno, O'Hearn, Parkinson (POPL 2005); Chalice (Leino, Müller, Smans, FOSAD 2009); Viper (Müller, Schwerhoff, Summers, VMCAI 2016) | permission amount per location; permission transfer at fork/join and lock acquire | separation-logic proof obligations, discharged by SMT in practice | i `-` as deployed (Viper/Chalice use SMT); `~` if restricted to a fixed split/join calculus; ii `+`; iii `+`; iv `~`; v `+` | fractions give elegant read-sharing, but full permission arithmetic drags in a solver; a fixed halving discipline is the deterministic subset |
| Concurrent separation logic with invariants/protocols | O'Hearn (TCS 2007; CONCUR 2004); Iris (POPL 2015; JFP 2018); Concurrent Abstract Predicates (Dinsdale-Young, Dodds, Gardner, Parkinson, Vafeiadis, ECOOP 2010); Verus state-machine sharding (Hance et al., OSDI 2023) | resource invariant per lock; ghost protocol state | proof obligations | i `-` proof search; ii `+`; iii `+`; iv `-` non-local; v `+` expressive enough for lock-free code | the only family that handles genuinely shared concurrent state; cost is a proof language, which WF must render as explicit finite `use` steps |
| Purity + uniqueness (parallelism from the absence of effects) | Futhark (PLDI 2017); Cogent (ICFP 2016) | purity of the operator; uniqueness of the array being updated | syntax-directed | i `+`; ii `+`; iii `+`; iv `+`; v `+` for data-parallel shapes, `-` for general imperative statement overlap | restricted programming model; does not cover general statement-level overlap over mutable structures |

---

## R6. Modular frame / proof-fact retention (which facts survive a write or a call)

| Mechanism family | Representative works | Information the checker needs | How it is checked | Fit i / ii / iii / iv / v | Cost or failure mode |
|---|---|---|---|---|---|
| Separation-logic frame rule (framing is *free* from disjointness) | Reynolds (LICS 2002); O'Hearn, Reynolds, Yang (CSL 2001); Iris (POPL 2015) | footprint of each command as a separating conjunct | structural rule; no obligation when footprints are syntactically separate | i `+` when separation is syntactic (a linear context split), `-` when it needs entailment search; ii `+`; iii `+`; iv `+` frame is invisible when it works; v `+` | if the writer must *prove* disjointness rather than exhibit it by distinct linear names, the search returns |
| Explicit modifies clauses / dynamic frames | Dafny (Leino, LPAR 2010); Boogie (Barnett, Chang, DeLine, Jacobs, Leino, FMCO 2005); Kassios dynamic frames (FM 2006); Low* (Protzenko et al., ICFP 2017) | a set-valued `modifies`/`reads` expression per callable | frame axiom generated per call; discharged by SMT | i `-` set reasoning goes to the solver; ii `+`; iii `+`; iv `+` the clause itself is a clean, local declaration; v `+` | the declaration form is excellent for an AI writer; the *discharge* is the problem — WF needs a syntactic footprint algebra, not set entailment |
| Implicit dynamic frames | Smans, Jacobs, Piessens (ECOOP 2009); Viper (VMCAI 2016) | accessibility predicates `acc(x.f)` in pre/postconditions | permission accounting + SMT for the functional part | i `-` as deployed; ii `+`; iii `+`; iv `~`; v `+` | hybrid: the permission part is decidable accounting, the value part is not — a natural place to draw WF's automatic/explicit line |
| Ownership-based framing (framing derived from the type system) | Prusti (Astrauskas, Müller, Poli, Summers, OOPSLA 2019); Flux (Lehmann, Geller, Vazou, Jhala, PLDI 2023); Creusot (Denis, Jourdan, Marché, ICFEM 2022) | the borrow structure already present in the program | core frame comes from types; only value facts go to the solver | i `~` types decide framing (good), values still need SMT; ii `+`; iii `+`; iv `+` no separate frame language to learn; v `+` | **the key precedent for WF**: framing as a type-system consequence, so no frame clauses are written and no set reasoning is needed |
| Store typing as the frame (the type *is* the footprint) | Alias Types (ESOP 2000); L3 (TLCA 2005); Capability Calculus (TOPLAS 2000) | the full store type before and after each operation | syntax-directed rewriting of the store type | i `+` completely deterministic; ii `+`; iii `+`; iv `+` errors are local to one store-type mismatch; v `+` | store types must be complete, so unaffected memory is still named; scaling to large modules needs existential/region abstraction |
| Region logic (frames stated with region expressions) | Banerjee, Naumann, Rosenberg (ECOOP 2008) | region-valued ghost expressions; separator assertions | proof obligations over region sets | i `-`; ii `+`; iii `+`; iv `~`; v `+` | more expressive than syntactic separation, at the price of set-level reasoning |

---

## R7. Signature-only modular checking (callee leaves a hole, frees, reallocs, returns an interior pointer)

| Mechanism family | Representative works | Information the checker needs | How it is checked | Fit i / ii / iii / iv / v | Cost or failure mode |
|---|---|---|---|---|---|
| Store-type pre/post in the signature | Alias Types (ESOP 2000); Capability Calculus (TOPLAS 2000) | `∀ρ. (store-in) -> (store-out)`, including locations that become holes or vanish | syntax-directed subsumption at the call site | i `+`; ii `+`; iii `+`; iv `+` the signature literally says "leaves a hole at ρ"; v `+` | signatures get long; location polymorphism and existential packing are mandatory, not optional |
| Linear capability passing (pointer value vs. capability separated) | L3 (TLCA 2005); Verus `Tracked<PointsTo>` (OOPSLA 2023) | which capabilities are consumed, returned, or reshaped by the callee | linear context split at the call | i `+` for the linear part; ii `+`; iii `+`; iv `+` realloc is naturally "consume cap for ρ, return cap for ρ'"; v `+` | every call site threads permissions explicitly; higher-order code needs capability-polymorphic signatures |
| Permission-passing signatures with flow annotations | Mezzo (Pottier, Protzenko, ICFP 2013; TOPLAS **unverified**) | permissions consumed/produced; `consumes` annotations on parameters | permission subtyping at call sites, mostly syntax-directed | i `+` mostly; ii `-` for the `adopt`/`give`/`take` dynamic-ownership escape; iii `~`; iv `+`; v `+` | Mezzo's own experience report is the cautionary data: the escape hatch existed because purely static sharing was too restrictive |
| Region-polymorphic signatures with effects | Cyclone (PLDI 2002); Tofte, Talpin (Inf. & Comp. 1997); DPJ (OOPSLA 2009) | region parameters, outlives constraints, effect summary | constraint check per call | i `+`; ii `+`; iii `-` in Cyclone; iv `+`; v `~` | regions express lifetime and disjointness but not per-object holes or double-release |
| Modifies/reads clauses on the signature | Dafny (LPAR 2010); Low* (ICFP 2017) | footprint sets, `fresh`, `old` | SMT-discharged frame axioms | i `-`; ii `+`; iii `+`; iv `+` very legible signatures; v `+` | best-in-class *notation* for "this call reallocs / this call frees"; the discharge mechanism is what WF must replace |
| Lifetime-parameterized signatures returning interior pointers | Rust `fn get(&mut self) -> &mut T`; Oxide (arXiv 2019/2021); Prusti (OOPSLA 2019) | lifetime parameters and outlives bounds; reborrow structure | region inference + borrow check | i `+`; ii `~`; iii `-`; iv `~` errors mention inferred regions the writer never wrote; v `+` | this is exactly the "interior pointer out of a call" case; Rust solves it, but with inference that produces the least local diagnostics in the language |

---

## R8. Storage placement and relocation (stack / heap / arena / inline-in-container; realloc; interior pointers)

| Mechanism family | Representative works | Information the checker needs | How it is checked | Fit i / ii / iii / iv / v | Cost or failure mode |
|---|---|---|---|---|---|
| Regions/arenas as first-class typed entities | Tofte, Talpin (Inf. & Comp. 1997); Cyclone (PLDI 2002); Capability Calculus (TOPLAS 2000) | region of each allocation; region lifetimes; capability to allocate/free in a region | type-and-effect + capability rules | i `+`; ii `+` (Cyclone's dynamic regions excepted); iii `-` Cyclone; iv `+` explicit `rnew(r)` is very regular; v `+` bump allocation is the fast path | region-only freeing is coarse; long-lived regions retain garbage; needs a per-object mechanism alongside |
| Location names + existential packing (relocation = renaming) | Alias Types (ESOP 2000); L3 (TLCA 2005) | old and new location names; the capability transfer between them | type rule: `realloc: cap(ρ) -> ∃ρ'. cap(ρ')` | i `+`; ii `+`; iii `+`; iv `+` relocation has a *name change*, which is the most explainable form; v `+` | every pointer to the old name becomes ill-typed, which is correct but forces the writer to re-derive every derived pointer explicitly |
| Take/put on inline (unboxed) storage | Cogent (ICFP 2016); ATS views (PADL 2005) | which inline slots are full/empty; boxed vs unboxed in the type | linear type rules; slot state in the record type | i `+`; ii `+`; iii `+`; iv `+`; v `+` inline-in-container without a hidden tag | type-level slot state multiplies record types; nested containers compound it |
| Projections / yielded borrows for interior access | Hylo/Val (JOT 2022) subscripts; Swift `_read`/`_modify` accessors (**unverified**); Rust `IndexMut` | which access path is projected; the span over which the projection is live | static exclusivity over access paths | i `+`; ii `~` Swift falls back to dynamic exclusivity for some storage; iii `+`; iv `+` projections scope naturally to a block; v `+` | interior pointers escaping the projection must be forbidden; that restriction is what keeps it decidable |
| Ownership-annotated IR with explicit placement | Swift SIL Ownership SSA (**unverified**); MLIR bufferization + ownership-based deallocation (**unverified**) | per-SSA-value ownership kind (owned/guaranteed/trivial); buffer ownership | IR verifier rules; deallocation pass | i `+` verifier is syntactic; ii `-` for the inserted ownership bits; iii n/a; iv n/a; v `+` | precedent for the *lowering* target: an ownership-typed IR whose verifier is decidable and whose passes may exploit the facts |
| Regions as an optimization overlay over a dynamic scheme | Vale regions (Ovadia, **not peer-reviewed**) | which references are in an immutable/isolated region for a span | static region check removes the generational check inside the region | i `+`; ii `-` outside regions; iii `+`; iv `+`; v `~` | interesting *strategy* (static regions to eliminate dynamic checks) but starts from a runtime-checked baseline WF rejects |

---

## R9. Shared mutation among several holders

Several long-lived pointers to one object, occasional writes, sequential and concurrent.

| Mechanism family | Representative works | Information the checker needs | How it is checked | Fit i / ii / iii / iv / v | Cost or failure mode |
|---|---|---|---|---|---|
| Identity/permission separation via branded tokens | GhostCell (Yanovski, Dang, Jung, Dreyer, ICFP 2021) | a brand `'id` shared by cells; a single token whose borrow state grants read or write to *all* cells of that brand | ordinary borrow/type check on the token; no per-cell state | i `+`; ii `+` **zero runtime cost**; iii `-` in Rust (soundness rests on an `unsafe` core, proven in RustBelt); iv `+` one uniform pattern; v `+` | coarse granularity: the token is a single lock-like permission over the whole brand; two disjoint sub-structures cannot be written concurrently without splitting brands |
| Ghost permission objects tied to raw pointers | Verus `PointsTo`/`Tracked` + atomic invariants (Lattuada et al., OOPSLA 2023); L3 (TLCA 2005); Hance et al. state-machine sharding (OSDI 2023) | linear ghost permissions, separate from the runtime pointers; an invariant relating shared state to ghost state | linear checking of ghost terms + proof obligations for the invariant | i `~` linear part deterministic, invariant part is SMT in Verus; ii `+` ghost erased; iii `+`; iv `~` writer must construct the protocol; v `+` | the most expressive fit for "several long-lived pointers, occasional writes"; WF would need the invariant discharge to be explicit finite `use` steps |
| Deny capabilities / capability lattice | Pony (AGERE! 2015) | per-reference capability; viewpoint adaptation for field access | type rules | i `+`; ii `+`; iii `+`; iv `~` large capability matrix; v `~` relies on per-actor GC | shared-mutable across actors is expressible only by transferring `iso`; genuine concurrent sharing needs `val` (immutable) |
| Fractional permissions for read-sharing | Boyland (SAS 2003); Bornat et al. (POPL 2005); Chalice (FOSAD 2009); Viper (VMCAI 2016) | permission fraction per location; splitting/joining points | permission arithmetic; SMT in deployed tools | i `~` decidable if restricted to a fixed split discipline; ii `+`; iii `+`; iv `+` fractions are a small concept; v `+` | handles many-readers-one-writer cleanly; does not by itself handle *occasional writes* through long-lived aliases without a protocol |
| Session-typed / protocol-mediated sharing | Balzer, Pfenning (ICFP 2017, manifest sharing); Concurrent Abstract Predicates (ECOOP 2010); Iris STS (POPL 2015) | a protocol (STS/session type) governing acquire-release of the shared resource | type rules (session) or proof obligations (STS) | i `~` session part can be syntax-directed; `-` for STS proof search; ii `~` acquire/release may lower to real synchronization (fine: it is a typed outcome with control flow); iii `+`; iv `~`; v `~` | manifest sharing makes the acquire/release points *visible in the type*, which matches WF's "falsifiable condition becomes a typed outcome" rule remarkably well |
| Runtime-enforced interior mutability | Rust `RefCell`/`Rc`; Mezzo `adopt`/`take` (ICFP 2013); Vale generational refs | per-object borrow flag / owner tag / generation | runtime check and trap | i `+`; ii `-` **disqualifying**; iii `-` (`Rc<RefCell<_>>` also defeats R4); iv `+`; v `-` | the baseline WF exists to eliminate; useful only as the comparison point for expressiveness loss |

---

## E. Cross-requirement couplings observed in the literature

| Design | What one mechanism serves | What the coupling buys | What it costs |
|---|---|---|---|
| Rust `&mut`/`&` (RFC 2094; RustBelt POPL 2018; Stacked Borrows POPL 2020) | R1, R4, R5 (via `Send`/`Sync`), R6 (framing from borrows, used by Prusti/Flux), R9 (by *excluding* it) | one concept the writer learns once; aliasing facts come free with safety | R9 becomes unrepresentable, forcing `RefCell`/`unsafe` back in, which then falsifies R4 for everyone; and lifetime inference makes R7 diagnostics non-local. Tree Borrows exists precisely to re-describe what the coupling means once `unsafe` participates |
| Rust drop + drop flags | R2 coupled to R3 (affinity) | writer never writes a release | a hidden runtime bit — the coupling silently converts a *type* obligation into a *runtime* one, violating WF (ii) |
| Cogent linearity (ICFP 2016) | R1, R2, R3, R4, R8 all from one linear discipline | a very small spec; a verification story that scales | no sharing at all; R9 is simply absent, and recursion/loops need a restricted style |
| Regions (Tofte-Talpin 1997; Cyclone PLDI 2002) | R1 + R8 together | allocation and lifetime in one concept, with fast bump allocation | R2 becomes coarse (free the region, not the object); R4 is weak, because same-region does not mean same-object |
| DPJ (OOPSLA 2009) | R4 + R5 from one region/effect system, **deliberately decoupled from R1/R3** | parallelism claims are checked from *effects on code*, not from ownership of values; the object model stays conventional | effect annotations duplicate information the type system almost has; region path expressions are a second naming system |
| **Decoupling: pointer vs. capability** — Alias Types (ESOP 2000), L3 (TLCA 2005), Verus `PointsTo` (OOPSLA 2023) | the *value* (a pointer, freely copyable) is split from the *right* (a linear capability) | R3 (copying a pointer) stops implying R1/R4 consequences; R7 signatures can say "consumes cap(ρ), returns cap(ρ')" for realloc; capabilities can be ghost and erased | the writer threads capabilities by hand everywhere; higher-order and recursive structures need capability polymorphism and existentials |
| **Decoupling: identity vs. permission** — GhostCell (ICFP 2021) | the cell carries *identity* (a brand); the token carries *permission* | R9 becomes expressible with no runtime cost, and R4 survives because the token's borrow is still exclusive | permission granularity collapses to the whole brand; concurrency within a brand needs a second mechanism |
| **Decoupling: value vs. ghost proof** — Verus (OOPSLA 2023) | linear ghost types carry R1/R2/R9 obligations; ordinary Rust types carry the data | proofs erase; the same mechanism handles sequential and concurrent sharing | the functional side is SMT-discharged; WF would keep the linear/ghost structure and replace the discharge with explicit steps |
| **Decoupling: framing from specification** — Prusti (OOPSLA 2019), Flux (PLDI 2023) | R6 is derived from R1's type structure rather than declared | no `modifies` clauses to write, no set reasoning for the frame | only works where the ownership discipline already holds; shared state falls back to explicit permissions |
| **Decoupling: access convention from reference type** — Hylo/Val (JOT 2022), Swift SE-0176 | R3 (how a value is passed) is separated from R1/R4 (whether a reference exists at all) | no first-class references at all, so R4 facts are structural; projections give interior access with a lexical span | an aliased mutable object graph is simply not expressible; shared structure needs indices into a container, not pointers |
| **Decoupling: protocol from ownership** — Balzer, Pfenning (ICFP 2017) | R9 acquire/release is *manifest in the type* rather than hidden in a lock | sharing points are visible, checkable, and become ordinary control flow | adds a session-type layer; interaction with R4 (what the optimizer may assume across an acquire) is not worked out in that line |

---

## F. Assessment (my judgment, not established literature)

Clearly labeled as opinion: best-matched families per requirement, with the main open risk.

| Req | Best-matched family (1-2) | Why | Main open risk |
|---|---|---|---|
| R1 | Store/alias typing with linear capabilities (Alias Types, L3); ATS/Cogent views as the surface syntax | fully syntax-directed, so (i) holds with no inference; errors are local, which is what an AI writer needs; no runtime residue | store types for large aggregates and recursive structures need an abstraction mechanism (existentials/regions) that could grow the spec past "small and regular" |
| R2 | Strict linear obligations (Austral/Cogent/Linear Haskell style), with conditional release written as real control flow | directly satisfies "no drop flags": the obligation is discharged syntactically on every path | joining linear contexts at merges forces a lot of explicit code around early exits; needs a good `defer`-free idiom or the writer will fight it |
| R3 | Cogent-style `take`/`put` over inline storage, plus multiplicity annotations for copy/affine/linear | `take`/`put` *is* the hole-and-refill requirement, with slot state in the type rather than in a runtime flag | type-level slot state on nested aggregates explodes combinatorially; needs notation that keeps signatures readable |
| R4 | Per-location capabilities (Alias Types/L3) for pointer data; DPJ-style region+effect for arrays and loops | per-storage and per-span facts, not per-parameter, which is exactly the gap the project has identified in Rust | the array/index disjointness fragment must be fixed and decidable; this is where a solver would otherwise creep in |
| R5 | DPJ-style effects over an index-parameterized region partition | designed for iteration overlap, declarative, and checkable by disjointness rules rather than search | non-affine index patterns (indirection, gather/scatter) fall outside any fixed fragment and must degrade to explicit writer-supplied proof steps |
| R6 | Store typing as the frame (Alias Types/L3), with ownership-derived framing as the modular story (Prusti/Flux precedent) | framing becomes a type-system consequence, so nothing is written and nothing is searched | facts about *values* (not just locations) still need a retention rule; the boundary between "framed automatically" and "restated by the writer" must be specified, not discovered |
| R7 | Linear capability passing in signatures (L3/Verus style), with Dafny/Low*-quality *notation* for "frees / reallocs / returns interior pointer" | the semantics come from a decidable linear discipline; the readability comes from an explicit clause the writer states | capability-polymorphic signatures for higher-order and generic code; this is where signature size may become the real cost |
| R8 | Location names with existential repacking for relocation (L3), over region/arena placement (Capability Calculus) | relocation as a *name change* is the most explainable rule; arenas give the fast allocation path with a linear free capability | derived/interior pointers must all be re-derived after a realloc; the ergonomics of that in a tight loop are unproven at scale |
| R9 | GhostCell-style identity/permission separation for the sequential and read-mostly cases; Balzer-Pfenning manifest sharing (or a Verus-style ghost protocol with explicit steps) for genuine concurrent sharing | both keep (ii): the permission is static or the acquire is a typed outcome with real control flow; neither adds a per-access runtime check | granularity (a brand-wide token is too coarse for fine-grained concurrency) and spec size (a protocol layer is a second sub-language); this is the requirement least well served by any existing deterministic mechanism |


# Part D. Threads, locks and external resources under the storage-identity model


Research note, 2026-09-16. Read-only survey for the access-effects investigation.
Not an amendment, not a design decision, not a claim of compiler support.
Baseline: `spec/kernel-spec.md` §13; `docs/constitution.md`;
`design/language/{system-interface,effects,ownership,parallelism}.md`;
`research/investigations/{io-model,access-effects,containers-and-resources}/`.

## 0. The four constraints everything below is judged against

- **CAP-1**: no writer-visible capability category, no additional concurrency
  permission. `own`, `&`, `&uniq`, place overlap and the effect row are the
  *complete* authority and interference vocabulary. A later thread construct
  "must derive transfer and sharing permission from these same ownership rules
  and the represented type". Data-race impossibility is law; general race
  conditions are out of scope.
- **PAR-1/PAR-2 are permissions, never obligations**: denial selects sequential
  lowering, never rejection. Their guarantee is stronger than race freedom — it
  is *equality with the source-order result* in every execution.
- **Constitution**: no rule distinguishes a value or function by whether its
  implementation crosses the host boundary.
- **Derivation discipline**: specification-fixed, deterministic, terminating; no
  SMT for acceptance; proofs erased; no compiler-inserted runtime safety check.

One distinction is load-bearing throughout. *A compiler-inserted runtime check
that rescues an unproved operation* is forbidden. *A trusted runtime mechanism
that safety rests on* is different: PAR-1's identity guarantee is already
"conditional on contract compliance, exactly as [SCOPE-3]'s freedom from
undefined behavior is conditional on its trusted computing base". A mutex, if WF
gets one, is a **TCB extension**, not an inserted check — a decision, not a
derivation.

---

## A. Threads and transfer

### A.1 Prior art: what moves, how disjointness is established, what runs

| Mechanism | What is transferred at fork | How disjointness is established | What runs at runtime | Determinism promised |
|---|---|---|---|---|
| CSL parallel rule (O'Hearn, TCS 2007; semantics Brookes, TCS 2007) | A *split of the assertion*: `{P1*P2} C1‖C2 {Q1*Q2}`. Heap ownership, logically. | The `*` at the split point, plus side conditions on program variables: no variable free in `P1,C1,Q1` is modified by `C2`, and conversely. Variables-as-resource removes the side conditions. | Nothing. The split is erased. | No; race freedom + partial correctness only |
| Chalice (Leino & Müller, ESOP 2009) | *Fractional permissions* named in the forked method's precondition; `fork tk := o.m()` moves them out, `join tk` brings the postcondition's back. | Permission arithmetic — no thread can hold more than the full fraction. `tk` is a linear token. | Nothing for permissions. Real locks for monitors; `waitlevel` deadlock order is static. | No |
| Verus (Lattuada et al., OOPSLA 2023; IronSync, Hance et al., OSDI 2023) | *Linear ghost tokens* (`PointsTo<T>` and tracked resources) moved into the spawned closure; the child's exit tokens come back through the join handle. | Two layers: Rust's own `Send`/borrow discipline for the value transfer, token linearity for the logical transfer. Proof obligations discharged by SMT. | Nothing for ghost (erased). Real `std::thread`, real atomics/locks. | No |
| Rust `Send`/`Sync` (RustBelt, Jung et al., POPL 2018) | *Ownership of values*, at the type level. `Send` is a whole-type judgment; `Sync` is `&T: Send`. | Not a footprint: the borrow checker's lifetime bound (`'static`, or a scope for `thread::scope`) plus the auto-trait derivation. | Nothing for the discipline. `Arc` refcounts, `Mutex`, etc. run if used. | No |
| Pony reference capabilities (Clebsch et al., AGERE! 2015) | A *reference capability* travels with the reference; the sendable set is `iso`, `val`, `tag`. | Denial: each capability states which aliases are denied locally and globally; sendability falls out of the denial set. `recover` re-establishes `iso`. | Actor scheduler; ORCA concurrent GC. | No |
| DPJ (Bocchino et al., OOPSLA 2009) | *Nothing.* Tasks share the heap; `reads`/`writes` effect summaries over region path lists are compared at `cobegin`/`foreach`. | Static RPL disjointness; index-parameterized arrays for the per-element case. | Nothing. Determinism is entirely static. | **Yes** — deterministic by construction |
| Regent / Legion (Slaughter et al., SC 2015; Bauer et al., SC 2012) | *Privileges* (`reads`/`writes`/`reduces`) over logical regions, granted per task. | Type checker checks privileges statically; **partition disjointness and inter-task dependences are computed by the runtime**, which builds a dynamic dependence graph. | A dynamic dependence analysis and a distributed runtime. | Sequential semantics, but enforced dynamically |

### A.2 Fit against WF's three constraints

| Mechanism | Deterministic, non-SMT? | Runtime safety mechanism? | New sharing category (CAP-1 conflict)? | Usable in WF |
|---|---|---|---|---|
| CSL parallel rule | Yes — it *is* PAR-1's footprint judgment | No | No | **Already present.** PAR-1 is the CSL rule with syntactic footprints |
| DPJ effect summaries | Yes | No | Regions are effect subjects, not sharing classes | **Closest match.** WF identities do DPJ regions' job at finer resolution |
| Chalice fork/join tokens | Permission arithmetic yes; surrounding proofs no | No | Fractions are a sharing class | Token shape usable (#4); fractions unnecessary — `&` loans end lexically, so nothing must recombine by counting |
| Verus ghost tokens | Shape yes, discharge no (SMT) | No | No — ordinary linear values | **Right shape.** WF must fix syntactically what Verus sends to a solver |
| Rust `Send`/`Sync` | Yes — structural auto-trait derivation | No | Yes, a type-level sendability class | Not needed: WF's identity footprint is a *finer* fact than `Send` |
| Pony capabilities | Yes | Scheduler + GC | Yes, six capabilities | Excluded as a source category; the insight (sendability from mode + type) is what CAP-1 already mandates |
| Regent privileges | Privileges yes; **partition disjointness is dynamic** | **Yes** | Coherence modes are a sharing class | The dynamic half is what WF forbids. PAR-2's affine refinement is WF's *static* substitute for Legion's runtime partition check |

### A.3 What the checker must know at spawn and at join

Under the identity model the fork point is a **partition of the checker's
context**, not a transfer of tokens. Concretely:

| At | The checker must know | Why the identity model makes it harder or easier than CSL |
|---|---|---|
| Spawn | 1. The child's complete read/write footprint over identities, after call-boundary projection. | Easier: this is already PAR-1's footprint, with field and proved index/range refinement. |
| Spawn | 2. For each identity in that footprint, the **entry state** the child requires (`initialized` / `uninitialized` / and any `ite(c,·,·)`). | Harder: CSL ships a single assertion; WF ships a per-identity state that may be conditional, and the condition's captured value must be in scope on both sides of the fork. |
| Spawn | 3. The **frame**: every identity the parent continues to touch in the fork/join window must be disjoint from the child's written footprint, in both directions, including argument-expression reads. | This is PAR-1's rule verbatim; a spawn is a PAR-1 window with an explicit join. |
| Spawn | 4. Which **affine release obligations** cross. An obligation is an owner binding; if it goes to the child, the parent no longer has it. | Harder than CSL: a never-joined child leaks, and the leak is *the parent's* obligation that has gone somewhere the parent cannot discharge it. |
| Spawn | 5. Loans held by the child for the child's whole extent, which under PAR-1 is "through the call's return" and under spawn is "through join". | An exclusive loan crossing the fork denies the parent every overlapping access until join. That is the scoped-thread guarantee. |
| Join | 6. The child's **exit states** for every transferred identity, including any `ite` over conditions the child captured. | This is the sharp one. A condition captured *inside* the child is not a term the parent has. Either the child's exit contract collapses conditional states to a join (over-approximate, may reject later reads), or the condition must be published as an ordinary returned value the parent can match on. |
| Join | 7. That every obligation that crossed is either discharged in the child or returned at join, on every exit edge of the child. | Otherwise the affine discipline has a hole at the thread boundary. |
| Join | 8. That the join actually dominates every later use of the returned identities. | With a lexically scoped join this is syntactic; with an escaping handle it is a dataflow fact and the handle must be linear. |

**The structural finding.** PAR-1 *is already* a statically-joined fork/join: the
join is the end of the window, and `CONCURRENCY-CATALOG.md` §12 shows
`std::thread::scope` mapping onto it with the verdict *direct, nothing changes*.
So a thread construct buys exactly two things PAR-1 does not have:

1. **Arity not written in source.** Catalog §0.1 consequence 3 and §7 identify
   this as "the single most frequently recurring cost" — PAR-1's width is a
   source constant because per-lane windows need exclusive loans rooted outside
   the loop, which PAR-2 denies. The catalog's own `[new]` note says an
   **iteration-exclusive affine *range* loan** `[c*i+b, c*(i+1)+b)`, proved
   disjoint by the arithmetic PAR-2 already runs for `a*i+b`, removes it. That
   is a PAR-2 refinement, **not a thread construct**, and it is the cheapest item
   in this whole report.
2. **A child that outlives its block.** This is the only thing that needs a
   handle, and the handle is Chalice's `tk` and Verus's `JoinHandle`: a linear
   value whose type carries the child's exit contract over the transferred
   identities. Everything in rows 6–8 above is the cost of that handle.

**Assessment.** Rows 1–5 need **no new vocabulary**: they are PAR-1's judgment
with the window end moved from "end of block" to "the join statement". Row 6 is
the one genuine semantic addition the identity model forces and CSL does not —
**conditional states must cross a join**; the conservative rule (a child's exit
contract may not mention a condition the parent cannot name, so `ite` states
collapse at the thread boundary) is deterministic, terminating, and costs only
precision. Nothing in rows 1–8 requires a runtime check: scoped spawn is
compatible with every WF constraint, and the escaping handle is too, at the cost
of one linear type.

---

## B. Locks and atomics

### B.1 The question restated

"Several holders, occasional writes, across threads, paying synchronization only
at the write" has exactly three answers in the literature, and they are not
variants of one mechanism.

| Route | What the readers hold | Read cost | Write cost | Static obligation |
|---|---|---|---|---|
| **R1 — phase / epoch** (WF today) | An ordinary shared loan, valid for the whole read phase | **Zero** | The phase boundary | None beyond the loan rules |
| **R2 — lock as fact custodian** (CSL, Chalice, Mezzo, Balzer/Pfenning) | Nothing outside a critical section | One acquire/release per *read* | One acquire/release | The invariant, and its precision |
| **R3 — protocol / fictional sharing** (CAP, Iris, STS, Verus atomics) | A **stable** fact, one the writer's permitted actions cannot falsify | Zero, or one relaxed load | One atomic RMW | Stability of every retained fact under the other party's action set |

Only **R3** literally answers the question. R2 pays at every hold.

`CONCURRENCY-CATALOG.md` §8 already records R1's outcome for read-mostly data:
RCU's grace period becomes the lexical region boundary, `synchronize_rcu()`
disappears, reclamation is immediate and exact, reader cost is **zero** against
an `RwLock` read's ~15–25 ns, verdict *restructure, no loss, structural win*.
The only loss is publish latency bounded by the read-phase length. That is a
strong result and it is the reason R2 and R3 are not obviously worth their price.

### B.2 Prior art for R2 and R3

| Mechanism | Declaration states | Acquire / open | Release / close | Runtime |
|---|---|---|---|---|
| CSL resource invariant (O'Hearn, TCS 2007) | `RI_r`, owned by the resource while unlocked | Adds `RI_r` to the context | Demands `RI_r` back | The CCR/mutex. `RI_r` must be **precise** or the conjunction rule is unsound |
| Chalice monitors (Leino & Müller, ESOP 2009) | A monitor invariant; `share o` circulates it | Yields the invariant's permissions | Requires them back; `waitlevel` is a static lock order | The monitor |
| Mezzo locks (Balabonski, Pottier & Protzenko, TOPLAS 2016) | `lock p` holds permission `p`; handle duplicable, `p` affine | Yields `p` | Consumes `p` | The lock. Mezzo's *adoption/abandon* is a **dynamic** ownership test — WF cannot take that half |
| Shared session types (Balzer & Pfenning, ICFP 2017) | Adjoint modalities stratify a linear and a shared layer | Turns a shared channel into a linear one | Returns it, and **equi-synchronizing** requires release *at the type it was acquired at* | The acquire blocks |
| CAP (Dinsdale-Young et al., ECOOP 2010) | A shared region plus an **action set**; threads hold guards | Opens the region for one atomic step | Re-establishes the interpretation | The atomic. Every retained assertion must be **stable** under others' actions |
| Fictional separation (Jensen & Birkedal, ESOP 2012) | A monoid/PCM imposing a *fiction* of disjointness on shared state | — | — | — |
| STS protocols (Turon et al., ICFP 2013; Iris, POPL 2015 / JFP 2018) | A transition system plus tokens bounding others' transitions | — | — | — |
| Verus atomics (Lattuada et al., OOPSLA 2023) | An atomic paired with ghost state under an invariant (`AtomicInvariant` / `atomic_with_ghost!`; *names unverified*) | Opens it for **one** atomic instruction | Closes it | The atomic RMW |

The common price of R3 is **stability**. WF's checker retains *exact* facts:
`ite(c, s1, s2)` states, exact affine value images, discharged bounds results,
captured range endpoints. None of those are stable under an unsequenced writer.
Low\*'s answer to the same problem is instructive: it admits external interference
only through **monotonic references and `witnessed` predicates** — facts that,
once true, can never be falsified (Protzenko et al., ICFP 2017; *the exact
library shape is unverified*). Monotone facts are the stable fragment.

### B.3 An allocator sketch: lock-protected metadata, exclusively owned blocks

```text
// --- declaration -------------------------------------------------------
// The lock is an ordinary nominal. Its declaration names the identities it
// custodies and the invariant those identities satisfy while unheld.
lock AllocLock custody 'meta
  invariant free_list_wf('meta)
  holds { live('meta), initialized('meta), layout('meta, FreeList) }

// --- acquire -----------------------------------------------------------
fn acquire(lock: &AllocLock) -> guard: own Guard<'meta>
  reads(lock), writes(lock)
  ensures live('meta), initialized('meta), free_list_wf('meta)
  ensures exclusive_loan(guard, 'meta)      // guard is LINEAR

// --- release -----------------------------------------------------------
fn release(guard: own Guard<'meta>)
  requires free_list_wf('meta)
  reads('meta), writes(lock)
  // consumes every fact about 'meta

// --- the operation -----------------------------------------------------
fn alloc(lock: &AllocLock, n: own u64)
  -> own Result<exists 'b. Owner<'b, Block>, OutOfMemory>
  reads(lock), writes(lock)                 // 'meta is NOT in the caller row
{
  let g = acquire(lock: lock);              // context GAINS 'meta at invariant
  let r = pop_free(meta: &uniq g, n: n);    // reads('meta), writes('meta)
  release(guard: move g);                   // context LOSES all 'meta facts
  return r;                                 // fresh 'b, live, uninitialized
}

fn fill(b: &uniq ptr<'b, Block>) reads('b), writes('b)   // no lock anywhere
```

| | |
|---|---|
| **Declaration states** | One custody set of identities and one invariant over them. `'meta` is existentially owned by the lock — never a caller identity, never in a caller-facing row. That is what makes `fill` and `alloc` non-conflicting: `fill` writes `'b1`, `alloc` writes `lock`. |
| **Acquire changes** | Adds the custody facts *at the invariant and no stronger*; takes an exclusive loan on every custody identity for the guard's extent; introduces one linear guard that must be consumed on every exit edge. |
| **Release changes** | Checks the invariant, then **erases every fact about every custody identity**, not only the custody facts — every refinement the body derived (an exact free-list length, an `ite` state, a discharged index bound) is dropped, because another thread may now act. This *fact firewall* is mechanical: a set-difference on the fact store keyed by supporting identity, which `design/language/effects.md` already needs for `writes`-projection killing. |
| **Statically checked** | Every block access; every block's release obligation; the invariant at both ends of every critical section; guard linearity; that no path touches `'meta` outside a guard extent. |
| **Runtime** | Mutual exclusion, and nothing else. No generation test, no alias test, no ownership check. |

### B.4 The finding that matters, and it is a refusal

Two `alloc` calls both exhibit `writes(lock)`. Under PAR-1 they therefore **deny
overlap and run sequentially** — safe, and useless, since a lock exists precisely
so two threads may both call `alloc`. Making them overlap requires a rule saying
`writes(L)` on a lock nominal does not deny thread-overlap because the lock
orders the accesses. That rule has two consequences of different severity:

1. It is a **new interference vocabulary entry** — a self-synchronizing effect
   subject. CAP-1 says the kernel has no such category. A specification-level
   decision, not a derivation.
2. It **breaks PAR-1's guarantee**, which is "bindings and every Whitefoot state
   place equal the source-order result", in every execution. Under a lock, which
   thread wins the free list is schedule-dependent. The program stays race-free
   and the invariant holds, but it is **not deterministic**.

So a lock cannot live under PAR-1. It needs a **separate permission with a weaker
guarantee**: race freedom plus invariant preservation, without source-order
equality — a second guarantee level in a language whose overlap story currently
has one. DPJ and Regent both keep determinism by declining exactly this.

**Atomics (R3) are worse; recommend against as a minimal addition.** They need a
memory model, a stability judgment over every retained fact, and an action-set
declaration per region, and they give up determinism too. The io-model has
already taken the other road: `DESIGN.md` §8 states io_uring SQ/CQ pages, IOCP
ports, MMIO and device-owned DMA state "cannot appear as ordinary Whitefoot
buffers"; the adapter represents them as atomic or channel protocols, in C, under
an ordinary ownership contract. That is a **trusted-implementation boundary**,
not a source-language exception, and the constitution forbids only the latter.

---

## C. External resources as ordinary objects

### C.1 A descriptor is not one identity; it is three

`RESEARCH.md`'s IO witness already says this: "wrapper, open-file description,
file contents, namespace, or provider accounting may be different subjects."
Making that concrete is most of the modelling work, and it needs no new rule.

| Subject | Identity | Who may change it | Carries the close obligation? |
|---|---|---|---|
| Descriptor slot / wrapper | `'fd` | This program only | **Yes** (affine owner) |
| Open-file description (cursor, flags) | `'ofd` | This program only; `dup` gives two `'fd` over one `'ofd` | No |
| File contents | `'file` | **Any agent**, reachable by any path in the namespace | No |
| Namespace entry | `'dir` | Any agent | No |
| Provider accounting | the `HandleFactory` value | This program (credits) | Yes, per `design/language/system-interface/handle-factory.md` |

### C.2 State tables

**File descriptor.** `'fd` and `'ofd` are program-ordered: no external agent can
close your descriptor or move your cursor. `'file` is never WF storage — it is
only ever behind a call.

| State | Operations admitted | Transition |
|---|---|---|
| (no identity) | `open_read(permit, root, path)` | → `Open` on `Ok`, fresh `'fd` + `'ofd`; permit consumed either way |
| `Open{read}` | `read_at` (`reads('fd,'file)`, `writes(dest)`); `read` (`writes('ofd)`) | stays `Open` |
| `Open{write}` | `write_at` (`writes('file)`); `write` (`writes('ofd,'file)`) | stays `Open` |
| `Open` | `dup` | fresh `'fd2` with its own obligation, **same** `'ofd` |
| `Open` | `close(own file)` | → `Ended`; obligation discharged |
| `Ended` | none | terminal; every alias loses access (RESEARCH.md: release is permanent) |

**Socket.** Per `design/language/system-interface.md`, `TcpConnection` is an
ordinary public struct of **separately closable direction owners** — read-half
and write-half are separate identities with separate obligations, and pairing
halves from different connections is well-typed and must stay correct.

| State | Operations | Transition |
|---|---|---|
| `Unbound` | `bind(permit, addr)` / `connect(permit, addr)` | → `Bound` / → `Connected` on `Ok` |
| `Bound` | `listen` | → `Listening` |
| `Listening` | `accept(&uniq src)` → `AcceptOutcome` | a `&uniq` state machine (io-model §5); yields a fresh `'sock` pair |
| `Connected` | `recv` (`writes('rx, dest)`), `send` (`writes('tx)`) | stays; **the peer is an external agent** |
| `Connected` | `shutdown(own tx)` | → `HalfClosed{read}`; the write half's obligation discharged alone |
| any | `close(own half)` | that half → `Ended` |

The peer matters only in that **no fact about stream contents is ever retained**:
every `recv` result comes from the call's postcondition. That is the ordinary
rule already — the checker has no way to know a byte is coming.

**Memory-mapped region.** This is the only one that is different.

| Kind | State | May another agent write the storage? | Verdict |
|---|---|---|---|
| Anonymous / `MAP_PRIVATE` | `Mapped{len, prot}` → `Unmapped` | No | **Ordinary storage.** An anonymous map is an arena whose provider is the OS. `mmap` is a `HandleFactory`-shaped acquisition; `munmap` is an ordinary consuming release. Zero additions. |
| `MAP_SHARED`, file-backed or cross-process | `Mapped{...}` | **Yes, with no edge in this program's order** | **Needs the one addition below.** |
| Device registers, DMA buffers, ring pages | — | Yes, and with ordering requirements | Today: not WF objects at all (io-model §8). |

### C.3 "The OS may change it" and the frame

| Formalism | How externally-writable state is framed |
|---|---|
| Low\*/F\* (Protzenko et al., ICFP 2017) | Explicit `modifies` clauses over location sets plus a region tree; the frame is what the modifies set omits. External interference is admitted only through **monotonic references / `witnessed` stable predicates** — facts that can never be falsified. *(Library shape unverified.)* |
| VST (Mansky, Honoré & Appel, ESOP 2020) | The outside world is a **first-order** object with its own transition relation; external-call specifications connect it to the higher-order logic, and the soundness theorem is stated against it explicitly rather than assumed. |
| Iris (Jung et al., POPL 2015; JFP 2018) | External/ghost state as resource-algebra elements under an invariant; only the invariant's **stable** consequences may be retained. |

All three converge on one answer: **facts about externally-writable storage must
be restricted to the stable (typically monotone) fragment; everything else is
re-derived by a call.** WF's minimal version of that is the degenerate case:
retain *nothing*.

### C.4 The minimal addition, and the constitutional test

> An identity may be declared **foreign**: another agent may write it without an
> edge in this program's order. The checker retains no contents fact about a
> foreign identity across any operation — every read's result comes from the
> operation's own postcondition. Its *extent* facts (live, length, layout) are
> ordinary and program-ordered, because mapping and unmapping are this program's
> calls.

Three properties make this minimal rather than an exception:

1. It changes **no** ownership, effect, or proof rule. It is a fact-retention
   policy, and the machinery — killing facts by supporting footprint — already
   exists for `writes` projection.
2. It is **not** a host-crossing distinction, and this is the constitutional
   test. The qualifier must be spelled over *interference*, not over *origin*: "a
   foreign identity is one another agent may write without an edge in this
   order". A purely internal WF object shared with another WF thread has exactly
   the same property and must be able to carry the same qualifier. Spelled that
   way, no rule distinguishes a value by whether its implementation crosses the
   host boundary. Spelled as "external" or "native", it would.
3. It composes with §B: R3's shared regions, if ever added, are foreign
   identities with a non-degenerate stable fragment. The degenerate version here
   is the first point on that scale, not a separate mechanism.

**`fd` and socket need no addition at all.** Their state machines are
program-ordered; their contents are behind calls. Only the mmap class needs it,
and today WF avoids even that by keeping such storage out of the source language.

---

## D. Minimal additions

Ordered by cost. "Runtime cost" is what executes; proofs erase.

| # | Need | Mechanism | What the checker must know | Runtime cost | Prior art | Confidence |
|---|---|---|---|---|---|---|
| 1 | Parallel width not written in source | **PAR-2 iteration-exclusive affine range loan** `[c*i+b, c*(i+1)+b)` | The same affine arithmetic it already runs for `a*i+b`, plus range-loan disjointness | **None** | DPJ index-parameterized arrays (OOPSLA 2009); Regent partitions (but Regent checks dynamically) | High. This is the catalog's own `[new]`; it removes the single most recurring cost in `CONCURRENCY-CATALOG.md` and needs no thread construct |
| 2 | Real threads for a lexically scoped fork/join | **None.** PAR-1 already is this; only the lowering changes | Nothing new | Pool fork/join edges; catalog measured 2.98x on 4 cores (75% of ideal) for the sibling-pair phase | CSL parallel rule (TCS 2007); `thread::scope`; DPJ `cobegin` | High. Catalog §12 verdict is *direct, nothing changes* |
| 3 | Conditional states crossing a join | Collapse `ite(c,·,·)` at any fork/join boundary whose condition the other side cannot name; publish the condition as an ordinary returned value if it is needed | Which captured conditions are in scope on both sides | None | No direct precedent — CSL ships one assertion, not per-identity conditional states | Medium. The conservative rule is obviously sound; whether it rejects useful programs is **untested** |
| 4 | A child that outlives its block | Linear `Task<'ids>` handle carrying the child's exit contract; join consumes it | Transferred identity set + entry states at spawn; exit states at join; handle consumed on every exit edge; join dominates every later use | A real wait | Chalice `fork tk` / `join tk` (ESOP 2009); Verus `JoinHandle` + tracked ghost (OOPSLA 2023) | Medium-high on the shape; the obligation-leak case (never joined) is **unresolved** |
| 5 | Lock over identities no caller owns | Ordinary nominal declaring a **custody set + invariant**; `acquire` → linear guard; `release` → invariant check then **erase every fact about the custody set** | Custody set; invariant at both ends; guard linearity; no custody access outside a guard extent | One mutex per critical section | CSL resource invariants (TCS 2007); Chalice monitors (ESOP 2009); Mezzo locks (TOPLAS 2016); shared session `acquire`/`release` (ICFP 2017) | High on the mechanism. **Precision of the invariant** is a known soundness side condition I have not checked against WF's fact model |
| 6 | Two lock-bearing calls actually overlapping | A **second permission level**: race freedom + invariant preservation, *without* PAR-1's source-order equality | That the only conflicting subject is the lock itself | The mutex | None with WF's determinism promise; DPJ and Regent both decline this | **This is the decision, not a derivation.** It adds an interference category CAP-1 excludes and gives up determinism. Flagged, not recommended |
| 7 | "Several holders, occasional writes", paying only at the write | Protocol/STS shared region + stability judgment + a memory model | Which facts are stable under the region's action set; the action set itself; ordering | One atomic RMW | CAP (ECOOP 2010); fictional separation (ESOP 2012); STS (ICFP 2013); Iris (POPL 2015/JFP 2018); Verus atomics (OOPSLA 2023) | **Recommend against.** A second model, not an addition. R1 (phase/epoch, catalog §8) already beats `RwLock` on the read path at zero reader cost |
| 8 | Externally-writable storage (`MAP_SHARED`, device registers, DMA) | **Foreign identity**: no contents fact retained across any operation; extent facts stay ordinary and program-ordered | That the qualifier is present. Nothing else | **None** — the load is an ordinary load; ordering is the adapter's | Low\*/F\* `modifies` + monotonic/`witnessed` (ICFP 2017); VST external state (ESOP 2020); Iris invariants | High, *given* it is spelled over interference rather than origin (§C.4). Today WF sidesteps it entirely (io-model §8) |
| 9 | fd / socket / anonymous mmap | **Nothing.** Three identities per descriptor (wrapper / description / contents) is a modelling convention, not a mechanism | Which subject each operation reads and writes | None | RESEARCH.md's own IO witness; `dup`; `design/language/system-interface.md` split direction owners | High |

### Uncertainty register

- **#3** is the only place the identity model demands something CSL does not, and
  I have no prior art for it. The collapse rule is untested against any program.
- **#5**: CSL requires resource invariants to be **precise** for the conjunction
  rule's soundness. Whether WF's fact model has an analogous side condition is
  **not established here**.
- **#4**: a spawned child that is never joined holds obligations the parent
  cannot discharge. CAP-1 puts general race conditions out of scope but says
  nothing about obligation leaks across a thread boundary. **Open.**
- Verus API names (`PointsTo`, `AtomicInvariant`, `atomic_with_ghost!`) are from
  memory and **unverified** against the current `vstd`. The papers are verified.
- Low\*'s monotonic-reference / `witnessed` library shape is **unverified**; the
  ICFP 2017 paper and its `modifies`-clause discipline are verified.
- Every performance number quoted is copied from `CONCURRENCY-CATALOG.md` and
  `compute-runtime/RESULTS.md`; none was re-measured for this note.

### Verified citations

O'Hearn, *Resources, concurrency and local reasoning*, TCS 375, 2007 · Brookes,
*A semantics for concurrent separation logic*, TCS 375, 2007 · Leino & Müller,
*A Basis for Verifying Multi-threaded Programs*, ESOP 2009 · Bocchino et al.,
*A Type and Effect System for Deterministic Parallel Java*, OOPSLA 2009 ·
Dinsdale-Young, Dodds, Gardner, Parkinson & Vafeiadis, *Concurrent Abstract
Predicates*, ECOOP 2010 · Jensen & Birkedal, *Fictional Separation Logic*, ESOP
2012 · Bauer, Treichler, Slaughter & Aiken, *Legion*, SC 2012 · Turon, Dreyer &
Birkedal, *Unifying refinement and Hoare-style reasoning…*, ICFP 2013 · Clebsch,
Drossopoulou, Blessing & McNeil, *Deny capabilities for safe, fast actors*,
AGERE! 2015 · Jung et al., *Iris*, POPL 2015 · Slaughter, Lee, Treichler, Bauer
& Aiken, *Regent*, SC 2015 · Balabonski, Pottier & Protzenko, *The Design and
Formalization of Mezzo*, TOPLAS 38(4), 2016 · Protzenko et al., *Verified
Low-Level Programming Embedded in F\**, ICFP 2017 · Balzer & Pfenning, *Manifest
Sharing with Session Types*, PACMPL 1(ICFP), 2017 · Jung, Jourdan, Krebbers &
Dreyer, *RustBelt*, POPL 2018 · Jung et al., *Iris from the ground up*, JFP 28,
2018 · Mansky, Honoré & Appel, *Connecting Higher-Order Separation Logic to a
First-Order Outside World*, ESOP 2020 · Lattuada, Hance, Cho, Brun, Subasinghe,
Zhou, Howell, Parno & Hawblitzel, *Verus*, PACMPL 7(OOPSLA1), 2023 · Hance,
Zhou, Lattuada, Achermann, Conway, Stutsman, Zellweger, Hawblitzel, Howell &
Parno, *Sharding the State Machine* (IronSync), OSDI 2023.
