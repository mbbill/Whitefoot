; Serves compute-bench: the Mandelbrot host adapter. It is LLVM IR rather than
; C because every user function the compiler emits has internal linkage, so
; nothing outside the module can call @wf_render_points or @wf_release_points.
; It builds the two buffer descriptors and forwards; it computes nothing.
; The Makefile appends this file to each emitted module and sed-renames the
; two entry points to the -par or -seq spelling, so one text serves both.
; Eighteen lines of IR, the ceiling section 2 of the specification records.
define void @wf_bench_mandelbrot(ptr %real, ptr %imaginary, i64 %count, i64 %limit, ptr %out, ptr %out_len) {
  %a = insertvalue { ptr, i64 } poison, ptr %real, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %c = insertvalue { ptr, i64 } poison, ptr %imaginary, 0
  %d = insertvalue { ptr, i64 } %c, i64 %count, 1
  %r = call { ptr, i64 } @wf_render_points({ ptr, i64 } %b, { ptr, i64 } %d, i64 0, i64 %count, i64 %limit)
  %p = extractvalue { ptr, i64 } %r, 0
  %n = extractvalue { ptr, i64 } %r, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  ret void
}
define void @wf_bench_mandelbrot_release(ptr %data, i64 %length) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %length, 1
  %x = call i64 @wf_release_points({ ptr, i64 } %b)
  ret void
}
