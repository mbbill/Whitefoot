# Sustained-batch report contract. Called by quadrature-batch.sh; retire with
# the quadrature cost-attribution panel. Numerical equality is checked in C.
function bad(why) { print "quadrature batch report: " why > "/dev/stderr"; failed=1; exit 1 }
function integer(x) { return x ~ /^[0-9]+$/ }
BEGIN {
    split("smooth center-peak left-peak right-peak outside-peak loose depth-zero depth-cap empty reverse",names," ")
    split("59 3287 2473 2473 503 3 1 8191 1 183",nodes," ")
    # Pinned binary64 fixtures from the independent explicit-stack oracle,
    # also checked by the ordinary quadrature qualification for all forms.
    split("0x1.dac670561e696p-1 0x1.8a205fd558741p-5 0x1.54b66ed3898f4p-5 0x1.54b66ed3898f4p-5 0x1.8f66c9347d918p-7 0x1.1b91b91b91b92p-1 0x1.170b527c76338p-3 0x1.54b66ed3898f2p-5 0x0p+0 -0x1.1b6e192eb8a66p-1",values," ")
    for(i=1;i<=10;++i)if(input==names[i]){expected_nodes=nodes[i];expected_value=values[i]}
    parallel=(form=="wf-auto" || form=="wf-leaf" || form=="wf-refusal")
    native_wf=(form=="wf-native" || form=="wf-value" || form=="wf-value-right")
    native=(native_wf || form=="tbb" || form=="parlay" || form=="parlay-left")
    if(!expected_nodes || (width!=1 && width!=4) || !integer(repeats) || repeats<1 || repeats>65536 ||
        (stats!=0 && stats!=1) || (control!=0 && control!=1) || !integer(spawn) || spawn>24 ||
        (!native && spawn!=0) || (!native && !parallel && form!="native" && form!="cpp-seq" &&
        form!="wf-seq" && form!="wf-leaf-seq" && form!="wf-refusal-seq"))bad("validator arguments")
}
NR==1 {
    if($0!="# quadrature batch v1: input form workers spawn_depth repeats warmup perf_control stats nodes_per_call wall_ns user_us system_us voluntary involuntary minor_faults major_faults wf_lanes")bad("columns")
    next
}
NR==2 {
    if(NF!=17 || $1!=input || $2!=form || $3!=width || $4!=spawn || $5!=repeats ||
        $6!=8 || $7!=control || $8!=stats || $9!=expected_nodes)bad("batch identity")
    for(i=3;i<=17;++i)if(!integer($i))bad("noninteger observation")
    offers=(parallel && (form=="wf-auto" || expected_nodes>1)) || (native_wf && spawn && expected_nodes>1)
    if($17!=((offers && width==4)?4:0))bad("WF pool width")
    next
}
NR==3 {
    if(NF!=7 || $1!="#" || $2!="quadrature" || $3!="batch" || $4!="PASS:" ||
        $5!="outputs=" repeats+8 || $6!="expected=" expected_value ||
        $7!="mismatch=0")bad("footer")
    next
}
{ bad("extra row") }
END { if(!failed && NR!=3)bad("incomplete report"); if(failed)exit 1 }
