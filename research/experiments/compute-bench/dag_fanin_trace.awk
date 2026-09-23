# Research-only adapter and passive recurrence observer for dag-fanin-emit.
# Retire with the runtime-DAG investigation. No scheduler call is intercepted.
{
    gsub(/@main\(/, "@dag_original_main(")
    gsub(/@wf__main_body\(/, "@dag_original_main_body(")
    if ($0 ~ /^define i64 @wf_dag_task\(/) {
        tasks++
        if (observed) sub(/@wf_dag_task\(/, "@wf_dag_task_body(")
    }
    print
}
END {
    if (tasks != 1) {
        print "dag-fanin: expected exactly one scalar dag_task definition" > "/dev/stderr"
        exit 1
    }
    print "\ndefine i32 @dag_probe_trace_image() {"
    print "  ret i32 " (observed ? 1 : 0)
    print "}"
    if (observed) {
        print "\ndeclare i64 @dag_trace_begin(i64, i64, i64)"
        print "declare void @dag_trace_end(i64, i64)"
        print "define i64 @wf_dag_task(i64 %id, i64 %steps, i64 %seed, i64 %predecessors) {"
        print "  %observed_seed = call i64 @dag_trace_begin(i64 %id, i64 %steps, i64 %seed)"
        print "  %result = call i64 @wf_dag_task_body(i64 %id, i64 %steps, i64 %observed_seed, i64 %predecessors)"
        print "  call void @dag_trace_end(i64 %id, i64 %result)"
        print "  ret i64 %result"
        print "}"
    }
}
