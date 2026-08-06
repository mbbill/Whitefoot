# v0.19 arg_get parameter-spelling repair

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — v0.19 activated on main, 2026-08-06
- **Authority (historical):** owner authorization 2026-08-06 for the repair
  batch; exact-byte approvals recorded in `governance/APPROVALS.md`

## Outcome

v0.19 renames [SYS-2]'s `arg_get` value parameter from `index` to
`position`, repairing the v0.18 internal contradiction (a fixed GRAM-5 atom
cannot be a call-site fieldinit IDENT under FORM-3, so no complete legal
`arg_get` call existed — task-0007 finding; sole collision by systematic
sweep of every SYS-2 spelling against the 67 fixed terminals). The first
approved candidate revision omitted the machine-checked META-5
`Selection ground:` element; the integrity gate caught it at installation, a
prematurely committed red-gate attempt was dropped (a piped gate had
swallowed the failing exit code — the mcts-recorded pitfall; gates now run
unpiped before activation commits), and the corrected bytes were re-approved
and installed. Derived material in the same work: active identity constants,
the whitefoot-spec approved-candidate pin, catalog spelling + byte-exact
SYS-2 test, the coupled GRAM-11 assertion, the derivation-record amendment,
Outline revision 10.

## Evidence and validation

- `governance/APPROVALS.md` two 2026-08-06 v0.19 entries (superseding);
  installed spec SHA `01fb10d2d61cc87cce72cc98071eda98c7411fdc95af4ef29b79ac9a49cb5398`
  verified at install; grammar-preserving verifier 64/74/75; both gates green
  by unpiped exit codes; 119 rules; coverage 119/119.

## Follow-ups

- The positive `arg_get` call path is unblocked; 0014's runtime argv cases
  exercise it end-to-end once the lowering chain (0010-0012) lands.
