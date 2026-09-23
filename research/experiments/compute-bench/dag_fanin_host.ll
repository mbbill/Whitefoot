; Research-only runtime-DAG entries. TaskCell is { i64 value, i64 evaluations }.
; host-adapter.awk chooses the same world as ordinary runtime bootstrap.
define void @dag_probe_spine(i64 %count, ptr %costs, i64 %length, ptr %output, i64 %seed) {
  %ca = insertvalue { ptr, i64 } poison, ptr %costs, 0
  %cb = insertvalue { ptr, i64 } %ca, i64 %length, 1
  %oa = insertvalue { ptr, i64 } poison, ptr %output, 0
  %ob = insertvalue { ptr, i64 } %oa, i64 %length, 1
  %r = call i8 @wf_dag_spine(i64 %count, { ptr, i64 } %cb, { ptr, i64 } %ob, i64 %seed)
  ret void
}

define void @dag_probe_spine_phased(i64 %count, ptr %costs, i64 %length, ptr %output, i64 %seed) {
  %ca = insertvalue { ptr, i64 } poison, ptr %costs, 0
  %cb = insertvalue { ptr, i64 } %ca, i64 %length, 1
  %oa = insertvalue { ptr, i64 } poison, ptr %output, 0
  %ob = insertvalue { ptr, i64 } %oa, i64 %length, 1
  %r = call i8 @wf_dag_spine_phased(i64 %count, { ptr, i64 } %cb, { ptr, i64 } %ob, i64 %seed)
  ret void
}

define void @dag_probe_spine_phased_scalar(i64 %count, i64 %leaf_steps, i64 %length, ptr %output, i64 %seed) {
  %oa = insertvalue { ptr, i64 } poison, ptr %output, 0
  %ob = insertvalue { ptr, i64 } %oa, i64 %length, 1
  %r = call i8 @wf_dag_spine_phased_scalar(i64 %count, i64 %leaf_steps, { ptr, i64 } %ob, i64 %seed)
  ret void
}

define void @dag_probe_notify(i64 %mask, ptr %costs, ptr %output, ptr %receipts, i64 %seed) {
  %ca = insertvalue { ptr, i64 } poison, ptr %costs, 0
  %cb = insertvalue { ptr, i64 } %ca, i64 4, 1
  %oa = insertvalue { ptr, i64 } poison, ptr %output, 0
  %ob = insertvalue { ptr, i64 } %oa, i64 4, 1
  %ra = insertvalue { ptr, i64 } poison, ptr %receipts, 0
  %rb = insertvalue { ptr, i64 } %ra, i64 4, 1
  %r = call i8 @wf_dag_notify(i64 %mask, { ptr, i64 } %cb, { ptr, i64 } %ob, { ptr, i64 } %rb, i64 %seed)
  ret void
}

define void @dag_probe_n(i64 %mode, ptr %costs, ptr %output, i64 %seed) {
  %ca = insertvalue { ptr, i64 } poison, ptr %costs, 0
  %cb = insertvalue { ptr, i64 } %ca, i64 4, 1
  %oa = insertvalue { ptr, i64 } poison, ptr %output, 0
  %ob = insertvalue { ptr, i64 } %oa, i64 4, 1
  %r = call i8 @wf_dag_n(i64 %mode, { ptr, i64 } %cb, { ptr, i64 } %ob, i64 %seed)
  ret void
}

define void @dag_probe_wide_four(ptr %costs, ptr %output, i64 %seed) {
  %ca = insertvalue { ptr, i64 } poison, ptr %costs, 0
  %cb = insertvalue { ptr, i64 } %ca, i64 4, 1
  %oa = insertvalue { ptr, i64 } poison, ptr %output, 0
  %ob = insertvalue { ptr, i64 } %oa, i64 4, 1
  %r = call i8 @wf_dag_wide_four({ ptr, i64 } %cb, { ptr, i64 } %ob, i64 %seed)
  ret void
}

define void @dag_probe_runtime(i64 %form, i64 %owners, i64 %count, ptr %costs, i64 %successor_count, ptr %successors, i64 %output_count, ptr %output, i64 %seed, ptr %report) {
  %ca = insertvalue { ptr, i64 } poison, ptr %costs, 0
  %cb = insertvalue { ptr, i64 } %ca, i64 %count, 1
  %sa = insertvalue { ptr, i64 } poison, ptr %successors, 0
  %sb = insertvalue { ptr, i64 } %sa, i64 %successor_count, 1
  %oa = insertvalue { ptr, i64 } poison, ptr %output, 0
  %ob = insertvalue { ptr, i64 } %oa, i64 %output_count, 1
  call void @wf_dag_runtime(ptr %report, i64 %form, i64 %owners, { ptr, i64 } %cb, { ptr, i64 } %sb, { ptr, i64 } %ob, i64 %seed)
  ret void
}
