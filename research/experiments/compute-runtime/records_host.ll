define i64 @wf_research_record_summary(ptr %data, i64 %count, i64 %first, i64 %end) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %result = call i64 @wf_record_summary({ ptr, i64 } %b, i64 %first, i64 %end)
  ret i64 %result
}

define void @wf_research_records_parallel(ptr %data, i64 %count, ptr %offsets, i64 %offset_count, i64 %first, i64 %end, ptr %output_data, ptr %output_count) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %c = insertvalue { ptr, i64 } poison, ptr %offsets, 0
  %d = insertvalue { ptr, i64 } %c, i64 %offset_count, 1
  %result = call { ptr, i64 } @wf_summarize_records({ ptr, i64 } %b, { ptr, i64 } %d, i64 %first, i64 %end)
  %pointer = extractvalue { ptr, i64 } %result, 0
  %length = extractvalue { ptr, i64 } %result, 1
  store ptr %pointer, ptr %output_data
  store i64 %length, ptr %output_count
  ret void
}

define void @wf_research_records_sequential(ptr %data, i64 %count, ptr %offsets, i64 %offset_count, i64 %first, i64 %end, ptr %output_data, ptr %output_count) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %c = insertvalue { ptr, i64 } poison, ptr %offsets, 0
  %d = insertvalue { ptr, i64 } %c, i64 %offset_count, 1
  %result = call { ptr, i64 } @wf__par_seq_summarize_records({ ptr, i64 } %b, { ptr, i64 } %d, i64 %first, i64 %end)
  %pointer = extractvalue { ptr, i64 } %result, 0
  %length = extractvalue { ptr, i64 } %result, 1
  store ptr %pointer, ptr %output_data
  store i64 %length, ptr %output_count
  ret void
}

define i64 @wf_research_records_release(ptr %data, i64 %count) {
  %a = insertvalue { ptr, i64 } poison, ptr %data, 0
  %b = insertvalue { ptr, i64 } %a, i64 %count, 1
  %result = call i64 @wf_record_result_release({ ptr, i64 } %b)
  ret i64 %result
}
