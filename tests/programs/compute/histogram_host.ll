; Forward the input view and runtime dimensions, and unpack the owned result.
define void @wf_bench_histogram(ptr %data, i64 %count, i64 %block_size, i64 %buckets, ptr %out, ptr %out_len) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %r = call { ptr, i64 } @wf_histogram({ ptr, i64 } %b, i64 %block_size, i64 %buckets)
  %p = extractvalue { ptr, i64 } %r, 0
  %n = extractvalue { ptr, i64 } %r, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  ret void
}
define void @wf_bench_histogram_release(ptr %data, i64 %count) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %r = call i64 @wf_release_histogram({ ptr, i64 } %b)
  ret void
}
