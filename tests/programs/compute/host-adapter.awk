# Bind formal program adapters to the world selected by the real runtime.
# Compiler/program tests and explicit experiments share this binding path.
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
    if (!match($0, / = call [^{@]*(\{[^}]*\} )?@wf_[A-Za-z0-9_]+\(/)) {
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
    id++
    prefix = "wfb.world." id
    print "  %" prefix ".pool = call i32 @wf__par_pool_active()"
    print "  %" prefix ".active = icmp ne i32 %" prefix ".pool, 0"
    print "  br i1 %" prefix ".active, label %" prefix ".par, label %" prefix ".seq"
    print prefix ".par:"
    print "  %" prefix ".a = call " result " " substr($0, at)
    print "  br label %" prefix ".join"
    print prefix ".seq:"
    print "  %" prefix ".b = call " result " @" clones[callee] substr($0, at + length(callee) + 1)
    print "  br label %" prefix ".join"
    print prefix ".join:"
    print "  " destination " = phi " result " [ %" prefix ".a, %" prefix ".par ], [ %" prefix ".b, %" prefix ".seq ]"
}
