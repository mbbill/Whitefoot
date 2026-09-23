# Bind formal program adapters to the world selected by the real runtime.
# Compiler/program tests and explicit experiments share this binding path.
# Adapters are straight-line wrappers; this does not rewrite existing CFG edges.
# Remove it when exported function entries replace appended LLVM adapters.
NR == FNR {
    if ($0 ~ /^define .* @wf__par_seq_/) {
        name = $0
        sub(/^.* @wf__par_seq_/, "", name)
        sub(/\(.*/, "", name)
        clones["wf_" name] = "wf__par_seq_" name
    }
    print
    next
}
{
    indirect = match($0, /^[ \t]*call void @wf_[A-Za-z0-9_]+\(/)
    if (!indirect && !match($0, / = call [^{@]*(\{[^}]*\} )?@wf_[A-Za-z0-9_]+\(/)) {
        print
        next
    }
    at = index($0, "@wf_")
    callee = substr($0, at + 1)
    sub(/\(.*/, "", callee)
    if (!(callee in clones)) {
        print
        next
    }
    destination = $0
    sub(/ =.*/, "", destination)
    sub(/^[ \t]*/, "", destination)
    result = substr($0, index($0, "call ") + 5, at - index($0, "call ") - 5)
    sub(/[ \t]*$/, "", result)
    if (indirect) {
        # Keep assigned-call labels and output unchanged, even after this call.
        void_id++
        prefix = "wfb.world.void." void_id
    } else {
        id++
        prefix = "wfb.world." id
    }
    print "  %" prefix ".pool = call i32 @wf__par_pool_active()"
    print "  %" prefix ".active = icmp ne i32 %" prefix ".pool, 0"
    print "  br i1 %" prefix ".active, label %" prefix ".par, label %" prefix ".seq"
    print prefix ".par:"
    print "  " (indirect ? "" : "%" prefix ".a = ") "call " result " " substr($0, at)
    print "  br label %" prefix ".join"
    print prefix ".seq:"
    print "  " (indirect ? "" : "%" prefix ".b = ") "call " result " @" clones[callee] substr($0, at + length(callee) + 1)
    print "  br label %" prefix ".join"
    print prefix ".join:"
    if (!indirect)
        print "  " destination " = phi " result " [ %" prefix ".a, %" prefix ".par ], [ %" prefix ".b, %" prefix ".seq ]"
}
