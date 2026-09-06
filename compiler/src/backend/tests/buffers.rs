use crate::backend::qualification::{SystemTarget, qualify_program};
use crate::backend::target::{TargetLayout, TargetLayoutFailure, TargetObject, validate_program};

use super::system::with_ir;
use super::*;

const AFFINE_INVARIANT_BOUNDED_ALLOCATION: &[u8] =
    br#"fn allocate(n: own u64, half: own u64) -> result: own unit pure contract {
  requires half <= 500_u64;
} {
  let doubled = half * 2_u64;
  let within = n <= doubled;
  if within {
    invariant tight: n <= 1000_u64;
    let values = buffer_new(n, 0_u16);
  }
  return unit;
}

command fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;

const U64_STORE_TAKE: &[u8] = br#"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  doc "One eight-byte slot taken from the general store, whose actual alignment the selected allocator has to promise.";
  region {
    match heap_vector::<u64>(store: &uniq heap, count: 1_u64) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: taken) => {
        let values = move taken;
        return exit_status(code: 0_u8);
      }
    }
  }
}
"#;

const U64_STORE_CELL: &[u8] = br#"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  doc "One eight-byte cell taken from the same store, the other half of the same obligation S39.";
  region {
    match heap_box(store: &uniq heap, value: 7_u64) {
      Ok(value: made) => {
        let cell = move made;
        return exit_status(code: 0_u8);
      }
      Err(error: back) => {
        return exit_status(code: 70_u8);
      }
    }
  }
}
"#;

/// LEFT ON `buffer<T>` DELIBERATELY: the run has no twin for this subject,
/// and the reason is now a measured property of the store surface rather than
/// a missing arm. The store's take *is* target-validated — the case below is
/// the alignment half of exactly that validation — but a byte **ceiling**
/// against the allocator-parameter domain is not part of it: a take the store
/// cannot satisfy hands back `None`, which is an arm of the source program
/// [BLK-2], so an unproved runtime count is an ordinary program and there is
/// no boundary a run can sit just inside and just outside of. The buffer's
/// fill has no such arm, which is why this boundary is its own.
#[test]
fn affine_invariant_ceiling_controls_the_exact_selected_target_boundary() {
    with_ir(AFFINE_INVARIANT_BOUNDED_ALLOCATION, |program| {
        let host = TargetLayout::host().expect("the backend test runs on a qualified host");
        let system_target = SystemTarget::for_triple(host.triple())
            .expect("the host triple has one qualified system target");
        let qualification = qualify_program(system_target, program)
            .expect("the invariant-bounded allocation fixture must qualify");

        let exact = host.with_runtime_allocation_limits_for_test(2000, 8);
        assert_eq!(validate_program(exact, &qualification, program), Ok(()));

        let one_byte_short = host.with_runtime_allocation_limits_for_test(1999, 8);
        assert_eq!(
            validate_program(one_byte_short, &qualification, program),
            Err(TargetLayoutFailure::Unrepresentable(
                TargetObject::RuntimeSizedAllocation
            ))
        );
    });
}

/// The target stage never reports a runtime-sized allocation failure for a
/// store take the semantic stage accepted.
///
/// This is the half of `compiler/README.md`'s known defect that the store
/// surface answers. `buffer_new(n, 0_u8)` at an unproved runtime `n` passes
/// semantic checking and stops four stages later with
/// `Unrepresentable(RuntimeSizedAllocation)` and no rule; the same count at
/// `heap_vector::<u8>` reaches the same target stage and emits, because the
/// row hands back an `Option` and a store that cannot satisfy the take is the
/// `None` arm the writer already wrote [BLK-2]. What still refuses an
/// unproved count is [OP-9] itself, at the source, with a rule and a residual
/// — at any element type whose stride makes the fit goal underivable.
#[test]
fn a_store_take_of_an_unbounded_runtime_count_emits_rather_than_stopping_at_the_target() {
    const UNBOUNDED_STORE_TAKE: &[u8] = br#"command fn main(command.args as args: own Args, command.heap as heap: own Heap) -> status: own ExitStatus reads(args, heap), writes(heap), allocates(heap) {
  doc "The count is the invocation's own argument count, which no source fact bounds.";
  let n = 0_u64;
  region {
    set n = args_count(args: &args);
  }
  region {
    match heap_vector::<u8>(store: &uniq heap, count: n) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: taken) => {
        let values = move taken;
        return exit_status(code: 0_u8);
      }
    }
  }
}
"#;
    with_ir(UNBOUNDED_STORE_TAKE, |program| {
        let host = TargetLayout::host().expect("the backend test runs on a qualified host");
        let system_target = SystemTarget::for_triple(host.triple())
            .expect("the host triple has one qualified system target");
        let qualification = qualify_program(system_target, program)
            .expect("the unbounded store take must qualify");
        assert_eq!(validate_program(host, &qualification, program), Ok(()));
    });
    let output = compile_and_run(&compile(UNBOUNDED_STORE_TAKE));
    assert!(output.status.success());
}

/// The store surface's own alignment boundary, for the run and for the cell.
///
/// A general store hands out raw storage its host allocator supplies, so the
/// element's *actual* target alignment must be one that allocator promises —
/// the same obligation `buffer_new` and `box_new` carry, now read off the
/// store's two rows [BLK-2, S39, STOR-6]. Both directions are pinned at the
/// exact boundary: eight-byte alignment admits an eight-byte slot and a
/// four-byte guarantee refuses it, and refuses it as a runtime-sized
/// allocation rather than as a representation failure, because what is short
/// is the allocator's promise and not the language ceiling.
#[test]
fn a_store_take_and_a_store_cell_must_fit_the_selected_allocator_alignment() {
    for fixture in [U64_STORE_TAKE, U64_STORE_CELL] {
        with_ir(fixture, |program| {
            let host = TargetLayout::host().expect("the backend test runs on a qualified host");
            let system_target = SystemTarget::for_triple(host.triple())
                .expect("the host triple has one qualified system target");
            let qualification = qualify_program(system_target, program)
                .expect("the store-alignment fixture must qualify");

            // The byte domain stays the host's own: a store take is not judged
            // against an allocator byte ceiling at all, and cutting the
            // address-index domain to the take's own size would refuse the
            // run's thirty-two-byte descriptor before the alignment is
            // reached. Only the alignment guarantee moves here.
            let byte_domain = i64::MAX as u64;
            let exact = host.with_runtime_allocation_limits_for_test(byte_domain, 8);
            assert_eq!(validate_program(exact, &qualification, program), Ok(()));

            let one_alignment_step_short =
                host.with_runtime_allocation_limits_for_test(byte_domain, 4);
            assert_eq!(
                validate_program(one_alignment_step_short, &qualification, program),
                Err(TargetLayoutFailure::Unrepresentable(
                    TargetObject::RuntimeSizedAllocation
                ))
            );
        });
    }
}

#[test]
fn weigh_invariant_proves_domains_then_erases_before_llvm() {
    let source = br#"fn weigh(weights: own Slice<u8>, count: own u64) -> total: own u32 reads(weights) contract {
  define capacity = len_of(weights);
  requires count <= capacity;
  requires count <= 1000_u64;
  ensures total <= 255000_u32;
} {
  let sum = 0_u32;
  for (
    i in 0_u64..count,
    invariant per_byte: sum <= 255_u32 * i
  ) {
    let w = weights[i];
    let wide = cvt::<u8, u32>(w);
    set sum = sum + wide;
  }
  return sum;
}

command fn main() -> status: own ExitStatus pure {
  let empty = fixed_vector::<u8, 4>();
  let one = place_back(vector: move empty, value: 7_u8);
  let two = place_back(vector: move one, value: 7_u8);
  let three = place_back(vector: move two, value: 7_u8);
  let weights = place_back(vector: move three, value: 7_u8);
  let code = 0_u8;
  region {
    let window = slice_of(&weights);
    let total = weigh(weights: window, count: 4_u64);
    if total != 28_u32 {
      set code = 1_u8;
    }
  }
  return exit_status(code: code);
}
"#;
    let llvm = compile(source);
    let weigh = emitted_function(&llvm, "weigh");

    // INV-1 and OP-2 discharge before lowering. The loop therefore contains
    // one plain integer addition and no runtime representation of `per_byte`.
    assert!(weigh.contains("add i32"));
    assert!(!weigh.contains(".with.overflow."));
    assert!(!weigh.contains("call void @wf_trap"));
    assert!(!llvm.contains("per_byte"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// LEFT ON `buffer<T>` DELIBERATELY: its subject includes the buffer's own
/// refusal edge — `buffer.fill.oom.` reaching `wf_resource_abort()` — and the
/// run has no such edge. A store take is refusable in the source: it hands
/// back an `Option` the writer matches, so the refusal is an arm of the
/// program rather than an emitted abort, and there is nothing for these
/// assertions to name. Migrating the rest would mean dropping them.
#[test]
fn primitive_buffers_cross_functions_update_and_free_once() {
    let source = br#"fn bounded_count(n: own u64) -> result: own u64 pure contract {
  ensures result <= 4611686018427387903_u64;
} {
  if n <= 4611686018427387903_u64 {
    return n;
  } else {
    return 4611686018427387903_u64;
  }
}

fn make(n: own u64) -> result: own buffer<u16> pure {
  let bounded = bounded_count(n: n);
  return buffer_new(bounded, 3_u16);
}

fn replacement() -> result: own u16 pure {
  return 9_u16;
}

command fn main() -> status: own ExitStatus pure {
  let values = make(n: 4_u64);
  let length = len_of(values);
  let stored = 0_u16;
  let code = 0_u8;
  if 2_u64 < length {
    set values[2_u64] = replacement();
    set stored = values[2_u64];
  } else {
    set code = 3_u8;
  }
  if code == 0_u8 {
    if length != 4_u64 {
      set code = 1_u8;
    }
    if stored != 9_u16 {
      set code = 2_u8;
    }
  }
  return exit_status(code: code);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let make = emitted_function(&llvm, "make");
    // The verified scalar normalizer summary discharges allocation, while a
    // local length branch discharges both indexed sites. The RHS is evaluated
    // once before the target commits one store, with no runtime proof fallback.
    let rhs = main
        .find("call i16 @wf_replacement")
        .expect("SET-1 must evaluate its RHS once");
    let store = main
        .find("store i16 %v")
        .expect("SET-1 must commit one element store");
    assert!(rhs < store);
    assert!(!main.contains("call void @wf_trap"));
    assert_eq!(main.matches("call void @free").count(), 1);
    assert!(!make.contains("call void @free"));

    // The proved count ceiling times the u16 stride fits the selected target's
    // byte domain. Target layout therefore admits the dynamic allocation and
    // the emitter needs only the allocator's null-result edge.
    assert!(make.contains("call ptr @malloc"));
    assert!(make.contains("icmp ne ptr"));
    assert!(make.contains("buffer.fill.oom."));
    assert!(make.contains("call void @wf_resource_abort()"));
    for absent in [
        "buffer.fill.target.",
        "@wf_target_domain_abort",
        "@.wf_resource.target_domain",
    ] {
        assert!(
            !llvm.contains(absent),
            "a target-qualified allocation must not emit {absent}:\n{llvm}"
        );
    }

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// LEFT ON `buffer<T>` DELIBERATELY: it asserts that a proved length emits
/// no `buffer.fill.target.` guard. The run surface emits no target guard for
/// any length, proved or not, so the same assertion over a run would pass
/// vacuously and would stop being evidence of anything.
#[test]
fn buffer_length_qualifies_same_element_reallocation_without_a_target_guard() {
    let source = br#"fn refill(source: own buffer<u8>) -> result: own buffer<u8> reads(source) {
  let length = len_of(source);
  return buffer_new(length, 0_u8);
}

command fn main() -> status: own ExitStatus pure {
  let initial = buffer_new(4_u64, 7_u8);
  let copied = refill(source: move initial);
  let length = len_of(copied);
  if length != 4_u64 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let refill = emitted_function(&llvm, "refill");
    assert!(refill.contains("call ptr @malloc"));
    for absent in [
        "buffer.fill.target.",
        "@wf_target_domain_abort",
        "@.wf_resource.target_domain",
    ] {
        assert!(
            !llvm.contains(absent),
            "a buffer-length target invariant must not emit {absent}:\n{llvm}"
        );
    }

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn op9_overflow_is_rejected_before_lowering() {
    let source = br#"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_vector::<u64>(store: &uniq heap, count: 18446744073709551615_u64) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: fresh) => {
        let values = move fresh;
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-9"));
    assert!(
        failure
            .detail()
            .contains("UndischargedAllocationFitObligation")
    );
}

#[test]
fn an_out_of_bounds_run_set_is_an_op4_compile_rejection() {
    // The take's count fixes the run's length, so 2 < 2 is underivable and
    // the program rejects at compile time with the residual the buffer
    // origin gave, byte for byte [OP-4, ENT-6].
    let source = br#"fn replacement() -> result: own u8 pure {
  return 9_u8;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_vector::<u8>(store: &uniq heap, count: 2_u64) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: fresh) => {
        let values = move fresh;
        set values[2_u64] = replacement();
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-4"));
    assert!(failure.detail().contains("2_u64 < len_of(values)"));
}

#[test]
fn an_empty_run_has_zero_length_and_a_normal_release() {
    let source = br#"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_vector::<u8>(store: &uniq heap, count: 0_u64) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: fresh) => {
        let values = move fresh;
        let length = len_of(values);
        if length != 0_u64 {
          return exit_status(code: 1_u8);
        }
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let output = compile_and_run(&compile(source));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn run_cleanup_is_explicit_on_return_and_break_edges() {
    let source = br#"fn cleanup(flag: own Bool, store: &uniq Heap) -> result: own unit reads(store), writes(store), allocates(store) {
  doc "Every edge that leaves this scope holding a run carries that run's release: the early return, the loop break, and the final return.";
  region {
    match heap_vector::<u8>(store: &uniq deref(store), count: 2_u64) {
      None() => {
        return unit;
      }
      Some(value: fresh) => {
        let values = move fresh;
        if flag {
          return unit;
        }
        loop @done {
          match heap_vector::<u16>(store: &uniq deref(store), count: 1_u64) {
            None() => {
              break @done;
            }
            Some(value: spare) => {
              let scratch = move spare;
              break @done;
            }
          }
        }
        return unit;
      }
    }
  }
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  let true_value = True();
  let false_value = False();
  region {
    cleanup(flag: true_value, store: &uniq heap);
  }
  region {
    cleanup(flag: false_value, store: &uniq heap);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let cleanup = emitted_function(&llvm, "cleanup");
    // Three release sites, exactly as the buffer shape had: the early
    // return, the loop break, and the final return each carry the release of
    // what that edge holds [STOR-3].
    assert_eq!(cleanup.matches("call void @free").count(), 3);
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_mutable_buffer_checksum_executes() {
    let output = compile_and_run(&compile(include_bytes!(
        "../../../../tests/conformance/cases/x-buffer-mutable-checksum-run.wf"
    )));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn borrowed_columns_cross_helpers_without_transferring_ownership() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-buffer-borrowed-columns-run.wf"
    ));
    let fill = emitted_function(&llvm, "fill");
    let fold = emitted_function(&llvm, "fold");
    let main = emitted_function(&llvm, "main");
    assert!(fill.contains("store i64"));
    assert!(fold.contains("load i64"));
    assert!(!fill.contains("call void @free"));
    assert!(!fold.contains("call void @free"));
    // Both declared length requirements and the counted-range binder facts are
    // checked before lowering. Neither helper retains a runtime proof check.
    assert_eq!(fill.matches("call void @wf_trap").count(), 0);
    assert_eq!(fold.matches("call void @wf_trap").count(), 0);
    // Each counted loop retains exactly its own continuation comparison. No
    // second comparison remains for either proved buffer bound.
    assert_eq!(fill.matches("icmp ult i64").count(), 1);
    assert_eq!(fold.matches("icmp ult i64").count(), 1);
    // B7c4b-1: the two columns are store runs held in one struct and lent as
    // views, so the requirement branches are the four length checks, the two
    // non-wrap checks the view formations submit, the two store refusals, the
    // checksum branch and the success exit — ten in all — and the releases are
    // the general store's, one per run on each edge that leaves holding them.
    assert!(!main.contains("call void @wf_trap"));
    assert_eq!(main.matches("call i8 @wf.sys.exit_status.v1").count(), 10);
    assert_eq!(main.matches("call void @free").count(), 17);
    assert!(main.contains("call i8 @wf_fill"));
    assert!(main.contains("call i64 @wf_fold"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// LEFT ON `buffer<T>` DELIBERATELY: [BLK-4] refuses a `&uniq` whose
/// referent reaches a run, so a pool of runs cannot be lent as one struct
/// pointer at all. The address-path property this pins — one caller-storage
/// update through a single `ptr` parameter — has no run shape to hold it;
/// `compiler_independent_borrowed_pool_tree_executes` above records what the
/// migrated pool does instead, which is to lend two views and a scalar.
#[test]
fn borrowed_struct_projection_updates_caller_storage_through_one_address_path() {
    let source = br#"struct Pool {
  left: buffer<u64>;
  right: buffer<u64>;
  count: u64;
}

fn update(pool: &uniq Pool) -> result: own unit reads(pool.left), writes(pool.left, pool.count) {
  let spare = len_of(deref(pool).left);
  let ok = 1_u64 < spare;
  if ok {
    set deref(pool).left[1_u64] = 13_u64;
    set deref(pool).count = 1_u64;
  }
  return unit;
}

fn observe(pool: &Pool) -> result: own u64 reads(pool.left, pool.count) {
  let spare = len_of(deref(pool).left);
  let ok = 1_u64 < spare;
  let count = deref(pool).count;
  if ok {
    let value = deref(pool).left[1_u64];
    return value +wrap count;
  } else {
    return count;
  }
}

command fn main() -> status: own ExitStatus pure {
  let left = buffer_new(2_u64, 0_u64);
  let right = buffer_new(2_u64, 0_u64);
  let pool = Pool(left: move left, right: move right, count: 0_u64);
  let code = 0_u8;
  let apply = True();
  if apply {
    region {
      update(pool: &uniq pool);
    }
  }
  region {
    let observed = observe(pool: &pool);
    if observed != 14_u64 {
      set code = 1_u8;
    }
  }
  return exit_status(code: code);
}
"#;
    let llvm = compile(source);
    let update = emitted_function(&llvm, "update");
    let observe = emitted_function(&llvm, "observe");
    let main = emitted_function(&llvm, "main");
    assert!(update.starts_with("define internal i8 @wf_update(ptr "));
    assert!(observe.starts_with("define internal i64 @wf_observe(ptr "));
    assert!(main.contains("call i8 @wf_update(ptr "));
    assert!(main.contains("call i64 @wf_observe(ptr "));
    assert!(!update.contains("call void @free"));
    assert!(!observe.contains("call void @free"));
    assert_eq!(main.matches("call void @free").count(), 2);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_borrowed_pool_tree_executes() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-borrowed-pool-tree-run.wf"
    ));
    let build = emitted_function(&llvm, "build");
    let checksum = emitted_function(&llvm, "checksum");
    let main = emitted_function(&llvm, "main");
    // B7c4b-1: [BLK-4] refuses a `&uniq` whose referent reaches a run, so the
    // pool is lent as two views and a scalar borrow rather than as one struct;
    // each helper therefore takes two descriptors by value where it took one
    // struct pointer.
    assert!(build.starts_with("define internal %wf.t4 @wf_build({ ptr, i64 } "));
    assert!(build.contains(", i32 "));
    assert!(checksum.starts_with("define internal %wf.t4 @wf_checksum({ ptr, i64 } "));
    assert!(checksum.contains(", i64 "));
    assert!(!build.contains("call void @free"));
    assert!(!checksum.contains("call void @free"));
    // Bounds and arithmetic failures are typed results rather than written proofs,
    // so build and checksum contain no trap edge and main still has its five
    // status exits. B7c4b-1: the two runs come from one bump extent laid out in
    // this activation's frame, so the program reaches the host allocator on no
    // path and there is no free to count.
    assert!(!build.contains("call void @wf_trap"));
    assert!(!checksum.contains("call void @wf_trap"));
    assert!(!main.contains("call void @wf_trap"));
    assert_eq!(main.matches("call i8 @wf.sys.exit_status.v1").count(), 5);
    assert_eq!(main.matches("call void @free").count(), 0);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// The case counts lines, words and bytes over two chunks and combines the
/// two summaries. It was migrated from `buffer<u8>` to `FixedVector<u8, n>`,
/// which makes `summarize` a const generic over its chunk length: the two
/// chunk lengths therefore reach the backend as two monomorphized instances,
/// each taking its own inline run by value instead of one heap descriptor,
/// and the whole program allocates nothing. The assertions are those facts.
#[test]
fn compiler_independent_wc_chunk_summary_executes() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-wc-chunk-summary-run.wf"
    ));
    let four_bytes = emitted_function(&llvm, "summarize$instance$2");
    let one_byte = emitted_function(&llvm, "summarize$instance$3");
    let combine = emitted_function(&llvm, "combine");
    assert!(four_bytes.starts_with("define internal i8 @wf_summarize$instance$2(ptr "));
    assert!(four_bytes.contains(", { [4 x i8], i64, i64 } "));
    assert!(one_byte.starts_with("define internal i8 @wf_summarize$instance$3(ptr "));
    assert!(one_byte.contains(", { [1 x i8], i64, i64 } "));
    assert!(combine.starts_with("define internal i8 @wf_combine(ptr "));
    assert_eq!(
        combine
            .lines()
            .next()
            .expect("combine signature")
            .matches("ptr %v")
            .count(),
        3
    );
    assert!(!llvm.contains("call ptr @malloc"));
    assert!(!llvm.contains("call void @free"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn projected_buffer_target_is_formed_once_before_rhs() {
    let source = br#"struct Columns {
  left: buffer<u16>;
  right: buffer<u16>;
}

fn replacement() -> result: own u16 pure {
  return 9_u16;
}

fn update(columns: own Columns) -> result: own Columns reads(columns.left), writes(columns.left) {
  let spare = len_of(columns.left);
  let ok = 1_u64 < spare;
  if ok {
    set columns.left[1_u64] = replacement();
  }
  return move columns;
}

command fn main() -> status: own ExitStatus pure {
  let left = buffer_new(2_u64, 0_u16);
  let right = buffer_new(2_u64, 0_u16);
  let columns = Columns(left: move left, right: move right);
  let updated = update(columns: move columns);
  let updated_room = len_of(updated.left);
  let updated_ok = 1_u64 < updated_room;
  if updated_ok {
    let value = updated.left[1_u64];
    if value != 9_u16 {
      return exit_status(code: 1_u8);
    }
  } else {
    return exit_status(code: 2_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let update = emitted_function(&llvm, "update");
    // The length read projects the field once for the explicit control; the
    // target projects it once more at the store, with no language trap.
    assert_eq!(update.matches("extractvalue %wf.t0").count(), 2);
    let guard = update
        .find("icmp ult i64")
        .expect("the explicit control must test the projected buffer length");
    let rhs = update
        .find("call i16 @wf_replacement")
        .expect("the RHS must execute once");
    let store = update
        .find("store i16")
        .expect("the target must receive one store");
    assert!(guard < rhs && rhs < store);
    assert!(!update.contains("call void @wf_trap"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn nested_struct_cleanup_releases_run_fields_in_reverse_order() {
    let source = br#"struct Pair['s] {
  first: Vector<'s, u8>;
  second: Vector<'s, u16>;
}

struct Owner['s] {
  prefix: Vector<'s, u32>;
  pair: Pair<'s>;
  suffix: Vector<'s, u64>;
}

fn release['s](owner: own Owner<'s>, store: &uniq Heap<'s>) -> result: own unit writes(store) {
  doc "Holds the whole nested owner and nothing else, so its one return edge carries exactly the four field releases in reverse declared order.";
  return unit;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_vector::<u8>(store: &uniq heap, count: 1_u64) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: first) => {
        match heap_vector::<u16>(store: &uniq heap, count: 1_u64) {
          None() => {
            return exit_status(code: 70_u8);
          }
          Some(value: second) => {
            let pair = Pair(first: move first, second: move second);
            match heap_vector::<u32>(store: &uniq heap, count: 1_u64) {
              None() => {
                return exit_status(code: 70_u8);
              }
              Some(value: prefix) => {
                match heap_vector::<u64>(store: &uniq heap, count: 1_u64) {
                  None() => {
                    return exit_status(code: 70_u8);
                  }
                  Some(value: suffix) => {
                    let owner = Owner(prefix: move prefix, pair: move pair, suffix: move suffix);
                    region {
                      release(owner: move owner, store: &uniq heap);
                    }
                    return exit_status(code: 0_u8);
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
"#;
    let llvm = compile(source);
    // `release` holds the whole nested owner and nothing else, so its one
    // return edge carries exactly the four field releases in reverse declared
    // order — the number and the order the buffer fields had.
    let release = emitted_function(&llvm, "release");
    assert_eq!(release.matches("call void @free").count(), 4);
    // A store take is refusable where `buffer_new` aborted, so `main` also
    // carries the releases each refusal edge owes: the four arms hold 0, 1, 2
    // and 3 runs, and the success edge hands the owner to `release`.
    let main = emitted_function(&llvm, "main");
    assert_eq!(main.matches("call void @free").count(), 6);
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn a_projected_run_move_releases_only_residual_siblings() {
    let source = br#"struct Pair['s] {
  first: Vector<'s, u8>;
  second: Vector<'s, u8>;
}

struct Owner['s] {
  prefix: Vector<'s, u8>;
  pair: Pair<'s>;
  suffix: Vector<'s, u8>;
}

fn take['s](owner: own Owner<'s>) -> result: own Vector<'s, u8> pure {
  doc "Takes one field out; the three residual siblings are released here, on the store whose provider this scope holds.";
  return move owner.pair.first;
}

command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_vector::<u8>(store: &uniq heap, count: 1_u64) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: first) => {
        match heap_vector::<u8>(store: &uniq heap, count: 1_u64) {
          None() => {
            return exit_status(code: 70_u8);
          }
          Some(value: second) => {
            let pair = Pair(first: move first, second: move second);
            match heap_vector::<u8>(store: &uniq heap, count: 1_u64) {
              None() => {
                return exit_status(code: 70_u8);
              }
              Some(value: prefix) => {
                match heap_vector::<u8>(store: &uniq heap, count: 1_u64) {
                  None() => {
                    return exit_status(code: 70_u8);
                  }
                  Some(value: suffix) => {
                    let owner = Owner(prefix: move prefix, pair: move pair, suffix: move suffix);
                    region {
                      let retained = take(owner: move owner);
                    }
                    return exit_status(code: 0_u8);
                  }
                }
              }
            }
          }
        }
      }
    }
  }
}
"#;
    let llvm = compile(source);
    let take = emitted_function(&llvm, "take");
    // Three residual siblings released where the projected field left, the
    // number the buffer shape had.
    assert_eq!(take.matches("call void @free").count(), 3);
    // One retained run released in `main`, plus the releases the four
    // refusable takes owe on their arms — 0, 1, 2 and 3 runs held.
    assert_eq!(
        emitted_function(&llvm, "main")
            .matches("call void @free")
            .count(),
        7
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn compiler_independent_struct_of_buffers_checksum_executes() {
    let output = compile_and_run(&compile(include_bytes!(
        "../../../../tests/conformance/cases/x-struct-of-buffers-checksum-run.wf"
    )));
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn affine_element_buffers_construct_replace_vacate_and_drop_per_element() {
    let source = br#"command fn main() -> status: own ExitStatus pure {
  let slots = buffer_vacant::<box<u64>>(3_u64);
  let first = box_new(11_u64);
  let wrapped = Some<box<u64>>(value: move first);
  let vacant = replace slots[0_u64] = move wrapped;
  match vacant {
    None() => {
    }
    Some(value: stray) => {
      return exit_status(code: 1_u8);
    }
  }
  let second = box_new(22_u64);
  let wrapped2 = Some<box<u64>>(value: move second);
  let vacant2 = replace slots[2_u64] = move wrapped2;
  match vacant2 {
    None() => {
    }
    Some(value: stray2) => {
      return exit_status(code: 2_u8);
    }
  }
  let taken = replace slots[0_u64] = None<box<u64>>();
  match taken {
    None() => {
      return exit_status(code: 3_u8);
    }
    Some(value: payload) => {
      let observed = deref(payload);
      if observed != 11_u64 {
        return exit_status(code: 4_u8);
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    // OP-9 was discharged statically. Target qualification supplies the
    // selected layout, so no language overflow guard remains.
    assert!(!main.contains("@llvm.umul.with.overflow.i64"));
    assert!(main.contains("ptrtoint (ptr getelementptr (%wf.t"));
    assert!(!main.contains("buffer.vacant.overflow"));
    // Every element starts as the tag-zero None(): the aggregate
    // zeroinitializer stored through the init loop.
    let body = main
        .find("buffer.vacant.body")
        .expect("the all-None init loop must be emitted");
    assert!(main[body..].contains("zeroinitializer"));
    // The SET-2 element commit is one aggregate load and one aggregate
    // store through the same element address arithmetic.
    assert!(main.contains("load %wf.t"));
    assert!(main.contains("store %wf.t"));
    // The scope-exit drop is the per-element loop [STOR-3]: the buffer
    // helper drops each element through the enum helper, then frees.
    let helper_start = llvm
        .find("define private void @wf.drop.buffer.t")
        .expect("an element type with a drop derives the buffer drop loop");
    let helper_end = llvm[helper_start..]
        .find("\n}\n")
        .map(|offset| helper_start + offset)
        .expect("buffer drop helper must be complete");
    let helper = &llvm[helper_start..helper_end];
    assert!(helper.contains("call void @wf.drop.t"));
    assert_eq!(helper.matches("call void @free").count(), 1);
    assert!(main.contains("call void @wf.drop.buffer.t"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn trivially_droppable_affine_elements_keep_the_single_free() {
    let source = br#"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_vector::<Option<u32>>(store: &uniq heap, count: 4_u64) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: fresh) => {
        let slots = move fresh;
        for @vacate (
          at in 0_u64..4_u64,
          invariant grown: len_of(slots) >= at,
          invariant spare: room_of(slots) + at >= 4_u64,
          invariant flat: head_of(slots) <= 0_u64
        ) {
          let empty = None<u32>();
          set slots = place_back(vector: move slots, value: move empty);
        }
        let filled = Some<u32>(value: 7_u32);
        let vacant = replace slots[2_u64] = move filled;
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    // An element type whose own drop derives no action keeps the composite
    // action exactly the heap free [STOR-3]: no drop loop is generated.
    assert!(!llvm.contains("@wf.drop.buffer"));
    assert!(!llvm.contains("@wf.drop.run"));
    let main = emitted_function(&llvm, "main");
    assert_eq!(main.matches("call void @free").count(), 1);
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn a_vacant_run_op9_overflow_is_rejected_before_lowering() {
    let source = br#"command fn main(command.heap as heap: own Heap) -> status: own ExitStatus reads(heap), writes(heap), allocates(heap) {
  region {
    match heap_vector::<Option<u32>>(store: &uniq heap, count: 18446744073709551615_u64) {
      None() => {
        return exit_status(code: 70_u8);
      }
      Some(value: fresh) => {
        let slots = move fresh;
      }
    }
  }
  return exit_status(code: 0_u8);
}
"#;
    let failure = compile_rejection(source);
    assert_eq!(failure.rule_id(), Some("OP-9"));
    assert!(
        failure
            .detail()
            .contains("UndischargedAllocationFitObligation")
    );
}
