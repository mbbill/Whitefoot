use super::*;

#[test]
fn array_and_buffer_slices_share_one_read_only_descriptor_path() {
    let source = br#"const bytes: array<u8, 4> = [1_u8, 2_u8, 3_u8, 4_u8];

fn sum ['r](values: own slice<'r, u8>) -> own u64 reads('r), traps {
  let offset: own u64 = 0_u64;
  let total: own u64 = 0_u64;
  let length: own u64 = len<u8>(values);
  loop @items {
    let done: own Bool = ieq<u64>(offset, length);
    match done {
      True() => {
        break @items;
      }
      False() => {
      }
    }
    let byte: own u8 = index<u8>(values, offset);
    let word: own u64 = cvt<u8, u64>(byte);
    set total = iadd.wrap<u64>(total, word);
    set offset = iadd.wrap<u64>(offset, 1_u64);
  }
  return total;
}

fn main() -> own unit allocates(heap), traps {
  region 'static_view {
    let view: own slice<'static_view, u8> = slice_of<'static_view, u8>(&'static_view bytes);
    let total: own u64 = sum<'static_view>(values: move view);
    check ieq<u64>(total, 10_u64) else trap "array slice";
  }
  let local: own array<u8, 4> = array_new<u8, 4>(3_u8);
  region 'local_view {
    let view: own slice<'local_view, u8> = slice_of<'local_view, u8>(&'local_view local);
    let total: own u64 = sum<'local_view>(values: move view);
    check ieq<u64>(total, 12_u64) else trap "local array slice";
  }
  let runtime: own buffer<u8> = buffer_new<u8>(4_u64, 2_u8);
  region 'runtime_view {
    let view: own slice<'runtime_view, u8> = slice_of<'runtime_view, u8>(&'runtime_view runtime);
    let total: own u64 = sum<'runtime_view>(values: move view);
    check ieq<u64>(total, 8_u64) else trap "buffer slice";
  }
  return unit;
}
"#;
    let llvm = compile(source);
    let sum = emitted_function(&llvm, "sum");
    let main = emitted_function(&llvm, "main");
    assert!(sum.contains("slice.index.cont"));
    assert!(sum.contains("getelementptr inbounds i8"));
    assert!(!sum.contains("call void @free"));
    assert_eq!(main.matches("call void @free").count(), 1);

    let output = compile_and_run(&llvm);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn slice_index_retains_the_op4_trap_before_address_formation() {
    let source = br#"fn main() -> own unit traps {
  let bytes: own array<u8, 2> = array_new<u8, 2>(0_u8);
  region 'view {
    let window: own slice<'view, u8> = slice_of<'view, u8>(&'view bytes);
    let value: own u8 = index<u8>(window, 2_u64);
  }
  return unit;
}
"#;
    let llvm = compile(source);
    let main = emitted_function(&llvm, "main");
    let bounds = main
        .find("icmp ult i64")
        .expect("slice index must compare against its retained runtime length");
    let trap = main[bounds..]
        .find("call void @wf_trap")
        .map(|offset| bounds + offset)
        .expect("slice index must retain an OP-4 trap edge");
    let address = main[trap..]
        .find("getelementptr inbounds i8")
        .map(|offset| trap + offset)
        .expect("slice element address must follow the successful guard");
    assert!(bounds < trap && trap < address);

    let output = compile_and_run(&llvm);
    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("trap record is UTF-8");
    assert!(stderr.starts_with(
        "{\"rule_id\":\"OP-4\",\"message\":\"\",\"function\":\"main\",\"node_path\":["
    ));
    assert_eq!(stderr.lines().count(), 1);
}
