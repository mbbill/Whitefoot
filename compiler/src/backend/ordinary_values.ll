; Ordinary linked definitions with the exact Whitefoot FunctionAbi.
; A range reference crosses the call as its element pointer and count, as
; every WF call passes it. Their private C bodies take a pointer to one view,
; since C aggregate parameter coercions differ between target ABIs.
; This file is library implementation, with no compiler operation dispatch.

declare void @wf__body_host_copy_bytes(ptr, ptr, ptr, i64, i64)

define void @wf_std.text.host_copy_bytes(ptr %result, ptr %value, ptr %destination.data, i64 %destination.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_host_copy_bytes(ptr %result, ptr %value, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_host_copy_utf8(ptr, ptr, ptr, i64, i64)

define void @wf_std.text.host_copy_utf8(ptr %result, ptr %value, ptr %destination.data, i64 %destination.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_host_copy_utf8(ptr %result, ptr %value, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_read_at(ptr, ptr, ptr, ptr, i64, i64, i64)

define void @wf_std.fs.read_at(ptr %result, ptr %factory, ptr %file, ptr %destination.data, i64 %destination.len, i64 %offset, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_read_at(ptr %result, ptr %factory, ptr %file, ptr %view, i64 %offset, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_write_once(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_std.io.write_once(ptr %result, ptr %factory, ptr %output, ptr %source.data, i64 %source.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %source.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %source.len, ptr %view.len, align 8
  call void @wf__body_write_once(ptr %result, ptr %factory, ptr %output, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_open_directory(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_std.fs.open_directory(ptr %result, ptr %factory, ptr %root, ptr %name.data, i64 %name.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %name.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %name.len, ptr %view.len, align 8
  call void @wf__body_open_directory(ptr %result, ptr %factory, ptr %root, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_directory_next(ptr, ptr, ptr, i64, i64)

define void @wf_std.fs.directory_next(ptr %result, ptr %source, ptr %destination.data, i64 %destination.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_directory_next(ptr %result, ptr %source, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_open_file(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_std.fs.open_file(ptr %result, ptr %factory, ptr %root, ptr %name.data, i64 %name.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %name.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %name.len, ptr %view.len, align 8
  call void @wf__body_open_file(ptr %result, ptr %factory, ptr %root, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_read_next(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_std.io.read_next(ptr %result, ptr %factory, ptr %input, ptr %destination.data, i64 %destination.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_read_next(ptr %result, ptr %factory, ptr %input, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_receive_next(ptr, ptr, ptr, i64, i64)

define void @wf_std.net.receive_next(ptr %result, ptr %receive, ptr %destination.data, i64 %destination.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_receive_next(ptr %result, ptr %receive, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_send_once(ptr, ptr, ptr, i64, i64)

define void @wf_std.net.send_once(ptr %result, ptr %send, ptr %source.data, i64 %source.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %source.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %source.len, ptr %view.len, align 8
  call void @wf__body_send_once(ptr %result, ptr %send, ptr %view, i64 %start, i64 %end)
  ret void
}
