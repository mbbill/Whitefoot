use super::*;

#[test]
fn source_enum_cleanup_switches_on_the_active_variant() {
    let source = br#"struct PairBuffers {
  left: buffer<u8>;
  right: buffer<u8>;
}

enum Owner {
  Empty();
  Full(value: PairBuffers);
}

fn abandon(owner: own Owner) -> own unit pure {
  return unit;
}

fn consume(owner: own Owner) -> own unit traps {
  match move owner {
    Empty() => {
    }
    Full(value: pair) => {
      let byte: own u8 = pair.left[0_u64];
      check ieq<u8>(byte, 11_u8) else trap "transferred payload drift";
    }
  }
  return unit;
}

fn main() -> own unit allocates(heap), traps {
  let abandoned_left: own buffer<u8> = buffer_new<u8>(1_u64, 7_u8);
  let abandoned_right: own buffer<u8> = buffer_new<u8>(1_u64, 9_u8);
  let abandoned_pair: own PairBuffers = PairBuffers(left: move abandoned_left, right: move abandoned_right);
  let abandoned: own Owner = Full(value: move abandoned_pair);
  abandon(owner: move abandoned);
  let empty: own Owner = Empty();
  abandon(owner: move empty);
  let consumed_left: own buffer<u8> = buffer_new<u8>(1_u64, 11_u8);
  let consumed_right: own buffer<u8> = buffer_new<u8>(1_u64, 13_u8);
  let consumed_pair: own PairBuffers = PairBuffers(left: move consumed_left, right: move consumed_right);
  let consumed: own Owner = Full(value: move consumed_pair);
  consume(owner: move consumed);
  return unit;
}
"#;
    let llvm = compile(source);
    let helper_start = llvm
        .find("define private void @wf.drop.t1")
        .expect("resource enum must have one drop helper");
    let helper_end = llvm[helper_start..]
        .find("\n}\n\n")
        .map(|offset| helper_start + offset + 3)
        .expect("drop helper must close");
    let helper = &llvm[helper_start..helper_end];
    assert!(helper.contains("switch i32 %tag"));
    assert!(helper.contains("i32 0, label %variant.0"));
    assert!(helper.contains("i32 1, label %variant.1"));
    assert_eq!(helper.matches("call void @free").count(), 2);
    assert_eq!(
        emitted_function(&llvm, "abandon")
            .matches("call void @wf.drop.t1")
            .count(),
        1
    );
    let consume = emitted_function(&llvm, "consume");
    assert!(!consume.contains("call void @wf.drop.t1"));
    assert_eq!(consume.matches("call void @free").count(), 2);

    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "resource enum program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn result_buffer_transfer_error_and_abandonment_execute() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-result-buffer-transform-run.wf"
    ));
    let helper_start = llvm
        .find("define private void @wf.drop.")
        .expect("Result<buffer<u8>, DecodeError> must have a drop helper");
    let helper_end = llvm[helper_start..]
        .find("\n}\n\n")
        .map(|offset| helper_start + offset + 3)
        .expect("drop helper must close");
    let helper = &llvm[helper_start..helper_end];
    assert_eq!(llvm.matches("define private void @wf.drop.").count(), 1);
    assert!(helper.contains("switch i32 %tag"));
    assert_eq!(helper.matches("call void @free").count(), 1);

    let abandon = emitted_function(&llvm, "abandon");
    assert_eq!(abandon.matches("call void @wf.drop.").count(), 1);
    let transform = emitted_function(&llvm, "transform");
    assert_eq!(transform.matches("call void @free").count(), 3);

    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "Result buffer program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn option_buffer_some_none_and_transfer_execute() {
    let source = br#"fn abandon(value: own Option<buffer<u8>>) -> own unit pure {
  return unit;
}

fn consume(value: own Option<buffer<u8>>) -> own unit traps {
  match move value {
    None() => {
      check False() else trap "Some became None";
    }
    Some(value: bytes) => {
      let byte: own u8 = bytes[0_u64];
      check ieq<u8>(byte, 17_u8) else trap "Some payload drift";
    }
  }
  return unit;
}

fn main() -> own unit allocates(heap), traps {
  let abandoned_bytes: own buffer<u8> = buffer_new<u8>(1_u64, 5_u8);
  let abandoned_some: own Option<buffer<u8>> = Some(value: move abandoned_bytes);
  abandon(value: move abandoned_some);
  let abandoned_none: own Option<buffer<u8>> = None();
  abandon(value: move abandoned_none);
  let consumed_bytes: own buffer<u8> = buffer_new<u8>(1_u64, 17_u8);
  let consumed_some: own Option<buffer<u8>> = Some(value: move consumed_bytes);
  consume(value: move consumed_some);
  return unit;
}
"#;
    let llvm = compile(source);
    let helper_start = llvm
        .find("define private void @wf.drop.")
        .expect("Option<buffer<u8>> must have a drop helper");
    let helper_end = llvm[helper_start..]
        .find("\n}\n\n")
        .map(|offset| helper_start + offset + 3)
        .expect("drop helper must close");
    let helper = &llvm[helper_start..helper_end];
    assert_eq!(helper.matches("call void @free").count(), 1);
    assert_eq!(
        emitted_function(&llvm, "abandon")
            .matches("call void @wf.drop.")
            .count(),
        1
    );
    let consume = emitted_function(&llvm, "consume");
    assert!(!consume.contains("call void @wf.drop."));
    assert_eq!(consume.matches("call void @free").count(), 1);

    let output = compile_and_run(&llvm);
    assert!(
        output.status.success(),
        "Option buffer program failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
