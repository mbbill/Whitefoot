use super::*;

#[test]
fn base64_has_typed_results_and_no_host_allocation() {
    let llvm = compile(include_bytes!(
        "../../../../tests/conformance/cases/x-base64-rfc-vectors-run.wf"
    ));
    let encode = emitted_function(&llvm, "encode");
    let main = emitted_function(&llvm, "main");
    // The three inputs are const runs and the three outputs come from
    // one bump extent, so nothing in this program reaches the host allocator
    // and no edge carries a free.
    assert!(encode.starts_with("define void @wf_encode(ptr %wf.result, { ptr, i64 } "));
    // Check the type actually stored to the return destination, independent
    // of nominal numbering: tag, u64 success, fieldless one-bit IndexError.
    let result_stores = encode
        .lines()
        .filter(|line| line.ends_with(", ptr %wf.result"))
        .filter_map(|line| line.trim_start().strip_prefix("store "))
        .collect::<Vec<_>>();
    assert!(!result_stores.is_empty());
    for store in result_stores {
        let result_type = store.split_whitespace().next().expect("store operand type");
        assert!(llvm.contains(&format!("{result_type} = type {{ i32, i64, i1 }}")));
    }
    assert_eq!(encode.matches("call ptr @malloc").count(), 0);
    assert_eq!(encode.matches("call void @free").count(), 0);
    assert_eq!(main.matches("call void @wf_encode(ptr ").count(), 3);
    assert_eq!(main.matches("call ptr @malloc").count(), 0);
    assert_eq!(main.matches("call void @free").count(), 0);
}
