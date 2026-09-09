# Whole-process paired ratios; raw FIR call/tail measurements remain in raw/.
# Above one means the named numerator costs more than stats-on research.
BEGIN {
    FS=OFS="\t"
    if(expected_passes!~/^[0-9]+$/ || expected_passes<1 || expected_passes>20 || widths=="") bad=1
    nw=split(widths,ws," ")
    for(w=1;w<=nw;w++) {
        if(ws[w]!~/^(1|2|4|8|16|32|64)$/ || width_seen[ws[w]]++) bad=1
        for(size=4096;size<=65536;size*=16) {
            split("16 64 1024",tiles," ")
            for(t=1;t<=3;t++) expected["fir" FS size "-" tiles[t] FS ws[w]]=1
            for(s=0;s<=6;s++) expected["mandelbrot" FS s "-" size FS ws[w]]=1
        }
    }
}
NR==1 { if ($0!="suite\tcase\tworkers\tpass\tvariant\twall_ns\tuser_ns\tsystem_ns\trss_bytes\tvoluntary\tinvoluntary\tstatus") bad=1; next }
{
    if(NF!=12 || $12!=0 || $6<=0 || $9<=0 || $4<1 || $4>expected_passes ||
       $5!~/^(old|research|research-off|replica)$/) bad=1
    for(i=3;i<=12;i++) if(i!=5 && $i!~/^[0-9]+$/) bad=1
    key=$1 FS $2 FS $3; sample=key SUBSEP $4 SUBSEP $5
    if(!(key in expected)) bad=1
    if(seen[sample]++) bad=1
    if(!(key in cells)) order[++count]=key
    cells[key]=1; passes[key,$4]=1
    wall[sample]=$6; cpu[sample]=$7+$8; rss[sample]=$9
    switches[sample]=$10+$11
}
function sort(a,n, i,j,x) {for(i=2;i<=n;i++){x=a[i];j=i-1;while(j>0&&a[j]>x){a[j+1]=a[j];j--}a[j+1]=x}}
function median(a,n) {sort(a,n);return n%2?a[(n+1)/2]:(a[n/2]+a[n/2+1])/2}
END {
    split("old research research-off replica",all_variants," ")
    for(key in expected) for(p=1;p<=expected_passes;p++) for(v=1;v<=4;v++)
        if(!((key SUBSEP p SUBSEP all_variants[v]) in seen)) bad=1
    if(NR<2 || bad) {print "invalid or incomplete comparison data" > "/dev/stderr";exit 1}
    print "suite","case","workers","variant/research","pairs","wall_median","wall_min","wall_max","cpu_median","rss_median","switches_delta_median"
    split("old research-off replica",variants," ")
    for(c=1;c<=count;c++) for(v=1;v<=3;v++) {
        key=order[c];n=0
        for(p=1;p<=20;p++) if((key SUBSEP p) in passes) {
            base=key SUBSEP p SUBSEP "research"; sample=key SUBSEP p SUBSEP variants[v]
            if(!(sample in seen)||!(base in seen)||cpu[base]<=0) {bad=1;continue}
            wr[++n]=wall[sample]/wall[base];cr[n]=cpu[sample]/cpu[base]
            mr[n]=rss[sample]/rss[base];sr[n]=switches[sample]-switches[base]
        }
        if(!n){bad=1;continue}
        wm=median(wr,n);cm=median(cr,n);mm=median(mr,n);sm=median(sr,n)
        printf "%s\t%s\t%d\t%.4f\t%.4f\t%.4f\t%.4f\t%.4f\t%.0f\n",key,variants[v],n,wm,wr[1],wr[n],cm,mm,sm
    }
    if(bad) {print "invalid or incomplete comparison data" > "/dev/stderr";exit 1}
}
