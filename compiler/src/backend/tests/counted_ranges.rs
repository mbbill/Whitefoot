use super::{compile, compile_and_run, emitted_function};

/// The dedicated counted CFG executes empty, reversed, singleton, MAX-edge,
/// captured-endpoint, shared-binder-borrow, nested-break, and enclosing-break
/// paths through the normal native backend. `main` reports any result drift
/// through its status; the counted worker itself remains pure and has no trap
/// fallback.
#[test]
fn counted_ranges_execute_exact_half_open_edges_without_a_hidden_trap() {
    let source = br#"fn exercise() -> result: own u64 pure {
  let total = 0_u64;
  for @empty (i in 4_u64..4_u64) {
    set total = total +wrap 100_u64;
  }
  for @reversed (i in 5_u64..2_u64) {
    set total = total +wrap 100_u64;
  }
  for @singleton (i in 0_u64..1_u64) {
    set total = total +wrap 1_u64;
  }
  let max_visits = 0_u64;
  let max_seen = 0_u64;
  for @max_one (i in 18446744073709551614_u64..18446744073709551615_u64) {
    set max_visits = max_visits +wrap 1_u64;
    set max_seen = i;
    set total = total +wrap 2_u64;
  }
  for @max_empty (i in 18446744073709551615_u64..18446744073709551615_u64) {
    set total = total +wrap 100_u64;
  }
  if ine(max_visits, 1_u64) {
    return 1000_u64;
  }
  if ine(max_seen, 18446744073709551614_u64) {
    return 1001_u64;
  }
  let upper = 3_u64;
  for @captured (i in 0_u64..upper) {
    region 'r {
      let held = &'r i;
      let seen = deref(held);
      set total = total +wrap seen;
    }
    set upper = 0_u64;
  }
  for @outer (i in 0_u64..4_u64) {
    for @inner (j in 0_u64..4_u64) {
      set total = total +wrap 1_u64;
      break @inner;
    }
    if ieq(i, 1_u64) {
      break @outer;
    }
  }
  return total;
}

command fn main() -> status: own ExitStatus pure {
  let result = exercise();
  if ine(result, 8_u64) {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    let llvm = compile(source);
    let worker = emitted_function(&llvm, "exercise");
    assert!(worker.contains("phi i64"));
    assert!(worker.contains("icmp ult i64"));
    assert!(!worker.contains("wf_trap"));

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
