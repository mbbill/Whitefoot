//! Standing gates for the dossier's §9.1 native cost-shape table, read off the
//! real `tests/programs/wfgrep.wf` command rather than off a shape written to
//! pass them.
//!
//! [QUAL-3] fixes the required emitted shape and says in as many words that
//! "the evidence establishing it is inspection of emitted code and symbols,
//! not a machine-checked language judgment". These tests are that inspection,
//! made standing: each one names the §9.1 row it discharges and asserts the
//! structural property that row requires, so a later lowering change that
//! reintroduces an allocation, a copy, a dispatch, a handle lookup, a lock, or
//! a per-call signal operation fails here instead of being discovered by a
//! measurement much later.
//!
//! Task 0012 already inspected these properties on small programs written for
//! one operation each. This module answers the different question the plan
//! actually asks: whether the shape survives on the whole first real program,
//! after the host optimizer has inlined every wrapper and rearranged the
//! control flow of a five-declaration, seven-hundred-line command. Two levels
//! of evidence appear here for that reason:
//!
//! 1. the emitted body of each [QUAL-1]-selected approved implementation, in
//!    `wfgrep`'s own module — what the compiler emits for the row; and
//! 2. `wfgrep`'s optimized `main` — what survives of it in a real program.
//!
//! The complete §9.1 row inventory, and where each row's evidence lives, so
//! that what is machine-checked here is not confused with what is not:
//!
//! | §9.1 row | Evidence |
//! |---|---|
//! | target selection | `target_selection_is_one_link_time_table_decision` |
//! | selected argument | `an_argument_lease_allocates_nothing_and_copies_no_byte` |
//! | raw argument bytes | `the_raw_byte_route_carries_no_unicode_gate` |
//! | UTF-8 text conversion | absence side gated here; the presence side is the `run-syshost-copyutf8-*` conformance cases. The Windows column the row also names has no implementation in the first slice, so it is not inspectable and is not claimed. |
//! | `RelativePath` construction | `relative_path_retypes_the_lease_without_allocating` |
//! | `open_read` | `open_read_is_one_direct_relative_open_on_the_capabilitys_own_descriptor` |
//! | `read_once` / `write_once` | `each_transfer_is_one_host_call_with_a_cold_outcome_mapper` |
//! | `DirectoryRead`/`ReadFile` release | `every_release_close_is_one_discarded_attempt` |
//! | value releases | `releasing_a_value_or_an_output_reaches_no_host_facility` |
//! | `Output` release | the same test, plus the deterministic-host run below. The row's second half is a *recording* obligation, not a gate: a failure surfaced only at close or writeback is outside this slice's error model, is recorded as such in the dossier, and is observed rather than assumed by `deterministic_target::an_output_sink_that_fails_only_at_close_is_never_closed_by_its_release`. |
//! | output batching | `the_output_batch_costs_one_host_write_per_full_batch` — a count over a real run rather than a shape, so it is gated against task 0013's deterministic host, where host attempts are observable. |
//! | buffer initialization reuse | `the_reused_buffers_are_initialized_once_at_allocation` — the *initialized* control, which answers the structural question only. |
//! | initialization cost | `research/experiments/buffer-initialization-cost/` — the *uninitialized* control, which is the only one that can answer whether paying for initialization is material at all. It is a measurement, and per `tests/codegen/README.md` noisy timing is experimental evidence rather than an every-commit invariant. |
//!
//! §9.1's own closing paragraph says the remaining `material` judgments "are
//! structural inspections rather than quantitative gates, and carry no
//! threshold by design". Nothing here invents a numeric threshold for them.
//!
//! One recorded observation is corrected here rather than gated, because
//! gating it would assert something untrue. Task 0015's closure recorded, as
//! an informal reading, that `wfgrep`'s newline scan is "recognized as
//! memchr". It is not. The single `@memchr` call in the optimized module is
//! `relative_path`'s embedded-NUL check; the newline scan is a scalar
//! byte-at-a-time loop that retains its bounds trap, and the `<16 x i8>`
//! vector operations in the module belong to the boundary-carry shift and the
//! batch append, not to the scan. No §9.1 row requires a `memchr`, and §12.2's
//! per-byte-call rejection is satisfied — the deterministic-host run below
//! moves six thousand bytes with five host calls in total — so this is a
//! correction to a note, not a defect. A future scan-recognition change is a
//! performance question for its own attributed slice.

use std::sync::OnceLock;

use super::deterministic_target::{HostScript, run_on_deterministic_host};
use super::{compile, host_optimized_module, optimized_main};

/// The first real Whitefoot program (task 0015), following the dossier's
/// §10.1 witness trace step for step. It is the anchor for every row below:
/// a cost-shape gate on a program written for the gate proves nothing about
/// the program the project actually has to compile.
const WFGREP: &[u8] = include_bytes!("../../../../tests/programs/wfgrep.wf");

/// `wfgrep`'s module as the backend emits it.
fn emitted() -> &'static str {
    static MODULE: OnceLock<String> = OnceLock::new();
    MODULE.get_or_init(|| compile(WFGREP))
}

/// `wfgrep`'s module as the host optimizer leaves it at the shipped level.
fn optimized() -> &'static str {
    static MODULE: OnceLock<String> = OnceLock::new();
    MODULE.get_or_init(|| host_optimized_module(emitted()))
}

/// `wfgrep`'s optimized entry: the whole program, every wrapper inlined.
fn entry() -> &'static str {
    optimized_main(optimized())
}

/// The emitted body of one approved implementation, by symbol.
fn approved_row<'module>(module: &'module str, symbol: &str) -> &'module str {
    let needle = format!(" @{symbol}(");
    let start = module
        .match_indices(&needle)
        .find_map(|(at, _)| {
            let line = module[..at].rfind('\n').map_or(0, |newline| newline + 1);
            module[line..at].starts_with("define").then_some(line)
        })
        .unwrap_or_else(|| panic!("the module must define {symbol}"));
    let end = module[start..]
        .find("\n}\n")
        .map(|offset| start + offset + 2)
        .unwrap_or_else(|| panic!("the definition of {symbol} must close"));
    &module[start..end]
}

/// Whether one printed line opens a basic block.
///
/// Instructions are indented; a block label and the enclosing `define` are
/// not. Distinguishing them is all the block structure these gates need.
fn opens_a_block(line: &str) -> bool {
    !line.starts_with([' ', '\t'])
        && !line.starts_with("define")
        && !line.starts_with('}')
        && line.contains(':')
}

/// The basic block of `function` that holds the first occurrence of `needle`.
///
/// [QUAL-3] scopes its "no allocation, no copy of the transferred data, no
/// lock, no per-call signal operation" requirement to the transfer *path*, not
/// to the whole program: `wfgrep` legitimately copies bytes elsewhere, in
/// `host_copy_bytes` and in its own batch append. Reading the block that holds
/// the host call is what makes the scoped claim checkable instead of
/// approximate.
fn basic_block<'function>(function: &'function str, needle: &str) -> &'function str {
    let hit = function
        .find(needle)
        .unwrap_or_else(|| panic!("{needle} must appear in the function"));
    let mut start = 0;
    let mut end = function.len();
    let mut offset = 0;
    let mut passed = false;
    for line in function.split_inclusive('\n') {
        if opens_a_block(line) {
            if passed {
                end = offset;
                break;
            }
            start = offset;
        }
        if offset <= hit && hit < offset + line.len() {
            passed = true;
        }
        offset += line.len();
    }
    &function[start..end]
}

/// The symbol one printed instruction calls, if it calls one.
///
/// The callee is the first `@name(` on the line: a symbol operand such as a
/// trap record or a constant appears only after it.
fn call_target(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if !(trimmed.starts_with("call ")
        || trimmed.starts_with("tail call ")
        || trimmed.contains(" = call ")
        || trimmed.contains(" = tail call "))
    {
        return None;
    }
    let at = trimmed.find('@')? + 1;
    let rest = &trimmed[at..];
    let end = rest.find(|character: char| {
        !(character.is_ascii_alphanumeric() || character == '_' || character == '.')
    })?;
    rest[end..].starts_with('(').then(|| &rest[..end])
}

/// Every symbol one region of a module calls.
fn call_targets(region: &str) -> Vec<&str> {
    region.lines().filter_map(call_target).collect()
}

/// How many calls one region of a module makes.
fn calls(region: &str) -> usize {
    call_targets(region).len()
}

/// The one semantic identity of each operation `wfgrep` uses, and the approved
/// implementation [QUAL-1] fixes for it on the native command target.
///
/// The three text-facing routes — `host_bytes_len`, `host_utf8_len`, and
/// `host_copy_utf8` — are deliberately absent: `wfgrep` reaches its pattern
/// and its paths only through the raw byte route.
const SELECTED_ROWS: &[(u32, &str)] = &[
    (0, "args_count"),
    (1, "arg_get"),
    (3, "host_copy_bytes"),
    (6, "relative_path"),
    (7, "open_read"),
    (8, "read_once"),
    (9, "write_once"),
    (10, "exit_status"),
];

/// §9.1 row 1 — target selection is one link-time table decision.
#[test]
fn target_selection_is_one_link_time_table_decision() {
    let module = emitted();
    for (identity, operation) in SELECTED_ROWS {
        let selection = format!(
            "; QUAL-1 semantic id {identity} -> @wf.sys.{operation}.v1 implementation version 1"
        );
        assert_eq!(
            module.matches(&selection).count(),
            1,
            "{operation} must resolve to exactly one approved implementation at compile time"
        );
    }
    // Nothing else was selected: the table is consulted once per operation the
    // program uses, not once per call and not for operations it never reaches.
    assert_eq!(
        module.matches("; QUAL-1 semantic id ").count(),
        SELECTED_ROWS.len(),
        "the program selects exactly the operations it uses"
    );
    // No runtime operation switch, target tag, or per-call dispatch table
    // [QUAL-3].
    assert!(!module.contains("@wf.sys.dispatch"));
    assert!(!module.contains("@wf.sys.table"));
    assert!(!module.contains("@wf.sys.target.tag"));
    // And nothing selects an implementation at run time in the finished
    // program: every call in the entry names a symbol.
    for indirect in [
        "call i64 %",
        "call i32 %",
        "call i8 %",
        "call void %",
        "call ptr %",
    ] {
        assert!(
            !entry().contains(indirect),
            "an indirect call would be a run-time selection:\n{indirect}"
        );
    }
}

/// §9.1 row 2 — a selected argument is one inline pointer/length lease over
/// immutable command backing.
#[test]
fn an_argument_lease_allocates_nothing_and_copies_no_byte() {
    let row = approved_row(emitted(), "wf.sys.arg_get.v1");
    // The lease is a bounds test, one slot load out of the invocation vector,
    // one length pass, and a `{ptr, i64}` pair built in registers.
    assert_eq!(calls(row), 1, "the lease makes one length pass:\n{row}");
    assert!(row.contains("call i64 @strlen(ptr %text)"));
    assert!(row.contains("%slot = getelementptr inbounds ptr, ptr %base, i64 %position"));
    assert!(row.contains("%lease.value = insertvalue { ptr, i64 } %lease.base, i64 %length, 1"));
    for forbidden in [
        "@calloc",
        "@malloc",
        "@realloc",
        "memcpy",
        "memmove",
        "@wf.sys.handle",
        "@wf.sys.slot",
    ] {
        assert!(
            !row.contains(forbidden),
            "an argument lease must not reach {forbidden}:\n{row}"
        );
    }
    // In the finished program the lease is gone as a call entirely: three
    // source `arg_get` sites leave three length passes and no other trace.
    assert_eq!(entry().matches("@strlen(").count(), 3);
}

/// §9.1 rows 3 and 4 — the raw byte route is a length pass plus a
/// caller-buffer copy, with no Unicode gate in front of it.
#[test]
fn the_raw_byte_route_carries_no_unicode_gate() {
    let row = approved_row(emitted(), "wf.sys.host_copy_bytes.v1");
    // The range is validated and traps before the source is read or the
    // destination is written; the length is compared against the caller's
    // capacity; the accepted case is one copy into the caller's buffer.
    assert!(row.contains("call void @wf_trap(ptr %record, i64 %record.length)"));
    assert!(row.contains("%room = icmp ule i64 %required, %capacity"));
    assert_eq!(
        row.matches("@llvm.memcpy").count(),
        1,
        "the lossless route copies once, into the caller's buffer:\n{row}"
    );
    for forbidden in ["@calloc", "@malloc", "@realloc", "utf8", "@wf.sys.io.error"] {
        assert!(
            !row.contains(forbidden),
            "the raw route must not reach {forbidden}:\n{row}"
        );
    }
    // The text-facing routes are not merely unused here — they contribute no
    // code at all to a program that never asks for text, so no argument or
    // path in `wfgrep` passes through a Unicode check on its way to source
    // [SYS-2, §3.3].
    for text_route in ["host_utf8_len", "host_copy_utf8", "host_bytes_len"] {
        assert!(
            !emitted().contains(&format!("@wf.sys.{text_route}")),
            "{text_route} must contribute nothing to a raw-route program"
        );
    }
}

/// §9.1 row 5 — `RelativePath` construction is validation and a type
/// transition over the consumed lease.
#[test]
fn relative_path_retypes_the_lease_without_allocating() {
    let row = approved_row(emitted(), "wf.sys.relative_path.v1");
    // Validation is a leading-separator test and one embedded-NUL scan.
    assert!(row.contains("%rooted = icmp eq i32 %first.value, 47"));
    assert!(row.contains("%embedded = call ptr @memchr(ptr %text, i32 0, i64 %length)"));
    assert_eq!(calls(row), 1, "validation is one scan:\n{row}");
    // Success carries the *same* lease value out under the new type. Nothing
    // is allocated, nothing is copied, and no native unit is exposed.
    assert!(
        row.contains("%ok = insertvalue %wf.t16 %ok.tag, { ptr, i64 } %value, 1"),
        "success must retype the consumed lease itself:\n{row}"
    );
    for forbidden in ["@calloc", "@malloc", "@realloc", "memcpy", "memmove"] {
        assert!(
            !row.contains(forbidden),
            "path construction must not reach {forbidden}:\n{row}"
        );
    }
}

/// §9.1 row 6 — `open_read` is one direct native open-relative operation on
/// the capability's own descriptor.
#[test]
fn open_read_is_one_direct_relative_open_on_the_capabilitys_own_descriptor() {
    let row = approved_row(emitted(), "wf.sys.open_read.v1");
    // The directory descriptor is the capability's; the path pointer is the
    // lease itself. No concatenation, no ambient working-directory lookup.
    assert!(row.contains("%text = extractvalue { ptr, i64 } %path, 0"));
    assert!(row.contains("call i32 (i32, ptr, i32, ...) @openat(i32 %root, ptr %text, i32 0)"));
    for forbidden in [
        "@calloc",
        "@malloc",
        "memcpy",
        "@getcwd",
        "@chdir",
        "@realpath",
    ] {
        assert!(
            !row.contains(forbidden),
            "the open path must not reach {forbidden}:\n{row}"
        );
    }
    // In the finished program the single source `open_read` site is one direct
    // `openat` against the bound `DirectoryRead`, and the only other open is
    // the bootstrap's one-time acquisition of the initial directory [QUAL-3].
    let program = entry();
    assert_eq!(program.matches("@openat(").count(), 1);
    assert_eq!(program.matches("@open(").count(), 1);
    assert!(program.contains("@open(ptr nonnull @.wf.sys.working.directory"));
    let opening = basic_block(program, "@openat(");
    assert_eq!(
        calls(opening),
        1,
        "the open is the only call on its path:\n{opening}"
    );
    // Its failure arm is the cold mapper, reached only when the open failed.
    assert!(
        opening.contains("%cwd"),
        "the dirfd is the capability's own"
    );
}

/// §9.1 row 7 — `read_once` and `write_once` are bounds checks, at most one
/// host transfer, one count check, and a cold outcome mapper.
#[test]
fn each_transfer_is_one_host_call_with_a_cold_outcome_mapper() {
    let module = emitted();
    for (operation, facility, empty) in [
        (
            "wf.sys.read_once.v1",
            "@read(",
            "%vacant = icmp eq i64 %capacity, 0",
        ),
        (
            "wf.sys.write_once.v1",
            "@write(",
            "%vacant = icmp eq i64 %count, 0",
        ),
    ] {
        let row = approved_row(module, operation);
        // The range is validated first and traps before any host action
        // [SYS-8].
        assert!(row.contains("%wrapped = icmp ult i64 %end, %offset"));
        assert!(row.contains("call void @wf_trap(ptr %record, i64 %record.length)"));
        // A zero-length range issues no host call at all.
        assert!(
            row.contains(empty),
            "{operation} must short-circuit an empty range:\n{row}"
        );
        // Exactly one host transfer, and one check of the count it reported.
        assert_eq!(
            row.matches(facility).count(),
            1,
            "{operation} is at most one host attempt:\n{row}"
        );
        assert!(row.contains("%progress = icmp sgt i64"));
        for forbidden in [
            "@calloc",
            "@malloc",
            "@realloc",
            "memcpy",
            "memmove",
            "@pthread_mutex_lock",
            "@flockfile",
            "@fwrite",
            "@sigaction",
            "@sigprocmask",
            "@signal(",
        ] {
            assert!(
                !row.contains(forbidden),
                "the transfer path must not reach {forbidden}:\n{row}"
            );
        }
    }
    // The [SYS-7] class mapper is emitted cold, and it is the only thing the
    // failure arms reach.
    assert!(module.contains("@wf.sys.io.error(i32 %code, i8 %origin) noinline cold"));

    let program = entry();
    // No wrapper residue: qualification requires the compiler wrapper to be
    // inlined [QUAL-3], and in the finished program not one survives.
    assert!(
        !program.contains("@wf.sys."),
        "every approved-implementation wrapper must be inlined into the entry"
    );
    // The cold mapper is not merely marked cold: the optimizer moved every
    // one of its call sites out of the entry into outlined cold functions, so
    // the thirty-class mapping is unreachable on a successful transfer.
    assert!(!program.contains("@wf.sys.io.error("));
    assert!(
        optimized().matches("@wf.sys.io.error(").count() > 1,
        "the mapper must still exist for the failure paths"
    );
    // One host call site per source operation: one `read_once`, five
    // `write_once`, two of them to standard output and three to standard
    // error, which are separate owners and stay separate descriptors.
    assert_eq!(program.matches("@read(").count(), 1);
    assert_eq!(program.matches("@write(").count(), 5);
    assert_eq!(program.matches("@write(i32 1,").count(), 2);
    assert_eq!(program.matches("@write(i32 2,").count(), 3);
    // Each of those calls is alone on its path: the block that holds it
    // computes an address and makes one call, so nothing allocates, copies the
    // transferred bytes, takes a lock, or touches a signal disposition beside
    // the transfer [QUAL-3].
    for site in ["@read(", "@write(i32 1,", "@write(i32 2,"] {
        let block = basic_block(program, site);
        assert_eq!(
            calls(block),
            1,
            "{site} must be alone on its path:\n{block}"
        );
    }
    // One-time normalization belongs to the bootstrap, not to any transfer.
    assert_eq!(program.matches("@signal(").count(), 1);
    assert!(program.contains("@signal(i32 13,"));
    // No lock, no buffered-stdio layer, no reallocation anywhere in the
    // program's own code.
    for forbidden in [
        "@pthread_mutex_lock",
        "@os_unfair_lock_lock",
        "@flockfile",
        "@funlockfile",
        "@fwrite",
        "@fputs",
        "@setvbuf",
        "@realloc",
        "@sigaction",
        "@sigprocmask",
        "@memcpy(",
        "@memmove(",
    ] {
        assert!(
            !program.contains(forbidden),
            "wfgrep must not reach {forbidden}"
        );
    }
}

/// §9.1 row 8 — a closing owner releases with at most one direct native close
/// attempt, and an ambiguous close is never retried.
#[test]
fn every_release_close_is_one_discarded_attempt() {
    let program = entry();
    // Only the two closing owners close: `DirectoryRead` and `ReadFile`. The
    // program holds both, so both appear.
    let closes = program.matches("@close(").count();
    assert!(closes >= 2, "both closing owners must release:\n{program}");
    // Every close result is named once and never read again. Nothing compares
    // it, branches on it, or feeds it to a retry: the diagnostic is discarded,
    // which is what makes "never retry an ambiguous fd close" a property of
    // the emitted code rather than a convention [SYS-5].
    let mut inspected = 0;
    for line in program.lines() {
        let trimmed = line.trim_start();
        if !trimmed.contains("@close(") {
            continue;
        }
        let name = trimmed
            .split_once(" = ")
            .map(|(name, _)| name)
            .unwrap_or_else(|| panic!("a close result must be named:\n{line}"));
        assert_eq!(
            program.matches(&format!("{name} ")).count()
                + program.matches(&format!("{name},")).count()
                + program.matches(&format!("{name})")).count(),
            1,
            "the close diagnostic must be discarded, not inspected: {name}"
        );
        inspected += 1;
    }
    assert_eq!(inspected, closes);
}

/// §9.1 rows 9 and 10 — the value releases and the `Output` release reach no
/// host facility at all.
#[test]
fn releasing_a_value_or_an_output_reaches_no_host_facility() {
    let program = entry();
    // `Args`, `HostString`, `RelativePath`, and `ExitStatus` release by
    // logical consume. `Output` releases by logical source detach: no close,
    // no flush, no target call. Standard output and standard error are
    // descriptors 1 and 2 here, and neither is ever closed or flushed —
    // operating-system process teardown owns them [SYS-12].
    for forbidden in [
        "@close(i32 1)",
        "@close(i32 2)",
        "@fflush",
        "@fclose",
        "@fsync",
        "@fdatasync",
        "@shutdown",
    ] {
        assert!(
            !program.contains(forbidden),
            "an Output release must not reach {forbidden}"
        );
    }
    // The complete inventory of what the finished program calls. Every entry
    // is accounted for by a first-slice operation, by a required check, or by
    // the compiler's own runtime; nothing here is a release reaching a host
    // facility, a handle table, or a hidden external effect [§12.2]. A release
    // that started making a target call would have to add a name to this list.
    for target in call_targets(program) {
        let accounted = matches!(
            target,
            // The bootstrap's one-time working-directory acquisition and
            // signal normalization [QUAL-3].
            "open" | "signal"
            // The first slice's own host operations.
            | "openat" | "read" | "write" | "close"
            // The lease length pass and the path NUL scan.
            | "strlen" | "memchr"
            // Buffer allocation and language cleanup.
            | "calloc" | "free"
            // Required checks and the mandatory diagnostic record.
            | "wf_trap" | "abort"
        ) || target.starts_with("llvm.")
            // The optimizer's own cold outlining of the failure arms, which is
            // where the [SYS-7] class mapper ended up.
            || target.starts_with("main.cold.");
        assert!(
            accounted,
            "wfgrep calls @{target}, which no first-slice row accounts for"
        );
    }
}

/// §9.1 row 12 — one buffer initialization on allocation, then reuse across
/// every read and every flush.
///
/// This is the *initialized* control: it answers "does initialization happen
/// once", which is a structural question. Whether paying for initialization at
/// all is material is the separate §9.1 row 13 question, which only an
/// uninitialized control can answer, and which lives in
/// `research/experiments/buffer-initialization-cost/`.
#[test]
fn the_reused_buffers_are_initialized_once_at_allocation() {
    let program = entry();
    // `wfgrep` asks for exactly four buffers — input, batch, pattern, and the
    // diagnostic report — and gets exactly four allocations, each carrying its
    // own initialization in the allocation itself.
    assert_eq!(
        program.matches("@calloc(").count(),
        4,
        "four source buffers, four allocations"
    );
    for size in ["@calloc(i64 1, i64 4096)", "@calloc(i64 1, i64 512)"] {
        assert!(program.contains(size), "missing allocation {size}");
    }
    assert_eq!(program.matches("@calloc(i64 1, i64 4096)").count(), 3);
    assert_eq!(program.matches("@calloc(i64 1, i64 512)").count(), 1);
    // Nothing reallocates, and nothing re-initializes: there is no fill loop,
    // no `memset`, and no second allocation anywhere in the module, so a
    // per-read re-initialization or a reallocation after a flush cannot be
    // hiding in a path this test did not walk.
    for forbidden in ["@malloc(", "@realloc(", "@reallocf(", "memset", "bzero"] {
        assert!(
            !optimized().contains(forbidden),
            "the reused buffers must not reach {forbidden}"
        );
    }
    // All four allocations precede every host transfer in the entry, and no
    // allocation follows one. Combined with the exact count above, that places
    // every initialization in the program's one-time prologue rather than in
    // the drain, the match loop, or the flush.
    let last_allocation = program
        .rfind("@calloc(")
        .expect("the program allocates its buffers");
    for transfer in ["@openat(", "@read(", "@write("] {
        let first = program
            .find(transfer)
            .unwrap_or_else(|| panic!("{transfer} must appear"));
        assert!(
            last_allocation < first,
            "every allocation must precede the first {transfer}"
        );
    }
}

/// §9.1 row 11 — one reusable fixed buffer, normally one host write per full
/// batch.
///
/// This row is a count over a real run, not a shape: "reject syscall-per-match
/// and buffer reinitialization after flush" is a statement about how many host
/// attempts a run makes for how many matches. Task 0013's deterministic host
/// is what makes that observable, because it traces every attempt against
/// every descriptor.
#[test]
fn the_output_batch_costs_one_host_write_per_full_batch() {
    // Three thousand matching lines of two bytes each: six thousand bytes of
    // output through a four-thousand-and-ninety-six-byte batch.
    const MATCHES: u64 = 3_000;
    let mut fixture = Vec::new();
    for _ in 0..MATCHES {
        fixture.extend_from_slice(b"x\n");
    }

    let run = run_on_deterministic_host(
        WFGREP,
        &HostScript::new().file(&fixture),
        &[b"x", b"lines.txt"],
    );
    assert_eq!(
        run.output.status.code(),
        Some(0),
        "every line matches; trace was {:?}",
        run.trace()
    );

    // The host echoes each accepted payload into its own trace, and this
    // program's payload is lines, so a trace entry is not reliably a trace
    // *line*. Counting occurrences of each marker instead is exact here
    // because no marker can occur inside a payload of `x` and newline.
    let trace = run.trace();
    let published: Vec<u64> = trace
        .match_indices("wf_test write fd=1 count=")
        .map(|(at, marker)| {
            trace[at + marker.len()..]
                .split_whitespace()
                .next()
                .expect("a write trace carries its requested count")
                .parse()
                .expect("a requested count is a number")
        })
        .collect();

    // Two host writes for three thousand matches: the batch fills at 4096
    // bytes and is published whole, then the final partial batch is published
    // on the way out. A syscall-per-match implementation would show three
    // thousand here, which is exactly what §12.2 rejects.
    assert_eq!(published, vec![4_096, 1_904], "trace was {trace:?}");
    assert!(published.len() as u64 * 1_000 < MATCHES);

    // The batch is not refilled or re-initialized after a flush: the second
    // publication starts from an empty batch and carries only what followed
    // the first, so the two requested counts sum to exactly the output the
    // matches produced and no byte is published twice.
    assert_eq!(
        published.iter().sum::<u64>(),
        fixture.len() as u64,
        "trace was {trace:?}"
    );

    // The same run witnesses the release rows on the real program, which the
    // structural gates above can only establish statically. Exactly two closes
    // fire — one for the `DirectoryRead` and one for the `ReadFile`, each
    // attempted once — and neither `Output` is closed or flushed, because its
    // release is a logical source detach [SYS-5, SYS-12].
    assert_eq!(
        trace.matches("wf_test close ").count(),
        2,
        "trace was {trace:?}"
    );
    assert_eq!(trace.matches("wf_test close fd=41 outcome=ok").count(), 1);
    assert_eq!(trace.matches("wf_test close fd=42 outcome=ok").count(), 1);
    assert!(!trace.contains("wf_test close fd=1 "));
    assert!(!trace.contains("wf_test close fd=2 "));
    // Three reads for a two-chunk file: two that delivered bytes and one that
    // observed the end. One host attempt per source `read_once`, never a retry
    // and never a second attempt to confirm the end [SYS-8]. Nothing was ever
    // published to standard error: the successful path reports nothing.
    // And the whole run is the §12.2 per-byte-call rejection, measured rather
    // than argued: six thousand bytes in and six thousand bytes out cost five
    // host calls altogether — three reads and two writes — plus one open, one
    // openat, and two closes. Not one host call per byte, per line, per match,
    // or per field.
    assert_eq!(trace.matches("wf_test ").count(), 9, "trace was {trace:?}");
    assert_eq!(
        trace.matches("wf_test read fd=42 ").count(),
        3,
        "trace was {trace:?}"
    );
    assert!(!trace.contains("wf_test write fd=2 "));
}
