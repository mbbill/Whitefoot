; Forward the input range reference and runtime dimensions, and unpack the
; owned result cell.
; The owned runtime-capacity result is a `Box<Array<T>>` cell pointer, not the
; `{ ptr, i64 }` descriptor v0.59 returned: a runtime-capacity shape exists only
; as the content of a cell [TYPE-9], the cell is what the call hands back, and
; one compiler-derived free at the owner's scope exit releases it [STOR-1,
; STOR-3]. The adapter therefore loads the cell's `{ data, count }` content for
; the oracle to read and hands the cell itself back as the retained handle; the
; release entry consumes exactly that handle, which is the only value the
; release row accepts.
define void @wf_bench_histogram(ptr %data, i64 %count, i64 %block_size, i64 %buckets, ptr %out, ptr %out_len, ptr %out_cell) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %r = call ptr @wf_histogram({ ptr, i64 } %b, i64 %block_size, i64 %buckets)
  %content = load { ptr, i64 }, ptr %r
  %p = extractvalue { ptr, i64 } %content, 0
  %n = extractvalue { ptr, i64 } %content, 1
  store ptr %p, ptr %out
  store i64 %n, ptr %out_len
  store ptr %r, ptr %out_cell
  ret void
}
define void @wf_bench_histogram_release(ptr %cell) {
  %x = call i64 @wf_release_histogram(ptr %cell)
  ret void
}
