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
//! control flow of a seven-declaration, seven-hundred-line command. Two levels
//! of evidence appear here for that reason:
//!
//! 1. the emitted body of each [QUAL-1]-selected approved implementation, in
//!    `wfgrep`'s own module — what the compiler emits for the row; and
//! 2. `wfgrep`'s own emitted code — what survives of it in a real program.
//!
//! Level 2 read `main` alone until task 0021, because until then every one of
//! `wfgrep`'s helpers was inlined into it. Borrow-mode parameters of system
//! types then let the five copies of the write-until-accepted loop become one
//! `publish_all`, and that function costs 245 against the host inliner's 225
//! threshold at the shipped level, so it stays out of line at four of its five
//! call sites. [QUAL-3]'s inlining condition is scoped to the *compiler
//! wrapper* of an approved implementation — still inlined at every site, and
//! still asserted below — and places no such requirement on an ordinary
//! Whitefoot function. So level 2 now reads the program's own code: `main`
//! plus whichever declared functions survive as separate definitions.
//!
//! Task 0016's closure recorded the rule that governs that move: transfer-site
//! counts equal *source*-site counts, and a change in what the optimizer does
//! with a site requires re-deriving the gate from source, never relaxing it.
//! The one row this touches is the publication count. `wfgrep` used to have
//! five source `write_once` sites and now has one, reached from five source
//! publications, so the count that was five emitted `@write` sites is now five
//! publication entries — two naming standard output and three naming standard
//! error, exactly as before — counted in whichever of the two forms the
//! inliner left each site in. Nothing became an inequality and no row lost its
//! claim.
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
//! byte-at-a-time loop whose bounds obligation is statically discharged, and the `<16 x i8>`
//! vector operations in the module belong to the boundary-carry shift and the
//! batch append, not to the scan. No §9.1 row requires a `memchr`, and §12.2's
//! per-byte-call rejection is satisfied — the deterministic-host run below
//! moves six thousand bytes with five host calls in total — so this is a
//! correction to a note, not a defect. A future scan-recognition change is a
//! performance question for its own attributed slice.

use std::sync::OnceLock;

use super::deterministic_target::{HostScript, run_emitted_on_deterministic_host};
use super::system::with_ir_for;
use super::{host_optimized_module, optimized_main};
use crate::backend::emit_llvm;
use crate::backend::emitter::emit_llvm_for_target;
use crate::backend::qualification::SystemTarget;

/// The first real Whitefoot program (task 0015), following the dossier's
/// §10.1 witness trace step for step. It is the anchor for every row below:
/// a cost-shape gate on a program written for the gate proves nothing about
/// the program the project actually has to compile.
const WFGREP: &[u8] = include_bytes!("../../../../tests/programs/wfgrep.wf");

/// `wfgrep`'s two emitted modules — the host target's and the deterministic
/// test target's — from one shared front-end pass. The checked program is
/// identical for both targets; only the emission call differs, and a second
/// full analysis of the same bytes was measured at 136s of pure setup. The
/// module's own shape assertions are the differential guarding this merge.
fn modules() -> &'static (String, String) {
    static MODULES: OnceLock<(String, String)> = OnceLock::new();
    MODULES.get_or_init(|| {
        with_ir_for(WFGREP, crate::Inventory::OpenByName, |program| {
            (
                emit_llvm(program)
                    .expect("lowered program must emit")
                    .into_string(),
                emit_llvm_for_target(program, SystemTarget::deterministic_test())
                    .expect("the deterministic test target admits the program")
                    .into_string(),
            )
        })
    })
}

/// `wfgrep`'s module as the backend emits it.
fn emitted() -> &'static str {
    &modules().0
}

/// `wfgrep`'s module as the host optimizer leaves it at the shipped level.
fn optimized() -> &'static str {
    static MODULE: OnceLock<String> = OnceLock::new();
    MODULE.get_or_init(|| host_optimized_module(emitted()))
}

/// `wfgrep`'s optimized entry.
fn entry() -> &'static str {
    optimized_main(optimized())
}

/// Every function `wfgrep` declares, in source order.
///
/// Task 0023 re-derived this list from source when the double-walk slice
/// fused the literal match into `main`'s scan walk: `line_matches` is no
/// longer declared. Per task 0016's rule the list moves only by source
/// derivation, never by relaxation.
const DECLARED_FUNCTIONS: &[&str] = &[
    "io_class",
    "append_slice",
    "copy_range",
    "publish_all",
    "byte_at",
    "name_before",
    "put_decimal",
    "report_failure",
    "search_file",
    "walk",
];

/// `wfgrep`'s own emitted code, function by function: the optimized entry
/// plus every declared function the host inliner left standing.
///
/// Value names are function-local, so any check that reads a named result
/// walks this list rather than the concatenation below.
fn program_functions() -> &'static [&'static str] {
    static FUNCTIONS: OnceLock<Vec<&'static str>> = OnceLock::new();
    FUNCTIONS.get_or_init(|| {
        let module = optimized();
        let mut functions = vec![entry()];
        for name in DECLARED_FUNCTIONS {
            let needle = format!(" @wf_{name}(");
            if module
                .match_indices(&needle)
                .any(|(at, _)| definition_start(module, at).is_some())
            {
                functions.push(approved_row(module, &format!("wf_{name}")));
            }
        }
        functions
    })
}

/// `wfgrep`'s own emitted code as one region, for the checks that count
/// symbols rather than read named results.
fn program() -> &'static str {
    static PROGRAM: OnceLock<String> = OnceLock::new();
    PROGRAM.get_or_init(|| program_functions().join("\n"))
}

/// The start of the `define` line that introduces `symbol` at `at`, if that
/// occurrence is a definition rather than a call.
fn definition_start(module: &str, at: usize) -> Option<usize> {
    let line = module[..at].rfind('\n').map_or(0, |newline| newline + 1);
    module[line..at].starts_with("define").then_some(line)
}

/// The first argument of the call to `callee` on one printed instruction.
///
/// Scans at paren depth so that a `range(i32 1, 3)` annotation inside the
/// operand does not end it early.
fn first_argument<'line>(line: &'line str, callee: &str) -> Option<&'line str> {
    let open = line.find(&format!("@{callee}("))? + callee.len() + 2;
    let rest = &line[open..];
    let mut depth = 0_usize;
    for (offset, character) in rest.char_indices() {
        match character {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' if depth == 0 => return Some(rest[..offset].trim()),
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => return Some(rest[..offset].trim()),
            _ => {}
        }
    }
    None
}

/// The standard descriptor each of `wfgrep`'s publications names.
///
/// A publication is one source `publish_all` call. In the emitted program each
/// appears in exactly one of two forms: a call into the out-of-line
/// `publish_all`, whose first argument is the descriptor, or — where the host
/// inliner expanded that site — the `@write` of the expanded copy, carrying
/// the same literal descriptor. The out-of-line body's own `@write` names the
/// parameter instead of a literal and is not a publication site; it is the one
/// transfer the one source `write_once` site emits.
fn publications() -> Vec<u32> {
    let mut descriptors = Vec::new();
    for line in program().lines() {
        let Some(target) = call_target(line) else {
            continue;
        };
        let argument = match target {
            "wf_publish_all" | "write" => first_argument(line, target),
            _ => None,
        };
        let Some(argument) = argument else {
            continue;
        };
        if let Some(descriptor) = argument
            .split_whitespace()
            .next_back()
            .and_then(|token| token.parse::<u32>().ok())
        {
            descriptors.push(descriptor);
        }
    }
    descriptors
}

/// The emitted body of one definition, by symbol.
fn approved_row<'module>(module: &'module str, symbol: &str) -> &'module str {
    let needle = format!(" @{symbol}(");
    let start = module
        .match_indices(&needle)
        .find_map(|(at, _)| definition_start(module, at))
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
    // The [SYS-14] traversal surface and the [SYS-11] file-open-by-name
    // candidate, which the recursive search reaches: the root and every child
    // directory by name, one enumeration per directory, and every regular file
    // by its enumerated name. `open_read` stays on the list because the search
    // still takes the path route when its root names a single file.
    (11, "open_directory"),
    (12, "open_list"),
    (13, "list_once"),
    (14, "open_file"),
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
        let row = approved_row(module, &format!("wf.sys.{operation}.v1"));
        assert!(
            !row.contains("@wf_trap"),
            "{operation} must consume statically discharged domains without a runtime trap"
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
    // program: every call the program makes names a symbol.
    for indirect in [
        "call i64 %",
        "call i32 %",
        "call i8 %",
        "call void %",
        "call ptr %",
    ] {
        assert!(
            !program().contains(indirect),
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
    assert_eq!(program().matches("@strlen(").count(), 3);
}

/// §9.1 rows 3 and 4 — the raw byte route is a length pass plus a
/// caller-buffer copy, with no Unicode gate in front of it.
#[test]
fn the_raw_byte_route_carries_no_unicode_gate() {
    let row = approved_row(emitted(), "wf.sys.host_copy_bytes.v1");
    // The two range obligations were discharged at the source call. The
    // wrapper therefore starts directly from the authorized half-open extent;
    // only the recoverable source-length fit remains dynamic.
    assert!(row.contains("%extent = sub nuw i64 %end, %start"));
    assert!(row.contains("%room = icmp ule i64 %required, %extent"));
    assert!(
        !row.contains("@wf_trap"),
        "a system wrapper must not reintroduce a runtime range trap:\n{row}"
    );
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
    // is allocated, nothing is copied, and no native unit is exposed. The
    // outcome type's ordinal is the module's own numbering, which every
    // declared type in the program shifts, so only the retyping itself is
    // pinned here.
    assert!(
        row.contains("%ok = insertvalue %wf.t") && row.contains("%ok.tag, { ptr, i64 } %value, 1"),
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
    // In the finished program every source open site is one direct `openat`
    // against a bound `DirectoryRead`, and the only other open is the
    // bootstrap's one-time acquisition of the initial directory [QUAL-3].
    // The search has exactly five open sites, derived from the source and not
    // from the module: `main` opens the search root with `open_directory` and,
    // when that root is not a directory, the same name with `open_read`;
    // `walk` opens one enumeration with `open_list`, each child directory with
    // `open_directory`, and each regular file with `open_file`.
    assert_eq!(program().matches("@openat(").count(), 5);
    assert_eq!(program().matches("@open(").count(), 1);
    assert!(program().contains("@open(ptr nonnull @.wf.sys.working.directory"));
    let opening = basic_block(entry(), "@openat(");
    let host_calls: Vec<_> = call_targets(opening)
        .into_iter()
        .filter(|target| !target.starts_with("llvm."))
        .collect();
    assert_eq!(
        host_calls,
        vec!["openat"],
        "the open is the only host call on its path:\n{opening}"
    );
    // The entry's first open is the search root's, which the name route
    // reaches, so its block also holds the one bounded copy that terminates
    // the validated component for the facility [SYS-14]. That copy is an
    // intrinsic over a fixed-size stack slot, not a second host call and not a
    // copy of anything the open transfers.
    assert_eq!(
        opening.matches("@llvm.memcpy").count(),
        1,
        "the name route copies the component once:\n{opening}"
    );
    // Its failure arm is the cold mapper, reached only when the open failed.
    assert!(
        opening.contains("%cwd"),
        "the dirfd is the capability's own"
    );
}

/// §9.1 row 7 — `read_once` and `write_once` consume statically authorized
/// ranges, make at most one host transfer, sanitize one host count into an
/// absolute endpoint, and use a cold outcome mapper.
#[test]
fn each_transfer_is_one_host_call_with_a_cold_outcome_mapper() {
    let module = emitted();
    for (operation, facility) in [
        ("wf.sys.read_once.v1", "@read("),
        ("wf.sys.write_once.v1", "@write("),
    ] {
        let row = approved_row(module, operation);
        // SYS-8's two source obligations authorize `sub nuw`; the wrapper has
        // no range-validation branch or language trap fallback.
        assert!(row.contains("%extent = sub nuw i64 %end, %start"));
        assert!(
            !row.contains("@wf_trap"),
            "a system transfer must not retain a runtime range trap:\n{row}"
        );
        // A zero-length range issues no host call at all.
        assert!(
            row.contains("%vacant = icmp eq i64 %extent, 0"),
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

    let program = program();
    // No wrapper residue: qualification requires the compiler wrapper to be
    // inlined [QUAL-3], and in the finished program not one survives.
    assert!(
        !program.contains("@wf.sys."),
        "every approved-implementation wrapper must be inlined into the program"
    );
    // The cold mapper is not merely marked cold: the optimizer moved every
    // one of its call sites out of the program's own code into outlined cold
    // functions, so the thirty-class mapping is unreachable on a successful
    // transfer.
    assert!(!program.contains("@wf.sys.io.error("));
    assert!(
        optimized().matches("@wf.sys.io.error(").count() > 1,
        "the mapper must still exist for the failure paths"
    );
    // One host transfer per source operation. `wfgrep` writes `read_once` and
    // `write_once` once each, so the emitted program holds one `@read` and one
    // `@write` for each surviving copy of the body that holds them: the one
    // source `read_once` site in `search_file`, and the one source
    // `write_once` site in `publish_all`, whose out-of-line definition the
    // inliner left standing.
    assert_eq!(program.matches("@read(").count(), 1);
    assert_eq!(
        program.matches("@write(").count(),
        1,
        "one transfer per surviving copy of the one source write_once site"
    );
    // Every publication whose destination the optimizer resolved to a literal
    // descriptor. Derived from source: `search_file` publishes twice to the
    // standard-output owner — one flush of a full batch and one of the
    // remainder — and the standard-error owner is reached by
    // `report_failure`'s one assembled diagnostic plus `main`'s two startup
    // diagnostics (general startup failure and an overlong root name). The
    // two owners are separate and stay separate descriptors [SYS-12].
    let published = publications();
    assert_eq!(published.iter().filter(|fd| **fd == 1).count(), 2);
    assert_eq!(published.iter().filter(|fd| **fd == 2).count(), 3);
    // Each transfer is alone on its path: the block that holds it computes an
    // address and makes one call, so nothing allocates, copies the transferred
    // bytes, takes a lock, or touches a signal disposition beside the transfer
    // [QUAL-3].
    for function in program_functions() {
        for site in ["@read(", "@write("] {
            if !function.contains(site) {
                continue;
            }
            let block = basic_block(function, site);
            assert_eq!(
                calls(block),
                1,
                "{site} must be alone on its path:\n{block}"
            );
        }
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
    // The three closing resource kinds close: `DirectoryRead`, `DirectoryList`,
    // and `ReadFile`. The program holds all three, so all three appear. The
    // emitted `open_file` helper has two mutually exclusive provisional-error
    // cleanup paths, independently locked in `system.rs`; the host optimizer
    // tail-merges those paths into one physical close site in this whole
    // program. That site is distinguished below from resource releases.
    let closes = program().matches("@close(").count();
    assert!(
        closes >= 4,
        "three closing resource kinds and one tail-merged provisional cleanup must appear:\n{}",
        program()
    );
    // Every close result is named once and never read again. Nothing compares
    // it, branches on it, or feeds it to a retry: the diagnostic is discarded,
    // which makes "never retry an ambiguous fd close" a property of emitted
    // code rather than a convention [SYS-5]. This applies equally to normal
    // resource releases and to `open_file`'s tail-merged provisional cleanup.
    // Value names are function-local, so each function is read on its own.
    let mut releases = 0;
    let mut provisional_cleanups = 0;
    for function in program_functions() {
        for line in function.lines() {
            let trimmed = line.trim_start();
            if !trimmed.contains("@close(") {
                continue;
            }
            let name = trimmed
                .split_once(" = ")
                .map(|(name, _)| name)
                .unwrap_or_else(|| panic!("a close result must be named:\n{line}"));
            let occurrences = function.matches(&format!("{name} ")).count()
                + function.matches(&format!("{name},")).count()
                + function.matches(&format!("{name})")).count();
            if name.starts_with("%release.") {
                releases += 1;
            } else {
                assert!(
                    name.starts_with("%inspection.close") || name.starts_with("%kind.close"),
                    "an unclassified close site escaped the release/provisional split: {name}"
                );
                provisional_cleanups += 1;
            }
            assert_eq!(
                occurrences, 1,
                "a close diagnostic must be discarded, not inspected or retried: {name}"
            );
        }
    }
    assert!(
        releases >= 3,
        "all three closing resource kinds must release"
    );
    assert_eq!(provisional_cleanups, 1);
    assert_eq!(releases + provisional_cleanups, closes);
}

/// §9.1 rows 9 and 10 — the value releases and the `Output` release reach no
/// host facility at all.
#[test]
fn releasing_a_value_or_an_output_reaches_no_host_facility() {
    let program = program();
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
    // is accounted for by a first-slice operation, by a written claim, or by
    // the compiler's own runtime; nothing here is a release reaching a host
    // facility, a handle table, or a hidden external effect [§12.2]. A release
    // that started making a target call would have to add a name to this list.
    for target in call_targets(program) {
        let accounted = matches!(
            target,
            // The bootstrap's one-time working-directory acquisition and
            // signal normalization [QUAL-3].
            "open" | "signal"
            // The first slice's own host operations, including the [SYS-14]
            // enumeration facility this target's [QUAL-1] row names.
            | "openat" | "fstat" | "read" | "write" | "close" | "__getdirentries64"
            // Darwin's native error-slot access on failed host operations.
            | "__error"
            // The lease length pass and the path NUL scan.
            | "strlen" | "memchr"
            // Buffer allocation and language cleanup.
            | "calloc" | "free"
            // Written claims and their mandatory diagnostic record.
            | "wf_trap" | "abort"
            // The lane protocol of a permitted overlap group [PAR-1
            // candidate], and the one question its bootstrap asks. The module
            // this census reads is the default compilation, which actualizes
            // nothing and therefore names none of these. They are listed for
            // the `--par` build of the same source, where all five are the
            // module's own weak definitions: the claim refuses every lane and
            // the rest return, `wf__par_pool_active` answers that no pool
            // started — so that build runs its own sequential clone, which
            // reaches no host facility either, and the four protocol entries
            // go unused. That is a statement about the emitted module, not
            // about every link of it — with the parallel runtime linked,
            // `wf__par_pool_active` and `wf__par_claim` both reach
            // `pthread_create`. This census inspects the module, and the pool
            // the runtime adds is trusted computing base below the language,
            // like malloc's own internals. This row is a permission the
            // target may take, not an operation of the first slice, so no
            // §9.1 count moves with it.
            | "wf__par_claim" | "wf__par_publish" | "wf__par_join" | "wf__par_release"
            | "wf__par_pool_active"
        ) || target.starts_with("wf__par_thunk_")
            // The sequential clone of a function on a path to a hand-out: the
            // same body under a reserved symbol, so it calls exactly what its
            // original calls and adds no row here either.
            || target.starts_with("wf__par_seq_")
            || target.starts_with("llvm.")
            // The program's own declared functions, and the optimizer's cold
            // outlining of their failure arms, which is where the [SYS-7]
            // class mapper ended up.
            || DECLARED_FUNCTIONS.iter().any(|name| {
                target == format!("wf_{name}") || target.starts_with(&format!("wf_{name}.cold."))
            })
            // The entry body — `@main` hands the program to the exhaustion
            // floor, which runs this — and the optimizer's cold outlining of
            // its own failure arms.
            || target.starts_with("wf__main_body.cold.");
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
    let program = program();
    // `wfgrep` asks for exactly eleven buffers, and gets exactly eleven
    // allocations, each carrying its own initialization in the allocation
    // itself. Derived from source, function by function: `main` allocates the
    // pattern (4096), the root name (256), the root path (1024), and the
    // diagnostic report (1280); `walk` allocates its enumeration batch (8192),
    // its collected names (65664), its visit order (64 u64 slots, 512
    // bytes), one child path (1024), and its own report (1280); `search_file`
    // allocates the read input (4096) and the publication batch (8192).
    //
    // This is where the recursive search moved the cost: `walk`'s five and
    // `search_file`'s two are per call, so a walk of D directories holding F
    // files allocates 5D + 2F + 4 times, where the argv-list version allocated
    // four times for the whole run. That is a measured property of the new
    // shape, recorded here rather than hidden — it is one of the numbers the
    // flagship re-attribution has to explain.
    assert_eq!(
        program.matches("@calloc(").count(),
        11,
        "eleven source buffers, eleven allocations"
    );
    for (size, count) in [
        ("@calloc(i64 1, i64 4096)", 2),
        ("@calloc(i64 1, i64 8192)", 2),
        ("@calloc(i64 1, i64 1280)", 2),
        ("@calloc(i64 1, i64 1024)", 2),
        ("@calloc(i64 1, i64 65664)", 1),
        ("@calloc(i64 1, i64 512)", 1),
        ("@calloc(i64 1, i64 256)", 1),
    ] {
        assert_eq!(
            program.matches(size).count(),
            count,
            "allocation {size} must appear {count} times"
        );
    }
    // Nothing reallocates, and nothing re-initializes: there is no fill loop,
    // no `memset`, and no second allocation anywhere in the module, so a
    // per-read re-initialization or a reallocation after a flush cannot be
    // hiding in a path this test did not walk. `@malloc` is on the list
    // because a buffer whose fill byte is not zero would take it, and no
    // buffer in this program does.
    for forbidden in ["@malloc(", "@realloc(", "@reallocf(", "memset", "bzero"] {
        assert!(
            !optimized().contains(forbidden),
            "the reused buffers must not reach {forbidden}"
        );
    }
    // Allocation begins in each function's prologue: the first allocation a
    // function makes precedes every host transfer it reaches, so no buffer is
    // first created in the drain, the match loop, or the flush.
    //
    // The stronger claim the argv-list version could make — that the *last*
    // allocation also precedes the first transfer — no longer holds textually,
    // and its failure is inlining rather than re-initialization: the host
    // inliner expands `search_file`'s body into `main`'s single-file arm, so
    // `main` textually holds a callee's prologue allocations after `main`'s own
    // root open. The per-buffer count above is what now carries "one
    // allocation per source buffer"; that a call allocates once rather than per
    // read is a source fact — every buffer is bound at its function's entry —
    // which no inspection of the merged module can restate.
    for function in program_functions() {
        let Some(first_allocation) = function.find("@calloc(") else {
            continue;
        };
        for transfer in ["@openat(", "@read(", "@write(", "@__getdirentries64("] {
            let Some(first) = function.find(transfer) else {
                continue;
            };
            assert!(
                first_allocation < first,
                "allocation must begin before the first {transfer}"
            );
        }
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
    // Three thousand matching lines of two bytes each, published through an
    // eight-thousand-one-hundred-and-ninety-two-byte batch. The scripted host
    // holds one regular file, so the search's root `open_directory` reports
    // `ENOTDIR` and the run reaches the file through the path route — which is
    // exactly the shape this row needs: one file, one read cursor, one batch.
    const MATCHES: u64 = 3_000;
    let mut fixture = Vec::new();
    for _ in 0..MATCHES {
        fixture.extend_from_slice(b"x\n");
    }

    let run = run_emitted_on_deterministic_host(
        &modules().1,
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

    // A handful of host writes for three thousand matches: the batch fills and
    // is published whole, then the remainder is published on the way out. A
    // syscall-per-match implementation would show three thousand here, which
    // is exactly what §12.2 rejects.
    //
    // The exact split is no longer a fixed vector, because a published record
    // now carries the file's path and the line's ordinal, so record lengths
    // differ by the ordinal's digit count and a fixed vector would be a
    // transcription of this fixture rather than a property. What the row
    // asserts instead is the property itself: every batch but the last is
    // published within one record of full, the sum is exactly the output the
    // matches produced with no byte published twice, and the call count is two
    // orders of magnitude below the match count.
    const BATCH: u64 = 8_192;
    let expected: u64 = (1..=MATCHES)
        .map(|ordinal| {
            "lines.txt:".len() as u64 + ordinal.to_string().len() as u64 + ":x\n".len() as u64
        })
        .sum();
    assert!(published.len() >= 2, "trace was {trace:?}");
    assert!(
        (published.len() as u64) * 100 < MATCHES,
        "one write per full batch, not per match: {published:?}"
    );
    // The program flushes when the next record would not fit under its own
    // conservative reservation — the path length plus twenty-four bytes for
    // the ordinal and the two separators, plus the line's own span. Here that
    // is nine plus twenty-four plus two, so a full batch lands within
    // thirty-five bytes of the cap and never above it.
    const RESERVE: u64 = 40;
    for count in &published[..published.len() - 1] {
        assert!(
            *count > BATCH - RESERVE && *count <= BATCH,
            "a flushed batch must be within one record of full: {published:?}"
        );
    }
    // The batch is not refilled or re-initialized after a flush: each
    // publication starts from an empty batch and carries only what followed the
    // previous one, so the requested counts sum to exactly the output the
    // matches produced and no byte is published twice.
    assert_eq!(
        published.iter().sum::<u64>(),
        expected,
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
    // than argued: six thousand bytes in and roughly fifty thousand out cost a
    // dozen host calls altogether. Not one host call per byte, per line, per
    // match, or per field.
    assert!(
        trace.matches("wf_test ").count() < 20,
        "the whole run is a dozen host calls: {trace:?}"
    );
    assert_eq!(
        trace.matches("wf_test read fd=42 ").count(),
        3,
        "trace was {trace:?}"
    );
    // The refused directory open of the one regular file, then the path-route
    // open that reached it.
    assert_eq!(
        trace.matches("wf_test openat root=41 -> notdir").count(),
        1,
        "trace was {trace:?}"
    );
    assert_eq!(
        trace.matches("wf_test openat root=41 fd=42").count(),
        1,
        "trace was {trace:?}"
    );
    assert!(!trace.contains("wf_test write fd=2 "));
}
