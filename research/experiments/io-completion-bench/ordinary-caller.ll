; C-side pointer ABI for matched public and private-body routes. The public
; route loads the exact WF view aggregate; C must not guess its coercion.
; Retire with ordinary-caller.c when this attribution control is superseded.
declare void @wf_open_file(ptr, ptr, ptr, {ptr, i64}, i64, i64)
declare void @wf_read_at(ptr, ptr, ptr, {ptr, i64}, i64, i64, i64)
declare void @wf__body_open_file(ptr, ptr, ptr, ptr, i64, i64)
declare void @wf__body_read_at(ptr, ptr, ptr, ptr, i64, i64, i64)

define void @bench_public_open(ptr %r, ptr %f, ptr %root, ptr %name, i64 %start, i64 %end) {
  %v = load {ptr, i64}, ptr %name
  call void @wf_open_file(ptr %r, ptr %f, ptr %root, {ptr, i64} %v, i64 %start, i64 %end)
  ret void
}
define void @bench_public_read(ptr %r, ptr %f, ptr %file, ptr %window, i64 %offset, i64 %start, i64 %end) {
  %v = load {ptr, i64}, ptr %window
  call void @wf_read_at(ptr %r, ptr %f, ptr %file, {ptr, i64} %v, i64 %offset, i64 %start, i64 %end)
  ret void
}
define void @bench_body_open(ptr %r, ptr %f, ptr %root, ptr %name, i64 %start, i64 %end) {
  call void @wf__body_open_file(ptr %r, ptr %f, ptr %root, ptr %name, i64 %start, i64 %end)
  ret void
}
define void @bench_body_read(ptr %r, ptr %f, ptr %file, ptr %window, i64 %offset, i64 %start, i64 %end) {
  call void @wf__body_read_at(ptr %r, ptr %f, ptr %file, ptr %window, i64 %offset, i64 %start, i64 %end)
  ret void
}
