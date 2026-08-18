use super::*;

#[test]
fn statement_scoped_child_reborrows_resume_their_parent() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-child-reborrow-run.wf"
    ));
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// Borrows of directly stored scalar and enum content are the address of that
/// storage, so a write through `&uniq` is visible to the owner and a read
/// through `&'r` reloads it [OWN-2, OWN-5, TYPE-7].
#[test]
fn general_scalar_and_enum_borrows_execute_through_host_llvm() {
    for source in [
        include_bytes!("../../../../tests/conformance/cases/own2-pos-three-modes.wf").as_slice(),
        include_bytes!("../../../../tests/conformance/cases/own5-pos-read-through-holder.wf")
            .as_slice(),
        include_bytes!("../../../../tests/conformance/cases/own7-pos-distinct-noverlap.wf")
            .as_slice(),
        include_bytes!("../../../../tests/conformance/cases/own11-pos-loop-inner-region.wf")
            .as_slice(),
        include_bytes!("../../../../tests/conformance/cases/x-typ-uniq-deref-write-roundtrip.wf")
            .as_slice(),
        include_bytes!("../../../../tests/conformance/cases/x-enum-borrow-payload-live.wf")
            .as_slice(),
        include_bytes!(
            "../../../../tests/conformance/cases/x-integ-coin-borrow-match-score-twice.wf"
        )
        .as_slice(),
    ] {
        let llvm = compile(source);
        let output = compile_and_run(&llvm);
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(output.stderr.is_empty());
    }
}

/// A borrow-returning call is typed by the callee's declared result mode: the
/// call site receives an address, and a discarded borrow result is evaluated
/// without any drop of the referent it does not own [OWN-2, TYPE-7, STOR-3].
///
/// Regression: the call definition used the referent value type and the
/// borrow-typed callee result made the module invalid IR, an internal error
/// on accepted source.
#[test]
fn a_discarded_borrow_returning_call_compiles_and_runs() {
    let llvm = compile(
        br#"fn source['r](x: &'r i32) -> &'r i32 pure {
  return x;
}

fn main() -> own unit traps {
  let v = 5_i32;
  region 'a {
    let h = &'a v;
    source<'a>(x: h);
  }
  check ieq(v, 5_i32) else trap "owner value changed";
  return unit;
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// The v0.31-candidate chain executes end to end (test-only extension
/// checker): a holder's candidate child feeds a borrow-returning callee, the
/// bound result becomes a holder, and a statement-scoped grandchild of that
/// result carries the callee write back into the owner's storage.
///
/// A *suffixed* reborrow (`&uniq 'r deref(p).left`) remains an explicit
/// RegionsAndBorrows capability stop in both admitted positions, so the
/// executable chain stays on whole-referent reborrows.
#[test]
fn extension_chains_execute_and_write_the_owners_storage() {
    let llvm = emit_reborrow_extension(
        br#"fn passthru['r0](x: &uniq 'r0 i32) -> &uniq 'r0 i32 pure {
  return &uniq 'r0 deref(x);
}

fn bump['r](n: &uniq 'r i32) -> own unit writes('r) {
  set deref(n) = 42_i32;
  return unit;
}

fn main() -> own unit traps {
  let v = 1_i32;
  region 'a {
    let h = &uniq 'a v;
    let r = passthru<'a>(x: &uniq 'a deref(h));
    region 'c {
      bump<'c>(n: &uniq 'c deref(r));
    }
  }
  check ieq(v, 42_i32) else trap "chain write lost";
  return unit;
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// A borrow-mode parameter crosses the call boundary as an address, so the
/// callee's write lands in the caller's storage.
#[test]
fn a_unique_scalar_borrow_parameter_writes_the_callers_storage() {
    let llvm = compile(
        br#"fn bump['r](n: &uniq 'r i32) -> own unit writes('r) {
  set deref(n) = 42_i32;
  return unit;
}

fn main() -> own unit traps {
  let a = 0_i32;
  region 'r {
    bump<'r>(n: &uniq 'r a);
  }
  check ieq(a, 42_i32) else trap "callee write lost";
  return unit;
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// [OWN-13] matching an enum through a shared borrow leaves the scrutinee
/// live and derives a shared binder on an affine struct payload;
/// `deref(binder).field` reads through that provenance without transferring
/// ownership, and the matched-through root remains usable afterwards. This
/// is the own13-pos-borrow-affine-payload capability stated with conforming
/// source: the binder spelling is distinct from its field [GRAM-10], the
/// unit has a `main` [FN-7], and the read through the caller region is
/// declared [EFF-2].
#[test]
fn borrow_match_preserves_provenance_on_an_affine_payload() {
    let llvm = compile(
        br#"struct Pair {
  left: i32;
  right: i32;
}

enum Packet {
  Data(item: Pair);
  Empty();
}

fn inspect['r](packet: &'r Packet) -> own i32 reads('r) {
  match deref(packet) {
    Data(item: payload) => {
      return deref(payload).left;
    }
    Empty() => {
      return 0_i32;
    }
  }
}

fn main() -> own unit traps {
  let pair = Pair(left: 41_i32, right: 1_i32);
  let packet = Data(item: move pair);
  let fallback = Empty();
  region 'r {
    let held = &'r packet;
    let read = inspect<'r>(packet: held);
    check ieq(read, 41_i32) else trap "payload left";
    let hollow = &'r fallback;
    let zero = inspect<'r>(packet: hollow);
    check ieq(zero, 0_i32) else trap "empty arm";
  }
  return unit;
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
