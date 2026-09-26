; Ordinary linked definitions with the exact Whitefoot FunctionAbi.
; A range reference crosses the call as its element pointer and count, as
; every WF call passes it. Their private C bodies take a pointer to one view,
; since C aggregate parameter coercions differ between target ABIs.
; A result whose scalar leaves fit the return registers crosses as its LLVM
; first-class value, one register per leaf; its private C body writes the
; ordinary representation through a pointer, since C packs small struct
; returns differently on each target.
; This file is library implementation, with no compiler operation dispatch.

declare void @wf__body_host_utf8_len(ptr, ptr)

; `Result<u64, Utf8Error>` is `{ i32, i64, i1 }`: the tag, the `Ok` value and
; the one-bit `Utf8Invalid` tag, which the C body stores as a byte.
define { i32, i64, i1 } @wf_host_utf8_len(ptr %value) {
entry:
  %result = alloca { i32, i64, i8 }, align 8
  call void @wf__body_host_utf8_len(ptr %result, ptr %value)
  %tag = load i32, ptr %result, align 8
  %ok.field = getelementptr inbounds { i32, i64, i8 }, ptr %result, i32 0, i32 1
  %ok = load i64, ptr %ok.field, align 8
  %error.field = getelementptr inbounds { i32, i64, i8 }, ptr %result, i32 0, i32 2
  %error.byte = load i8, ptr %error.field, align 8
  %error = trunc i8 %error.byte to i1
  %with.tag = insertvalue { i32, i64, i1 } poison, i32 %tag, 0
  %with.ok = insertvalue { i32, i64, i1 } %with.tag, i64 %ok, 1
  %returned = insertvalue { i32, i64, i1 } %with.ok, i1 %error, 2
  ret { i32, i64, i1 } %returned
}

declare void @wf__body_host_copy_bytes(ptr, ptr, ptr, i64, i64)

define void @wf_host_copy_bytes(ptr %result, ptr %value, ptr %destination.data, i64 %destination.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_host_copy_bytes(ptr %result, ptr %value, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_host_copy_utf8(ptr, ptr, ptr, i64, i64)

define void @wf_host_copy_utf8(ptr %result, ptr %value, ptr %destination.data, i64 %destination.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_host_copy_utf8(ptr %result, ptr %value, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_read_at(ptr, ptr, ptr, ptr, i64, i64, i64)

define void @wf_read_at(ptr %result, ptr %factory, ptr %file, ptr %destination.data, i64 %destination.len, i64 %offset, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_read_at(ptr %result, ptr %factory, ptr %file, ptr %view, i64 %offset, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_write_once(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_write_once(ptr %result, ptr %factory, ptr %output, ptr %source.data, i64 %source.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %source.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %source.len, ptr %view.len, align 8
  call void @wf__body_write_once(ptr %result, ptr %factory, ptr %output, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_open_directory(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_open_directory(ptr %result, ptr %factory, ptr %root, ptr %name.data, i64 %name.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %name.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %name.len, ptr %view.len, align 8
  call void @wf__body_open_directory(ptr %result, ptr %factory, ptr %root, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_directory_next(ptr, ptr, ptr, i64, i64)

define void @wf_directory_next(ptr %result, ptr %source, ptr %destination.data, i64 %destination.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_directory_next(ptr %result, ptr %source, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_open_file(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_open_file(ptr %result, ptr %factory, ptr %root, ptr %name.data, i64 %name.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %name.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %name.len, ptr %view.len, align 8
  call void @wf__body_open_file(ptr %result, ptr %factory, ptr %root, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_read_next(ptr, ptr, ptr, ptr, i64, i64)

define void @wf_read_next(ptr %result, ptr %factory, ptr %input, ptr %destination.data, i64 %destination.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_read_next(ptr %result, ptr %factory, ptr %input, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_receive_next(ptr, ptr, ptr, i64, i64)

define void @wf_receive_next(ptr %result, ptr %receive, ptr %destination.data, i64 %destination.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %destination.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %destination.len, ptr %view.len, align 8
  call void @wf__body_receive_next(ptr %result, ptr %receive, ptr %view, i64 %start, i64 %end)
  ret void
}

declare void @wf__body_send_once(ptr, ptr, ptr, i64, i64)

define void @wf_send_once(ptr %result, ptr %send, ptr %source.data, i64 %source.len, i64 %start, i64 %end) {
entry:
  %view = alloca { ptr, i64 }, align 8
  store ptr %source.data, ptr %view, align 8
  %view.len = getelementptr inbounds { ptr, i64 }, ptr %view, i32 0, i32 1
  store i64 %source.len, ptr %view.len, align 8
  call void @wf__body_send_once(ptr %result, ptr %send, ptr %view, i64 %start, i64 %end)
  ret void
}
