define i64 @wf_bench_batch(ptr %owner, ptr %output, i64 %count) {
  %a = insertvalue { ptr, i64 } poison, ptr %output, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %r = call i64 @wf_batch(ptr %owner, { ptr, i64 } %b)
  ret i64 %r
}
