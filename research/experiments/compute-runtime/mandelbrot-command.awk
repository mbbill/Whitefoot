# Whole-process, matched-pass comparisons; five processes are five samples.
# Called by mandelbrot-command.sh. Retire with the command workload.
BEGIN {
    FS=OFS="\t"
    if(expected_workers!=1 && expected_workers!=2 && expected_workers!=4) invalid("expected worker count required")
}
function invalid(message) {
    print "invalid command screen: " message > "/dev/stderr"
    bad=1
    exit 2
}
NR==1 {
    if ($0 != "shape\tcount\tlimit\trepetitions\tpass\tform\trequested_workers\twall_ns\tuser_ns\tsystem_ns\trss_bytes\tvoluntary\tinvoluntary\tstatus") invalid("header")
    next
}
{
    if (NF!=14 || $1!~/^[0-6]$/ || ($2!=4096 && $2!=65536) || $3!=256 ||
        $4!=($2==4096?32:2) || $5!~/^[0-4]$/ ||
        ($7!=1 && $7!=2 && $7!=4) || $14!=0) invalid("cell or status at line " NR)
    if ($6!="par" && $6!="replica" && $6!="static" && $6!="seq" && $6!="serial" &&
        $6!="work60000" && $6!="work240000" && $6!="nosplit") invalid("form")
    if (($6=="seq" || $6=="serial") && $7!=1) invalid("serial width")
    for (i=8;i<=11;i++) if ($i!~/^[0-9]+$/) invalid("metric")
    if (!(($12~/^[0-9]+$/ && $13~/^[0-9]+$/) || ($12=="NA" && $13=="NA"))) invalid("context switches")
    if ($8<=0 || $11<=0) invalid("nonpositive wall or RSS")
    k=$1 SUBSEP $2 SUBSEP $7 SUBSEP $5 SUBSEP $6
    if (k in wall) invalid("duplicate sample")
    wall[k]=$8; cpu[k]=$9+$10; rss[k]=$11
    if ($7>width) width=$7
}
function median(a,    i,j,t) {
    for(i=0;i<5;i++) for(j=i+1;j<5;j++) if(a[i]>a[j]) {t=a[i];a[i]=a[j];a[j]=t}
    return a[2]
}
function compare(shape,count,w,left,right,    p,k,l,r,aa,quiet,available,m,low,high,state,metric) {
    quiet=1
    for(p=0;p<5;p++) {
        k=shape SUBSEP count SUBSEP w SUBSEP p
        aa=wall[k SUBSEP "par"]/wall[k SUBSEP "replica"]
        if(aa<0.95 || aa>1.05) quiet=0
    }
    for(metric=1;metric<=3;metric++) {
        available=1
        for(p=0;p<5;p++) {
            k=shape SUBSEP count SUBSEP w SUBSEP p
            l=k SUBSEP left; r=k SUBSEP right
            if(metric==1) {a[p]=wall[l]/wall[r]}
            if(metric==2) {
                if(cpu[r]==0 || cpu[l]==0) {available=0; a[p]=0} else a[p]=cpu[l]/cpu[r]
            }
            if(metric==3) {a[p]=rss[l]/rss[r]}
        }
        m=median(a); low=a[0]; high=a[4]
        state="reported"
        if(metric==1 || metric==2) {
            state=quiet?"no-observed-gap":"noisy-open"
            if(quiet && low>1.05) {
                state="gap"
                # Tuning controls are diagnostic, not the selected default.
                if(left=="par" || left=="seq") gaps++
            }
        }
        if(!available) state="unavailable-open"
        print shape,count,w,left "/" right,(metric==1?"wall":metric==2?"cpu":"rss"), \
            (available?sprintf("%.4f",m):"NA"),(available?sprintf("%.4f",low):"NA"), \
            (available?sprintf("%.4f",high):"NA"),state
    }
}
END {
    if(bad) exit 2
    if(width!=expected_workers) invalid("missing planned worker count")
    split("par replica static work60000 work240000 nosplit seq serial",forms," ")
    for(s=0;s<7;s++) for(n=4096;n<=65536;n*=16) for(w=1;w<=width;w*=2)
        for(p=0;p<5;p++) for(f=1;f<=(w==1?8:6);f++)
            if(!((s SUBSEP n SUBSEP w SUBSEP p SUBSEP forms[f]) in wall)) invalid("missing sample")
    print "shape","count","workers","comparison","metric","median_ratio","min_ratio","max_ratio","screen"
    for(s=0;s<7;s++) for(n=4096;n<=65536;n*=16) for(w=1;w<=width;w*=2) {
        compare(s,n,w,"par","static")
        compare(s,n,w,"par","replica")
        compare(s,n,w,"work60000","par")
        compare(s,n,w,"work240000","par")
        compare(s,n,w,"nosplit","par")
        compare(s,n,w,"work60000","static")
        compare(s,n,w,"work240000","static")
        if(w==1) compare(s,n,w,"seq","serial")
    }
    # This is an initial regression screen, not full performance qualification.
    # Noisy cells remain unresolved; five observations are not population tails.
    if(gaps) exit 1
}
