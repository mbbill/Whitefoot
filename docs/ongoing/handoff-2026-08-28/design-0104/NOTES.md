# 0104 proof-first design (agent: proof-first angle)

Task: write $SCRATCH/wf-0104-design/proof-first.md
Part A streaming chunk loop pipelining; Part B bytes->path.
READ ONLY the repo. Base ref: integration/2026-08-28b @ 16228216.
Source snapshots dumped in ./src/ (git show <ref>:<path>) -- line numbers
MATCH the branch content, cite as <repo path>:<line>.

## STEP LOG
- [x] step 0: dumped sources. worktree clean, HEAD b2e2e267 (not the base); read via git show.
- [ ] step 1: read FIRST-PRINCIPLES.md
- [ ] step 2: spec sections PAR-1..3, SYS-7..14, PATH-1, OWN-5..7, STOR-1
- [ ] step 3: LOOP-PIPELINE.md
- [ ] step 4: staged_permission.rs
- [ ] step 5: patterns P15-P17, done/0098, done/0100, writer programs
- [ ] step 6: write proof-first.md Part A
- [ ] step 7: Part B
- [x] step 1: base rev files exported to $SCRATCH/wf-0104-design/base/ from 16228216. CITE LINE NUMBERS FROM base/ COPIES (== 16228216).
- [x] step 2: read FIRST-PRINCIPLES.md (1371 ln). key: sec 9.1 permit ceremony 571-591; sec 13 completion=ownership transfer 815-888; sec 16 runtime rules 960-993; sec 23 open Qs 1298-1323.
- [x] step 3: spec read: PAR-1 1992-2024, PAR-2 2026-2051, PAR-3 2053-2082 (c=first may-suspend 2056; exits in P 2057; edge on submission outcome is E 2058; replication 2067-2070; resource footprint 2072). SYS-7 2522, SYS-8 2537-2576, SYS-10 2602-2622, SYS-11 2624-2633, SYS-12 2635, SYS-14 2663-2692 (name=one component 2676; no op turns name into HostString/RelativePath 2680-2681). PATH-1 2385-2392, PATH-2 2394-2400. HOST-1 2359, HOST-3 2375-2383. OWN-5 548-574, OWN-6 576-590, OWN-7 592-596, OWN-11 609-611. STOR-1 638-646. QUAL-1..3 2402-2430.
- [x] step 1: read AGENTS.md, FIRST-PRINCIPLES.md, PAR-1..3, HOST-1..3, PATH-1..2,
      QUAL-1..3, SYS-4..14. Spec dumped to $SCRATCH/wf-0104-design/spec.md
      (== `git show 16228216:spec/kernel-spec.md`), so spec.md line numbers == repo line numbers.
      KEY LINES: PAR-3 head 2053; c definition 2056; "Every edge that leaves B" 2057;
      "An edge the statement performing c takes" 2058; replication clause ~2073;
      SYS-8 2537; "A short success is not end of input" 2556; "Repetition..." 2557;
      PATH-1 2385-2392; PATH-2 2394; SYS-14 2663; "A name is one path component" 2676;
      "This specification declares no operation turning an enumerated name" 2680; 2681.
- [x] step 4: read LOOP-PIPELINE.md fully. Key cites (base/research/.../LOOP-PIPELINE.md): mechanism 107-132; four dispositions 340-356; must-refuse programs 382-470; iteration-privacy 472-559; early exits 579-605; host-resource hole 636-675; K window 681-719; ring/restore 742-787; in-order commit 789-835; compute lane 837-873; two latent bugs 875-904; runtime needs 941-979; spec delta 995-1170; falsifier 1298-1357; cost 1396-1424; open Qs 1559-1613; probe corrections 1794-1839 (F2 restated: narrow submits 8192 reads today, open count is 0; --no-overlap still uses ring; REQUIRED 119.47 bar no longer discriminates).
- [x] step 1-2: read FIRST-PRINCIPLES (skim) + spec anchors. KEY LINES (spec/kernel-spec.md):
  PAR-3 = 2053..2082 (already exists on base!). 2056 cut c; 2057 exits in P;
  2058 "An edge the statement performing c takes on the outcome of that submission ...
  is an edge of E and not of P" <- THE WALL for Part A break-on-ReadEnd.
  2063 prologues do not overlap; 2067 replicate; 2069 "a contract stating only which
  bytes ... may have changed [SYS-8] establishes no written byte" -> SYS-8:2565 still
  says "may have changed", so replication is currently UNUSABLE. 2070 unresolved extent.
  2072 host resources retire-and-retry.
  SYS-8 = 2537..2576; 2551 at most one progress-producing attempt; 2554 read_at positioned,
  ReadBytes only for next>start; 2556 "A short success is not end of input"; 2564 start<=next<=end
  via ENT-3.S10; 2565/2566 change sets; 2570 sanitized count.
  SYS-10 = 2602..2622 (permit ceremony; 2603 factory loan ends inline; 2605 permit one attempt,
  no descriptor promise).  SYS-14 = 2663..2690; 2676 "A name is one path component";
  2680 "declares no operation turning an enumerated name into a HostString or a RelativePath";
  2681 open_* take a caller-owned name range; 2683 component validation -> InvalidPath;
  2687 "a symbolic link is not followed by either operation".
  PATH-1 = 2385..2392 (2387 no NUL + no target-root prefix; 2389 retype inline lease, no alloc;
  2390 no normalization; 2391 component/absolute/join DEFERRED).
  PATH-2 = 2394..2400 (2396 process-equivalent, may lie outside; 2397 "not a confinement claim";
  2399 confined type DEFERRED).
  HOST-3 = 2375..2383; 2381 "A producer whose backing is not command-lifetime yields no value
  of this type: it introduces a distinct owned-backing string resource" <- Part B door.
  OWN-5 548..574, OWN-7 592..596, STOR-1 638..646, STOR-3 651..660 (656 for-body edges).
- [x] step 2: read LOOP-PIPELINE.md (dumped locally), staged_permission.rs (dumped locally).
      PAR-3 shipped as its own rule (design proposed amending PAR-2; owner chose +1 rule).
      Ledger format (driver.rs:1644-1680 test):
        "PAR stage  <file>:<line>  for   permitted   staged at <call>; N places classified"
        "PAR place  <file>:<line>  <disposition>  <citation>  <reason>"
      Dispositions: read-only | serialized-P | serialized-E | replicated | denied.
      EXIT_IN_REMAINDER / EXIT_SELECTED_BY_SUBMISSION at staged_permission.rs:365,366.
- [x] step 5: read staged_permission.rs (conditions 1-7 at 36-95; cut/segment code 1036-1062; exit classification 1099-1113; EXIT_* at 365-366). ledger format permission_ledger.rs:238,251,494.
- [x] step 6: writer programs at $SCRATCH/wf-0100-verify/writer/work/: sizes.wf:74-93 = the @slurp chunk loop (Part A), sizes.wf:117 = open_file(name:&content,start:begin,end:index) = the Part B wall; list.txt has sample/a.txt, flat.txt is the workaround. largest.wf:55,78,91,198 = buffer_vacant<DirectoryRead>+replace descent.
- [x] step 7: KEY FACTS. open_read has NO descriptor-status inspection in spec (only open_file, spec:2685) -> ReadFile may be a FIFO. file_adapter.c:172-190 already calls openat with a whole path (multi-component costs runtime ZERO). macOS SDK sys/fcntl.h:158 O_NOFOLLOW_ANY 0x20000000, :128 O_RESOLVE_BENEATH 0x1000 (comment says "only for open(2)"). IoError has NotDirectory + CrossDevice, no ELOOP class. HOST-3:2381 already names the answer for Part B (distinct owned-backing type) but B-plain needs no type at all. 0100 doc:755-765 = W4 (Part A) and :787-801 = B6 (Part B) already recorded as owner open points.
- [ ] step 8: write runtime-first.md
- [x] steps 3-5 done. Extra verified facts:
  * read_at lowers to `pread` (emitter/system.rs:409,1440) and IORING_OP_READ with
    explicit `off` (linux_io_uring.c:512,515). BOTH POSITIONED.
  * ESPIPE (29) is in NEITHER errno class table (qualification.rs LINUX_ERROR_CLASSES /
    DARWIN_ERROR_CLASSES) -> a pread on a pipe reaches source as IoError::Other.
    So read_at cannot read a pipe today AT ALL. Kills the pipe hazard for speculation.
  * ELOOP: Linux 40, Darwin 62 -> InvalidPath. ENAMETOOLONG: Linux 36, Darwin 63 ->
    InvalidPath. EMFILE/ENFILE 24/23 -> ResourceExhausted. No new IoError class needed.
  * open flags already carry O_NOFOLLOW: qualification.rs:917,948,966;
    component_directory_open_flags / component_file_open_flags. root_prefix = b'/'.
  * component_validation() emitter/system.rs:1723-1755 (nonempty, <=limit, no NUL,
    no separator) -> invalid_component() :1704 yields InvalidPath(0,0).
  * ledger formats: permission_ledger.rs:238 "PAR stage" / :251 "PAR place".
  * writer program $SCRATCH/wf-0100-verify/writer/work/sizes.wf
    already passes a name RANGE out of a read buffer to open_file (line ~117), so the
    ONE-component case already works; only separators fail. largest.wf:55,78,91,173,198
    is the buffer_vacant<DirectoryRead>+replace descent.
- [ ] step 6-7: write proof-first.md
- [x] step 3: read patterns P15-P17, 0098, 0100, the third writer's work/ (sizes.wf,
      largest.wf), read_heavy_narrow.wf, RESULTS.md read-heavy tables, qualification.rs
      errno tables.
      DECISIVE FACTS FOUND:
      * ELOOP already maps to InvalidPath on BOTH families (qualification.rs:290 Darwin
        InvalidPath=[62,63]; :328 Linux InvalidPath=[36,40]). So a no-follow refusal needs
        NO new IoError class. 9 files match IoError exhaustively (5 conformance cases +
        wfgrep.wf + 3 research shapes) -> a new class would cost those 9.
      * Falsifier bar for Part A: RESULTS.md read-heavy 4 KiB uncached Linux:
        C.narrow.default 3058.12 ms, C.wide8.default 1463.43, N.uring32 1459.84,
        N.direct 2993.34. 64 KiB cold: C.narrow 1664.13, C.wide8 1228.53, N.uring8 1274.99.
      * The writer's count_file chunk loop is denied by THREE walls: cond 2 (ReadEnd break
        in E), cond 5 carried cursor ("the body reaches it on both sides of the cut",
        staged_permission.rs:1618), cond 3 hoisted scratch.
      * disposition_of at staged_permission.rs:1565 -- 4 dispositions, no carried datum.
      * SYS-2 declaration record count 203 = 18 types + 40 ctors + 63 fields + 16 ops
        + 22 region params + 44 value params.
- [ ] step 4: write writer-first.md.
- [ ] runtime-first copy #2 (14:5x): runtime-first.md exists, 1105 lines, ENDS MID-PART-B
      (last heading B.6 at 1067, text references a B.7 that does not exist).
      Remaining: B.7 confinement/permit composition, B.8 lowering, B.9 ledger,
      B.10 safety, B.11 errno map, B.12 oracle+falsifier+tests, B.13 cost,
      B.14 open questions. Do NOT rewrite Parts A/B.0-B.6.
- [ ] writer-first RESUME (agent copy #2, 15:0x): writer-first.md is 244 lines, ends
      mid-Part-A at A.3.3 (open_read status inspection). DO NOT rewrite A.0-A.3.3.
      Remaining: A.4 judgment conditions, A.5 lowering, A.6 ledger line, A.7 safety,
      A.8 oracle+falsifier, A.9 what the writer writes differently, A.10 cost,
      A.11 open questions; then Part B B.0-B.12 (writer example, spec sentences,
      judgment, lowering, ledger, safety, errno, tests, cost, open Qs).
- [ ] proof-first copy #2 (14:5x): proof-first.md exists, 1229 lines, Part A COMPLETE
      (A.0-A.12), Part B complete through B.7 (safety argument). MISSING and to append:
      B.8 ledger line, B.9 differential oracle + falsifier + tests, B.10 cost,
      B.11 open questions for the owner. Do NOT rewrite Part A or B.0-B.7.
