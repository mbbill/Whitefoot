; Ordinary linked definitions with the exact Whitefoot FunctionAbi.
; View descriptors stay SSA aggregates in WF. Their private C bodies take
; pointers, since C aggregate parameter coercions differ between target ABIs.
; This file is library implementation, with no compiler operation dispatch.

declare void @wf__body_host_copy_bytes(ptr, ptr, ptr, i64, i64)

define void @wf_host_copy_bytes(ptr %result, ptr %value, { ptr, i64 } %destination, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store { ptr, i64 } %destination, ptr %view, align 8
  call void @wf__body_host_copy_bytes(ptr %result, ptr %value, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_host_copy_utf8(ptr, ptr, ptr, i64, i64)

define void @wf_host_copy_utf8(ptr %result, ptr %value, { ptr, i64 } %destination, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store { ptr, i64 } %destination, ptr %view, align 8
  call void @wf__body_host_copy_utf8(ptr %result, ptr %value, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_read_at(ptr, ptr, ptr, ptr, i64, i64, i64)

define void @wf_read_at(ptr %result, ptr %factory, ptr %file, { ptr, i64 } %destination, i64 %offset, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store { ptr, i64 } %destination, ptr %view, align 8
  call void @wf__body_read_at(ptr %result, ptr %factory, ptr %file, ptr %view, i64 %offset, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_write_once(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_write_once(ptr %result, ptr %factory, ptr %output, { ptr, i64 } %source, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store { ptr, i64 } %source, ptr %view, align 8
  call void @wf__body_write_once(ptr %result, ptr %factory, ptr %output, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_open_directory(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_open_directory(ptr %result, ptr %factory, ptr %root, { ptr, i64 } %name, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store { ptr, i64 } %name, ptr %view, align 8
  call void @wf__body_open_directory(ptr %result, ptr %factory, ptr %root, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_directory_next(ptr, ptr, ptr, i64, i64)

define void @wf_directory_next(ptr %result, ptr %source, { ptr, i64 } %destination, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store { ptr, i64 } %destination, ptr %view, align 8
  call void @wf__body_directory_next(ptr %result, ptr %source, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_open_file(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_open_file(ptr %result, ptr %factory, ptr %root, { ptr, i64 } %name, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store { ptr, i64 } %name, ptr %view, align 8
  call void @wf__body_open_file(ptr %result, ptr %factory, ptr %root, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_read_next(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_read_next(ptr %result, ptr %factory, ptr %input, { ptr, i64 } %destination, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store { ptr, i64 } %destination, ptr %view, align 8
  call void @wf__body_read_next(ptr %result, ptr %factory, ptr %input, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_receive_next(ptr, ptr, ptr, i64, i64)

define void @wf_receive_next(ptr %result, ptr %receive, { ptr, i64 } %destination, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store { ptr, i64 } %destination, ptr %view, align 8
  call void @wf__body_receive_next(ptr %result, ptr %receive, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_send_once(ptr, ptr, ptr, i64, i64)

define void @wf_send_once(ptr %result, ptr %send, { ptr, i64 } %source, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store { ptr, i64 } %source, ptr %view, align 8
  call void @wf__body_send_once(ptr %result, ptr %send, ptr %view, i64 %start, i64 %end)
  ret void
}
