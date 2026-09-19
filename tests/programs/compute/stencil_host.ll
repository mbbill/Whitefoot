; Serves compute-bench and the compiler's independent native stencil oracle.
; Only forward runtime dimensions and unpack the owned result cell.
; The owned runtime-capacity result is a `Box<Array<T>>` cell pointer, not the
; `{ ptr, i64 }` descriptor v0.59 returned: a runtime-capacity shape exists only
; as the content of a cell [TYPE-9], the cell is what the call hands back, and
; one compiler-derived free at the owner's scope exit releases it [STOR-1,
; STOR-3]. The adapter therefore loads the cell's `{ data, count }` content for
; the oracle to read and hands the cell itself back as the retained handle; the
; release entry consumes exactly that handle, which is the only value the
; release row accepts.
define void @wf_bench_stencil(i64 %width, i64 %height, i64 %steps, ptr %out, ptr %out_len, ptr %out_cell) {
  %r = call ptr @wf_stencil(i64 %width, i64 %height, i64 %steps)
  %content = load { ptr, i64 }, ptr %r
  %p = extractvalue { ptr, i64 } %content, 0
  %n = extractvalue { ptr, i64 } %content, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  store ptr %r, ptr %out_cell
  ret void
}
define void @wf_bench_stencil_release(ptr %cell) {
  %x = call i64 @wf_release_stencil(ptr %cell)
  ret void
}
