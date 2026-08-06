# 0015 — Sequential wfgrep

Frozen coordination history. This record reports how the task was carried
out; it is not authority.

- **Status:** `DONE` — landed on main at `501cf1e`, 2026-08-06
- **Authority (historical):** `ACTIVE` `docs/current-plan.md` Work item 2,
  wave 7

## Outcome

The first real Whitefoot program: `tests/programs/wfgrep.wf` (744 lines,
five declarations), following dossier §10.1 step for step — four standard
inputs, raw-byte pattern via `host_copy_bytes`, one reusable input buffer
with boundary carry in ordinary storage, matching over only the returned
prefix, one reusable output batch with a write-until-accepted loop, exit
0/1/2, `grep -h -F` frozen semantics. Oracle harness
(`compiler/tests/programs/wfgrep.rs`, 9 cases) with an in-harness trusted
reference (BSD/GNU grep disagree on unterminated lines and non-text
patterns); corpus covers empty/short/unterminated/exact-buffer/multichunk/
boundary-straddling/non-UTF-8 content and patterns/multi-file/many-flush/
line-too-long; the three OS-mechanism cases pass, with broken pipe
discriminating recoverable `BrokenPipe` from SIGPIPE death. No reachable
trap on hostile input. Programs binary 18 → 27 cases; both gates green by
unpiped exit codes.

## Language findings (the deliverable)

1. **Unsupported specified capability, top implementation gap:** a
   borrow-mode parameter of a system nominal type stops as
   `RegionsAndBorrows` unsupported (`semantic/check/types.rs` admits
   non-own parameters only for buffer/slice/struct/box; v0.19 restricts
   nothing). Consequence: no helper can touch a system value — every system
   call and the five copies of the write-until-accepted loop sit inline in
   a ~500-line `main`. Candidate for the next plan; correctly not absorbed
   here.
2. **ERR-2's price, measured:** one two-class diagnostic distinction costs
   a complete 30-arm `IoError` match — `io_class` is 95 lines, 13% of the
   program (the decision-3 trade, now witnessed).
3. **Language gap witness (feeds STORE-2's reopening input, not a blocker
   here):** a growable line buffer is inexpressible — `buffer<T>` has no
   in-place growth and STOR-1 rejects rebinding an affine place — so
   wfgrep has a fixed maximum line length and an honest `line too long`
   error where real grep grows its buffer. First-slice scope held by
   design; recorded in the outline at STORE-2.
4. **Emitted shape (informal; 0016 owns the gate):** 4 allocations per
   invocation (calloc-folded), one openat/read site, one write site per
   write_once, no memcpy libcall, the newline scan recognized as memchr —
   nothing per file, line, or match.
5. APFS refuses non-UTF-8 names (EILSEQ), so the lossless path route is
   witnessed portably through a nonexistent path whose exact bytes survive
   the full chain.
6. Awkward-but-legal: no byte literals → 13 hand-written array constants;
   function-unique region names → ~25 spellings in `main`; the ReadEnd
   trailing line served by synthesizing `\n` into the input buffer.

## Evidence and validation

- Landed commits: `2c22d11` (claim), `501cf1e` (program + oracle).
- Harness additions (`CompiledProgram` argv-as-raw-bytes/cwd/closed-stdout,
  `FixtureDirectory` byte-named files and symlinks) are shared with 0014
  per the recorded cross-link.
