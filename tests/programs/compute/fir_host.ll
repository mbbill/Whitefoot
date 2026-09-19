; Scalar/pointer host ABI for the formal fir fixture.
; The shared adapter binder selects the entry-equivalent execution world.
; The owned runtime-capacity result is a `Box<Array<T>>` cell pointer, not the
; `{ ptr, i64 }` descriptor v0.59 returned: a runtime-capacity shape exists only
; as the content of a cell [TYPE-9], the cell is what the call hands back, and
; one compiler-derived free at the owner's scope exit releases it [STOR-1,
; STOR-3]. The adapter therefore loads the cell's `{ data, count }` content for
; the oracle to read and hands the cell itself back as the retained handle; the
; release entry consumes exactly that handle, which is the only value the
; release row accepts.
define void @wf_bench_fir(ptr %input, i64 %input_len, ptr %taps, i64 %tap_len, i64 %first, i64 %end, i64 %last_tap, ptr %out, ptr %out_len, ptr %out_cell) {
  %a = insertvalue { ptr, i64 } poison, ptr %input, 0
  %b = insertvalue { ptr, i64 } %a, i64 %input_len, 1
  %c = insertvalue { ptr, i64 } poison, ptr %taps, 0
  %d = insertvalue { ptr, i64 } %c, i64 %tap_len, 1
  %r = call ptr @wf_filter({ ptr, i64 } %b, { ptr, i64 } %d, i64 %first, i64 %end, i64 %last_tap)
  %content = load { ptr, i64 }, ptr %r
  %p = extractvalue { ptr, i64 } %content, 0
  %n = extractvalue { ptr, i64 } %content, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  store ptr %r, ptr %out_cell
  ret void
}
define void @wf_bench_fir_release(ptr %cell) {
  %x = call i64 @wf_release_samples(ptr %cell)
  ret void
}
