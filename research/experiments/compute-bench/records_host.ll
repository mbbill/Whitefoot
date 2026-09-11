; Serves compute-bench: the records host adapter. It is LLVM IR rather than C
; because every user function the compiler emits has internal linkage, so
; nothing outside the module can call @wf_summarize_records or
; @wf_record_result_release. It builds the two buffer descriptors -- the input
; bytes and the record offsets -- and forwards; it computes nothing.
; The Makefile appends this file to each emitted module and sed-renames the
; two entry points to the -par or -seq spelling, so one text serves both.
; Eighteen lines of IR, the ceiling section 2 of the specification records.
define void @wf_bench_records(ptr %data, i64 %data_len, ptr %offsets, i64 %offsets_len, i64 %first, i64 %end, ptr %out, ptr %out_len) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %data_len, 1
  %c = insertvalue { ptr, i64 } poison, ptr %offsets, 0
  %d = insertvalue { ptr, i64 } %c, i64 %offsets_len, 1
  %r = call { ptr, i64 } @wf_summarize_records({ ptr, i64 } %b, { ptr, i64 } %d, i64 %first, i64 %end)
  %p = extractvalue { ptr, i64 } %r, 0
  %n = extractvalue { ptr, i64 } %r, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  ret void
}
define void @wf_bench_records_release(ptr %data, i64 %length) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %length, 1
  %x = call i64 @wf_record_result_release({ ptr, i64 } %b)
  ret void
}
