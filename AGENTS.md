# Whitefoot — agent onboarding

Whitefoot is a systems language for AI-written, human-approved code. Entire bug
classes are unrepresentable (memory corruption, races, silent overflow,
uninitialized reads); there is no unsafe escape. The checker's proofs double
as optimizer facts: safety checks are always on unless a machine-verified proof
discharges them — speed is earned by proof, never by weakening a check.

## Read order

1. `THE-PLAN.md` is the sole source for roadmap and authorization.
2. The tail of
   `optimizer-language-research/implementation/decision-gates.md`.
3. The relevant live `mcts_mem/` node and its `.alt/` history before a
   non-trivial design change.
4. As needed: `CONSTITUTION.md`, `PATTERNS.md`,
   `spec/kernel-spec-v0.9.md`, and
   `optimizer-language-research/notes/user-directives.md`.

## Verify

- `make check` is always required. It checks repository structure,
  specification governance and integrity, the retained focused reference
  model, conformance data, the standalone grammar evidence, and the active Rust
  foundation. Its green result is an exact development-capability statement,
  never a release claim.
- `make -C compiler check` is also required before and after compiler work. The
  root gate incorporates it.
- A release claim uses the separate release gate defined by `THE-PLAN.md`; a
  green development gate is not a completeness claim.

## Standing rules

- English only: every new or modified repository artifact, identifier, comment,
  diagnostic, fixture, test name, document, and file or directory name uses
  English prose. Formal notation, programming-language tokens, numeric data,
  and external proper names are allowed.
- `AGENTS.md` and `CLAUDE.md` must remain byte-identical.
- The active numbered specification and evidence baseline is
  `spec/kernel-spec-v0.9.md`, SHA-256
  `bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`.
  Compiler code does not reinterpret or edit that numbered file. Exact v0.8
  remains immutable historical authority for its versioned evidence.
- Kernel-spec changes are owner-gated in advance. Present the exact delta, get
  explicit approval, create a new numbered version and update every live
  reference, then run `make approve-spec REASON="..."` to regenerate the
  guarded baseline and append the governance entry.
- Earn a specification change with independent evidence, never implementation
  convenience. A compiler/spec discrepancy stops for investigation; compiler
  behavior cannot define the language.
- Conformance source and expected verdicts, frozen oracle digests, and active
  reference-semantics tests are owner-gated. Additive tests are free; modifying,
  deleting, weakening, or regenerating protected material needs exact logged
  approval followed by `make approve-spec REASON="..."`.
- A red spec guard means stop and obtain approval. Never regenerate a baseline
  merely to make the gate green.
- The conformance corpus is implementation-independent authority. Compiler
  capability, internal errors, timeouts, verifier failures, and backend
  failures live in adapter results, not normative expectations.
- Production acceptance has one semantic kernel. The originating invocation
  must project, decode, and completely replay its canonical artifact through
  that same kernel before lowering authority can exist. There is no second
  production semantic verifier; replay checks the artifact boundary and is not
  independent semantic evidence.
- [PROG-2] gives `SourceBundle` transport exact language meaning: one ordered,
  nonempty logical-source sequence forms one flattened program root; record
  order fixes top-level declaration order, and paths never create namespaces.
- Facts that can increase optimizer authority require hostile adversarial
  review before shipment. A green gate is not a review.
- Never trade a source check for speed. Proof-elision is the only path.
- Durability: each completed step gets one commit and one append-only
  `decision-gates.md` entry.
- Keep files cohesive and reviewable. Split by invariant-bearing
  responsibility, not arbitrary line counts, corpus functions, or one-use
  forwarding modules.
- Report results in plain performance and correctness language; keep internal
  project codenames in repository logs.
- Subagent tiering: sonnet only for mechanical work, opus for most tasks, and
  top tier for subtle soundness reasoning. Never haiku.

## Layout

- `spec/` — exact language versions and derivation evidence.
- `conformance/` — compiler-independent source and expected behavior.
- `codegen-corpus/` — implementation-independent proof/code-shape premises
  and hostile near misses; its old democ runner is dormant until replaced.
- `prototype/checker/` — retained focused reference model, never compiler or
  language authority.
- `compiler/` — the active safe-Rust production compiler workspace.
- `grammar-verifier/` — separately runnable independent grammar-change
  evidence; never production compiler authority.
- `tools/` — active repository, governance, and verification tooling.
- `experiments/` — measured evidence and open development workloads.
- `optimizer-language-research/` — owner directives, decision log, design
  dossiers, and historical research evidence.
- `mcts_mem/` — current design decisions plus rejected alternatives.
- `archive/` — inert historical material. No active tool, build, test, or
  source import may read from it.

## Current authority

The owner replaced the self-host-first wfc/democ route on 2026-07-20. The old
implementations are archived and there is no disposable Rust compiler. The
owner-approved exact v0.9 specification is active; v0.8 remains immutable
history. Phases 1 through 3 are complete, including the independently checked
grammar repair, protected migration, and active-target switch. Phase 4's
canonical frontend is next under the exact gates in `THE-PLAN.md`. Later
specification changes, protected changes, release claims, and any future
self-hosting remain separately gated.

The active foundation contains `whitefoot-contract`, `whitefoot-lexer`,
`whitefoot-source-audit`, and the binary-only
`whitefoot-lexical-observer`. The observer is evidence only, and the source
audit checks exact source/specification binding only. No production parser,
semantic kernel, artifact, backend, compiler executable, or release capability
exists yet.
