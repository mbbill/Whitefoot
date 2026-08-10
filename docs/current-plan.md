# Current Plan

Status: PROPOSED (awaiting owner selection, 2026-08-10): activate the held
provenance gate as stage 5b of the selected obligation-discharge direction.
This proposal authorizes no execution. If selected, a separate change must mark
it ACTIVE before any successor task is registered or substantive work begins.

Derived from: [Direction Outline revision 29](roadmap.md), item `PROOF-8`
(primary), with `BOUND-1`, `VERIFY-1`, and `VERIFY-2` as boundary and evidence
constraints. `CAND-8` remains the selected flagship but stays parked until the
complete obligation-discharge direction reaches its completion boundary.

Before any specification approval request, the lead must first give the owner
a complete plain-language Chinese explanation of the exact language behavior,
implementation, protected and accepted-set impact, real-program result,
archive action, limitations, and complete digest; then stop and wait for an
explicit response.

## Direction and proposed milestone

Exact-approved v0.26 is active at `spec/kernel-spec.md`, SHA-256
`18aa00e307642e608f2a3406642db9980dd3620291a7e434985e20a65eb0e476`.
Task 0048 made each admitted function requirement one atomic typed goal:
ordinary calls prove it before transfer, bodies receive it as S4, ordinary
callees execute no requirement prologue, and both real process entries retain
one checked boundary. Installed acceptance and the complete gate are green.

The compiler also retains finite provenance metadata but emits no provenance
rejection. Task 0046's held review reaches all three canonical Huffman subjects
with a finite explicit-dataflow rule. v0.26 closes the remaining helper-shaped
`requires` bypass and records the requirement-to-protected-leaf bridge needed
to activate that rule without inventing a second goal language.

## Proposed current step — stage 5b provenance-gate activation

### Why

The current named claims distinguish machine-proven obligations from checked
writer assertions, but they do not distinguish a local invariant failure from
malformed external input. In the frozen boundary-fed DEFLATE unit, the held
rule classifies nineteen of thirty-three protected subjects as external. Six
already discharge without assertion evidence; thirteen obligation nodes under
eleven claims instead turn externally controlled failure into an abort.

Task 0046 fixed the held rule's explicit-offset and payload-projection defects
without adding implicit-flow analysis. Stage 7 then made a function requirement
an ordinary caller proof obligation and retained finite bridge metadata, so a
helper can no longer hide the same protected leaf behind a runtime callee
prologue. The smallest next step is therefore to activate the already bounded
explicit-dataflow policy and migrate only the real externally controlled
failures it identifies to value paths.

### Do

1. Re-derive the candidate from current authority, not historical wording.
   Use active v0.26, task 0046's held design review, and the current v0.26
   requirement bridge as inputs. The held candidate in
   `governance/spec-evolution/provenance-gate-candidate.md` remains evidence,
   not specification text, and its v0.24-era anchors must never be fuzzy-patched
   into the stable file.

2. Freeze the current consumer before changing semantics. The compilation-unit
   order and SHA-256 identities are:

   - `raw_deflate.wf` —
     `c8fa0d58301e5346041c1886eaa3e277f9d3926212b6a5420e52b22eada300f0`;
   - `raw_deflate_dynamic.wf` —
     `cca35bbd3c5985c1e6753e0b0ca5311be7287d2021c01b46f14506b06734fcee`;
   - `raw_deflate_dynamic_decode.wf` —
     `03bab2ab19d9087bdd4fc3edebb060499a54d388623cab695f8bbdc10cd0ac9c`;
   - `raw_deflate_boundary.wf` —
     `3fbd1281b1e9f4f9a161cf7d846622ae277611eaf9d34ce3ba576f3a81d140c4`.

   Freeze the current conformance manifest SHA-256
   `65393c118817f207ef268a35d8b67931409b30c7c04ea3a2f2ffc7c41b80c73a`,
   all 407 existing case identities and rows, and all 30 rule-coverage
   annotations. Additive PRV cases are allowed; changing any existing case or
   manifest field stops for exact protected review.

3. Draft the smallest complete v0.27 provenance judgment at the stable
   `spec/kernel-spec.md` path. The candidate may add only the closed PRV-1,
   PRV-2, and PRV-3 rules needed here. It adds no token, terminal, grammar
   production, source construct, operation row, trusted assertion, optimizer
   assumption, or writer-spelled provenance annotation.

4. PRV-1 is one finite two-point explicit-dataflow classification:

   - process-entry inputs are external, while system results and writes follow
     one closed component table: `args_count` and `host_bytes_len` are
     external; both `arg_get` payloads are external; `host_copy_bytes` has an
     internal `Ok(value:)`, external `Err(error:)`, and external
     `destination`; both `host_utf8_len` payloads are external;
     `host_copy_utf8` has an internal `Ok(value:)`, external `Err(error:)`, and
     external `destination`; both `relative_path` and both `open_read`
     payloads are external; `read_once` has an internal `ReadBytes(count:)`,
     external `ReadFailed(error:)`, and external `destination` and `file`;
     `write_once` has an internal `Ok(value:)`, external `Err(error:)`, and
     external `output`; and `exit_status` is internal. No unlisted component
     inherits an external class by association;
   - direct enum payload projections are tracked separately and nested payload
     joins conservatively seed every direct projection;
   - storage is per binding and per whole root, flow-insensitive and monotone;
   - a place read joins its root and every explicit subscript-offset operand,
     field selection preserves that class, and `len` remains internal;
   - checked-operation results, propagation, returns, user-call results, and
     write components compose through one finite least fixed point; and
   - branch/control dependence, write-address dependence, path-sensitive
     storage, recursive payload paths, and implicit-flow analysis are absent.

5. PRV-2 retains finite parameter-datum, result, write, and concrete
   protected-leaf identities plus deterministic witnesses chosen only after
   convergence. Compose them with v0.26's requirement occurrences and
   subject-only bridges. Do not replace exact goal identity with a recognizer,
   mention-all-parameters rule, whole-goal support rule, or a second proof
   language.

6. PRV-3 gates only the constrained subject of a protected obligation:

   - an internal subject keeps the existing entailment judgment;
   - an external subject must discharge in the unasserted state with S2/S3
     removed, so a preceding `check` or `claim` cannot authorize it;
   - a real value branch may establish the needed fact and pass;
   - an external value used only as a bound, base, or unrelated goal operand is
     not the constrained subject and does not trigger rejection; and
   - call-site gating follows the v0.26 bridge fixed point: an external actual
     protecting a downstream leaf requires the complete instantiated atomic
     goal to discharge in the caller's unasserted state. Two-hop and recursive
     bridges converge; a seedless cycle remains empty.

7. Treat real process entries explicitly. Command inputs are PRV-1 external.
   The compiler-owned entry check and the body's S4 axiom may not launder an
   external bridged protected leaf: that definition is checked with the
   retained S4-blinded entry rewalk and must use a value branch in the body.
   Entry requirements unrelated to a protected leaf retain v0.26's exactly-once
   wrapper behavior. A source call to the entry follows ordinary call-site
   gating. Do not implement or simulate a foreign adapter; the existing GATE
   boundary remains unsupported.

8. Implement the judgment as one ordinary safe-Rust semantic path over the
   checked metadata already installed in v0.26. Source acceptance and
   diagnostics consume the finite fixed point directly. Facts-on and facts-off
   compilation have identical acceptance and required runtime behavior. Do not
   special-case a project, function, claim name, source path, or test identity.

9. Migrate only the frozen real failures the gate identifies, with no error
   choice left to the executor. Remove the eleven gated claim declarations and
   map them exactly as follows:

   - `stored_header_zero_in_input`, `stored_header_one_in_input`,
     `stored_header_two_in_input`, `stored_header_three_in_input`, and
     `stored_copy_in_input` take the existing `Truncated` value path;
   - `length_symbol_in_tables` takes `InvalidHuffmanCode`;
   - `match_copy_in_history` takes `InvalidDistance`;
   - `order_slot_in_offsets`, `destination_in_symbols`, and
     `ordered_in_symbols` take `InvalidHuffmanTree`; and
   - `distance_position_in_lengths` takes `InvalidHuffmanTree` inside a changed
     `store_dynamic_length -> Result<unit, InflateError>`. Its three call sites
     in `decode_dynamic` use ordinary `propagate` bindings and do not duplicate
     the guard or move it into another requirement.

   The helper's exact effect row loses only the former claim-derived `traps`
   contribution and remains `reads('d), writes('l 'd)`; `decode_dynamic` keeps
   its independently justified row, and all new normal/error cleanup edges are
   checked explicitly. Preserve every other effect judgment, every successful
   output, and every stock/boundary/truncated/malformed/oversize/closed-output
   oracle.

10. Update specification-derived data, diagnostics, conformance coverage,
    compiler documentation, writer guidance, the Direction Outline, and design
    memory in the same activated slice. The MCTS tree currently records held
    metadata with no rejection; activation must make its live Items truthful
    and preserve any real superseded alternative through the skill workflow.

11. Follow the complete specification workflow. Prepare the v0.27 candidate at
    `spec/kernel-spec.md` and, in the same uncommitted reviewable change before
    approval, create `spec/kernel-spec-v0.26.md` as a byte-identical copy of the
    outgoing stable bytes. Hash and independently review both files, failing if
    the archive path is already occupied. The prepared archive and candidate
    remain non-authoritative and uncommitted until exact approval; an approved
    atomic activation lands the archive and stable candidate together.
    Independently review normative closure, accepted-set impact, compiler
    implementation, protected boundaries, real-program migration, derivation,
    and active pins. Then give the owner the required Chinese explanation and
    exact digest, stop, and wait. No approval-chain or activation-state byte is
    written before that response.

### Verify and accept

- Before source migration, reproduce the held v0.26 matrix exactly: 33
  protected obligations and 23 claim declarations; 19 external subjects; six
  unasserted-state discharges; 13 rejected obligation nodes under eleven
  claims; 14 internal subjects; canonical Huffman result 3/3; and diagnostic
  projection 14 rejecting calls / 24 external actual atoms.
- Preserve the frozen negative boundary controls: wfgrep 0/8 gated claims;
  `run-sysfile-multichunk` 0/4; and each too-small/invalid copy control 0/1.
- After the value-path migration, all-claims-blinded acceptance must be UTF-8
  `33/22/11/0`, SHA-256 `9/9/0/0`, complete DEFLATE `29/24/5/0`, and dynamic
  DEFLATE `24/19/5/0` in
  `total/proven/claim-supported/baseline-undischarged` order. The boundary-fed
  unit has 12 remaining claim declarations. Thirteen formerly claim-supported
  sites are authorized by real branches, never a retained prologue or hidden
  assertion.
- Exercise external+branch accept, external+check reject,
  external+claim reject, internal+claim accept, external-only-bound accept,
  allocation-equality call accept, nonexact-goal reject, direct/two-hop/
  recursive/mutual/seedless bridges, payload sibling isolation, read-offset
  propagation, the retained control/write-address non-propagation boundary,
  and an entry external bridged-requirement rejection.
- Keep every pre-existing protected case byte and manifest field unchanged.
  New PRV cases and coverage rows are additive. The adapter baseline remains
  `Pass=393 Fail=1 Skip=13` for existing identities, with only the retained
  OWN-3 unsupported failure; additive cases report their own dispositions and
  never hide that baseline.
- Verify previous-to-candidate and archive-to-stable native grammar paths,
  generated tables, specification integrity, exact diagnostics, focused
  semantic/lowering/backend tests, facts-on/facts-off equivalence, the complete
  adapter, the frozen four-source consumer, `make -C compiler check`,
  `make check`, and MCTS lint.
- Acceptance requires exact owner approval and atomic v0.27 activation,
  installed reruns of every frozen matrix and runtime oracle, no unreviewed
  protected drift, a green complete gate, truthful design memory, and terminal
  task closure before stage 8a begins.

### Stop condition

Stop with the smallest reproducer if the current v0.26 sources do not reproduce
the frozen 19/6/13-under-11/14, 3/3, and 14/24 matrices; if process-entry
gating needs a new source surface or error protocol; if correct classification
requires control-flow taint, write-address taint, path-sensitive storage,
recursive payload paths, Boolean decomposition, general induction, or a new
theorem prover; if the eleven real repairs cannot use the named existing value
paths and one `Result` propagation; or if an existing protected case, verdict,
rule list, status, documentation field, or runtime behavior must change. Return
that evidence for owner disposition rather than expanding the gate, weakening a
test, retaining a hidden assertion, or skipping ahead.

### Approval and task boundary

This proposal becomes executable authority only after the owner explicitly
selects it and a separate commit changes its status to ACTIVE without expanding
the written scope. Only then may a separate lifecycle commit register the next
free task number after refreshing the integration branch. The task registration
precedes substantive work. Plan selection is not specification approval; the
later v0.27 candidate still requires its own exact explanation, hard wait, and
owner response.

## Later dependency map — not execution authority

### 8a — postcondition proof-feasibility prerequisites

Freeze the smallest fact sources required by the two real `ensures` examples.
`read_bits` needs a verified mask/bitwise bound and outcome-sensitive normal
result; `append_slice` needs a fact connecting its loop-carried result to
capacity. If a small structural rule cannot establish them without general
induction or arithmetic entailment, return the blocker instead of hiding a new
proof engine inside postconditions.

### 8b — `ensures`

After 8a selects a fact fragment, add the smallest postcondition language that
exposes only verified normal-return facts to callers. Exercise branches, early
exit, cleanup, generics, unsupported forms, and false postconditions; the real
`read_bits` and `append_slice` obligations must discharge normally.

### 9a — deterministic claim ledger

Generate a deterministic read-only checked-program report for every remaining
named claim: obligation, provenance, justification, and stable source identity.
Clean builds reproduce its order and counts. Tooling precedes any language
marker.

### 9b — opt-in `deny-claims` partition

Design and implement the marker from ledger evidence. Its meaning is
transitive across ordinary calls and real generated adapters and explicitly
covers direct `claim`, ordinary trapping `check`, and callees that can claim.
Ordinary code keeps the existing lifecycle; the strict partition requires each
covered obligation to prove or take a value branch. This is not global law.

## Stable specification rule

The active specification stays at `spec/kernel-spec.md`. v0.25 is the current
immutable outgoing archive and v0.26 has no committed versioned sibling while
active. Every later candidate edits the stable file and is reviewed as a diff
plus complete digest. Its outgoing flat archive is prepared byte-identically in
the same uncommitted preapproval change, hashed and reviewed with the candidate,
and rejected if that path already exists. Exact-approved atomic activation
lands that prepared archive and installs the approved bytes at the stable path.
Archived specifications are never edited, renamed, or deleted.

## Cross-stage invariants

- One normal semantic and lowering path; no program-, corpus-, function-, or
  test-shaped behavior.
- A fact widens discharge only when normative entailment derives it. Required
  checks remain unless proof discharges their exact obligation.
- Expected or externally caused failure is a value path; a claim is reserved
  for a broken program invariant and remains an executed runtime check.
- Protected expectations never change without explicit owner approval.
  Unsupported capability never becomes source rejection.
- Each activated slice restores the complete gate and reruns the real consumer
  that earned it before the next slice begins.
- Durable decisions and rejected alternatives stay synchronized through the
  `mcts-mem-use` workflow; task records carry progress, not authority.

## Direction completion boundary

Wfgrep remains parked until stages 5b, 8a, 8b, 9a, and 9b are implemented end
to end, covered by positive, negative, near-miss, and invalidation evidence,
exercised by their named real programs, and recorded in the outline; the
complete repository gate is green; and remaining claims and unsupported gaps
are reported honestly. A reproduced prerequisite blocker returned for owner
disposition is the only earlier stop.

O11 Boolean-composition precision, general loop induction, arithmetic-term
entailment or arithmetic-mode dissolution, struct/witness invariants, the
OWN-3 predicate widening, move-on-copy generic policy, and further wfgrep work
remain outside this proposal.
