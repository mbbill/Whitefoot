; Descriptor-only entry for stable binary distribution and its independent oracle.
define void @wf_bench_radix_scatter(ptr %input, i64 %count, i32 %bit, ptr %out, ptr %out_len) {
  %a = insertvalue { ptr, i64 } poison, ptr %input, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %r = call { ptr, i64 } @wf_radix_scatter({ ptr, i64 } %b, i32 %bit)
  %p = extractvalue { ptr, i64 } %r, 0
  %n = extractvalue { ptr, i64 } %r, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  ret void
}
define void @wf_bench_radix_scatter_release(ptr %data, i64 %length) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %length, 1
  %r = call i64 @wf_release_radix_scatter({ ptr, i64 } %b)
  ret void
}
