; Scalar/pointer host ABI for the formal fir fixture.
; The shared adapter binder selects the entry-equivalent execution world.
define void @wf_bench_fir(ptr %input, i64 %input_len, ptr %taps, i64 %tap_len, i64 %first, i64 %end, i64 %last_tap, ptr %out, ptr %out_len) {
  %a = insertvalue { ptr, i64 } poison, ptr %input, 0
  %b = insertvalue { ptr, i64 } %a, i64 %input_len, 1
  %c = insertvalue { ptr, i64 } poison, ptr %taps, 0
  %d = insertvalue { ptr, i64 } %c, i64 %tap_len, 1
  %r = call { ptr, i64 } @wf_filter({ ptr, i64 } %b, { ptr, i64 } %d, i64 %first, i64 %end, i64 %last_tap)
  %p = extractvalue { ptr, i64 } %r, 0
  %n = extractvalue { ptr, i64 } %r, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  ret void
}
define void @wf_bench_fir_release(ptr %data, i64 %length) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %length, 1
  %x = call i64 @wf_release_samples({ ptr, i64 } %b)
  ret void
}
