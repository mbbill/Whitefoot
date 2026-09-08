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
    split("3 3 3 3 3 1 0 3 0 3", forks2," ")
    split("15 15 15 15 15 1 0 15 0 15", forks4," ")
    split("29 247 176 176 184 1 0 255 0 91", forks8," ")
    frontier=(form=="wf-frontier" || form=="wf-frontier-seq")
    refusal=(form=="wf-refusal" || form=="wf-refusal-seq")
    split("5 7 6 13 10 1 0 12 0 6", refusal_forks," ")
    parallel=(form=="wf-auto" || form=="wf-leaf" || form=="wf-refusal" || form=="wf-frontier")
    leaf=(form=="wf-leaf" || form=="wf-leaf-seq" || refusal || frontier)
    native_wf=(form=="wf-native" || form=="wf-value" || form=="wf-value-right")
    rayon=(form=="rayon" || form=="rayon-left")
    native=(form=="cpp-seq" || form=="tbb" || form=="parlay" || form=="parlay-left" || native_wf || rayon || form=="rust-seq")
    if (spawn=="")spawn=0
    if (!integer(spawn) || (spawn!=0 && spawn!=2 && spawn!=4 && spawn!=8 && spawn!=24) ||
        ((form!="tbb" && form!="parlay" && form!="parlay-left" && !native_wf && !frontier && !rayon) && spawn) || (frontier && spawn!=8))bad("spawn depth")
    wf_pool=(parallel || (native_wf && spawn>0))
    if ((mode!="check" && mode!="bench" && mode!="exhaust") || (form!="native" && form!="wf-seq" && !parallel && !leaf && !native) ||
        (width!=1 && width!=4) || (stats!=0 && stats!=1)) bad("validator arguments")
    if (mode=="exhaust" && (width!=4 || !((native_wf && spawn==24) || ((refusal || frontier) && parallel))))bad("exhaustion arguments")
    calls=(mode=="bench"?9:2)
}
NR==1 {
    expected="# quadrature mode=" mode " form=" form " workers=" width " stats=" stats " events=" stats " spawn_depth=" spawn
    if ($0!=expected) bad("command identity")
    next
}
NR==2 {
    if ($0!="# columns=input form call phase wall_ns user_us system_us voluntary involuntary steals pool_lanes publishes local_pops runs joins slot_refusals native_nodes native_forks migrated_branches") bad("columns")
    next
}
/^# input=/ {
    if (footer || (block && seen!=calls)) bad("missing calls")
    ++block;seen=0
    if (block>count || NF!=13 || field($2,"input=")!=names[block]) bad("input order")
    if (field($3,"nodes=")!=nodes[block] || field($4,"leaves=")!=(nodes[block]+1)/2 ||
        field($5,"capped=")!=capped[block] || field($6,"deepest=")!=depths[block] ||
        field($7,"evaluations=")!=3+2*nodes[block]) bad("work metadata")
    if (field($8,"expected=") !~ /^-?0x[01](\.[0-9a-f]+)?p[+-][0-9]+$/) bad("expected value")
    forks=field($9,"forks=")
    if (!integer(forks))bad("noninteger forks")
    forks+=0
    expected_forks=(spawn==0?0:spawn==2?forks2[block]:spawn==4?forks4[block]:spawn==8?forks8[block]:(nodes[block]-1)/2)
    if (forks!=expected_forks)bad("fork metadata")
    frontier_blocks=field($10,"frontier_blocks=");frontier_nodes=field($11,"frontier_nodes=")
    largest=field($12,"largest_subtree=");span=field($13,"node_span=")
    if(!integer(frontier_blocks) || !integer(frontier_nodes) || !integer(largest) || !integer(span))
        bad("noninteger frontier work")
    frontier_blocks+=0;frontier_nodes+=0;largest+=0;span+=0
    if(frontier_blocks!=forks+1 || frontier_nodes+forks!=nodes[block] || largest<1 ||
        largest>frontier_nodes || largest*frontier_blocks<frontier_nodes ||
        span<largest || span>nodes[block])bad("frontier work metadata")
    if((spawn==0 && (largest!=nodes[block] || span!=nodes[block])) ||
        (spawn>=depths[block] && (largest!=1 || span!=depths[block]+1)))bad("frontier work boundary")
    if(names[block]=="depth-cap") {
        cut=(spawn<12?spawn:12);subtree=2^(13-cut)-1
        if(largest!=subtree || span!=cut+subtree)bad("uniform frontier span")
    }
    next
}
/^# quadrature PASS:/ {
    if (footer || block!=count || seen!=calls || NF!=7) bad("footer position")
    if (field($4,"outputs=")!=count*calls || field($5,"stats=")!=stats ||
        field($6,"steals=")!=steals || field($7,"migrated=")!=migrated) bad("footer totals")
    if (stats && parallel && width==4 && mode!="exhaust" && !steals) bad("no actual steal")
    if (stats && rayon && width==4 && spawn && !migrated) bad("no Rayon migration")
    footer=1;next
}
{
    if (footer || !block || seen>=calls || NF!=19 || $1!=names[block] || $2!=form ||
        $3!=seen || $4!=(seen?"warm":"first")) bad("call identity")
    for (i=5;i<=19;++i) if(!integer($i))bad("noninteger observation")
    if ($11!=((wf_pool && width==4)?4:0))bad("actual pool width")
    if (!stats || !wf_pool || width==1) {
        if($10 || $12 || $13 || $14 || $15 || $16)bad("disabled/serial events")
    } else {
        opportunities=((native_wf || frontier)?forks:leaf?nodes[block]-(nodes[block]+1)/2:3*nodes[block]-(nodes[block]+1)/2+2)
        if (refusal && mode=="exhaust")opportunities=refusal_forks[block]
        if ($12!=$13+$10 || $12!=$14 || $12!=$15)bad("joined task conservation")
        if (refusal && mode!="exhaust") {
            if ($12+$16<refusal_forks[block] || $12+$16>opportunities)bad("refusal subtree opportunity bounds")
            if (!$16 && $12!=opportunities)bad("unrefused subtree opportunities")
        } else if ($12+$16!=opportunities)bad("task opportunities")
        if (mode=="exhaust" && ($12 || $16!=opportunities))bad("full owner-pool fallback")
    }
    if (stats && native) {
        if($17!=nodes[block] || $18!=forks || $19>2*forks || (width==1 && $19))bad("native conservation")
        if(native_wf && $19!=$10)bad("native WF steal witness")
    } else if($17 || $18 || $19)bad("disabled native observations")
    migrated+=$19
    steals+=$10;++seen;++rows
}
END {
    if (!failed && (!footer || rows!=count*calls))bad("incomplete report")
    if(failed)exit 1
}
