# Exact report grammar for the quadrature check/calibration callers. Retire
# with this experiment. Correct numerical values are checked by the host.
function bad(why) { print "quadrature report: " why > "/dev/stderr"; failed=1; exit 1 }
function integer(x) { return x ~ /^[0-9]+$/ }
function field(text, prefix) {
    if (index(text,prefix)!=1) bad("field prefix " prefix)
    return substr(text,length(prefix)+1)
}
BEGIN {
    count=split("smooth center-peak left-peak right-peak outside-peak loose depth-zero depth-cap empty reverse", names," ")
    split("59 3287 2473 2473 503 3 1 8191 1 183", nodes," ")
    split("0 0 0 0 0 0 1 4009 0 0", capped," ")
    split("5 14 14 14 10 1 0 12 0 7", depths," ")
    if ((mode!="check" && mode!="bench") || (form!="native" && form!="wf-seq" && form!="wf-auto") ||
        (width!=1 && width!=4) || (stats!=0 && stats!=1)) bad("validator arguments")
    calls=(mode=="check"?2:9)
}
NR==1 {
    expected="# quadrature mode=" mode " form=" form " workers=" width " stats=" stats " events=" stats
    if ($0!=expected) bad("command identity")
    next
}
NR==2 {
    if ($0!="# columns=input form call phase wall_ns user_us system_us voluntary involuntary steals pool_lanes publishes local_pops runs joins slot_refusals") bad("columns")
    next
}
/^# input=/ {
    if (footer || (block && seen!=calls)) bad("missing calls")
    ++block;seen=0
    if (block>count || NF!=8 || field($2,"input=")!=names[block]) bad("input order")
    if (field($3,"nodes=")!=nodes[block] || field($4,"leaves=")!=(nodes[block]+1)/2 ||
        field($5,"capped=")!=capped[block] || field($6,"deepest=")!=depths[block] ||
        field($7,"evaluations=")!=3+2*nodes[block]) bad("work metadata")
    if (field($8,"expected=") !~ /^-?0x[01](\.[0-9a-f]+)?p[+-][0-9]+$/) bad("expected value")
    next
}
/^# quadrature PASS:/ {
    if (footer || block!=count || seen!=calls || NF!=6) bad("footer position")
    if (field($4,"outputs=")!=count*calls || field($5,"stats=")!=stats ||
        field($6,"steals=")!=steals) bad("footer totals")
    if (stats && form=="wf-auto" && width==4 && !steals) bad("no actual steal")
    footer=1;next
}
{
    if (footer || !block || seen>=calls || NF!=16 || $1!=names[block] || $2!=form ||
        $3!=seen || $4!=(seen?"warm":"first")) bad("call identity")
    for (i=5;i<=16;++i) if(!integer($i))bad("noninteger observation")
    if ($11!=((form=="wf-auto" && width==4)?4:0))bad("actual pool width")
    if (!stats || form!="wf-auto" || width==1) {
        if($10 || $12 || $13 || $14 || $15 || $16)bad("disabled/serial events")
    } else {
        if ($12!=$13+$10 || $12!=$14 || $12!=$15 ||
            $12+$16!=3*nodes[block]-(nodes[block]+1)/2+2)bad("task conservation")
    }
    steals+=$10;++seen;++rows
}
END {
    if (!failed && (!footer || rows!=count*calls))bad("incomplete report")
    if(failed)exit 1
}
