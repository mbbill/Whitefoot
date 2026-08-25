<!-- Adversarial review of research/investigations/io-model/DESIGN.md revision 1,
     2026-08-25, sol-ultra agent "spec-sweeper". Checked in as evidence behind revision 2;
     paths sanitized to repo-relative and <scratch-root> forms. Findings were
     re-verified by the lead before adoption; §3f's original disposition in the
     runtime and sweep reports was OVERRULED by constitution T3 (see DESIGN.md). -->

# Per-sentence specification sweep

Scope: case-insensitive word-boundary grep found 136 occurrences of `external` and 31 of `blocks` on 117 physical lines in [spec/kernel-spec.md](/spec/kernel-spec.md). Every hit appears below. `pure` sentences whose meaning depends on those atoms follow the main sweep.

Status codes:

- `OK`: mechanical world-region rewrite exists.
- `RESISTS`: the rewrite needs a missing semantic rule identified in the holes section.
- `PROV`: `external` means input provenance, not an effect. It cannot be rewritten as a world region.
- `HOMONYM`: ordinary English or grammar terminology, unrelated to the effect atoms.

## Effect, ordering, trap, entry, and system-operation sentences

| Line | Verbatim specification text | World-region rewrite / result |
|---|---|---|
| K30 | “Failure of the external output sink may terminate the process before that explicit abort, but it never permits execution to continue.” | “Failure of the TCB-owned diagnostic world channel may terminate the process before that explicit abort, but it never permits execution to continue.” `RESISTS`: the diagnostic channel must be distinguished from source capability regions and serialized against in-flight source output. |
| K672 | “That action may perform one host call, and it carries exactly the effect row that contract fixes, which may include `external` and, where the contract permits synchronous waiting, `blocks`.” | “That action may perform one host call; its instantiated row may read or write the released capability’s world region, and its TCB metadata separately records whether its implementation may wait synchronously.” `RESISTS`: §3 assigns `blocks` metadata only to system operations, but releases also block. |
| K1089 | “After each signature's independently applicable EFF-1 judgment and the bound function declaration's EFF-2 judgment succeed, an effect row normalizes to six capabilities: the set of declared read regions, the set of declared write regions, the allocation set whose members are `heap` and each `arena` region, the presence or absence of `external`, the presence or absence of `blocks`, and the presence or absence of `traps`; `pure` is six empty capabilities.” | “...normalizes to four categories: kinded read-region set, kinded write-region set, allocation set, and `traps`; `pure` has all four empty.” `RESISTS`: memory regions and world regions need distinct kinds. |
| K1092 | “`external` and `blocks` are compared by presence exactly as `traps` is, and a `fn_sig` member may declare either.” | Delete. Contract equality instead compares kinded read/write region sets, allocation sets, and `traps`; blocking metadata is not callable-row data. |
| K1184 | “Its written effect row is any subset of `allocates(heap)`, `external`, `blocks`, and `traps`, in [EFF-1] canonical order; `pure` is the empty subset and no region-bearing effect is admitted.” | “Its row may contain `allocates(heap)`, effects over the world regions supplied with its selected command capabilities, and `traps`; `pure` is empty.” `RESISTS`: main currently declares no region parameters, so those regions have no legal binder. |
| K1208 | “The one canonical byte sequence for a complete four-input entry header is `command fn main(command.args as args: own Args, command.cwd as cwd: own DirectoryRead, command.stdout as out: own Output, command.stderr as err: own Output) -> status: own ExitStatus allocates(heap), external, blocks, traps {`.” | No complete rewrite exists until the entry contract defines the binders and alias relation for `DirectoryRead<'…>` and the two `Output<'…>` values. `RESISTS`. |
| K1343 | “[EFF-1] Row grammar: the `effects` and `effect` productions of the fence below, in exactly this canonical order (reads, writes, allocates, external, blocks, traps).” | “...in exactly this canonical order (reads, writes, allocates, traps).” |
| K1347 | `effect := "reads" "(" REGIONID+ ")" \| "writes" "(" REGIONID+ ")" \| "allocates" "(" ("heap" \| "arena" REGIONID)+ ")" \| "external" \| "blocks" \| "traps"` | `effect := "reads" "(" REGIONID+ ")" \| "writes" "(" REGIONID+ ")" \| "allocates" "(" ("heap" \| "arena" REGIONID)+ ")" \| "traps"`; the referenced REGIONIDs must be kind-checked. |
| K1351 | “`pure` is the unique spelling of the empty row and therefore excludes `external` and `blocks` exactly as it excludes every other category.” | “`pure` is the unique spelling of the empty row and therefore excludes every memory-region or world-region read/write, every allocation, and `traps`.” |
| K1356 | “`external` states that the call may observe or change state outside ordinary Whitefoot memory, including file contents, cursors, output, host namespaces, clock and random sequences, resource lifetime, and compiler-derived resource release [STOR-3].” | “A `reads('w)` or `writes('w)` entry over a world-kind region states that the call may observe or change the outside state represented by `'w`, including the applicable content, cursor, output, namespace, sequence, or resource-lifetime facet.” `RESISTS`: a single capability identity is insufficient for several of these facets. |
| K1357 | “`blocks` states that an ordinary call may block its current host thread.” | “Whether a target action may block a host thread is trusted lowering metadata and is not a source effect.” `RESISTS`: metadata must cover operations, releases, and transitive wrappers. |
| K1358 | “Both are payload-free: neither takes a REGIONID, resource name, family name, or any other argument, and `external(cwd)`, `changes(file)`, and every other resource-parameterized effect spelling is outside this grammar and outside this specification.” | “World effects use the existing `reads(REGIONID+)` and `writes(REGIONID+)` payloads; resource names and family names remain inadmissible.” `RESISTS`: the design must define world-region binders and kinds. |
| K1361 | “`external` and `blocks` are exact fixed grammar atoms and are therefore ineligible for IDENT under [FORM-3], like every other lowercase word this grammar fixes.” | Either “The spellings remain reserved despite having no effect production” or delete the sentence and declare both spellings newly eligible IDENTs. `RESISTS`: §3 chooses neither, so accepted syntax is undetermined. |
| K1362 | “The apostrophe- and at-prefixed lexical classes are untouched: REGIONID `'external` and LABEL `@blocks` remain well-formed spellings.” | This remains true. Bare-name reservation is a separate decision. |
| K1365 | “The body-syntactic contribution is syntactic over the complete function body: it exhibits `traps` iff the body contains a `claim` or a call to an operation or function whose selected row includes `traps`; it exhibits reads/writes/allocates per the operation table and borrow modes the body uses; and it exhibits `external` or `blocks` iff the body contains a call to any operation or function whose effect row includes that category.” | “...it exhibits kinded reads/writes/allocates by complete call-boundary projection; blocking behavior is retained separately in lowering metadata.” `RESISTS`: own-mode capability occurrences are not currently projected. |
| K1368 | “An optional `contract_block` consists only of erased definitions and proof clauses [FN-8, FN-9]; it contributes no read, write, allocation, external, blocking, or trapping category.” | “...it contributes no memory/world read or write, allocation, or trapping category and contains no target action carrying blocking metadata.” |
| K1372 | “Its compiler-owned captures, binder initialization, header comparison, and representable hidden update contribute no read, write, allocation, external, blocking, or trapping effect.” | “...contribute no memory/world read or write, allocation, or trap and execute no blocking target action.” |
| K1408 | “`external` and `blocks` carry no region payload, so the preceding call-boundary projection applies only to `reads`, `writes`, and `allocates` entries: the two categories transfer by presence and are unaffected by region-argument substitution, occurrence selection, and origin projection.” | “The preceding projection also selects world-kind occurrences in capability types, including own-mode actuals; blocking metadata does not enter EFF-2.” `RESISTS`: this is new projection machinery, not existing machinery. |
| K1419 | “Canonically, a nongeneric function whose only parameter is `own ReadFile` and whose complete body is exactly `return unit;` exhibits `external, blocks` and must declare exactly that row.” | Candidate: “A function `fn f['w](file: own ReadFile<'w>) ... writes('w)` exhibits `writes('w)` from the file release; its close action separately carries may-block metadata.” `RESISTS`: requires capability region syntax and instantiated release rows. |
| K1426 | “`pure` excludes traps, `external`, `blocks`, and all reads/writes/allocates; it does not promise termination.” | “`pure` excludes traps and all memory/world reads, writes, and allocations; it does not promise termination.” |
| K1427 | “These licenses are unchanged for every row that was `pure` before this version, and no license stated here reaches a row carrying `external` or `blocks` [EFF-5].” | “These licenses reach no row containing a memory/world read, write, allocation, or trap.” `RESISTS`: every semantically world-touching operation must receive a nonempty world row. |

### Complete [EFF-5] paragraph

| Line | Verbatim specification text | World-region rewrite / result |
|---|---|---|
| K1432 | “[EFF-5] Sequential external calls retain source program order.” | “Calls whose world footprints may conflict retain source program order.” `RESISTS`: this is strictly weaker than v0.36’s global promise. |
| K1433 | “Take two calls in one function whose resolved operation or callee rows each include `external`.” | “Take two calls whose rows access world regions related by the conservative may-alias relation and where at least one access is a write.” |
| K1434 | “If one precedes the other on a normal control-flow path of the conservative structural graph [FN-1] defines, then in every execution performing both, the earlier call's external effect is performed first.” | “...the earlier conflicting world access reaches its specified linearization point first.” `RESISTS`: submit, completion, and remote observation have no selected linearization point. |
| K1435 | “This holds when the two calls name different resources, different resource families, different owners, or the same owner.” | Cannot be preserved by per-region ordering. Replacement must say that different owners do not prove disjointness, while proven-disjoint world regions intentionally lose this global ordering. `RESISTS`. |
| K1436 | “A compiler-derived release action [STOR-3] whose row includes `external` participates on the same terms and occupies the position its normal edge gives it, after the releases that precede it in that edge's reverse declaration order.” | “A release’s instantiated world write participates against every may-alias world access and occupies its normal-edge position.” |
| K1437 | “A call whose row includes `external` is one such ordered point even when the external work is performed inside its callee; the callee's own external calls are performed within that call site's position in this order.” | “A call’s world footprint is a boundary summary even when the accesses occur inside its callee; each nested access must remain inside the boundary ordering required for its region.” |
| K1438 | “This ordering is a required property of every conforming lowering, at facts-off and at every optimization level; it is not an optimizer fact and no optimizer fact relaxes it.” | Preserve, referring to the newly defined world-trace ordering. |
| K1440 | “The rule orders the external calls that one execution performs.” | “The rule orders the conflicting world accesses that one execution performs.” |
| K1441 | “It is not a global runtime lock and not a total order over the whole program: this specification defines no worker, task, thread, or background-submission construct, and when such a construct is added it orders work across executions under its own rules rather than by widening this one.” | Preserve, but state explicitly that the new rule is not even a total order within one sequential execution across disjoint world regions. |
| K1442 | “Independently owned resources therefore remain the mechanism by which real concurrency is expressed later, and this rule constrains only what a single execution has already sequenced.” | “Proven-disjoint world alias domains, not independent ownership alone, permit concurrency.” `RESISTS`: capability separateness is not contact-point separateness. |
| K1444 | “No target-side fact proves two external calls independent or reorderable.” | “Only a checked world-region disjointness derivation, a TCB minting guarantee, or a separately approved fact family may prove two world accesses independent.” |
| K1445 | “A native handle or descriptor value, a separate open, a distinct target table entry, a distinct source spelling, the absence of a recorded alias link, and equal or unequal argument values are all outside the source language and prove nothing here.” | Preserve as the default. A world-region minting rule must positively establish disjointness rather than interpreting unequal capability values as proof. |
| K1446 | “Reordering, deduplicating, coalescing, hoisting, sinking, speculating, or eliminating an external call is unlicensed: [EFF-3] licenses those only for `pure`, and `pure` excludes `external`.” | “Those transformations remain unlicensed for any call with a nonempty world footprint; `pure` has none. Overlap of non-pure calls requires the separate world-order rule.” |
| K1447 | “A separately approved optional fact family may later license one exact transformation through a verifier binding the exact checked-program instance, target, backend, proposition, and authorized consequence [LEDGER-1]; that family's absence, rejection, or resource failure leaves source acceptance and facts-off lowering unchanged.” | Preserve. |

### [PAR-1], erroneous execution, and [PAR-2]

| Line | Verbatim specification text | World-region rewrite / result |
|---|---|---|
| K2012 | “Neither callee's effect row contains `external` or `blocks` [EFF-1], the effect row of every call written between the members likewise contains neither, and no statement of the window evaluates a system operation [EFF-1, SYS-2].” | “Every call and operation in the window has a complete world footprint; no pair denied by the world conflict/order relation overlaps. Blocking metadata does not itself deny.” `RESISTS`: direct operations, wrappers, and release actions need an I/O-frame lowering rule. |
| K2016 | “Under a permitted overlap every observable is the observable the same program produces by executing s1 and s2 in source order: the value of every binding and place, the trap-or-normal outcome, the exact [DIAG-3] record bytes, and the external-effect order [EFF-5] requires.” | Replace “external-effect order” with the exact world trace required by revised EFF-5. `RESISTS`: unrestricted read/read overlap does not yet preserve returned values. |
| K2017 | “That identity is conditional on contract compliance, exactly as [SCOPE-3]'s freedom from undefined behavior is conditional on its trusted computing base.” | Preserve. |
| K2018 | “For an execution in which no executed `claim` is false it holds in every execution, not in a typical execution or in some execution.” | Preserve. |
| K2019 | “An execution in which some executed `claim` is false is erroneous: the program has violated the sole writer-reachable language runtime contract [SCOPE-4], and this rule then requires exactly the following of that execution.” | Preserve only after deciding how submitted world work is quiesced or excluded. |
| K2020 | “The process writes exactly one complete [DIAG-3] record, naming one `claim` whose predicate evaluated false, and then aborts the whole process without unwinding and without language cleanup [TRAP-1].” | Preserve; add ordering against all in-flight world work. |
| K2021 | “No second record, and no partial or interleaved record, is written.” | Preserve; the diagnostic sink must not interleave with source output that aliases it. |
| K2022 | “Which such `claim` that record names may depend on the schedule, and is the only thing this specification permits a schedule to select.” | Cannot remain true if an overlapped world write may or may not occur. `RESISTS`. |
| K2023 | “Nothing else narrows for an erroneous execution: it has no undefined behavior [SCOPE-3], no overlapped pair reaches one place except as the disjointness condition above admits, and no statement of a permitted overlap produces an external effect at all, because neither callee's row may carry `external` [EFF-1].” | No faithful rewrite permits I/O in the same window. The conservative replacement is: “A window containing any world access is admitted only when every callable closure in it is trap-free; otherwise no world access occurs in that window.” `RESISTS`, critical. |
| K2024 | “The number of workers, the identity of the host thread that executes a statement, the schedule, and whether an overlap was performed at all are not observable, and no rule of this specification is stated in terms of them.” | Preserve. |
| K2025 | “An implementation that overlaps nothing therefore conforms: this permission is never an obligation, and no program depends on it being taken.” | Preserve. |
| K2026, first sentence | “When an execution of s1 or s2 does not reach its continuation, what survives is exactly what the erroneous-execution clauses above fix: the single complete [DIAG-3] record, the abort without unwinding or cleanup, and no external effect.” | “...and no source world access from the window.” This is sound only with the trap-free/world-free gate above. §3’s proposed “no write after the record” is weaker than this sentence. |
| K2039 | “No effect row of a call in B contains `external` or `blocks`, and no statement of B evaluates a system operation [EFF-1, SYS-2].” | Replace with complete loop-body world-footprint conflict and trap rules; blocking metadata is separate. |
| K2042 | “Under a permitted overlap every observable is the observable the same program produces by executing L's iterations in index order: the value of every binding and place, the trap-or-normal outcome, the exact [DIAG-3] record bytes, and the external-effect order [EFF-5] requires.” | Replace with the revised world trace, but only after proving iteration-level reads and writes satisfy it. `RESISTS`. |

### Gated FFI, [SYS-2], releases, and system contracts

| Line | Verbatim specification text | World-region rewrite / result |
|---|---|---|
| K2080–2084 | “The converse separation is equally exact: the system domain admits exactly the operations this specification names, while general FFI, arbitrary imported or exported foreign calls, raw host-ABI calls, and writer-declared external signatures remain reserved to this family and are unreachable through the system domain.” | “...writer-declared foreign signatures and any authority to mint or access world regions remain reserved to this family...” `RESISTS`: a gated call with ambient or unknown world reach needs an explicit top/conservative world capability or the deleted atom’s equivalent. |
| K2250 | `fn open_read['c, 'p](root: &'c DirectoryRead, path: &'p RelativePath) -> result: own Result<ReadFile, IoError> reads('c 'p), external, blocks;` | Root must become `DirectoryRead<'wd>` and contribute `reads('wd)`; success returns `ReadFile<'new-cursor, 'file-object>` or a conservative equivalent. The operation’s may-block property moves to TCB metadata. `RESISTS`: result-region generation is undefined. |
| K2251 | `fn read_once['f, 'd](file: &uniq 'f ReadFile, destination: &uniq 'd buffer<u8>, start: own u64, end: own u64) -> result: own ReadOutcome reads('f 'd), writes('f 'd), external, blocks;` | The capability type must carry cursor/object world regions. The row reads file content and writes cursor state in addition to current memory effects; may-block is metadata. |
| K2252 | `fn write_once['o, 's](output: &uniq 'o Output, source: &'s buffer<u8>, start: own u64, end: own u64) -> result: own Result<u64, IoError> reads('o 's), writes('o), external, blocks;` | Use separate borrow lifetime and world identity, for example `&uniq 'b Output<'w>` with memory effects over `'b/'s` and `writes('w)`; may-block is metadata. |
| K2254 | `fn open_directory['c, 'n](root: &'c DirectoryRead, name: &'n buffer<u8>, start: own u64, end: own u64) -> result: own Result<DirectoryRead, IoError> reads('c 'n), external, blocks;` | Read the parent/namespace world domain and return a capability carrying a new handle identity plus a conservatively aliased directory-object identity. `RESISTS`: `.`/`..` and repeated opens can return the same object. |
| K2255 | `fn open_list['c](directory: &'c DirectoryRead) -> result: own Result<DirectoryList, IoError> reads('c), external, blocks;` | Read the directory-object region and generate a fresh enumeration-cursor region on success; may-block is metadata. |
| K2256 | `fn list_once['l, 'd](list: &uniq 'l DirectoryList, destination: &uniq 'd buffer<u8>, start: own u64, end: own u64) -> result: own ListOutcome reads('l 'd), writes('l 'd), external, blocks;` | Read the directory-object world region and write the enumeration-cursor world region, plus current memory effects; may-block is metadata. |
| K2257 | `fn open_file['c, 'n](root: &'c DirectoryRead, name: &'n buffer<u8>, start: own u64, end: own u64) -> result: own Result<ReadFile, IoError> reads('c 'n), external, blocks;` | Same generativity and object-alias requirements as `open_read`. |
| K2263 | “The rows above are exactly that derivation together with each operation's fixed external and blocking classification; a system operation's row is declaration data and is never derived from a body, narrowed by a proof, or selected by a call site [ERR-4].” | “The rows contain the operation’s fixed memory/world footprint; its completion and may-block behavior is separate TCB declaration data. Neither is narrowed at a call site.” |
| K2265 | “An operation whose row contains `external` may observe or change state outside ordinary Whitefoot memory, and one whose row contains `blocks` may block its current host thread [EFF-1].” | “A world-kind read/write may observe/change its world domain; separate TCB metadata states how the target operation waits or completes.” |
| K2340 | “A lease owns no code-unit storage, several live leases may denote the same backing code units, and its compiler-derived release is a logical consume with no host call and no external effect [STOR-3].” | “...with no host call and no world-region access.” |
| K2366 | “That ID's record binds the operation's signature, complete outcome set, ownership transitions, memory and external effects [EFF-1], compiler-derived cleanup [STOR-3], and required target guarantees [QUAL-2].” | “...memory and world-region effects...” |
| K2398 | “External effects already performed are not rolled back: bytes already written remain written, an object already created remains created, and a persistent object or already-started external work retains the semantics its own family gives it.” | “Completed world writes are not rolled back, and submitted work retains its family semantics.” `RESISTS`: this contradicts any unconditional promise that no write occurs after the trap record. |
| K2440 | “Explicitly-abandonable means the type exposes a consuming abandon operation whose contract permits loss of unfinished external work, so abandonment is a source action rather than an accidental affine discard.” | “...permits loss of unfinished world-submitted work...” |
| K2452 | `\| DirectoryRead \| at most one native close attempt \| external, blocks \|` | Release row becomes an instantiated write to the capability-lifetime world region; close may block is separate release-action metadata. |
| K2453 | `\| ReadFile \| at most one native close attempt \| external, blocks \|` | Same rewrite for the file handle/cursor lifetime region. |
| K2456 | `\| DirectoryList \| at most one native close attempt \| external, blocks \|` | Same rewrite for the enumeration-handle lifetime region. |
| K2459 | “A logical consume performs no host call, no target call, no handle lookup, no byte copy, and no external effect.” | “...and no world-region access.” |
| K2466 | “Whole-process abort performs no release: a trap runs no language cleanup and returns no status [PROG-3, EFF-4, SCOPE-4], and the operating system reclaims process-local memory and handles while external writes are not rolled back.” | “...while completed or already-submitted world writes are not rolled back.” |
| K2616 | “Sequential calls across either owner preserve source order by the ordering rule that governs every external call, not by any aliasing analysis.” | “Calls across either owner preserve source order whenever their output world regions may alias.” `RESISTS`: the spec already records that these owners may be one sink. |
| K2634 | “`exit_status(code)` is its one constructor: it is total and pure, every `u8` is a valid command code, so the closed code range is 0 through 255 and there is no failure outcome, no allocation, no host call, and no external effect.” | “...no host call and no world-region access.” |
| K2685 | “It may not contain a user or system call, subscript, proof-required exact operation, checked-result operation, allocation, construction, write, move, borrow or reborrow, consuming projection, residual drop or cleanup, release, block, external operation, nested claim or trap, or any other partial, effectful, ownership-changing, or potentially nonterminating computation.” | “...release, block, operation reading or writing a world region, nested claim or trap...” |

## Provenance sentences that resist world-region rewriting

For every row below, the only semantics-preserving rewrite is terminological: replace the provenance class `external` with a name such as `boundary-derived`, and `unconditional-external` with `unconditional-boundary`. These statements must not be changed into world effects. K3205 explicitly says this provenance seed does not inspect an effect row.

| Line | Verbatim specification text | Result |
|---|---|---|
| K943 | “The length n is a protected subject under [PRV-2, PRV-3], so a claim about an unconditionally external n cannot launder it past the required real branch.” | `PROV` |
| K2271 | `\| args_count \| plain result external \| — \|` | `PROV` |
| K2272 | `\| arg_get \| Ok(value:) external; Err(error:) external \| — \|` | `PROV` |
| K2273 | `\| host_bytes_len \| plain result external \| — \|` | `PROV` |
| K2274 | `\| host_copy_bytes \| Ok(value:) dependent; Err(error:) external \| destination external \|` | `PROV` |
| K2275 | `\| host_utf8_len \| Ok(value:) external; Err(error:) external \| — \|` | `PROV` |
| K2276 | `\| host_copy_utf8 \| Ok(value:) dependent; Err(error:) external \| destination external \|` | `PROV` |
| K2277 | `\| relative_path \| Ok(value:) external; Err(error:) external \| — \|` | `PROV` |
| K2278 | `\| open_read \| Ok(value:) external; Err(error:) external \| — \|` | `PROV` |
| K2279 | `\| read_once \| ReadBytes(next:) dependent; ReadFailed(error:) external; ReadEnd() carries no result component \| destination external; file external \|` | `PROV` |
| K2280 | `\| write_once \| Ok(value:) dependent; Err(error:) external \| output external \|` | `PROV` |
| K2282 | `\| open_directory \| Ok(value:) external; Err(error:) external \| — \|` | `PROV` |
| K2283 | `\| open_list \| Ok(value:) external; Err(error:) external \| — \|` | `PROV` |
| K2284 | `\| list_once \| ListBytes(next:) dependent; ListBytes(entries:) internal; ListFailed(error:) external; ListEnd() carries no result component \| destination external; list external \|` | `PROV` |
| K2285 | `\| open_file \| Ok(value:) external; Err(error:) external \| — \|` | `PROV` |
| K2290 | “An external class seeds the unconditional-external bit; an internal class seeds no bit.” | `PROV` |
| K2292 | “No unlisted result, projection, parameter, field, or component inherits an external class by association.” | `PROV` |
| K2738 | “Operand provenance does not by itself prove a claim true: [PRV-2] and [PRV-3] still reject claim-only authorization of an unconditionally external constrained subject, while CLM-1 independently requires local authority and [CLM-2] independently requires a genuine admission consumer.” | `PROV` |
| K2837 | “Activating [PRV-2] or [PRV-3] for an already attached protected family, attaching a new protected family, changing a [SYS-2] component from internal to external, adding or removing a `BoundaryResult` seed or declassification, or adding a callable publication surface is an amendment-level accepted-set change, not implementation strengthening.” | `PROV` |
| K2969 | “Each result endpoint's [PRV-1] dependency additionally includes the concrete start actual, so this relation never launders an external start into an internal result.” | `PROV` |
| K3151 | “After complete-state success for a protected family, a [PRV-2] or [PRV-3] rejection makes the assertion-only route unavailable: the writer uses a dominating value branch whose false edge takes the domain outcome, or restructures so the external value no longer occupies the constrained-subject position.” | `PROV` |
| K3188 | “A write component unions every right-hand side written to a root overlapping that formal, together with each callee write component whose [EFF-2] projection reaches it; a system write adds no parameter datum and seeds the destination component's unconditional-external bit exactly when [SYS-2]'s closed table classifies that writable parameter external.” | `PROV` |
| K3190 | “A system result adds no formal parameter datum and seeds each plain or direct payload component's unconditional-external bit exactly as [SYS-2]'s closed table fixes; an internal component seeds no bit, while a dependent endpoint component additionally unions the concrete call's `start` actual dependency.” | `PROV` |
| K3203 | “Every source parameter component, command-entry parameter component, literal, named const, and otherwise untainted local initializer begins `Local`; this judgment does not classify external input provenance.” | `PROV` |
| K3205 | “This seed is unconditional: it does not inspect or substitute the callee body, arguments, effect row, [PRV-1] class, a system component's external/internal/dependent class, or an FN-9/S12 relation.” | `PROV`; direct proof that this meaning is independent of effect rows. |
| K3242 | “A subject component whose unconditional-external bit is true creates the local [PRV-3] candidate regardless of U and regardless of whether the component also carries parameter datums; retain those datums only as explanations and add no direct demand or bridge tuple for that subject.” | `PROV` |
| K3251 | “These facts may discharge a protected leaf in that same view, but they add no parameter datum, component predecessor, unconditional-external bit, protected family, constrained subject, demand kind, bridge kind, or callable component.” | `PROV` |
| K3258 | “After full success, a callee direct demand always composes through the selected actual component: a true unconditional-external bit creates the local call-argument candidate and retains any parameter datums only as explanations, while only a false bit permits each caller parameter datum in that component to add the corresponding direct demand to the caller.” | `PROV` |
| K3264 | “No synthetic external parameter datum is introduced.” | `PROV` |
| K3272 | “An unconditional-external bit is never replaced by or propagated as parameter-only demand metadata: it terminates at its local leaf under [PRV-3], or at its call argument under [PRV-2] for a direct demand or a B-failing bridge, retaining any parameter explanations only for diagnostics.” | `PROV` |
| K3273 | “At a `command` entry, each labelled input is unconditionally external [PRV-1]; a B-failing direct local leaf whose subject carries that bit is owned by PRV-3, while a B-failing inherited bridge whose selected actual carries that bit is owned by PRV-2 at that call's argument.” | `PROV` |
| K3285 | “Every component carries the pair `(unconditionally external, parameter datums)`, where the first member is one Boolean and the second is the finite [ENT-6] set.” | `PROV` |
| K3287 | “Under a concrete assignment of classes to ordinary parameter datums, the component is **external** exactly when its Boolean is true or at least one member of its parameter set is external; otherwise it is **internal**.” | `PROV` |
| K3289 | “Each labelled `command` entry parameter instead begins unconditionally external and creates no caller-substitutable sentinel.” | `PROV` |
| K3291 | “These entry and system components are the only unconditional external origins.” | `PROV` |
| K3307 | “An external value used only as a bound, base, target address, or unrelated goal operand therefore does not become a constrained subject by association.” | `PROV` |
| K3318 | “External origins are the labelled-entry `param` carrier or the system-operation `call` carrier and, for a system write, its exact writable parameter and caller actual.” | `PROV` |
| K3325 | “An unconditional-external bit is never represented by a synthetic parameter datum.” | `PROV` |
| K3329 | “For zero-based argument ordinal `q`, `Targets(c, q)` is the finite set of all direct-demand records whose selected actual component has a true unconditional-external bit, together with every bridge record for which the caller's B state fails and that selected component has a true unconditional bit.” | `PROV` |
| K3362 | “Either kind may also be repaired by restructuring the route so the external value no longer reaches the protected constrained subject.” | `PROV` |
| K3363 | “A `claim` is not a repair for an unconditionally external constrained subject.” | `PROV` |
| K3370 | “If B does not discharge it and any subject's PRV-1 pair has a true unconditional-external bit, the leaf is one hard rejection citing PRV-3 with `SourceNode` at its existing obligation-owning `psuffix` or `call` node and `SourceCoordinate` equal to that node's complete checked half-open extent, regardless of U and regardless of whether the pair also carries parameter datums.” | `PROV` |
| K3371 | “All external subjects and companion datums remain ordered diagnostic explanations but create no direct demand, bridge tuple, or second event.” | `PROV` |
| K3377 | “Every labelled input has a true unconditional-external bit, so a direct entry-local leaf whose constrained subject carries that bit must be justified by a real source branch present in U and B; a claim-only proof is the local PRV-3 rejection above.” | `PROV` |
| K3383 | “Thus a `claim` may not authorize an external constrained subject, while an internal subject may use one only when CLM-2 also proves that exact occurrence and every contribution individually necessary for an allowed terminal root.” | `PROV` |
| K3386 | “A PRV-3 payload contains the exact ENT-6 residual, the shortest post-convergence PRV-1 chain from the subject component to its labelled-entry or [SYS-2] origin, and the two legal repairs: a dominating real branch whose false edge takes the domain outcome, or a restructure in which the external value no longer occupies the constrained-subject position.” | `PROV` |

## Grep homonyms that require no semantic rewrite

| Line | Verbatim specification text | Result |
|---|---|---|
| K7 | “Selection ground: evidence-selected — the exclusivity investigation of batch 0081, chartered by the owner's direction of 2026-08-24 after an external review exhibited two read-only exclusive borrows overlapping: the probe suites and the completeness audit are recorded in the batch record, and the loans half is read off the borrow checker's existing loan vocabulary [OWN-5, OWN-12], so no writer construct, declaration, or marker is selected here.” | `HOMONYM`: outside review. |
| K19 | “An owner-approved program additionally has every retained claim's review record validated under [CLM-1]; that approval status is an external review judgment over the exact checker-accepted source and claim inventory, not another compiler fact source or a way to admit checker-rejected source.” | `HOMONYM`: outside review. |
| K66 | “Empty blocks still use an opening line followed by a closing-brace line.” | `HOMONYM`: brace blocks. |
| K140 | “An external terminal denotes one predicate over one formed token.” | `HOMONYM`: parser term. |
| K145 | “For each token independently, and without consulting grammar position, name lookup, the operation table, or another token, it evaluates the complete approved set of exact fixed-terminal predicates and external-terminal predicates in this specification and retains every matching predicate.” | `HOMONYM`: parser term. |
| K1029 | “A `let_stmt` selecting `value_if` enters both branch blocks the same way and follows [GIVE-1] exactly as the `value_match` sentence above does; an else-position `value_if` of a chain contributes its own branch edges to the same enclosing `let_stmt` [GIVE-1], not to a nested one.” | `HOMONYM`: branch blocks. |
| K1479 | “The only external boundary for foreign code is the gated FFI wall (§14); compiler-owned system operations [QUAL-1] are implemented by an approved target entry rather than by foreign code, and are not such a boundary [GATE-2].” | `HOMONYM`: foreign-code boundary; its authority implications are addressed separately at K2080–2084. |
| K1492 | “There is no source contract on main [FN-7], entry-goal evaluation, runtime wrapper condition, helper function, duplicate body, or second external entry.” | `HOMONYM`: externally invoked entry. |
| K1571 | “Every grammar production and external terminal predicate is owned by the numbered rule containing its unique definition.” | `HOMONYM`: parser term. |
| K1602 | “If the boundary token has the raw shape admitted by an expected external predicate before that predicate's explicit spelling restrictions, and fails only those restrictions, the rejection cites that predicate's owner.” | `HOMONYM`: parser term. |
| K1615 | “If several external predicates qualify under the first sentence, their owners rank by first rule occurrence in this specification.” | `HOMONYM`: parser term. |
| K1643 | “An input-envelope failure, resource failure, target-layout failure [STOR-6], target-qualification failure [QUAL-1], compiler-invariant failure, unsupported compiler capability, backend failure, or external-tool failure is not a source-language rejection, cites no language rule, and carries no expected-terminal set.” | `HOMONYM`: outside tool. |
| K1654 | “A referenced child production means a child production node, not an external terminal predicate such as `literal`.” | `HOMONYM`: parser term. |
| K1802 | “Backend, linker, runtime-environment, and external-tool failures remain non-language failures [DIAG-1].” | `HOMONYM`: outside tool. |

## `pure` and adjacent dependent sentences

| Line | Verbatim text | Rewrite / result |
|---|---|---|
| K6 | “No construct is added, no accepted program changes acceptance, no conformance verdict changes, and no required check is removed; the permitted-overlap set only narrows.” | This describes v0.36, not the proposed change. It cannot be reused: deleting two grammar atoms and changing entry/system rows changes source bytes and several current verdict cases. |
| K1094 | “A member declaring neither category therefore cannot bind a function that exhibits one, and a `pure` member cannot bind an externally effectful function.” | “A member omitting a world-region access cannot bind a function that exhibits it, and a `pure` member cannot bind a function with any world footprint.” |
| K1346 | `effects := "pure" \| effect ("," effect)*` | Preserve. |
| K1353 | “The two added categories take positions between `allocates` and `traps`, which leaves the pairwise canonical order of the four pre-existing categories unchanged.” | Delete; the two categories are removed. |
| K1355 | “A category states what a call may do, never which object it does it to.” | Must change: a world-region payload identifies the conservative world alias domain affected by the call. |
| K1359 | “A source row consequently carries no resource origin, and no rule derives a disjointness, reordering, or elimination conclusion from a row [EFF-5].” | Must change: world rows carry alias-domain identity and may feed only the precisely enumerated overlap/order conclusions. |
| K1416 | “A function whose body and release contribution are empty may therefore declare `pure` while carrying an erased contract.” | Preserve. |
| K1421 | “Declaring `pure` is an undeclared-but-exhibited rejection at that function's `effects` node.” | Preserve for the rewritten `ReadFile<'w>` release example, with the missing exhibited category now `writes('w)`. |
| K1424 | “[EFF-3] `pure` licenses deduplication and reordering of calls with equal arguments.” | Preserve only if every world-touching operation necessarily has a nonempty world row. |
| K1425 | “Elimination of an unused pure call additionally requires a termination proof; v0 provides no termination checker, so unused pure calls are not eliminated.” | Preserve. |

All other `pure` occurrences describe operation-table purity, proof syntax, laws, or examples. They do not derive purity from `external`/`blocks` and need no change.

# Semantic holes, ranked

## Critical 1: capability inequality does not prove contact-point disjointness

The design says:

> “through different capabilities they are disjoint (overlap permitted)”  
> — DESIGN.md:123–126

The active spec says the opposite twice:

> “This holds when the two calls name different resources, different resource families, different owners, or the same owner.”  
> — kernel-spec.md:1435

> “A native handle or descriptor value, a separate open, a distinct target table entry, a distinct source spelling, the absence of a recorded alias link, and equal or unequal argument values are all outside the source language and prove nothing here.”  
> — kernel-spec.md:1445

The concrete existing counterexample is stdout/stderr redirection. The spec says the two bindings are separate owners, but may be the same sink, and deliberately preserves their source order without alias analysis at K2614–2617. The checked program already retains that may-alias edge:

```rust
/// [SYS-12] fixes exactly one for the first slice: redirection may make the
/// `command.stdout` and `command.stderr` owners the same sink.
pub(crate) aliases: Vec<CheckedResourceAlias>,
```

[compiler/src/semantic/model.rs:1611](/compiler/src/semantic/model.rs:1611)

The conformance adapter realizes that case by duplicating one open file description:

```rust
// Two streams naming one sink are one destination sharing one
// open file description, which is what makes cross-owner call
// order observable in the combined bytes [EFF-5, SYS-12].
Some(open) => open.try_clone().expect("duplicate the shared sink"),
```

[compiler/tests/conformance/adapter.rs:143](/compiler/tests/conformance/adapter.rs:143)

The runtime case writes `AA` through stdout and `BB` through stderr at [run-sysout-redirect-same-sink-order.wf:8](/tests/conformance/cases/run-sysout-redirect-same-sink-order.wf:8) and line 19, then requires the combined bytes to be `AABB` at lines 56–79. Its manifest explicitly redirects both to `"combined"` and expects exit zero at [manifest.jsonl:448](/tests/conformance/manifest.jsonl:448).

Under §3’s rule, wrappers around those two writes have different capability world regions, disjoint memory places, and disjoint loans, so this proposed pair is admissible:

```wf
// Schematic proposed syntax.
let first_result = emit['stdout](output: &uniq 'a out, source: &'s first);
let second_result = emit['stderr](output: &uniq 'b err, source: &'t second);
```

If redirection aliases the sinks, overlap can publish `BBAA` or interleaved bytes. This violates both K1435 and the shipped runtime witness.

The required invariant is stronger than “capability identity approximates contact-point identity” at DESIGN.md:136–144:

> Unequal world regions may prove disjointness only when the TCB or a checked generativity rule proves that every observable state facet reached through them cannot alias. Uncertainty means overlap, not disjointness.

### Required POSIX alias judgments

| POSIX case | State that aliases | Required region treatment | Judgment on §3c policy |
|---|---|---|---|
| `dup`/descriptor cloning | Same open file description, hence shared cursor/status; for a socket, the same connection/stream | Duplication preserves every source world region. It never mints a fresh region. | “Reads free” is unsound for cursor-consuming reads. “Network writes free” is unsound for two duplicated handles to one connection. The current Whitefoot source surface exposes no general duplicate operation at K2432, but command redirection already creates this host shape. |
| Same path opened twice | Distinct handle cursors, but potentially the same filesystem object/content | Fresh cursor region per successful open; shared or conservatively global filesystem-object/content region | Read/read may overlap only on separate cursor state. Any file write must conflict with reads and writes through the object region. K2603–2604 already says a separate open does not prove a separate object. |
| stdout/stderr on one terminal or redirected file | Same terminal, pipe, file description, or other sink | Their output regions must be may-alias, or both must use one conservative Output-family region | §3’s “different capabilities are disjoint” rule is directly unsound. K2617 and the conformance case already pin this. |
| Hard links | Different path spellings/open descriptions can reach one inode/content object | Separate cursor regions; same or conservatively aliased file-content region | Path spelling and separate opens cannot prove disjointness, consistent with K1445. A singular per-capability region misses the shared object. |

The stated file/network policy is also not implementable from the current `Output` type. `Output` may be redirected to a terminal, file, pipe, or network endpoint, but [SYS-12] gives it one static type and no target-visible subtype at K2613–2626. A policy that globally orders files while freely overlapping network writes therefore needs either distinct capability contracts or a conservative common Output domain.

For distinct network connections, per-connection FIFO is not enough to preserve v0.36 semantics. DESIGN.md:23–26 itself says third parties make outside-world order observable. A peer can correlate messages on two connections. Free cross-connection overlap is sound only after the language explicitly removes that cross-region order from its trace semantics and supplies the fence promised at DESIGN.md:145–146. No such operation exists in SYS-2.

## Critical 2: the trap rewrite is weaker than the current guarantee and fails with in-flight I/O

DESIGN.md:120 maps the current trap job to:

> “no write to any world region after the record”

The current [PAR-1] guarantee is stronger:

> “no statement of a permitted overlap produces an external effect at all”  
> — K2023

> “the single complete [DIAG-3] record, the abort without unwinding or cleanup, and no external effect”  
> — K2026

The permission implementation deliberately allows claims in the transitive call closure. It explains that the old closure gate was removed at [permission.rs:85–107](/compiler/src/semantic/permission.rs:85), and returns “permitted” with no fifth condition at [permission.rs:904–907](/compiler/src/semantic/permission.rs:904).

Once disjoint world writes are admitted, this pair breaks source order even with perfect alias identities:

```wf
// Schematic proposed syntax.
let a = fail_if_contract_broken();  // row: traps; false claim aborts
let b = emit(output: &uniq 'b out); // row: writes('out)
```

There is no dataflow, memory conflict, loan conflict, or shared world region. In source order, a false claim in the first call prevents the second call from writing. Under overlap, the second call may write before the first call traps. K2022 says claim identity is the only schedule-selected fact; the world write becomes a second schedule-selected fact.

The reverse placement also fails. If an earlier write is submitted and a later call traps, completion can occur after the diagnostic record. The current system contract says already-started external work retains its family semantics at K2398, while DESIGN.md:264–267 explicitly leaves cancellation unsettled.

Required repair: any overlap window containing a world access must have a transitive `traps`-free closure, or the implementation must join/quiesce all world work before entering any potentially trapping call. The former is the presently specifiable conservative rule. The diagnostic channel must remain a distinct TCB trace and must not interleave with source output that aliases standard error.

## Critical 3: `reads('world)` does not by itself justify read/read overlap

DESIGN.md:127–129 claims that stdin and clock reads become ordinary reads and therefore overlap. That confuses absence of a write/write race with preservation of sequential observations.

A monotonic-clock counterexample is:

```wf
let a = now(clock: &'c clock); // reads('clock)
let b = now(clock: &'c clock); // reads('clock)
```

Shared/shared loans and read/read footprints conflict nowhere. If the second observation linearizes first, `a > b` can result even though source-order execution of a monotonic clock requires `a <= b`.

Stateful input reads are even clearer. `read_once` advances its cursor by exactly the returned byte count at K2545–2548, and [SYS-4] says an operation advancing a cursor changes state and therefore takes `&uniq` or consumes the owner at K2405–2407. Such an operation is a world write to the cursor, even though its payload direction is “read.”

Read/read overlap is sound only under one of these explicit rules:

1. both observations are proved commutative with source-order result attribution;
2. they may execute physically in parallel but their linearization points on one world region remain in source order; or
3. the language deliberately weakens the permitted result trace and records that as an accepted semantic change.

“A moving world races sequential execution too” at DESIGN.md:137–140 proves none of those.

## High 4: own mode needs no region payload; the capability type does

Current representation:

```rust
pub(crate) enum CheckedMode {
    Own,
    Shared(DeclarationId),
    Unique(DeclarationId),
}
```

[compiler/src/semantic/model.rs:16](/compiler/src/semantic/model.rs:16)

`CheckedType` has no system-resource-with-region variant; opaque resources are merely nominal IDs at [model.rs:242–262](/compiler/src/semantic/model.rs:242). The nominal kind records only the catalog index:

```rust
SystemResource {
    nominal: u8,
},
```

[model.rs:373](/compiler/src/semantic/model.rs:373)

The useful precedent is `Arena`, whose region is stored in the nominal instance:

```rust
Arena {
    region: DeclarationId,
    content: CheckedType,
},
```

[model.rs:360](/compiler/src/semantic/model.rs:360)

System nominals are presently interned once by `u8` catalog index at [check.rs:544](/compiler/src/semantic/check.rs:544) and [nominals.rs:248–315](/compiler/src/semantic/check/nominals.rs:248). Written type arguments on a system nominal are explicitly rejected at [types.rs:284–294](/compiler/src/semantic/check/types.rs:284). The spec matches the implementation: every opaque system type is bare, has no targs, and carries no region at K2163–2167.

The actual representation change is therefore:

- retain `CheckedMode::Own`;
- add kinded world identity to the capability type, for example `SystemResource { nominal, world_regions }`, interned by `(nominal, world_regions)`, or add an equivalent direct `CheckedType` variant;
- retain both identities in a borrow such as `&uniq 'loan Output<'world>`; `'loan` is an OWN lifetime and `'world` is an effect/alias identity;
- stop storing both kinds as indistinguishable `DeclarationId`s, because generated world regions may have no source declaration.

Changing `CheckedMode::Own` would be the wrong representation. It would also fail to represent the two independent regions in a borrowed capability.

## High 5: dynamic capability creation needs generativity and more than one state facet

`open_read`, `open_directory`, `open_list`, and `open_file` return new capability values at K2250 and K2254–2257. Their family rules distinguish:

- fresh per-handle cursor state at K2588–2589, K2603, and K2644–2645;
- potentially shared directory or file objects at K2590, K2600, and K2603–2604.

A successful `open_file` therefore cannot simply return either:

- a caller-selected fresh region, because the caller has no authority to assert physical disjointness; or
- the parent directory region, because that conflates the new cursor with every object reachable from the directory.

It needs at least a fresh handle/cursor identity and a conservative underlying-object alias domain. A duplicate preserves both. A same-path or hard-link open gets a fresh cursor but not a proved-fresh object region.

The source and type-scope problem is unresolved:

- main declares no region parameters at K1182;
- system resource types currently take no targs at K2166;
- ordinary function and system calls require explicit region arguments at K348 and K2295–2297;
- a successful open returns the capability inside `Result`, while STOR-5 currently treats lifetime-region-bearing values in generic payloads as forbidden or deferred at K684–697.

A world identity must therefore be a different kind from a storage lifetime. The design must choose either a generative unpack/binder surface or a compiler-created result skolem plus inference for world-kind call arguments. “No writer-facing construct” is not a complete answer.

Freshness also needs a quantified rule: one static call site executed in a loop cannot automatically prove all runtime results pairwise disjoint. Without an iteration-separation proof, its compile-time world identity must remain a may-alias class across executions.

Destruction has the symmetric hole. `DirectoryRead`, `ReadFile`, and `DirectoryList` releases perform native close attempts at K2452–2456. Their releases must instantiate a world write from the released type, and their may-block status must remain available even though no source call occurs.

## High 6: current [EFF-2] and permission projection do not extend through own capability types

The design’s example says existing [EFF-2] projection forces a caller to declare `writes('o)`. Current code does not.

Permission projects an `allocates(arena 'r)` entry directly through explicit region arguments:

```rust
for formal in &signature.allocates_arenas {
    ...
    Some(region) => footprint.writes.push(Access::Arena { ... })
}
```

[permission.rs:1056](/compiler/src/semantic/permission.rs:1056)

But parameter projection recognizes only:

```rust
let mode_region = match parameter.mode {
    CheckedMode::Own => None,
    CheckedMode::Shared(region) | CheckedMode::Unique(region) => Some(region),
};
let slice_region = match parameter.ty {
    CheckedType::Slice { region, .. } => Some(region),
    _ => None,
};
```

[permission.rs:1082](/compiler/src/semantic/permission.rs:1082)

An own actual is handled only as a consumed memory place at [permission.rs:1136–1145](/compiler/src/semantic/permission.rs:1136). `Access` has only `Place` and `Arena` variants at [permission.rs:172–200](/compiler/src/semantic/permission.rs:172).

The source-semantic EFF-2 projector has the same limitation: `CheckedMode::Own => None`, with a special case only for direct `Slice`, at [calls/user.rs:719–759](/compiler/src/semantic/check/expressions/calls/user.rs:719). System projection skips own parameters outright at [calls/system.rs:221–224](/compiler/src/semantic/check/expressions/calls/system.rs:221).

The arena comment already names the analogous defect:

> “an `own arena<'r, T>` parameter carries no mode region and no slice region: a callee row that declares `writes('r)` projects nothing onto it”  
> — permission.rs:187–195

Required implementation semantics:

- recognize world-region occurrences in capability types, including nested result types and own parameters;
- substitute formal world regions through explicit/inferred actuals independently of memory-place projection;
- add an `Access::World` footprint whose conflict test uses conservative world may-alias, not `ResolvedPlace::overlaps`;
- include generated/effect-only world regions using the arena-style no-value-actual path;
- instantiate release rows from the dropped type;
- apply the same rules in EFF-2 exactness, contract alpha-equality, [PAR-1], and [PAR-2].

For a function taking literally no capability, the current first slice provides no ambient route to I/O: K1211 says every system value originates at entry and reaches helpers only through a written parameter. If a future producer creates a capability locally, its generated world region must still project into that function’s row. If there is neither a capability occurrence nor an explicit/generated region actual, projection has no legal target; the function must be rejected or conservatively charged to an enclosing world domain.

## High 7: `blocks` has a release and transitive-lowering job omitted by §3

DESIGN.md:121 assigns `blocks` to “a TCB-known attribute of system operations.” Current semantics also attaches it to compiler-derived close actions:

- STOR-3 permits release rows containing `blocks` at K672.
- Three close releases carry `external, blocks` at K2452–2456.
- `EffectSet` currently propagates both booleans through transitive union at [check.rs:422–474](/compiler/src/semantic/check.rs:422).
- The catalog’s release row independently stores both properties at [catalog.rs:850–861](/compiler/src/resolution/catalog.rs:850).

Moving blocking out of source semantics is plausible, but the metadata must attach to every target action, including release/cleanup. Closed-world analysis must derive a transitive “contains target waiting point” lowering summary for user wrappers. A degraded POSIX backend must execute such actions on its waiter/blocking pool, never accidentally on the only runnable Whitefoot lane.

## High 8: EFF-5 narrowing is a semantic change, not a theorem

Even if world identities were perfect, K1435 orders calls across different resources and families. A conflict matrix over world regions orders only may-alias regions with a write. Therefore DESIGN.md:126’s claim that “EFF-5’s narrowing becomes a theorem” is false relative to v0.36.

There are only two honest migrations:

1. preserve v0.36 initially by adding one conservative global world-order region to every former-`external` operation, then refine selected families under later evidence; or
2. declare a versioned semantic weakening, define the new world trace and fence, and change the EFF-5 runtime evidence.

The fence cannot remain prose. DESIGN.md:145–146 promises an ordinary operation, but SYS-2’s exact fifteen-operation inventory at K2240–2257 contains none.

## Medium 9: conformance impact is larger than a syntax substitution

Actual grep counts over `tests/conformance`:

- 42 `.wf` case files mention `external` or `blocks`, case-insensitively.
- 29 case files mention `blocks`; all use or discuss the `external, blocks` pair.
- 27 case files contain a top-level source declaration whose effect row actually writes `external`.
- 17 manifest records mention either term.
- Total occurrences in `tests/conformance`: 65 `external`, 35 `blocks`.

The verdict-sensitive records are explicit at [manifest.jsonl:405–411](/tests/conformance/manifest.jsonl:405):

- `accept-sysrelease-return-unit-declared`
- `reject-syseff-return-unit-pure`
- `reject-syseff-declared-unexhibited`
- `accept-syseff-conditional-release-union`
- `reject-syseff-conditional-release-narrow`
- `accept-syseff-pure-immutable-only`
- `reject-syseff-pure-member-binds-release`

Expected consequences:

- The release-positive and pure-negative cases can retain their verdicts only after `ReadFile<'w>` release contributes `writes('w)`.
- `reject-syseff-declared-unexhibited` cannot merely delete the atoms: it would cease to test EFF-2’s declared-but-unexhibited direction. It needs a well-bound but unexhibited world row.
- Entry cases need new canonical capability types and region binders.
- The same-sink EFF-5 runtime case at manifest line 448 must remain passing by conservative aliasing, or be explicitly changed as evidence of an intentional semantic weakening.
- Provenance cases mentioning `external` are not effect migrations and must not change verdict.

## Medium 10: lexical reservation, PRV terminology, and gated FFI remain separate jobs

Removing the productions does not decide whether bare `external` and `blocks` become legal IDENTs. K1361 currently reserves them precisely because they are fixed grammar atoms. The design needs an explicit META-5 accepted-set choice.

The 46 provenance hit lines form a second semantic vocabulary. Because K3205 explicitly separates it from effect rows, it must either remain named `external` or be renamed mechanically to something such as `boundary-derived`. Treating it as a world-region property would corrupt PRV-2/PRV-3.

Finally, arbitrary gated foreign signatures may touch ambient or unclassifiable world state. K2080–2084 reserves those signatures to the gated family. Deleting the fallback atom requires either a conservative top-world capability/domain for gated calls or a proof that every such call receives complete typed world authority.

# Verdict

**UNSOUND AS DESIGNED.**

The decisive existing counterexample is the stdout/stderr same-sink program. §3 states that writes through different capability values are disjoint and may overlap, while K2614–2617 and the runnable conformance case establish that those two values can reach one sink and that their byte order is observable.

A second independent counterexample is a false-claim call followed by a disjoint world-writing call. Current [PAR-1] permits claims in callee closures only because its blanket row gate guarantees that the window performs no external effect. Removing that gate lets the later write occur even though source-order execution traps before it. The proposed “no write after the record” wording neither prevents a write before the record nor controls an already-submitted completion after it.

The `blocks` deletion can be sound after its metadata is generalized to operations, releases, and transitive lowering. The `external` deletion cannot land safely until the following amendments are part of the design.

# Exact amendments required in DESIGN.md

1. **Replace capability-equality disjointness.**

   Replace “through different capabilities they are disjoint” with:

   > Different capability values are never by themselves evidence of world disjointness. Two world regions are disjoint only when a TCB minting rule or checked generativity derivation proves that every state facet named by the two footprints cannot alias. When that proof is absent, their world footprints overlap.

2. **Define two region kinds.**

   Add:

   > Memory regions are OWN-3 lexical lifetimes. World regions are effect and alias identities. Substitution, equality, liveness, outlives, storage-bearing, and diagnostics distinguish the two kinds. A type `&uniq 'b Output<'w>` carries memory-loan region `'b` and world region `'w`; neither substitutes for the other.

3. **Put world identity in capability types, not `own`.**

   Add:

   > `own` remains a payload-free ownership mode. Each capability family declares a fixed vector of world-region parameters carried by its type. Checked type identity retains those parameters, and system nominal interning keys on the nominal family plus the instantiated world-region vector.

4. **Define entry-region binding and aliasing.**

   Add an exact canonical form for command inputs. It must state:

   - how the cwd, stdout, and stderr world identities are bound despite main’s current no-region-parameter rule;
   - whether those identities are writer-visible or compiler-supplied;
   - that stdout and stderr are conservatively may-alias;
   - how their regions appear in main’s exact effect row and in compiler-derived release.

5. **Define generated result regions.**

   Add:

   > A successful capability-producing operation may generate a result world region only for state the operation contract proves fresh. Failure generates no capability identity. A compile-time identity represents a may-alias class across repeated executions unless an iteration- or instance-separation proof says otherwise.

   The design must select either an explicit generative unpack binder or compiler-created result skolems with world-argument inference. It must update the claim that no writer-facing mechanism changes.

6. **Separate handle state from underlying object state.**

   Add family contracts with at least these rules:

   - a successful separate file open gets fresh cursor state;
   - separate opens, same paths, different paths, and hard links do not prove distinct file content;
   - `dup` preserves every source region;
   - directory opens may return the same directory object;
   - enumeration opens get fresh cursor state but may share directory content;
   - capability release writes the handle-lifetime region, not necessarily the persistent object region.

7. **Fix the near-term POSIX policy.**

   State exactly:

   - all Output values conservatively overlap unless the target proves their sinks disjoint;
   - stdout/stderr always carry a may-alias edge;
   - POSIX file-content writes use one conservative file-object domain unless stronger identity evidence exists;
   - file writes conflict with both reads and writes of that domain;
   - a duplicated network handle retains the same connection domain;
   - only a TCB-proved new connection may receive a fresh connection domain;
   - cross-connection ordering is either retained or explicitly removed from semantics and recovered through a declared fence.

8. **Define world read semantics.**

   Replace “reads overlap freely” with:

   > Read/read overlap is admitted only when the operation contract proves source-order result attribution under overlap, or when both reads retain source-ordered linearization points. An operation that advances a cursor, consumes input, samples an ordered sequence, or otherwise changes future observations writes the applicable world region.

9. **Replace EFF-5 with a complete trace law.**

   The new rule must define:

   - which world accesses are ordered;
   - the may-alias relation used for conflicts;
   - whether ordering refers to submission, linearization, completion, or remote observation;
   - nested-call and compiler-derived-release positions;
   - the semantics of disjoint-region calls;
   - the exact fence operation and its cross-region guarantee;
   - whether this is an intentional weakening of v0.36.

10. **Restore trap safety for world windows.**

    Add:

    > A permitted overlap containing any world access has a transitive trap-free callable closure, including compiler-derived releases and every interposed statement. Otherwise the window remains sequential.

    Also state that the diagnostic channel is TCB-owned, globally record-serialized, and cannot interleave with any source write that may alias its sink. Do not promise cancellation until the open cancellation question is resolved.

11. **Extend EFF-2 and permission projection explicitly.**

    Add requirements covering:

    - world-region occurrences in own, shared, and unique capability parameters;
    - formal-to-actual world substitution;
    - effect-only and generated result regions;
    - capability values nested in `Result` or other outcomes;
    - system calls and user calls;
    - compiler-derived release;
    - contract-member alpha-equality;
    - `Access::World` conflict checks in [PAR-1]/[PAR-2];
    - conservative failure when projection or alias identity is unresolved.

12. **Generalize blocking metadata.**

    Replace “attribute of system operations” with:

    > Every target action, including a system operation, compiler-derived release, close, completion wait, and backend adapter action, declares trusted completion/blocking metadata. Closed-world lowering derives the transitive action summary of user wrappers. This metadata is not part of EFF-1 exactness, but every backend must route a potentially blocking action so it cannot stall a Whitefoot compute lane that is required for progress.

13. **Rewrite the complete SYS-2 table, not only its suffix atoms.**

    Each affected signature must separately name:

    - memory borrow regions;
    - capability world regions;
    - generated result regions;
    - world read/write footprints;
    - target completion/blocking metadata.

    The system-type inventory sentence at K2166, operation counts at K2260, operation-record definition at K2306, and release table at K2444–2462 must change together.

14. **Resolve keyword and FFI policy.**

    State whether bare `external` and `blocks` remain reserved. For gated foreign calls, require complete capability/world footprints or charge the call to one explicit conservative top-world domain; absence of a footprint must never imply purity or disjointness.

15. **Keep provenance separate.**

    Add:

    > Deleting the EFF-1 atom `external` does not delete PRV-1’s external-input provenance class. That class is independent of effect rows. To avoid homonym confusion it is renamed `boundary-derived` throughout the specification, compiler metadata, diagnostics, and conformance prose, with no verdict change.

16. **Name the conformance migration before claiming completeness.**

    The design must enumerate:

    - all 27 source cases whose written rows contain `external`;
    - the seven release/purity verdict records at manifest lines 405–411;
    - entry canonical-form cases;
    - the same-sink EFF-5 runtime witness;
    - provenance-only cases that must retain verdicts.

    No existing verdict may be silently changed or a case weakened merely because the old atoms no longer parse.
