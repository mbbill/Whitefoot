; Serves compute-bench and the compiler's independent native stencil oracle.
; Only forward runtime dimensions and unpack the ordinary buffer descriptor.
define void @wf_bench_stencil(i64 %width, i64 %height, i64 %steps, ptr %out, ptr %out_len) {
  %r = call { ptr, i64 } @wf_stencil(i64 %width, i64 %height, i64 %steps)
  %p = extractvalue { ptr, i64 } %r, 0
  %n = extractvalue { ptr, i64 } %r, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  ret void
}
define void @wf_bench_stencil_release(ptr %data, i64 %length) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %length, 1
  %r = call i64 @wf_release_stencil({ ptr, i64 } %b)
  ret void
}
