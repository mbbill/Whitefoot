; Formal native probe boundary: a C descriptor pointer becomes the exact WF
; aggregate view at this LLVM call, independently of C's target ABI coercions.
declare void @wf_open_file(ptr, ptr, ptr, { ptr, i64 }, i64, i64)
declare void @wf_read_at(ptr, ptr, ptr, { ptr, i64 }, i64, i64, i64)

define void @wf_test_public_open(ptr %result, ptr %factory, ptr %root, ptr %name, i64 %start, i64 %end) {
  %view = load { ptr, i64 }, ptr %name
  call void @wf_open_file(ptr %result, ptr %factory, ptr %root, { ptr, i64 } %view, i64 %start, i64 %end)
  ret void
}

define void @wf_test_public_read(ptr %result, ptr %factory, ptr %file, ptr %destination, i64 %offset, i64 %start, i64 %end) {
  %view = load { ptr, i64 }, ptr %destination
  call void @wf_read_at(ptr %result, ptr %factory, ptr %file, { ptr, i64 } %view, i64 %offset, i64 %start, i64 %end)
  ret void
}
