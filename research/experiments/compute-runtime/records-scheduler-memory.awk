# Exact diagnostic grammar for the pinned Linux x86_64 Rayon caller worker.
# Addresses, build IDs, checkout paths and compiler-unit hashes may relocate;
# allocation sizes, counts, semantic stack identities and ordering may not.
function fail(why) {
    print "scheduler sanitizer report rejected: " why > "/dev/stderr";
    bad=1; exit 1
}
function frame(line, ordinal, text) {
    if (line !~ ("^    #" ordinal " 0x[0-9a-f]+ in ")) fail("stack frame syntax");
    text=line; sub(/^    #[0-9]+ 0x[0-9a-f]+ in /, "", text);
    return text
}
function object_frame(text, name, rest) {
    if (substr(text,1,length(name)+1)!=name " ") fail("object stack identity");
    rest=substr(text,length(name)+2);
    if (rest !~ ("^\\(/[^()]+/" object_basename "(\\+0x[0-9a-f]+)\\) \\(BuildId: [0-9a-f]+\\)$"))
        fail("object stack location");
}
function rust_frame(text, name, source, rest) {
    if (substr(text,1,length(name)+1)!=name " ") fail("Rust stack identity");
    rest=substr(text,length(name)+2);
    if (rest !~ ("^/rustc/[0-9a-f]+/library/std/src/" source ":[0-9]+:[0-9]+$"))
        fail("Rust stack location");
}
function pool_frame(text, name, rest) {
    if (substr(text,1,length(name)+1)!=name " ") fail("pool initialization identity");
    rest=substr(text,length(name)+2);
    if (rest !~ /^wf_records_rayon\.[0-9a-f]+-cgu\.[0-9]+$/) fail("pool compilation unit");
}
function allocation(    i,text) {
    if (kind=="worker") {
        if (workers++ || n!=7) fail("caller worker allocation count");
        object_frame(frame(stack[0],0),"posix_memalign");
        rust_frame(frame(stack[1],1),"std::sys::alloc::unix::aligned_malloc","sys/alloc/unix\\.rs");
        rust_frame(frame(stack[2],2),"<std::alloc::System as core::alloc::global::GlobalAlloc>::alloc","sys/alloc/unix\\.rs");
        rust_frame(frame(stack[3],3),"__rustc::__rdl_alloc","alloc\\.rs");
        pool_frame(frame(stack[4],4),force);
        rust_frame(frame(stack[5],5),"<std::sys::sync::once::futex::Once>::call","sys/sync/once/futex\\.rs");
        pool_frame(frame(stack[6],6),initialize);
    } else if (kind=="queue") {
        if (queues++ || n!=5 || workers!=1) fail("caller queue allocation count/order");
        object_frame(frame(stack[0],0),"calloc");
        object_frame(frame(stack[1],1),"<crossbeam_deque::deque::Block<rayon_core::job::JobRef>>::new");
        pool_frame(frame(stack[2],2),force);
        rust_frame(frame(stack[3],3),"<std::sys::sync::once::futex::Once>::call","sys/sync/once/futex\\.rs");
        pool_frame(frame(stack[4],4),initialize);
    } else if (kind=="extra") {
        # This branch belongs only to the explicit injected-leak negative
        # control. Its previously unseen caller stack is not a normal allowance.
        if (extras++ || n<2 || n>32) fail("injected allocation count");
        text=frame(stack[0],0);
        if (text !~ /^malloc [^[:space:]].+$/) fail("injected allocator");
        text=frame(stack[1],1);
        if (text !~ /^records_scheduler_leak_probe [^[:space:]].+$/) fail("injected allocation function");
        for(i=2;i<n;i++) if(frame(stack[i],i) !~ /[^[:space:]].+/) fail("injected caller stack");
    } else fail("unknown allocation");
    n=0; kind=""
}
BEGIN {
    # A shared workload image can retain this exact pinned lifecycle grammar.
    # Existing callers keep their original per-adapter binary identity.
    if (object_basename=="") object_basename="scheduler-memory-" binary;
    if (object_basename !~ /^[A-Za-z0-9_-]+$/) fail("binary identity parameter");
    init="<std::sync::once_lock::OnceLock<rayon_core::thread_pool::ThreadPool>>::initialize";
    get="<std::sync::once_lock::OnceLock<rayon_core::thread_pool::ThreadPool>>::get_or_init<wf_records_rayon::run::{closure#0}>::{closure#0}, !>";
    force="<std::sync::once::Once>::call_once_force::<" init "<" get "::{closure#0}>::{closure#0}";
    initialize=init "::<" get;
}
!started {
    if ($0=="") { started=1; next }
    print $0 > prefix; next
}
{
    if (!leak) fail("unexpected diagnostic after functional report");
    if (started==1) {
        if ($0!="=================================================================") fail("leak delimiter");
        started=2; next
    }
    if (started==2) {
        if ($0 !~ /^==[0-9]+==ERROR: LeakSanitizer: detected memory leaks$/) fail("leak heading");
        started=3; next
    }
    if (started==3) {
        if ($0!="") fail("leak heading separator");
        started=4; next
    }
    if (finished) fail("trailing diagnostic");
    if (kind!="") {
        if ($0=="") allocation();
        else { if(n>=32) fail("too many stack frames"); stack[n++]=$0 }
        next
    }
    if ($0=="Direct leak of 384 byte(s) in 1 object(s) allocated from:") kind="worker";
    else if ($0=="Indirect leak of 1520 byte(s) in 1 object(s) allocated from:") kind="queue";
    else if (extra==257 && $0=="Direct leak of 257 byte(s) in 1 object(s) allocated from:") kind="extra";
    else if ($0=="SUMMARY: AddressSanitizer: " (1904+extra) " byte(s) leaked in " (2+(extra!=0)) " allocation(s).") finished=1;
    else fail("unknown allocation or summary");
}
END {
    close(prefix);
    if (bad) exit 1;
    if (leak && (started!=4 || !finished || kind!="" || workers!=1 || queues!=1 || extras!=(extra!=0)))
        fail("incomplete lifecycle report");
    if (!leak && started) fail("unexpected trailing blank line");
}
