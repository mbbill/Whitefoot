use super::*;

/// Opaque descriptors remain replaceable owners even though their payload has
/// no source fields. The host supplies inert identities, never OS descriptors;
/// every owner is returned so unresolved release-origin routing is not needed.
#[test]
fn opaque_resource_borrows_write_back_through_calls_fields_and_reborrows() {
    let source = br#"struct Holder {
  before: u64;
  file: ReadFile;
  after: u64;
}

fn exclusive['r](target: &uniq 'r ReadFile) -> result: &uniq 'r ReadFile pure {
  return &uniq 'r deref(target);
}

fn paused(target: &uniq ReadFile) -> result: own unit pure {
  return unit;
}

fn resumed['r](target: &uniq 'r ReadFile) -> result: &uniq 'r ReadFile pure {
  region {
    paused(target: &uniq deref(target));
  }
  return move target;
}

fn exchange(target: &uniq ReadFile, incoming: own ReadFile) -> previous: own ReadFile reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn relay(target: &uniq ReadFile, incoming: own ReadFile) -> previous: own ReadFile reads(target), writes(target) {
  region {
    return exchange(target: &uniq deref(target), incoming: move incoming);
  }
}

fn through_return(target: &uniq ReadFile, incoming: own ReadFile) -> previous: own ReadFile reads(target), writes(target) {
  region {
    let borrowed = resumed(target: &uniq deref(target));
    return relay(target: move borrowed, incoming: move incoming);
  }
}

fn rotate(file: own ReadFile, incoming: own ReadFile) -> (current: own ReadFile, previous: own ReadFile) reads(file), writes(file) {
  region {
    let previous = exchange(target: &uniq file, incoming: move incoming);
    return move file, move previous;
  }
}

fn rotate_field(file: own ReadFile, incoming: own ReadFile) -> (current: own ReadFile, previous: own ReadFile, before: own u64, after: own u64) reads(file), writes(file) {
  let holder = Holder(before: 101_u64, file: move file, after: 103_u64);
  region {
    let previous = through_return(target: &uniq holder.file, incoming: move incoming);
    let Holder(before: before, file: current, after: after) = move holder;
    return move current, move previous, before, after;
  }
}

fn main() -> status: own ExitStatus pure {
  return exit_status(code: 0_u8);
}
"#;
    let host = r#"#include <stdint.h>
#include <stdlib.h>

struct Owners { int32_t current; int32_t previous; };
struct HeldOwners { int32_t current; int32_t previous; uint64_t before; uint64_t after; };
extern void wf_rotate(struct Owners *, int32_t, int32_t);
extern void wf_rotate_field(struct HeldOwners *, int32_t, int32_t);

void wf_test_unexpected_close(int32_t descriptor, void *record) {
    (void)descriptor;
    (void)record;
    abort();
}
void wf_test_unexpected_join(void *record, int64_t *value, int32_t *error) {
    (void)record;
    (void)value;
    (void)error;
    abort();
}

int main(void) {
    struct Owners owners = {0, 0};
    wf_rotate(&owners, 17, 29);
    if (owners.current != 29 || owners.previous != 17) return 1;
    struct HeldOwners result = {0};
    wf_rotate_field(&result, 37, 41);
    if (result.current != 41 || result.previous != 37) return 2;
    if (result.before != 101 || result.after != 103) return 3;
    return 0;
}
"#;
    for overlap in [
        OverlapLowering::Off,
        OverlapLowering::On,
        OverlapLowering::Completion,
    ] {
        let module = emit_lowered(source, overlap);
        assert!(module.contains("define internal void @wf_rotate("));
        assert!(module.contains("define internal void @wf_rotate_field("));
        let observed = super::owned_places::retain_calls(&module)
            .replace(
                "define internal void @wf_rotate(",
                "define void @wf_rotate(",
            )
            .replace(
                "define internal void @wf_rotate_field(",
                "define void @wf_rotate_field(",
            )
            .replace("define i32 @main(", "define i32 @wf_test_original_main(")
            .replace(
                "@wf__completion_file_close_submit(",
                "@wf_test_unexpected_close(",
            )
            .replace("@wf__completion_file_join(", "@wf_test_unexpected_join(");
        let output = compile_link_and_run(&observed, Some(host), &[]);
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

/// HostString is an opaque two-word lease. System operations must observe the
/// current lease value through both shared and exclusive user-call borrows.
#[test]
fn opaque_aggregate_borrows_preserve_replacement_and_system_observations() {
    let source = br#"struct Holder {
  value: HostString;
  marker: u64;
}

fn shared['r](target: &'r HostString) -> result: &'r HostString reads(target) {
  region {
    let first = initial(value: &deref(target));
  }
  let again = initial(value: target);
  return target;
}

fn exclusive['r](target: &uniq 'r HostString) -> result: &uniq 'r HostString pure {
  return &uniq 'r deref(target);
}

fn exchange(target: &uniq HostString, incoming: own HostString) -> previous: own HostString reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn relay(target: &uniq HostString, incoming: own HostString) -> previous: own HostString reads(target), writes(target) {
  region {
    return exchange(target: &uniq deref(target), incoming: move incoming);
  }
}

fn initial(value: &HostString) -> result: own u8 reads(value) {
  let bytes = fixed_vector::<u8, 12>();
  for (
    index in 0_u64..12_u64,
    invariant filled: len_of(bytes) >= index,
    invariant spare: room_of(bytes) + index >= 12_u64,
    invariant flat: head_of(bytes) <= 0_u64
  ) {
    place_back(vector: &uniq bytes, value: 0_u8);
  }
  region {
    let view = mut_slice_of(&uniq bytes);
    region {
      match host_copy_bytes(value: value, destination: &uniq view, start: 0_u64, end: 12_u64) {
        Ok(value: copied) => {
        }
        Err(error: too_long) => {
          return 0_u8;
        }
      }
    }
  }
  return bytes[0_u64];
}

fn main(command.args as args: own Args) -> status: own ExitStatus reads(args) {
  region {
    match arg_get(args: &args, position: 1_u64) {
      Err(error: absent) => {
        return exit_status(code: 10_u8);
      }
      Ok(value: first) => {
        region {
          match arg_get(args: &args, position: 2_u64) {
            Err(error: absent) => {
              return exit_status(code: 11_u8);
            }
            Ok(value: incoming) => {
              let holder = Holder(value: move first, marker: 101_u64);
              region {
                let borrowed = exclusive(target: &uniq holder.value);
                let previous = relay(target: move borrowed, incoming: move incoming);
                region {
                  let before = host_bytes_len(value: &previous);
                  if before != 5_u64 {
                    return exit_status(code: 1_u8);
                  }
                  let byte = initial(value: &previous);
                  if byte != 102_u8 {
                    return exit_status(code: 4_u8);
                  }
                }
              }
              region {
                let borrowed = shared(target: &holder.value);
                let after = host_bytes_len(value: borrowed);
                if after != 12_u64 {
                  return exit_status(code: 2_u8);
                }
                let byte = initial(value: borrowed);
                if byte != 115_u8 {
                  return exit_status(code: 5_u8);
                }
              }
              if holder.marker != 101_u64 {
                return exit_status(code: 3_u8);
              }
              return exit_status(code: 0_u8);
            }
          }
        }
      }
    }
  }
}
"#;
    for overlap in [
        OverlapLowering::Off,
        OverlapLowering::On,
        OverlapLowering::Completion,
    ] {
        let module = emit_lowered(source, overlap);
        let output = compile_link_and_run(
            &super::owned_places::retain_calls(&module),
            None,
            &[b"first", b"second value"],
        );
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

/// A returned unique field borrow still addresses its caller's owner slot.
/// Observe both owners and adjacent fields across retained call boundaries.
#[test]
fn ordinary_box_field_reborrows_preserve_both_owners_and_adjacent_fields() {
    let source = br#"struct Holder {
  before: u64;
  value: box<u64>;
  after: u64;
}

fn alias['r](value: &uniq 'r box<u64>) -> result: &uniq 'r box<u64> pure {
  return &uniq 'r deref(value);
}

fn exchange(target: &uniq box<u64>, incoming: own box<u64>) -> previous: own box<u64> reads(target), writes(target) {
  let previous = replace deref(target) = move incoming;
  return move previous;
}

fn replace_field(holder: &uniq Holder, incoming: own box<u64>) -> previous: own box<u64> reads(holder.value), writes(holder.value) {
  region {
    let selected = alias(value: &uniq deref(holder).value);
    region {
      return exchange(target: &uniq deref(selected), incoming: move incoming);
    }
  }
}

fn main() -> status: own ExitStatus pure {
  let first = box_new(17_u64);
  let holder = Holder(before: 101_u64, value: move first, after: 303_u64);
  let incoming = box_new(29_u64);
  region {
    let previous = replace_field(holder: &uniq holder, incoming: move incoming);
    if deref(previous) != 17_u64 {
      return exit_status(code: 1_u8);
    }
  }
  if deref(holder.value) != 29_u64 {
    return exit_status(code: 2_u8);
  }
  if holder.before != 101_u64 {
    return exit_status(code: 3_u8);
  }
  if holder.after != 303_u64 {
    return exit_status(code: 4_u8);
  }
  return exit_status(code: 0_u8);
}
"#;
    for overlap in [
        OverlapLowering::Off,
        OverlapLowering::On,
        OverlapLowering::Completion,
    ] {
        let module = emit_lowered(source, overlap);
        let output = compile_and_run(&super::owned_places::retain_calls(&module));
        assert_eq!(output.status.code(), Some(0), "{output:?}");
        assert!(output.stdout.is_empty(), "{output:?}");
        assert!(output.stderr.is_empty(), "{output:?}");
    }
}

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
        br#"fn source['r](x: &'r i32) -> result: &'r i32 pure {
  return x;
}

fn main() -> status: own ExitStatus pure {
  let v = 5_i32;
  region {
    let h = &v;
    source(x: h);
  }
  if v != 5_i32 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
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
/// This case exercises whole-referent reborrows; the ordinary Box field case
/// above also verifies a suffixed child through its caller's owner slot.
#[test]
fn extension_chains_execute_and_write_the_owners_storage() {
    let llvm = emit_reborrow_extension(
        br#"fn passthru['r0](x: &uniq 'r0 i32) -> result: &uniq 'r0 i32 pure {
  return &uniq 'r0 deref(x);
}

fn bump(n: &uniq i32) -> result: own unit writes(n) {
  set deref(n) = 42_i32;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let v = 1_i32;
  region {
    let h = &uniq v;
    let r = passthru(x: &uniq deref(h));
    region {
      bump(n: &uniq deref(r));
    }
  }
  if v != 42_i32 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
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
        br#"fn bump(n: &uniq i32) -> result: own unit writes(n) {
  set deref(n) = 42_i32;
  return unit;
}

fn main() -> status: own ExitStatus pure {
  let a = 0_i32;
  region {
    bump(n: &uniq a);
  }
  if a != 42_i32 {
    return exit_status(code: 1_u8);
  }
  return exit_status(code: 0_u8);
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

fn inspect(packet: &Packet) -> result: own i32 reads(packet) {
  match deref(packet) {
    Data(item: payload) => {
      return deref(payload).left;
    }
    Empty() => {
      return 0_i32;
    }
  }
}

fn main() -> status: own ExitStatus pure {
  let pair = Pair(left: 41_i32, right: 1_i32);
  let packet = Data(item: move pair);
  let fallback = Empty();
  region {
    let held = &packet;
    let read = inspect(packet: held);
    if read != 41_i32 {
      return exit_status(code: 1_u8);
    }
    let hollow = &fallback;
    let zero = inspect(packet: hollow);
    if zero != 0_i32 {
      return exit_status(code: 2_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

/// A borrowed enum binder keeps the representation selected for its declared
/// source mode. The enum itself is reached through an addressed struct field,
/// while the legacy buffer payload remains its value descriptor rather than
/// being passed as an address that its function ABI does not declare.
#[test]
fn a_projected_borrow_match_keeps_a_buffer_payloads_value_abi() {
    let llvm = compile(
        br#"enum Packet {
  Data(bytes: buffer<u8>);
  Empty();
}

struct Envelope {
  packet: Packet;
}

fn inspect(envelope: &Envelope) -> result: own u64 reads(envelope.packet) {
  match deref(envelope).packet {
    Data(bytes: payload) => {
      return len_of(deref(payload));
    }
    Empty() => {
      return 0_u64;
    }
  }
}

fn main() -> status: own ExitStatus pure {
  let bytes = buffer_new(3_u64, 7_u8);
  let packet = Data(bytes: move bytes);
  let envelope = Envelope(packet: move packet);
  region {
    let observed = inspect(envelope: &envelope);
    if observed != 3_u64 {
      return exit_status(code: 1_u8);
    }
  }
  return exit_status(code: 0_u8);
}
"#,
    );
    let output = compile_and_run(&llvm);
    assert!(output.status.success(), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    assert!(output.stderr.is_empty(), "{output:?}");
}
