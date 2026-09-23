; Ordinary scalar/pointer adapter for the research-only first-index probe.
define i64 @wf_probe_search_plain(ptr %input, i64 %input_len, ptr %offsets, i64 %offsets_len, i64 %count, i64 %mode) {
  %a = insertvalue { ptr, i64 } poison, ptr %input, 0
  %b = insertvalue { ptr, i64 } %a, i64 %input_len, 1
  %c = insertvalue { ptr, i64 } poison, ptr %offsets, 0
  %d = insertvalue { ptr, i64 } %c, i64 %offsets_len, 1
  %r = call i64 @wf_search_plain({ ptr, i64 } %b, { ptr, i64 } %d, i64 %count, i64 %mode)
  ret i64 %r
}

define i64 @wf_probe_search_trace(ptr %input, i64 %input_len, ptr %offsets, i64 %offsets_len, i64 %count, i64 %mode, ptr %trace, i64 %trace_len) {
  %a = insertvalue { ptr, i64 } poison, ptr %input, 0
  %b = insertvalue { ptr, i64 } %a, i64 %input_len, 1
  %c = insertvalue { ptr, i64 } poison, ptr %offsets, 0
  %d = insertvalue { ptr, i64 } %c, i64 %offsets_len, 1
  %e = insertvalue { ptr, i64 } poison, ptr %trace, 0
  %f = insertvalue { ptr, i64 } %e, i64 %trace_len, 1
  %r = call i64 @wf_search_trace({ ptr, i64 } %b, { ptr, i64 } %d, i64 %count, i64 %mode, { ptr, i64 } %f)
  ret i64 %r
}
