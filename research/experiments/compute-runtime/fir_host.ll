; Research-only adapter appended to the retained compiler-emitted FIR module.
; C passes scalars/pointers; descriptor aggregates are constructed explicitly.
; These calls qualify this emitted revision, not a public or stable WF ABI.
; The host normally selects the command entry's parallel/sequential world once
; under wf_floor. The formal wf-seq diagnostic explicitly selects sequential.
; Every tree remains alive until all lookups complete.

define ptr @wf_research_fir_parallel(ptr %input, i64 %input_count, ptr %taps, i64 %tap_count, i64 %first, i64 %end, i64 %last_tap, i64 %tile_size) {
  %in0 = insertvalue { ptr, i64 } zeroinitializer, ptr %input, 0
  %in1 = insertvalue { ptr, i64 } %in0, i64 %input_count, 1
  %tap0 = insertvalue { ptr, i64 } zeroinitializer, ptr %taps, 0
  %tap1 = insertvalue { ptr, i64 } %tap0, i64 %tap_count, 1
  %tree = call ptr @wf_filter_tiles({ ptr, i64 } %in1, { ptr, i64 } %tap1, i64 %first, i64 %end, i64 %last_tap, i64 %tile_size)
  ret ptr %tree
}

define ptr @wf_research_fir_sequential(ptr %input, i64 %input_count, ptr %taps, i64 %tap_count, i64 %first, i64 %end, i64 %last_tap, i64 %tile_size) {
  %in0 = insertvalue { ptr, i64 } zeroinitializer, ptr %input, 0
  %in1 = insertvalue { ptr, i64 } %in0, i64 %input_count, 1
  %tap0 = insertvalue { ptr, i64 } zeroinitializer, ptr %taps, 0
  %tap1 = insertvalue { ptr, i64 } %tap0, i64 %tap_count, 1
  %tree = call ptr @wf__par_seq_filter_tiles({ ptr, i64 } %in1, { ptr, i64 } %tap1, i64 %first, i64 %end, i64 %last_tap, i64 %tile_size)
  ret ptr %tree
}

define i64 @wf_research_fir_count(ptr %tree) {
  %count = call i64 @wf_sample_count(ptr %tree)
  ret i64 %count
}

define i64 @wf_research_fir_get(ptr %tree, i64 %index, ptr %output) {
  %hit = call i64 @wf_sample_get(ptr %tree, i64 %index, ptr %output)
  ret i64 %hit
}

define i64 @wf_research_fir_history(ptr %input, i64 %input_count, i64 %history_count, i64 %index, ptr %output) {
  %in0 = insertvalue { ptr, i64 } zeroinitializer, ptr %input, 0
  %in1 = insertvalue { ptr, i64 } %in0, i64 %input_count, 1
  %hit = call i64 @wf_history_get({ ptr, i64 } %in1, i64 %history_count, i64 %index, ptr %output)
  ret i64 %hit
}

define i64 @wf_research_fir_release(ptr %tree) {
  %status = call i64 @wf_release_tiles(ptr %tree)
  ret i64 %status
}
