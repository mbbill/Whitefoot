//! Actualization of the permission judgment: what a permitted overlap group
//! emits, what an unpermitted pair still emits, what links, and what the
//! program observes.
//!
//! The load-bearing property of this whole path is that overlapping changes
//! nothing observable. A test that only ran the overlapped program would pass
//! just as well against a runtime that never granted a lane, so the runs below
//! read the runtime's own grant count and refuse to accept a repeat that never
//! actually overlapped.
//!
//! Actualization is compile-time opt-in, so every case that expects a hand-out
//! emits through [`emit_with_overlap`], which is what `whitefootc --par`
//! compiles. Plain [`emit`] is the default compilation, and
//! `the_default_compilation_hands_nothing_out` is the case that pins what it
//! leaves out.

use std::path::Path;
use std::process::Command;

use crate::backend::emitter::emit_llvm_for_target;
use crate::backend::qualification::{SystemTarget, qualify_program};
use crate::backend::target::{
    PARALLEL_LANE_FRAME_ALIGNMENT, TargetLayout, TargetLayoutFailure, TargetObject,
    parallel_lane_frame_layout,
};

use super::system::{with_ir, with_parallel_ir};
use super::{
    HOST_OPTIMIZATION_ARGUMENTS, PARALLEL_COMPLETION_RUNTIME_SOURCE, PARALLEL_RUNTIME_SOURCE,
    append_completion_runtime, build_executable, compile_and_run, emit, emit_with_overlap,
    module_requires_parallel_runtime, module_requires_writer_scheduler, test_directory,
};

/// A pure recursive fold over a heap tree, the smallest shape that has
/// every eligible form at once: a self-recursive sibling pair inside `fold`,
/// sibling constructor pairs inside `pair`, `quad`, and `oct`, and a run of
/// four sibling calls in `main`. Its whole result is written to standard
/// output, so a difference anywhere in the tree is a difference in the bytes.
const OVERLAPPING_FOLD: &[u8] = br#"enum Node {
  Leaf(w: u64);
  Branch(left: box<Node>, right: box<Node>, w: u64);
}

fn leaf(w: own u64) -> result: own box<Node> allocates(heap) {
  let node = Leaf(w: w);
  return box_new(move node);
}

fn branch(left: own box<Node>, right: own box<Node>) -> result: own box<Node> allocates(heap) {
  let node = Branch(left: move left, right: move right, w: 0_u64);
  return box_new(move node);
}

fn pair(a: own u64, b: own u64) -> result: own box<Node> allocates(heap) {
  let l = leaf(w: a);
  let r = leaf(w: b);
  return branch(left: move l, right: move r);
}

fn quad(a: own u64, b: own u64, c: own u64, d: own u64) -> result: own box<Node> allocates(heap) {
  let l = pair(a: a, b: b);
  let r = pair(a: c, b: d);
  return branch(left: move l, right: move r);
}

fn oct(a: own u64, b: own u64, c: own u64, d: own u64, e: own u64, f: own u64, g: own u64, h: own u64) -> result: own box<Node> allocates(heap) {
  let l = quad(a: a, b: b, c: c, d: d);
  let r = quad(a: e, b: f, c: g, d: h);
  return branch(left: move l, right: move r);
}

fn mix(a: own u64, b: own u64) -> result: own u64 pure {
  let spun = irotl(a, 13_u32);
  let scattered = imulhi(b, 2654435761_u64);
  let blended = ixor(spun, b);
  return ixor(blended, scattered);
}

fn fold(node: &uniq box<Node>) -> result: own u64 reads(node), writes(node) {
  match deref(deref(node)) {
    Leaf(w: leaf_w) => {
      return deref(leaf_w);
    }
    Branch(left: l, right: r, w: slot) => {
      let a = fold(node: move l);
      let b = fold(node: move r);
      let mixed = mix(a: a, b: b);
      set deref(slot) = mixed;
      return mixed;
    }
  }
}

fn low_byte(v: own u64) -> result: own u8 pure {
  let low = iand(v, 255_u64);
  match cvt::<u64, u8>(low) {
    Ok(value: byte) => {
      return byte;
    }
    Err(error: problem) => {
      return 0_u8;
    }
  }
}

fn spell(destination: &uniq buffer<u8>, at: own u64, value: own u64) -> result: own u64 reads(destination), writes(destination) {
  let cursor = at;
  let rest = value;
  loop @octets {
    let limit = at +wrap 8_u64;
    let done = cursor >= limit;
    if done {
      break @octets;
    }
    let room = len(deref(destination));
    let writable = cursor < room;
    if writable {
      let byte = low_byte(v: rest);
      set deref(destination)[cursor] = byte;
    }
    set rest = irotr(rest, 8_u32);
    set cursor = cursor +wrap 1_u64;
  }
  return at +wrap 8_u64;
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let t0 = oct(a: 1_u64, b: 2_u64, c: 3_u64, d: 4_u64, e: 5_u64, f: 6_u64, g: 7_u64, h: 8_u64);
  let t1 = oct(a: 9_u64, b: 10_u64, c: 11_u64, d: 12_u64, e: 13_u64, f: 14_u64, g: 15_u64, h: 16_u64);
  let t2 = oct(a: 17_u64, b: 18_u64, c: 19_u64, d: 20_u64, e: 21_u64, f: 22_u64, g: 23_u64, h: 24_u64);
  let t3 = oct(a: 25_u64, b: 26_u64, c: 27_u64, d: 28_u64, e: 29_u64, f: 30_u64, g: 31_u64, h: 32_u64);
  let half0 = branch(left: move t0, right: move t1);
  let half1 = branch(left: move t2, right: move t3);
  let root = branch(left: move half0, right: move half1);
  let report = buffer_new(8_u64, 0_u8);
  region {
    let value = fold(node: &uniq root);
    region {
      let filled = spell(destination: &uniq report, at: 0_u64, value: value);
    }
  }
  region 'o {
    region {
      match write_once(output: &uniq 'o out, source: &report, start: 0_u64, end: 8_u64) {
        Ok(value: next) => {
          return exit_status(code: 0_u8);
        }
        Err(error: problem) => {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
}
"#;

const LANE_FRAME_LAYOUT_FUNCTIONS: &[u8] =
    br#"fn exact_frame(values: own array<u8, 255>) -> result: own u8 pure {
  return values[0_u64];
}

fn over_frame(values: own array<u8, 256>) -> result: own u8 pure {
  return values[0_u64];
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

fn lane_frame_program(length: u64) -> Vec<u8> {
    format!(
        "fn first(values: own array<u8, {length}>) -> result: own u8 pure {{\n  \
         return values[0_u64];\n}}\n\n\
         command fn main() -> status: own ExitStatus pure {{\n  \
         let left_values = array_new::<u8, {length}>(7_u8);\n  \
         let right_values = array_new::<u8, {length}>(9_u8);\n  \
         let left = first(values: move left_values);\n  \
         let right = first(values: move right_values);\n  \
         if left != 7_u8 {{\n    return exit_status(code: 1_u8);\n  }}\n  \
         if right != 9_u8 {{\n    return exit_status(code: 2_u8);\n  }}\n  \
         return exit_status(code: 0_u8);\n}}\n"
    )
    .into_bytes()
}

/// A pure call handed out while a second pure call, written as an `if`
/// condition, runs on this thread.
///
/// A writer may not spell a `Bool` match — [GRAM-6] demands `if` — but the
/// checker turns the `if` into one, so a call in condition position is a call
/// in scrutinee position and reaches the judgment exactly as a `let`
/// right-hand side does. That makes it an ordinary [PAR-1] candidate for
/// *compute* overlap, with no target operation anywhere in the pair — the
/// reach of a change whose motivation was completion I/O.
///
/// Both halves of the result are observable: the low byte of the handed-out
/// call's value, so a lost or unjoined hand-out shows, and a marker the
/// selected arm writes, so a condition decided wrongly shows too.
const IF_CONDITION_SIBLING: &[u8] = br#"fn mixdown(a: own u64, b: own u64) -> result: own u64 pure {
  let spun = irotl(a, 13_u32);
  let scattered = imulhi(b, 2654435761_u64);
  let blended = ixor(spun, b);
  return ixor(blended, scattered);
}

fn odd(v: own u64) -> result: own Bool pure {
  let low = iand(v, 1_u64);
  return low == 1_u64;
}

fn last_byte(v: own u64) -> result: own u8 pure {
  let low = iand(v, 255_u64);
  match cvt::<u64, u8>(low) {
    Ok(value: byte) => {
      return byte;
    }
    Err(error: problem) => {
      return 0_u8;
    }
  }
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  doc "A pure call handed out while a pure call written as an if condition runs.";
  let report = buffer_new(2_u64, 0_u8);
  let value = mixdown(a: 11_u64, b: 22_u64);
  if odd(v: 33_u64) {
    set report[1_u64] = 89_u8;
  }
  let byte = last_byte(v: value);
  set report[0_u64] = byte;
  region 'o {
    region {
      match write_once(output: &uniq 'o out, source: &report, start: 0_u64, end: 2_u64) {
        Ok(value: next) => {
          return exit_status(code: 0_u8);
        }
        Err(error: problem) => {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
}
"#;

/// Two sibling calls the judgment refuses: the second reads the first's
/// binding, so condition 1 denies the pair and nothing may be handed out.
const DEPENDENT_SIBLINGS: &[u8] = br#"fn twice(v: own u64) -> result: own u64 pure {
  return imax(v, v);
}

command fn main() -> status: own ExitStatus pure {
  let first = twice(v: 3_u64);
  let second = twice(v: first);
  let total = imax(first, second);
  return exit_status(code: 0_u8);
}
"#;

/// A program whose own functions are spelled like the runtime's entry points.
///
/// It overlaps, so the module carries the runtime symbols too, and both sets
/// have to coexist. `par_acquire_lane`, `par_publish`, `par_join`, and `par_release`
/// are ordinary IDENTs [FORM-3], so nothing may stop a writer from declaring
/// them.
const RUNTIME_SHAPED_NAMES: &[u8] = br#"fn par_acquire_lane(x: own u64) -> result: own u64 pure {
  return imax(x, x);
}

fn par_publish(x: own u64) -> result: own u64 pure {
  return imax(x, x);
}

fn par_join(x: own u64) -> result: own u64 pure {
  return imax(x, x);
}

fn par_release(x: own u64) -> result: own u64 pure {
  return imax(x, x);
}

fn par_thunk_0(x: own u64) -> result: own u64 pure {
  return imax(x, x);
}

command fn main() -> status: own ExitStatus pure {
  let a = par_acquire_lane(x: 1_u64);
  let b = par_publish(x: 2_u64);
  let c = par_thunk_0(x: 3_u64);
  let d = par_join(x: 4_u64);
  let e = par_release(x: 5_u64);
  let ab = imax(a, b);
  let cd = imax(c, d);
  let abcd = imax(ab, cd);
  let total = imax(abcd, e);
  return exit_status(code: 0_u8);
}
"#;

/// The backend's own symbols never collide with a source function's.
///
/// A source function is emitted as `wf_` plus its IDENT, and [FORM-3] spells
/// IDENT `[a-z][a-z0-9_]*`, so the `wf__par_` prefix the runtime uses is
/// unreachable from source. Without that reservation this program is accepted
/// by the checker and then rejected by the host toolchain with a raw
/// `invalid redefinition of function` — an accepted program failing to build,
/// which no source-level diagnostic explains.
#[test]
fn a_program_named_like_the_runtime_still_compiles_and_links() {
    let module = emit_with_overlap(RUNTIME_SHAPED_NAMES);
    assert!(
        module_requires_parallel_runtime(&module),
        "the fixture must actually hand work out:\n{module}"
    );
    assert!(
        module.contains("define internal i64 @wf_par_acquire_lane(i64 "),
        "the source function keeps its own symbol:\n{module}"
    );
    assert!(
        module.contains("define weak ptr @wf__par_acquire_lane(i64 %bytes) #0 {"),
        "the runtime keeps its reserved symbol:\n{module}"
    );
    let output = compile_and_run(&module);
    assert_eq!(output.status.code(), Some(0));
}

/// The selected target lays out the exact aggregate the worker thunk reads:
/// every parameter in declaration order followed by the result. A 255-byte
/// array plus a byte result reaches the 256-byte runtime boundary exactly;
/// adding one parameter byte remains a valid source function but makes this
/// optional schedule ineligible. Reducing the target address domain below the
/// exact aggregate is a target-layout failure rather than a capacity decline.
#[test]
fn selected_target_proves_the_complete_ordinary_lane_frame() {
    with_ir(LANE_FRAME_LAYOUT_FUNCTIONS, |program| {
        let host = TargetLayout::host().expect("the backend test runs on a qualified host");
        let system_target = SystemTarget::for_triple(host.triple())
            .expect("the host triple has one qualified system target");
        let qualification =
            qualify_program(system_target, program).expect("the lane-frame fixture must qualify");
        let exact = program
            .functions()
            .iter()
            .find(|function| function.name() == "exact_frame")
            .expect("the exact-boundary function must lower");
        let over = program
            .functions()
            .iter()
            .find(|function| function.name() == "over_frame")
            .expect("the over-boundary function must lower");

        let exact_layout = parallel_lane_frame_layout(host, &qualification, program, exact)
            .expect("the exact frame is target-representable")
            .expect("the exact frame fits the lane slot");
        assert_eq!(exact_layout.size(), crate::LANE_FRAME_BYTES);
        assert_eq!(exact_layout.align(), 1);
        assert!(exact_layout.align() <= PARALLEL_LANE_FRAME_ALIGNMENT);
        assert_eq!(
            parallel_lane_frame_layout(host, &qualification, program, over),
            Ok(None),
            "a target-representable frame beyond the lane capacity must decline overlap"
        );

        let short_domain = host.with_address_index_max_for_test(crate::LANE_FRAME_BYTES - 1);
        assert_eq!(
            parallel_lane_frame_layout(short_domain, &qualification, program, exact),
            Err(TargetLayoutFailure::Unrepresentable(
                TargetObject::ParallelLaneFrame
            )),
            "the aggregate itself must fit the selected target's address domain"
        );
    });
}

/// The two constants checked by selected-target lane layout are the runtime
/// slot's actual byte capacity and base alignment.
#[test]
fn ordinary_lane_frame_limits_match_the_runtime_slot() {
    let runtime = super::PARALLEL_RUNTIME_SOURCE;
    let declared = runtime
        .lines()
        .find_map(|line| line.strip_prefix("#define WF_PAR_FRAME_BYTES "))
        .expect("the runtime must state its frame capacity");
    assert_eq!(
        declared.trim().parse::<u64>().expect("a decimal capacity"),
        crate::LANE_FRAME_BYTES
    );
    assert!(
        runtime.contains(&format!(
            "_Alignas({PARALLEL_LANE_FRAME_ALIGNMENT}) unsigned char frame[WF_PAR_FRAME_BYTES];"
        )),
        "the runtime slot must provide the alignment target layout relies on"
    );
}

/// A frame at the runtime boundary is handed out with the selected-target
/// size as a constant. The same valid source shape one byte wider stays on the
/// ordinary sequential call path: no thunk, lane acquisition, or address-size
/// expression is emitted for it. Both modules pass the host LLVM toolchain and
/// preserve the source result.
#[test]
fn ordinary_overlap_uses_only_target_proved_lane_frames() {
    let exact = emit_with_overlap(&lane_frame_program(255));
    assert!(module_requires_parallel_runtime(&exact));
    assert!(exact.contains(&format!(
        "call ptr @wf__par_acquire_lane(i64 {})",
        crate::LANE_FRAME_BYTES
    )));
    assert!(
        !exact.contains("@wf__par_acquire_lane(i64 ptrtoint"),
        "lane size must come from selected-target layout, not emitted address arithmetic"
    );

    let over = emit_with_overlap(&lane_frame_program(256));
    assert!(
        !module_requires_parallel_runtime(&over),
        "a frame beyond the runtime slot must keep the optional overlap sequential"
    );
    assert!(!over.contains("@wf__par_thunk_"));

    let exact_output = compile_and_run(&exact);
    assert_eq!(exact_output.status.code(), Some(0));
    let over_output = compile_and_run(&over);
    assert_eq!(over_output.status.code(), Some(0));
}

/// One handed-out call emits its outlined thunk, a lane acquisition, the frame
/// stores and publication inside the granted edge, and a join whose refusal
/// edge makes the same call this thread would have made anyway.
#[test]
fn a_permitted_pair_is_outlined_offered_and_joined() {
    let module = emit_with_overlap(OVERLAPPING_FOLD);

    // The thunk is the outlined call: it loads the arguments out of the frame,
    // calls the same monomorphized function the inline edge calls, and stores
    // the result back into the frame. Its number is the module's, so the
    // assertion is on the shape rather than on which group came first.
    assert!(
        module.contains("(ptr %frame) #0 {\nentry:\n  %p0 = getelementptr inbounds "),
        "no outlined thunk over a frame:\n{module}"
    );
    assert!(
        module.contains("%result = call i64 @wf_fold(ptr %a0)"),
        "the thunk must call the same function the inline edge calls:\n{module}"
    );
    assert!(
        module.contains("  store i64 %result, ptr %slot\n  ret void\n"),
        "the thunk must leave its result in the frame:\n{module}"
    );
    // Every runtime entry point is the module's own weak definition, so a
    // module that hands work out is still a complete program.
    for weak in [
        "define weak ptr @wf__par_acquire_lane(i64 %bytes) #0 {",
        "define weak void @wf__par_publish(ptr %frame, ptr %fn) #0 {",
        "define weak void @wf__par_join(ptr %frame) #0 {",
        "define weak void @wf__par_release(ptr %frame) #0 {",
    ] {
        assert!(module.contains(weak), "no weak `{weak}`:\n{module}");
    }

    // `fold`'s own recursive pair: a lane is acquired and the first call is
    // published to it, the second runs inline on this thread, and only then is
    // the published one joined. The ordering is what makes the overlap window
    // exactly the second call.
    let body = function_body(&module, "@wf_fold");
    let acquisition = body
        .find("= call ptr @wf__par_acquire_lane(i64 ")
        .expect("fold must acquire a lane for its first recursive call");
    let publish = body
        .find("call void @wf__par_publish(ptr")
        .expect("fold must publish the acquired lane its outlined call");
    let inline = body
        .find("par.offered.")
        .and_then(|start| {
            body[start..]
                .find("call i64 @wf_fold(")
                .map(|at| start + at)
        })
        .expect("fold must run its second recursive call inline");
    let join = body
        .find("call void @wf__par_join(ptr")
        .expect("fold must join what it offered");
    assert!(
        acquisition < publish,
        "lane acquisition must precede the publish:\n{body}"
    );
    assert!(
        publish < inline,
        "the offer must precede the inline call:\n{body}"
    );
    assert!(
        inline < join,
        "the join must follow the inline call:\n{body}"
    );
    // The stores and the publish live inside the granted edge, so a refused
    // hand-out writes nothing and builds nothing.
    let offer_block = body
        .split("\npar.offer.")
        .nth(1)
        .expect("the granted edge must have its own block");
    let offer_block = offer_block
        .split_once("\npar.offered.")
        .expect("the granted edge must rejoin")
        .0;
    assert!(
        offer_block.contains("  store ") && offer_block.contains("@wf__par_publish"),
        "the frame stores and the publish must be inside the granted edge:\n{body}"
    );
    // The refused edge makes the same call the inline edge makes, so the two
    // edges are one lowering of one source call reached two ways.
    let refused = body
        .find("\npar.inline.")
        .and_then(|start| {
            body[start..]
                .find("call i64 @wf_fold(")
                .map(|at| start + at)
        })
        .expect("the refused edge must make the call on this thread");
    assert!(
        inline < refused,
        "the refusal edge belongs to the join, not the offer:\n{body}"
    );
    // Nothing between the offer and the join reads the offered value: the
    // value is defined by the phi in the block both edges branch to.
    let read = body
        .find("\npar.done.")
        .expect("the joined value must be read in the join's own block");
    assert!(
        join < read,
        "the value must be read after the join:\n{body}"
    );
    assert!(
        body[read..].contains(" = phi i64 [ "),
        "the joined value must be the phi of the two edges:\n{body}"
    );
}

/// A call written as an `if` condition is a compute-overlap join site, and the
/// program it joins publishes the same bytes at every worker count.
///
/// This is the compute half of the same change that let a `match` scrutinee be
/// judged: an `if` checks into a `Bool` match, so the call in its condition is
/// reached by exactly the machinery a `let` right-hand side is reached by, with
/// no rule of its own. Nothing in the program performs a target operation, so
/// the group here is the ordinary [PAR-1] compute lowering — acquire, publish,
/// the condition call inline on this thread, join, phi — and it is worth
/// pinning because the batch that opened this position was about I/O and would
/// not otherwise have covered it.
///
/// `WF_WORKERS` is the knob that matters: it decides whether a lane is granted,
/// which is what separates the published edge of the join from the refused one.
/// `0` and `1` are both the opt-out — fewer than two lanes of execution is the
/// sequential world either way — and `4` starts a pool that grants, so the loop
/// runs both edges.
#[test]
fn a_call_written_as_an_if_condition_joins_a_compute_overlap_group() {
    let module = emit_with_overlap(IF_CONDITION_SIBLING);
    let body = function_body(&module, "@wf_main");
    let acquisition = body
        .find("= call ptr @wf__par_acquire_lane(i64 ")
        .expect("the first call must acquire a lane");
    let publish = body
        .find("call void @wf__par_publish(ptr")
        .expect("the acquired lane must be given the outlined call");
    let condition = body
        .find("call i1 @wf_odd(")
        .expect("the condition call must run on this thread");
    let join = body
        .find("call void @wf__par_join(ptr")
        .expect("the handed-out call must be joined");
    assert!(
        acquisition < publish && publish < condition && condition < join,
        "the condition call is the overlap window and the join follows it:\n{body}"
    );
    let done = body
        .find("\npar.done.")
        .expect("the joined value must be defined in the join's own block");
    assert!(
        join < done && body[done..].contains(" = phi i64 [ "),
        "the joined value must be the phi of the two edges:\n{body}"
    );

    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    let mut runs = Vec::new();
    for workers in ["0", "1", "4"] {
        let output = Command::new(&executable)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run the if-condition overlap probe");
        assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
        // The handed-out value's low byte and the marker the selected arm
        // writes, in that order.
        assert_eq!(
            output.stdout, b"\x16Y",
            "WF_WORKERS={workers} published the wrong bytes"
        );
        runs.push((format!("WF_WORKERS={workers}"), output.stdout));
    }
    identical(&runs).expect("an if-condition join must not move one byte of the result");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// A recursion that hands one of its two calls out at every level, spelled at
/// one depth.
///
/// Every level scales the value it passes down, so no interprocedural fact
/// about the arguments collapses the sequential frame, and the two lowerings
/// of the same source are compared on the same terms. The whole result decides
/// the exit status, so neither build can drop the recursion.
const DEEP_RECURSION: &str = r#"fn leaf(v: own f64) -> result: own f64 pure {
  return fmul.strict(v, 0.5_f64);
}

fn spine(depth: own u64, v: own f64) -> result: own f64 pure {
  let done = depth == 0_u64;
  if done {
    return v;
  }
  let next = depth -wrap 1_u64;
  let scaled = fmul.strict(v, 1.0009765625_f64);
  let a = spine(depth: next, v: scaled);
  let b = leaf(v: v);
  return fadd.strict(a, b);
}

command fn main() -> status: own ExitStatus pure {
  let total = spine(depth: DEPTH_u64, v: 1.0009765625_f64);
  let bits = reinterpret::<f64, u64>(total);
  let low = iand(bits, 1_u64);
  match cvt::<u64, u8>(low) {
    Ok(value: byte) => {
      return exit_status(code: byte);
    }
    Err(error: wide) => {
      return exit_status(code: 2_u8);
    }
  }
}
"#;

/// Asking for overlap must not cost recursion depth when no lane is granted.
///
/// The frame of a handed-out call used to be a stack slot of the *calling*
/// function, so every activation of an eligible recursive function carried it
/// and its argument spills whether or not a lane was ever granted. The
/// measured price was about four times the stack per frame on a small
/// activation, and the death was a bare SIGSEGV with no diagnostic — a
/// recursion that ran without `--par` and did not run with it, at the same
/// schedule. The frame belongs to the lane now, so a refused hand-out builds
/// nothing.
///
/// The measurement is the frame width itself rather than survival at a chosen
/// depth, and it was rewritten on 2026-08-23 for that reason.
///
/// It used to run both builds under `ulimit -s 1024` at depth 18 600, chosen
/// to sit between what the old lowering reached under that limit (about
/// 16 200 frames) and what this one reached (about 21 600). That calibration
/// stopped meaning anything when the entry moved onto a stack the runtime
/// owns: `@main` trampolines to `wf__floor_run` regardless of world, so the
/// shell's limit binds neither build and both complete at depth 1 000 000,
/// fifty-four times what the case asked for. Against ceilings of 33 554 432
/// and 22 369 621 levels the margin was over a thousandfold, so the fourfold
/// frame regression this case exists to catch left it passing — as did every
/// other outcome.
///
/// A frame width is exact and the defect is a frame width, so that is what is
/// compared: the `--par` build's sequential clone against the plain build's
/// own function, byte for byte at one level. A per-activation slot for a
/// hand-out that was never granted shows up here immediately, with no depth to
/// calibrate and no host limit to depend on.
///
/// The original mechanism — a refused hand-out building no frame, which is what
/// the overlapped world still relies on whenever the pool is on and an acquisition is
/// refused — is held structurally by `handing_a_call_out_adds_no_stack_slot`,
/// whose count is taken over the overlapped world alone.
#[test]
fn handing_calls_out_keeps_the_sequential_recursion_depth() {
    let source = DEEP_RECURSION.replace("DEPTH", "1000").into_bytes();
    let overlapped_module = emit_with_overlap(&source);
    assert!(
        module_requires_parallel_runtime(&overlapped_module),
        "the fixture must hand work out, or this case is vacuous"
    );

    let plain_directory = test_directory();
    let overlapped_directory = test_directory();
    let plain = super::stack_ledger::ledger_lines(&emit(&source), &plain_directory);
    let overlapped = super::stack_ledger::ledger_lines(&overlapped_module, &overlapped_directory);

    let sequential = super::stack_ledger::reported_frame_bytes(&plain, "wf_spine");
    let clone = super::stack_ledger::reported_frame_bytes(&overlapped, "wf__par_seq_spine");
    assert_eq!(
        clone, sequential,
        "the --par build's sequential clone costs {clone} bytes a level where \
         the plain build's own function costs {sequential}, so handing calls \
         out is taxing activations that were never granted a lane"
    );

    std::fs::remove_dir_all(&plain_directory).expect("remove the test directory");
    std::fs::remove_dir_all(&overlapped_directory).expect("remove the test directory");
}

/// The world an unconfigured `--par` binary runs in reaches a deep recursion
/// too.
///
/// [`handing_calls_out_keeps_the_sequential_recursion_depth`] holds the clone
/// a pool-off binary runs; this holds the one a binary with no configuration
/// at all runs, which is the world the original defect lived in. A
/// per-activation frame slot cost about four times the stack there and died as
/// a bare SIGSEGV: a recursion that ran without `--par` and did not run with
/// it.
///
/// What is pinned is a bound and not a parity, because an overlapped
/// activation is genuinely not free: the acquisition handle, the recursion's own
/// argument and the value the join reads back are live across the call to
/// `wf__par_acquire_lane`, and whatever the register allocator cannot keep in
/// registers across that call is spilled into the frame.
///
/// The bound is that overhead in bytes rather than a multiple of the
/// sequential level, and it was rewritten that way on 2026-08-27 because the
/// multiple did not survive its second host. The two architectures cost the
/// same overlapped activation and differ in the sequential one they are
/// measured against — 48 bytes an overlapped level on both, against 32 a
/// sequential level on arm64 and 16 on x86-64 — so the same lowering reads as
/// a ratio of 1.5 on one host and 3 on the other, and the bound of two it used
/// to carry was a fact about the arm64 register allocator. The measured
/// overhead is two machine words on arm64 and four on x86-64; the bound is
/// six, which admits both with room and refuses an activation that has started
/// carrying storage for the hand-out itself.
///
/// The mechanism — the record belongs to the lane, so a refused hand-out
/// builds nothing — is held exactly, and with no tolerance to calibrate, by
/// [`handing_a_call_out_adds_no_stack_slot`]. This case is the frame the
/// optimizer actually emitted, which is the thing a structural count cannot
/// see.
///
/// It used to be a survival probe at depth 60 000 under `ulimit -s 8192`,
/// re-aimed on 2026-08-23 for the reason its own doc gave: the entry runs on a
/// stack the runtime owns and a lane runs on one the same size, so that
/// `ulimit` bounds neither thread, and the depth sat three orders of magnitude
/// inside a ceiling nothing in the shell could move. The instrument that
/// catches a moved frame is the frame, and the ledger prints it.
#[test]
fn the_shipped_default_keeps_a_deep_recursion() {
    /// How much more stack one overlapped level may cost than one sequential
    /// level, in bytes.
    const WIDEST_ADMITTED_OVERHEAD: u64 = 48;

    let source = DEEP_RECURSION.replace("DEPTH", "1000").into_bytes();
    let overlapped_module = emit_with_overlap(&source);
    assert!(
        module_requires_parallel_runtime(&overlapped_module),
        "the fixture must hand work out, or this case is vacuous"
    );

    let directory = test_directory();
    let lines = super::stack_ledger::ledger_lines(&overlapped_module, &directory);
    let overlapped = super::stack_ledger::reported_frame_bytes(&lines, "wf_spine");
    let sequential = super::stack_ledger::reported_frame_bytes(&lines, "wf__par_seq_spine");
    assert!(
        overlapped <= sequential + WIDEST_ADMITTED_OVERHEAD,
        "one overlapped level costs {overlapped} bytes against the sequential \
         clone's {sequential} in the same binary, which is more than \
         {WIDEST_ADMITTED_OVERHEAD} bytes of hand-out state, so the world a \
         --par binary runs in unconfigured is taxing activations far beyond \
         what lane acquisition keeps live"
    );

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The lane's frame is the lane's: asking for overlap adds no stack slot to
/// any function.
///
/// This is the whole resource bound of the hand-out. An earlier lowering put
/// the frame in the calling function's entry block, so every activation of an
/// eligible recursive function carried the slot and its argument spills
/// whether or not a lane was ever granted, and a `--par` build reached about a
/// quarter of the sequential build's recursion depth before dying on a bare
/// SIGSEGV. The comparison is against the default compilation of the same
/// source, so it measures the lowering rather than the program.
///
/// The count is taken over the *overlapped* world alone. A `--par` module
/// carries a second lowering of the eligible closure, and counting both copies
/// against one reference would compare a doubled module with a single one — a
/// failure that says nothing about whether a hand-out costs a slot. What the
/// clone costs is a separate and stronger question, answered by
/// `the_sequential_clone_is_the_sequential_lowering`: it is the sequential
/// lowering byte for byte, so its slots are the sequential build's slots.
#[test]
fn handing_a_call_out_adds_no_stack_slot() {
    let sequential = emit(OVERLAPPING_FOLD);
    let overlapped = emit_with_overlap(OVERLAPPING_FOLD);
    assert!(
        module_requires_parallel_runtime(&overlapped),
        "the fixture must hand work out, or this test is vacuous"
    );
    let actualized = without_clones(&overlapped);
    assert!(
        actualized.contains("@wf__par_publish"),
        "removing the clones must leave the overlapped world:\n{actualized}"
    );
    assert_eq!(
        actualized.matches("= alloca ").count(),
        sequential.matches("= alloca ").count(),
        "handing calls out must add no stack slot:\n{actualized}"
    );
}

/// The sequential clone is the sequential lowering: not similar to it, the
/// same bytes.
///
/// This is the load-bearing property of two-world compilation. The clone
/// exists so that every transform the default build gets fires on it — the one
/// that matters is LLVM's accumulator tail-recursion elimination, which the
/// hand-out's phi at `%par.done` forecloses and which was worth 2.96x on
/// `fib(38)` with the pool off. "Gets the same transforms" is not something a
/// test can ask LLVM directly; what it can ask is whether the input is the same
/// input, which is the whole property and is stronger than any list of
/// properties spelled out one at a time. A clone that drifted — a slot, a phi,
/// an operand read in a different order — would be a second lowering nobody
/// audited, and this case is what stops that.
///
/// The comparison restores the clone's own symbols, because the calls inside a
/// clone name clones: that renaming *is* the difference between the two
/// worlds, and after it there must be nothing left.
#[test]
fn the_sequential_clone_is_the_sequential_lowering() {
    let sequential = emit(OVERLAPPING_FOLD);
    let overlapped = emit_with_overlap(OVERLAPPING_FOLD);

    // The closure, spelled out: every function from which a handed-out call is
    // reachable. `pair`, `quad`, and `oct` hand out their sibling
    // constructors, `fold` its recursion, and `main` its four `oct` calls.
    // `leaf`, `branch`, `mix`, `low_byte`, and `spell` reach no hand-out at
    // all, so both worlds call the one copy of each and neither needs a clone.
    let mut cloned: Vec<_> = clone_symbols(&overlapped);
    cloned.sort_unstable();
    assert_eq!(
        cloned,
        [
            "@wf__par_seq_fold",
            "@wf__par_seq_main",
            "@wf__par_seq_oct",
            "@wf__par_seq_pair",
            "@wf__par_seq_quad",
        ],
        "the clone set must be the closure of the eligible calls:\n{overlapped}"
    );

    for symbol in &cloned {
        let clone = function_body(&overlapped, symbol);
        let restored = clone.replace("@wf__par_seq_", "@wf_");
        let reference = function_body(&sequential, &symbol.replace("@wf__par_seq_", "@wf_"));
        assert_eq!(
            restored, reference,
            "{symbol} is not the sequential lowering of its function"
        );
    }
}

/// The two worlds never call each other, and which one runs is decided once.
///
/// Both halves matter and they fail differently. A clone that called back into
/// the overlapped world would re-enter the lowering it exists to avoid, and
/// would do so *below* the one place the choice is made, so the program would
/// pay the tax again with nothing left to catch it. And a selection made
/// anywhere but the bootstrap would be a test executed per call or per
/// activation: at best a branch in a hot loop, at worst the per-task demand
/// signal this design exists to avoid — the shared word that took the
/// fine-grain oracle cell from 0.4905 s to 0.9254 s when it was measured.
///
/// The query is a separate weak definition, so the four entry points of the
/// lane protocol are exactly the bytes they were; a link that reads them, and
/// the cases that pin them, cannot be disturbed by it.
#[test]
fn the_bootstrap_selects_one_world_once() {
    let overlapped = emit_with_overlap(OVERLAPPING_FOLD);

    // Asked once, in the one place that runs once and is inside no loop and no
    // recursion.
    assert_eq!(
        overlapped
            .matches("call i32 @wf__par_pool_active()")
            .count(),
        1,
        "the world must be selected exactly once per process:\n{overlapped}"
    );
    // The bootstrap lives in the entry body: `@main` keeps the host's
    // signature and hands the program to the exhaustion floor, which runs this
    // on a stack the compiler sized.
    let bootstrap = function_body(&overlapped, "@wf__main_body");
    assert!(
        bootstrap.contains("  %par.pool = call i32 @wf__par_pool_active()")
            && bootstrap.contains("call i8 @wf_main(")
            && bootstrap.contains("call i8 @wf__par_seq_main("),
        "the bootstrap must branch between the two lowerings of the entry:\n{bootstrap}"
    );
    // With no runtime linked no pool can start, so the module's own answer is
    // the honest one and such a program runs the sequential lowering of itself.
    assert!(
        overlapped.contains("define weak i32 @wf__par_pool_active() #0 {\nentry:\n  ret i32 0\n}"),
        "the module must carry its own answer:\n{overlapped}"
    );
    for weak in [
        "define weak ptr @wf__par_acquire_lane(i64 %bytes) #0 {",
        "define weak void @wf__par_publish(ptr %frame, ptr %fn) #0 {",
        "define weak void @wf__par_join(ptr %frame) #0 {",
        "define weak void @wf__par_release(ptr %frame) #0 {",
    ] {
        assert!(
            overlapped.contains(weak),
            "adding the query must not disturb `{weak}`:\n{overlapped}"
        );
    }

    // No clone reaches the overlapped world. A clone may still call a function
    // that has no clone — that copy is shared because its lowering is the same
    // either way — so the forbidden targets are exactly the cloned ones.
    let cloned = clone_symbols(&overlapped);
    for symbol in &cloned {
        let body = function_body(&overlapped, symbol);
        for other in &cloned {
            let forbidden = format!(" @{}(", other.replace("@wf__par_seq_", "wf_"));
            assert!(
                !body.contains(&forbidden),
                "{symbol} calls the overlapped{forbidden}, so the clone world re-enters the \
                 lowering it exists to avoid:\n{body}"
            );
        }
    }
    // And nothing but the bootstrap reaches the clone world.
    let actualized = without_clones(&overlapped);
    let bootstrap_free = actualized.replace(function_body(&actualized, "@wf__main_body"), "");
    assert!(
        !bootstrap_free.contains("@wf__par_seq_"),
        "only the bootstrap may name a clone:\n{bootstrap_free}"
    );
}

/// A Windows `--par` module carries unresolved native-pool obligations instead
/// of the sequential weak definitions used by the POSIX optional-runtime
/// path.  Consequently, omitting `par_runtime_windows.c` is a link error and
/// can never turn a requested Windows backend into the sequential world.
#[test]
fn windows_parallel_modules_fail_closed_at_the_link_boundary() {
    let windows = SystemTarget::for_triple("x86_64-pc-windows-msvc")
        .expect("the supported Windows target must have a system row");
    let module = with_parallel_ir(OVERLAPPING_FOLD, |program| {
        emit_llvm_for_target(program, windows)
            .expect("the overlap fixture must emit for Windows")
            .into_string()
    });

    for declaration in [
        "declare ptr @wf__par_acquire_lane(i64)",
        "declare void @wf__par_publish(ptr, ptr)",
        "declare void @wf__par_join(ptr)",
        "declare void @wf__par_release(ptr)",
        "declare i32 @wf__par_pool_active()",
    ] {
        assert!(
            module.contains(declaration),
            "Windows must leave `{declaration}` for the native runtime:\n{module}"
        );
    }
    for fallback in [
        "define weak ptr @wf__par_acquire_lane",
        "define weak void @wf__par_publish",
        "define weak void @wf__par_join",
        "define weak void @wf__par_release",
        "define weak i32 @wf__par_pool_active",
    ] {
        assert!(
            !module.contains(fallback),
            "Windows must not carry sequential fallback `{fallback}`:\n{module}"
        );
    }
    assert!(
        module_requires_parallel_runtime(&module),
        "the Windows declarations must remain a driver-visible link obligation"
    );
}

/// A recursion far deeper than the runtime can hold offers for, whose whole
/// result is published as bytes so a wrong schedule is a wrong output.
///
/// One eligible pair per activation, and the handed-out member is the deep
/// side, so every level of the descent offers.
const DEEP_OVERLAPPED_SPINE: &str = r#"fn leafval(v: own f64) -> result: own f64 pure {
  return fmul.strict(v, 0.5_f64);
}

fn spine(depth: own u64, v: own f64) -> result: own f64 pure {
  let done = depth == 0_u64;
  if done {
    return v;
  }
  let next = depth -wrap 1_u64;
  let scaled = fmul.strict(v, 1.0009765625_f64);
  let a = spine(depth: next, v: scaled);
  let b = leafval(v: v);
  return fadd.strict(a, b);
}

fn low_byte(v: own u64) -> result: own u8 pure {
  let nibble = iand(v, 255_u64);
  match cvt::<u64, u8>(nibble) {
    Ok(value: byte) => {
      return byte;
    }
    Err(error: wide) => {
      return 0_u8;
    }
  }
}

fn spell(destination: &uniq buffer<u8>, value: own u64) -> result: own u64 reads(destination), writes(destination) {
  let cursor = 0_u64;
  let rest = value;
  loop @octets {
    let done = cursor >= 8_u64;
    if done {
      break @octets;
    }
    let room = len(deref(destination));
    let writable = cursor < room;
    if writable {
      let byte = low_byte(v: rest);
      set deref(destination)[cursor] = byte;
    }
    set rest = irotr(rest, 8_u32);
    set cursor = cursor +wrap 1_u64;
  }
  return cursor;
}

command fn main(command.stdout as out: own Output) -> status: own ExitStatus reads(out), writes(out), allocates(heap) {
  let total = spine(depth: DEPTH_u64, v: 1.0009765625_f64);
  let bits = reinterpret::<f64, u64>(total);
  let report = buffer_new(8_u64, 0_u8);
  region {
    let filled = spell(destination: &uniq report, value: bits);
  }
  region 'o {
    region {
      match write_once(output: &uniq 'o out, source: &report, start: 0_u64, end: 8_u64) {
        Ok(value: next) => {
          return exit_status(code: 0_u8);
        }
        Err(error: problem) => {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
}
"#;

/// A recursion deeper than the runtime holds offers for still computes the
/// sequential answer, at every worker count.
///
/// The runtime can only hold a bounded number of outstanding offers per
/// thread, so a descent this deep runs out partway down and every offer below
/// that point is refused — which puts three different edges in one run: the
/// reclaimed offer near the root, the refused offer below the bound, and
/// whatever a thief took. Nothing else here exercises the exhausted bound,
/// and a runtime that mishandled it would either lose a result or reuse a
/// frame that is still live, both of which move the published bytes.
#[test]
fn a_recursion_deeper_than_the_offer_bound_still_publishes_the_sequential_bytes() {
    const DEPTH: u32 = 4_000;

    let source = DEEP_OVERLAPPED_SPINE
        .replace("DEPTH", &DEPTH.to_string())
        .into_bytes();
    let module = emit_with_overlap(&source);
    assert!(
        module_requires_parallel_runtime(&module),
        "the fixture must hand work out, or this case is vacuous"
    );
    let directory = test_directory();
    let executable = build_executable(&module, &directory);

    let mut runs = Vec::new();
    for workers in ["1", "2", "4", "8"] {
        for _ in 0..3 {
            let output = Command::new(&executable)
                .env("WF_WORKERS", workers)
                .output()
                .expect("run the deep overlapped spine");
            assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
            assert_eq!(
                output.stdout.len(),
                8,
                "the spine must report eight bytes at WF_WORKERS={workers}"
            );
            runs.push((format!("WF_WORKERS={workers}"), output.stdout));
        }
    }
    // The reference is the same source compiled with no hand-outs at all, so
    // the comparison is against the sequential answer rather than against the
    // overlapped build agreeing with itself.
    let plain = test_directory();
    let sequential = build_executable(&emit(&source), &plain);
    let reference = Command::new(&sequential)
        .output()
        .expect("run the sequential spine");
    assert_eq!(reference.status.code(), Some(0));
    runs.push(("no hand-outs".to_owned(), reference.stdout));

    identical(&runs).expect("a deep overlapped recursion must publish the sequential bytes");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
    std::fs::remove_dir_all(&plain).expect("remove the test directory");
}

/// A pair the judgment denies emits exactly the sequential calls, with no
/// frame, no thunk, no offer, and no join anywhere in the module.
#[test]
fn a_denied_pair_emits_exactly_the_sequential_calls() {
    let module = emit_with_overlap(DEPENDENT_SIBLINGS);
    assert!(
        !module.contains("wf_par"),
        "a denied pair must name no part of the runtime:\n{module}"
    );
    let body = function_body(&module, "@wf_main");
    let calls: Vec<_> = body.match_indices("call i64 @wf_twice(").collect();
    assert_eq!(calls.len(), 2, "both calls stay ordinary calls:\n{body}");
}

/// A permitted pair whose first member is a borrowed binding is not handed
/// out, because promoting that binding reads the call's value at its
/// definition site — between the offer and the join, where the value does not
/// exist yet.
///
/// The judgment still permits the pair; this is the lowering refusing to
/// actualize a permission it cannot carry on one straight-line edge, and it
/// refuses by dropping the group rather than by moving the read.
#[test]
fn a_permitted_pair_whose_first_member_is_borrowed_is_not_handed_out() {
    let borrowed = br#"fn make() -> result: own u64 pure {
  return 7_u64;
}

fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

command fn main() -> status: own ExitStatus pure {
  let first = make();
  let second = make();
  region {
    let seen = peek(v: &first);
  }
  return exit_status(code: 0_u8);
}
"#;
    let module = emit_with_overlap(borrowed);
    assert!(
        !module.contains("wf_par"),
        "a borrowed first member must not be handed out:\n{module}"
    );

    // The same pair with only the *second* member borrowed is handed out: the
    // last member always runs on the calling thread, so its own value is read
    // after the join like every other.
    let trailing = br#"fn make() -> result: own u64 pure {
  return 7_u64;
}

fn peek(v: &u64) -> result: own u64 reads(v) {
  return deref(v);
}

command fn main() -> status: own ExitStatus pure {
  let first = make();
  let second = make();
  region {
    let seen = peek(v: &second);
  }
  return exit_status(code: 0_u8);
}
"#;
    assert!(
        emit_with_overlap(trailing).contains("call void @wf__par_publish(ptr "),
        "a borrowed last member does not stop the group"
    );
}

/// A module that hands nothing out is a complete program with no runtime, and
/// says so: the link path's own predicate is false, so no link anywhere adds
/// the runtime to it.
#[test]
fn a_module_that_hands_nothing_out_needs_no_runtime() {
    let module = emit_with_overlap(DEPENDENT_SIBLINGS);
    assert!(!module_requires_parallel_runtime(&module));
    let output = compile_and_run(&module);
    assert_eq!(output.status.code(), Some(0));

    // The overlapping module is the other half of the same statement: it does
    // hand work out, so linking the runtime is what gives it lanes.
    assert!(module_requires_parallel_runtime(&emit_with_overlap(
        OVERLAPPING_FOLD
    )));
}

/// The module carries a weak sequential answer to both runtime entry points,
/// and the runtime's own definitions replace them at link.
///
/// Without this the whole path could be silently sequential forever: every
/// other test here passes just as well when the weak refusal wins, because
/// refusing every lane is a correct execution.
#[test]
fn the_runtime_replaces_the_modules_weak_refusal() {
    let module = emit_with_overlap(OVERLAPPING_FOLD);
    let directory = test_directory();

    // Linked with nothing: the module's own weak definitions answer, every
    // lane is refused, and the program still produces its whole result.
    let alone = build_executable_without_parallel_runtime(&module, &directory);
    let sequential = Command::new(&alone)
        .output()
        .expect("run the linked module");
    assert_eq!(sequential.status.code(), Some(0));
    assert_eq!(
        sequential.stdout.len(),
        8,
        "the fold must report eight bytes"
    );

    // Linked with the runtime: the strong definitions win. The count is the
    // runtime's own, reported at process exit by the observer unit, so a
    // link that kept the weak refusal reports zero here and fails.
    let counted = CountedProgram::link(&module, &directory);
    let (granted, parallel) = counted.run(Some("4"));
    assert_eq!(parallel.status.code(), Some(0));
    assert_eq!(
        parallel.stdout, sequential.stdout,
        "granting lanes must not move one byte of the result"
    );
    if a_steal_is_observable(4) {
        let observed_grants = if granted == 0 {
            counted.grants_over_runs(Some("4"), GRANT_OBSERVATION_RUNS)
        } else {
            granted
        };
        assert!(
            observed_grants > 0,
            "the runtime granted no lane, so nothing was overlapped"
        );
    }

    // And the explicit opt-out: one lane of execution is the calling thread
    // alone, so the pool never starts and every offer is refused.
    let (opted_out, quiet) = counted.run(Some("1"));
    assert_eq!(quiet.stdout, sequential.stdout);
    assert_eq!(opted_out, 0, "WF_WORKERS=1 must never start the pool");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The shipped default is a pool: a `--par` binary run with `WF_WORKERS`
/// absent grants lanes, and only an explicit opt-out refuses them.
///
/// This is the whole of the default-behavior change, and it needs its own case
/// because every other case here names a worker count. Before it, an unset
/// variable meant the sequential world, so a `--par` binary handed to anybody
/// who did not know about the variable was byte-for-byte a sequential program
/// and the entire path was off for every real run. The grant count is the
/// runtime's own counter, so "the pool started" is read rather than assumed.
///
/// The opt-outs are pinned in the same case against the same executable, so a
/// change that turned the default on by making *every* setting start a pool
/// fails here rather than passing as a stronger version of the same news.
/// `abc` stands for the unparsable settings: a value that is not a number is
/// not a request for lanes, and reading it as one would be the runtime
/// inventing a count the caller did not ask for.
#[test]
fn an_absent_worker_setting_starts_the_pool_and_an_explicit_opt_out_does_not() {
    let module = emit_with_overlap(OVERLAPPING_FOLD);
    let directory = test_directory();

    // `wf__par_grants` counts steals, and a steal is a scheduling event: a
    // pool thread has to be given a CPU before the offering lane finishes the
    // work itself. On a saturated host that can fail to happen in one run of
    // a program this short, so the existential observation (the default build CAN
    // be granted lanes) is re-observed over [`GRANT_OBSERVATION_RUNS`] runs,
    // exactly as the WF_WORKERS=4 case above does. A pool that never grants
    // fails every one of them; the opt-out runs below stay exact.
    let counted = CountedProgram::link(&module, &directory);
    let (defaulted, published) = counted.run(None);
    assert_eq!(published.status.code(), Some(0));
    let observed_grants = if defaulted == 0 {
        counted.grants_over_runs(None, GRANT_OBSERVATION_RUNS)
    } else {
        defaulted
    };
    assert!(
        observed_grants > 0,
        "a --par binary with no worker setting must run in the overlapped \
         world and be granted lanes, or the path is off for every real run"
    );

    let mut runs = vec![("WF_WORKERS absent".to_owned(), published.stdout)];
    for setting in ["0", "1", "abc"] {
        let (granted, output) = counted.run(Some(setting));
        assert_eq!(output.status.code(), Some(0), "WF_WORKERS={setting}");
        assert_eq!(
            granted, 0,
            "WF_WORKERS={setting} is an opt-out and must never start the pool"
        );
        runs.push((format!("WF_WORKERS={setting}"), output.stdout));
    }
    identical(&runs).expect("the default must not move one byte of the result");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The repeat: the same program, at every worker count, over and over, is the
/// same bytes. Lanes are granted at every count above one, so each repeat is a
/// real overlapped execution rather than the refusal path run again.
#[test]
fn an_overlapped_program_reports_one_byte_sequence_at_every_worker_count() {
    let module = emit_with_overlap(OVERLAPPING_FOLD);
    let directory = test_directory();
    let executable = build_executable(&module, &directory);
    let mut runs = Vec::new();
    for workers in ["1", "2", "4", "8"] {
        for _ in 0..5 {
            let output = Command::new(&executable)
                .env("WF_WORKERS", workers)
                .output()
                .expect("run the overlapped program");
            assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
            runs.push((format!("WF_WORKERS={workers}"), output.stdout));
        }
    }
    // The sequential reference is the same program with no runtime linked at
    // all, so the repeat is compared against today's execution and not only
    // against itself.
    let alone = build_executable_without_parallel_runtime(&module, &directory);
    let reference = Command::new(&alone)
        .output()
        .expect("run the linked module");
    runs.push(("no runtime".to_owned(), reference.stdout));

    identical(&runs).expect("overlapping must not move one byte of the result");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// Overlap lowering is compile-time opt-in: the default compilation of a
/// program full of eligible sites names no part of the runtime.
///
/// This is what makes the feature free when it is not asked for. The judgment
/// still ran — the same source with the lowering switched on hands work out,
/// which is the second half of this test — so what the default drops is the
/// outlining and nothing else. The outlining is not free: it passes arguments
/// through a memory frame and is reached through a function pointer, so the
/// call cannot be inlined, and the batch audit measured that alone at about
/// 1.2x on the layout demo and 2.1x on `fib(38)` with no runtime linked and
/// `WF_WORKERS` unset.
#[test]
fn the_default_compilation_hands_nothing_out() {
    let default = emit(OVERLAPPING_FOLD);
    assert!(
        !default.contains("wf__par_"),
        "the default compilation must name no runtime symbol:\n{default}"
    );
    assert!(
        !default.contains("(ptr %frame) {"),
        "the default compilation must outline no thunk:\n{default}"
    );
    assert!(
        !module_requires_parallel_runtime(&default),
        "no link path may add the runtime to a default build"
    );

    // The same source asked for lanes: the sites were there all along, so the
    // assertions above are about the option and not about the program.
    let requested = emit_with_overlap(OVERLAPPING_FOLD);
    assert!(
        requested.contains("call void @wf__par_publish(ptr ")
            && requested.contains(", ptr @wf__par_thunk_"),
        "the fixture must hand work out when asked, or this test is vacuous"
    );
    assert!(module_requires_parallel_runtime(&requested));
}

/// The differential: the same source lowered *without* any overlap group
/// produces the same bytes as the overlapped lowering, at every worker count.
///
/// This is the comparison the rest of this module cannot make. Every other
/// test here links one emitted module two ways, so a defect introduced by the
/// outlining itself — a moved read, a hoisted operand, a join in the wrong
/// place — is present in the reference too and compares equal. The reference
/// here is the default compilation of the same source, whose calls were never
/// handed out, which is the only way an overlap's "changes nothing observable"
/// guarantee can be checked against something other than itself.
#[test]
fn the_overlapped_lowering_agrees_with_the_lowering_that_hands_nothing_out() {
    let sequential_module = emit(OVERLAPPING_FOLD);
    assert!(
        !module_requires_parallel_runtime(&sequential_module),
        "the reference module must contain no hand-out at all"
    );
    assert!(
        module_requires_parallel_runtime(&emit_with_overlap(OVERLAPPING_FOLD)),
        "the overlapped module must hand work out, or the comparison is vacuous"
    );

    let directory = test_directory();
    let reference = Command::new(build_executable(&sequential_module, &directory))
        .output()
        .expect("run the module that hands nothing out");
    assert_eq!(reference.status.code(), Some(0));
    assert_eq!(
        reference.stdout.len(),
        8,
        "the fold must report eight bytes"
    );

    let overlapped = build_executable(&emit_with_overlap(OVERLAPPING_FOLD), &directory);
    let mut runs = vec![("no overlap lowering".to_owned(), reference.stdout)];
    for workers in ["1", "2", "4", "8"] {
        let output = Command::new(&overlapped)
            .env("WF_WORKERS", workers)
            .output()
            .expect("run the overlapped program");
        assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
        runs.push((format!("WF_WORKERS={workers}"), output.stdout));
    }
    identical(&runs).expect("outlining a call must not move one byte of the result");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// The negative control for the repeat above: its comparison reports a
/// difference when one is present, so a green repeat is evidence about the
/// program rather than about the comparison.
#[test]
fn the_repeat_comparison_reports_an_injected_difference() {
    let same = vec![
        ("WF_WORKERS=1".to_owned(), b"abcdefgh".to_vec()),
        ("WF_WORKERS=2".to_owned(), b"abcdefgh".to_vec()),
    ];
    identical(&same).expect("equal runs must compare equal");

    for injected in [b"abcdefgi".to_vec(), b"abcdefg".to_vec(), Vec::new()] {
        let mut runs = same.clone();
        runs.push(("WF_WORKERS=4".to_owned(), injected));
        let report = identical(&runs).expect_err("a differing run must be reported");
        assert!(report.contains("WF_WORKERS=4"), "{report}");
    }
}

/// The negative control the repeat actually needs: a lowering with its joins
/// removed is caught.
///
/// The comparison control above proves the byte comparison reports a
/// difference it is handed. This one proves the *program* comparison reports a
/// real lowering defect: the branch's own emitted module, with both join calls
/// struck out, linked against the real runtime. A missed join lets the calling
/// thread read a frame slot the worker has not written and lets the frame's
/// activation return underneath the worker, so a run either publishes
/// different bytes or dies.
///
/// Detection is per-run and not certain — a granted lane sometimes finishes
/// before the read anyway — so the control runs the injected build up to twelve
/// times and requires that at least one run disagree with the reference. The
/// measured per-run detection rate is about four in five, which puts a false
/// green here below one in a hundred million.
///
/// The requirement is existential, so the loop stops at the first disagreement
/// and the twelve are the bound the *undetected* direction pays: a lowering
/// whose missing joins this comparison cannot see makes all twelve runs and
/// fails, exactly as before. What that bound removes is eleven runs of a
/// program that is expected to die — measured in batch 0093 at 30 seconds a
/// run on the four-core Linux runner, where a run that dies under a core-dump
/// handler is three orders of magnitude dearer than the same run here.
#[test]
fn the_repeat_reports_a_lowering_whose_joins_were_removed() {
    let module = emit_with_overlap(OVERLAPPING_FOLD);
    let directory = test_directory();

    let reference = Command::new(build_executable(&module, &directory))
        .env("WF_WORKERS", "1")
        .output()
        .expect("run the intact program");
    assert_eq!(reference.status.code(), Some(0));
    assert_eq!(reference.stdout.len(), 8);

    let joinless = module.replace(
        "  call void @wf__par_join(ptr ",
        "  call void @wf__par_join_removed(ptr ",
    );
    assert_ne!(joinless, module, "the injection must change the module");
    let joinless = format!(
        "{joinless}\ndefine internal void @wf__par_join_removed(ptr %handle) {{\nentry:\n  ret void\n}}\n"
    );
    let broken = build_executable(&joinless, &directory);

    let mut disagreements = 0;
    for _ in 0..12 {
        let output = Command::new(&broken)
            .env("WF_WORKERS", "4")
            .output()
            .expect("run the join-less program");
        if output.status.code() != Some(0) || output.stdout != reference.stdout {
            disagreements += 1;
            break;
        }
    }
    assert!(
        disagreements > 0,
        "removing every join changed nothing observable in twelve runs, \
         so this comparison cannot see a missed join"
    );

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}

/// Every run produced the same bytes, or the first run that did not.
pub(super) fn identical(runs: &[(String, Vec<u8>)]) -> Result<(), String> {
    let Some((first_name, first)) = runs.first() else {
        return Err("no run to compare".to_owned());
    };
    for (name, bytes) in &runs[1..] {
        if bytes != first {
            return Err(format!(
                "{name} produced {bytes:?}, but {first_name} produced {first:?}"
            ));
        }
    }
    Ok(())
}

/// Links one module without the parallel runtime, while retaining any target
/// runtime the normal completion-only build requires. This is the exact
/// sequential schedule of an overlap-capable module, not an invalid bare link.
pub(super) fn build_executable_without_parallel_runtime(
    module: &str,
    directory: &Path,
) -> std::path::PathBuf {
    let assembly = directory.join("alone.ll");
    let executable = directory.join("alone");
    std::fs::write(&assembly, module).expect("write the module");
    let mut command = Command::new("/usr/bin/clang");
    command.arg("-x").arg("ir").arg(&assembly);
    let _completion_units = append_completion_runtime(&mut command, module, directory);
    let linked = command
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        linked.status.success(),
        "a module that hands work out must link without the parallel runtime:\n{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    std::fs::remove_file(&assembly).expect("remove the module");
    executable
}

/// Whether this host can tell "the runtime granted no lane" apart from "no
/// worker was scheduled inside the window".
///
/// A steal is only observable if a worker reaches the offer before the
/// offering thread has already finished the work itself, which needs a core
/// that is not already carrying a lane. Measured in batch 0090 on GitHub's
/// runners: the four-lane observations reach zero over their whole sample on
/// the three-core macOS runner and are non-zero on every four-core host run,
/// so a zero there is a fact about the host rather than about the lowering.
/// Where the host has the cores, the observation is enforced exactly as it
/// always was; where it does not, the case says so on standard error rather
/// than reporting a lowering regression it cannot see.
pub(super) fn a_steal_is_observable(lanes: usize) -> bool {
    let cores = std::thread::available_parallelism().map_or(0, std::num::NonZero::get);
    if cores < lanes {
        eprintln!(
            "host-limited: {cores} schedulable cores cannot show a steal across {lanes} lanes, \
             so the grant observation is not made on this host"
        );
        return false;
    }
    true
}

/// The upper bound on the runs an existential grant observation makes before
/// it reports that the runtime granted nothing.
///
/// A steal is a scheduling event, so one run samples the host's schedule
/// rather than the lowering: the offering thread can finish the work itself
/// before any pool thread reaches the offer, and on a busy machine it often
/// does. Measured in batch 0090 on the three-core `macos-14` runner, where the
/// default-pool observation totalled zero over five runs in one gate run and
/// was granted on the first run of the next — five runs were sampling that
/// host's luck. Thirty-two runs of a fixture that finishes in milliseconds
/// cost one link and a fraction of a second, and a runtime that grants nothing
/// still totals zero over all of them.
///
/// [`CountedProgram::grants_over_runs`] stops at the first granted lane, so
/// this is what the *negative* direction pays and not what a healthy host
/// pays: the property these runs support is existential — some run was granted a
/// lane — and one grant settles it. A runtime that grants nothing still makes
/// every one of the thirty-two runs and still totals zero.
pub(super) const GRANT_OBSERVATION_RUNS: usize = 32;

/// One linked build of a module against the parallel runtime and the grant
/// observer, so a case that wants several runs of one module pays for the link
/// once.
///
/// Linking is the expensive half — clang compiles the whole runtime, the
/// exhaustion floor and the observer beside the emitted module, and a run of
/// these fixtures is milliseconds. The cases below ask one program several
/// questions: what it grants at four lanes, what it grants with the variable
/// absent, and that each named opt-out grants nothing. Through the
/// link-and-run helper this replaces, each of those questions linked the same
/// executable again — five links of one module in one case.
///
/// The observer reads `wf__par_grants`, which no Whitefoot construct can name;
/// it exists exactly so a pool that never grants a lane cannot pass for one
/// that does.
pub(super) struct CountedProgram {
    executable: std::path::PathBuf,
}

impl CountedProgram {
    /// Links `module` inside `directory`, which the caller removes when it is
    /// done with the fixture.
    pub(super) fn link(module: &str, directory: &Path) -> Self {
        Self {
            executable: link_counting_grants(module, directory),
        }
    }

    /// One run, with the grant count the observer reported at process exit.
    ///
    /// `workers` is `None` for the shipped default — the variable removed from
    /// the child's environment, which is how a `--par` binary is actually
    /// handed to somebody — and `Some(count)` for a run that names a count.
    pub(super) fn run(&self, workers: Option<&str>) -> (u64, std::process::Output) {
        counted_run(&self.executable, workers)
    }

    /// What the runtime granted over at most `runs` runs, stopping at the
    /// first run that was granted a lane.
    ///
    /// A steal is a race, so one run's count samples the schedule rather than
    /// stating a property of the lowering. A fixture whose whole range is
    /// worth only a few dozen offers can lose nearly all of them to the
    /// offering thread on a saturated machine — measured down to three grants
    /// at `WF_WORKERS=4` — which would fail a per-run `granted > 0` for a
    /// reason that has nothing to do with the code under test. A total keeps
    /// exactly what those assertions are for: a runtime that grants nothing
    /// totals zero and still fails.
    ///
    /// Every caller asserts `> 0`, which is an existential observation: the first
    /// grant is the whole observation, and the runs after it re-observe
    /// something already seen. Stopping there changes neither direction of the
    /// result — the total is positive exactly when some run of the sample was
    /// granted a lane, and a runtime that grants nothing still makes all
    /// `runs` runs and still returns zero.
    pub(super) fn grants_over_runs(&self, workers: Option<&str>, runs: usize) -> u64 {
        let mut total = 0;
        for run in 0..runs {
            let (granted, output) = self.run(workers);
            assert_eq!(
                output.status.code(),
                Some(0),
                "run {run} of the counted program must succeed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            total += granted;
            if total > 0 {
                break;
            }
        }
        total
    }
}

/// Links one module against the runtime and the observer, and returns the
/// executable. Linking is the expensive half, so a case that wants several runs
/// of one module pays for it once.
fn link_counting_grants(module: &str, directory: &Path) -> std::path::PathBuf {
    let assembly = directory.join("counted.ll");
    let runtime = directory.join("counted_runtime.c");
    let floor = directory.join("counted_floor.c");
    let observer = directory.join("observer.c");
    let executable = directory.join("counted");
    std::fs::write(&assembly, module).expect("write the module");
    let parallel_source = if module_requires_writer_scheduler(module) {
        PARALLEL_COMPLETION_RUNTIME_SOURCE
    } else {
        PARALLEL_RUNTIME_SOURCE
    };
    std::fs::write(&runtime, parallel_source).expect("write the runtime");
    // The floor joins every link the driver makes, and a lane's per-thread arm
    // lives in it, so this harness links what a shipped program links.
    std::fs::write(&floor, super::FLOOR_RUNTIME_SOURCE).expect("write the floor runtime");
    std::fs::write(
        &observer,
        "#include <stdio.h>\nextern unsigned long wf__par_grants;\n__attribute__((destructor)) static void wf__par_report(void) {\n    fprintf(stderr, \"grants=%lu\\n\", wf__par_grants);\n}\n",
    )
    .expect("write the observer");
    let mut command = Command::new("/usr/bin/clang");
    command
        .arg("-pthread")
        .arg("-x")
        .arg("ir")
        .arg(&assembly)
        .arg("-x")
        .arg("c")
        .arg(&runtime)
        .arg(&floor)
        .arg(&observer);
    let _completion_units = append_completion_runtime(&mut command, module, directory);
    let linked = command
        .args(HOST_OPTIMIZATION_ARGUMENTS)
        .arg("-o")
        .arg(&executable)
        .output()
        .expect("invoke host clang");
    assert!(
        linked.status.success(),
        "the runtime and its observer must link:\n{}",
        String::from_utf8_lossy(&linked.stderr)
    );
    for path in [assembly, runtime, observer] {
        std::fs::remove_file(path).expect("remove a counted-run artifact");
    }
    executable
}

/// One run of a linked module, with the grant count the observer reported.
fn counted_run(executable: &Path, workers: Option<&str>) -> (u64, std::process::Output) {
    let mut command = Command::new(executable);
    match workers {
        Some(count) => command.env("WF_WORKERS", count),
        None => command.env_remove("WF_WORKERS"),
    };
    let output = command.output().expect("run the counted program");
    let report = String::from_utf8_lossy(&output.stderr).into_owned();
    let granted = report
        .lines()
        .find_map(|line| line.strip_prefix("grants="))
        .and_then(|count| count.trim().parse::<u64>().ok())
        .unwrap_or_else(|| panic!("the observer must report a grant count, got {report:?}"));
    (granted, output)
}

/// Every sequential clone the module defines, by symbol.
pub(super) fn clone_symbols(module: &str) -> Vec<String> {
    module
        .lines()
        .filter(|line| line.starts_with("define "))
        .filter_map(|line| line.split_once(" @wf__par_seq_"))
        .filter_map(|(_, tail)| tail.split_once('('))
        .map(|(name, _)| format!("@wf__par_seq_{name}"))
        .collect()
}

/// The module with every sequential clone definition removed, leaving the
/// overlapped world and everything the two worlds share.
fn without_clones(module: &str) -> String {
    let mut kept = String::with_capacity(module.len());
    let mut rest = module;
    while let Some(offset) = rest.find("\ndefine ") {
        let (before, definition) = rest.split_at(offset + 1);
        kept.push_str(before);
        let end = definition
            .find("\n}\n")
            .map(|at| at + 3)
            .expect("a definition must close");
        let (definition, remainder) = definition.split_at(end);
        if !definition
            .lines()
            .next()
            .is_some_and(|header| header.contains(" @wf__par_seq_"))
        {
            kept.push_str(definition);
        }
        rest = remainder;
    }
    kept.push_str(rest);
    kept
}

/// The text of one emitted function definition, from its `define` line to its
/// closing brace.
pub(super) fn function_body<'module>(module: &'module str, symbol: &str) -> &'module str {
    let opening = format!("{symbol}(");
    let start = module
        .match_indices(&opening)
        .find_map(|(offset, _)| {
            let line = module[..offset]
                .rfind('\n')
                .map_or(0, |newline| newline + 1);
            module[line..offset].starts_with("define").then_some(line)
        })
        .unwrap_or_else(|| panic!("the module must define {symbol}:\n{module}"));
    let end = module[start..]
        .find("\n}\n")
        .map(|offset| start + offset + 2)
        .expect("a definition must close");
    &module[start..end]
}

/// The fixture above with one pure builtin written *between* the two recursive
/// calls, which is the shape the adjacency window admits.
///
/// It is produced by editing the one arm rather than by copying the whole
/// program, so the two sources differ in exactly the statement under test and
/// nothing else can drift between them. The interposed statement reads the
/// node's own `w` slot — a place neither child call reaches, since each reaches
/// only its own `&uniq` payload binder — and defines a binding neither call
/// reads, so the window is permitted.
fn windowed_fold() -> Vec<u8> {
    let source = std::str::from_utf8(OVERLAPPING_FOLD).expect("the fixture is UTF-8");
    let original = "      let a = fold(node: move l);\n      let b = fold(node: move r);\n      let mixed = mix(a: a, b: b);\n";
    let windowed = "      let a = fold(node: move l);\n      let seed = irotl(deref(slot), 3_u32);\n      let b = fold(node: move r);\n      let blended = mix(a: a, b: b);\n      let mixed = ixor(blended, seed);\n";
    assert!(
        source.contains(original),
        "the windowed fixture must be an edit of the adjacent one"
    );
    source.replace(original, windowed).into_bytes()
}

/// The window's differential: a fold whose two recursive calls are separated by
/// a builtin still hands work out, and still produces the bytes its own
/// sequential lowering produces at every worker count.
///
/// This is the case the checker's widening made reachable, and it is the one
/// the widening could break. The checker now hands the backend a group whose
/// members are not adjacent statements, so the fork, the interposed
/// instructions, and the join share one straight-line edge for the first time.
/// The reference is the default compilation of the same source — a lowering
/// with no hand-out at all — so a moved read or a misplaced join shows up as a
/// byte difference rather than being present identically on both sides.
#[test]
fn a_fold_whose_calls_are_separated_by_a_builtin_hands_out_and_agrees() {
    let source = windowed_fold();
    let overlapped_module = emit_with_overlap(&source);
    assert!(
        overlapped_module.contains("call void @wf__par_publish(ptr ")
            && overlapped_module.contains(", ptr @wf__par_thunk_"),
        "the windowed fold must still hand work out, or this test is vacuous:\n{overlapped_module}"
    );

    let sequential_module = emit(&source);
    assert!(
        !module_requires_parallel_runtime(&sequential_module),
        "the reference module must contain no hand-out at all"
    );

    let directory = test_directory();
    let reference = Command::new(build_executable(&sequential_module, &directory))
        .output()
        .expect("run the module that hands nothing out");
    assert_eq!(reference.status.code(), Some(0));
    assert_eq!(
        reference.stdout.len(),
        8,
        "the fold must report eight bytes"
    );

    let overlapped = build_executable(&overlapped_module, &directory);
    let mut runs = vec![("no overlap lowering".to_owned(), reference.stdout)];
    for workers in ["1", "2", "4", "8"] {
        for _ in 0..3 {
            let output = Command::new(&overlapped)
                .env("WF_WORKERS", workers)
                .output()
                .expect("run the overlapped program");
            assert_eq!(output.status.code(), Some(0), "WF_WORKERS={workers}");
            runs.push((format!("WF_WORKERS={workers}"), output.stdout));
        }
    }
    identical(&runs).expect("a statement between the two calls must not move one byte");

    std::fs::remove_dir_all(&directory).expect("remove the test directory");
}
