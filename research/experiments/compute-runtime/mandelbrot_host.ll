define i64 @wf_research_escape(double %x, double %y, i64 %limit) {
  %result = call i64 @wf_escape_count(double %x, double %y, i64 %limit)
  ret i64 %result
}

define void @wf_research_mandelbrot(ptr %real, ptr %imaginary, i64 %count, i64 %limit, i1 %parallel, ptr %data, ptr %length) {
  %a = insertvalue { ptr, i64 } poison, ptr %real, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %c = insertvalue { ptr, i64 } poison, ptr %imaginary, 0
  %d = insertvalue { ptr, i64 } %c, i64 %count, 1
  br i1 %parallel, label %par, label %seq
par:
  %p = call { ptr, i64 } @wf_render_points({ ptr, i64 } %b, { ptr, i64 } %d, i64 0, i64 %count, i64 %limit)
  br label %joined
seq:
  %s = call { ptr, i64 } @wf__par_seq_render_points({ ptr, i64 } %b, { ptr, i64 } %d, i64 0, i64 %count, i64 %limit)
  br label %joined
joined:
  %result = phi { ptr, i64 } [ %p, %par ], [ %s, %seq ]
  %pointer = extractvalue { ptr, i64 } %result, 0
  %size = extractvalue { ptr, i64 } %result, 1
  store ptr %pointer, ptr %data
  store i64 %size, ptr %length
  ret void
}

define void @wf_research_mandelbrot_release(ptr %data, i64 %length) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %length, 1
  %result = call i64 @wf_release_points({ ptr, i64 } %b)
  ret void
}
