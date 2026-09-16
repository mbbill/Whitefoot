; Descriptor-only entry for the compute-bench sort and independent oracle.
define void @wf_bench_merge_sort(ptr %input, i64 %count, ptr %out, ptr %out_len) {
  %a = insertvalue { ptr, i64 } poison, ptr %input, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %r = call { ptr, i64 } @wf_merge_sort({ ptr, i64 } %b)
  %p = extractvalue { ptr, i64 } %r, 0
  %n = extractvalue { ptr, i64 } %r, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  ret void
}
define void @wf_bench_merge_sort_release(ptr %data, i64 %length) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %length, 1
  %r = call i64 @wf_release_merge_sort({ ptr, i64 } %b)
  ret void
}
