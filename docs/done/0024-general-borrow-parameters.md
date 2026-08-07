# 0024 — General borrow-mode parameters and let-borrows

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `18ca21a` (ff), 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2

## Outcome

Borrow-mode parameters, let-borrows, deref reads, writes through unique
holders, matching through borrowed enums, and borrow-mode results — for
scalar and enum content on the normal path, generalizing the existing
struct-borrow machinery (`BorrowAddressed`/`ReborrowAddressed`/
`DerefAddressed`; `IrType::Address` with AddressOf/Load/Store; two
predicates as the single borrowability source; a `reference_value` marker
giving TYPE-7 one rule). Reconciled with 0025's TYPE-7 fixes into ONE
mechanism (`reads_implicitly_through_holder` with a `RequiredReferent` per
position; both regression sets green) — the two holder shapes genuinely
differ in this compiler (borrows carry referent type + provenance; boxes
carry holder type), so the unified predicate takes both facts. One
attribution correction matching a protected case (SET-1's shared-borrow
referent → OWN-5). Lane: Pass 310 → 342, Skip 49 → 14; 32 of the 44 target
cases green with per-case evidence; wfgrep and cost gates untouched and
green.

## Standing items (deliberately unresolved)

- 12 protected-source asks: 3 authored `pure` rows needing `reads('r)`
  (§9.1 attribution), 6 missing region arguments (TYPE-5), carried to the
  owner bundle with 0025's 10.
- 3 returned-reborrow cases hit a REAL spec gap: OWN-6 defines child
  reborrows only in call-argument position and defers bound/returned
  children; v0.19 does not settle `&'a deref(h)` in return position —
  recorded for the next spec batch, not invented.

## Evidence and validation

- Landed commits: `67469e7`/`03ee2e8`/`b854ec8`/`18ca21a`. Both gates green
  by unpiped exit codes; 449 lib tests; coverage 119/119; obligation-
  discharge (owner session) untouched.

> Correction (2026-08-07, task 0027): the "12 protected-source asks" label above is a closure-compression error; the asks itemized and approved were 9 (three EFF-2 `reads` rows, six TYPE-5 region arguments). The other three of the 12 are the OWN-6 returned-reborrow gap cases, which are spec-gap candidates, not source asks. Per-case enumeration recovered from the pre-closure record at 18ca21a; see docs/done/0027-protected-amendments.md.
