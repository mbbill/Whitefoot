#define _GNU_SOURCE
#define _DARWIN_C_SOURCE
/* Adaptive Lorentz-profile integration: correctness and initial generated-code
 * attribution. Retire with the owning quadrature experiment. No SIMD or FMA. */
#include "runtime.h"
#include "runtime_events.h"
#include "quadrature_native.h"
#include <errno.h>
#include <inttypes.h>
#include <limits.h>
#include <math.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <time.h>
#include <unistd.h>

extern int wf__floor_run(int, char **);
extern double wf_research_quadrature(double, double, double, double, double, uint64_t, bool);
extern double wf_research_quadrature_leaf(double, double, double, double, double, uint64_t, bool);
extern double wf_research_quadrature_refusal(double, double, double, double, double, uint64_t, bool);
extern double wf_research_quadrature_frontier(double, double, double, double, double, uint64_t, bool);
static double (*generated_run)(double,double,double,double,double,uint64_t,bool);
static bool parallel_form, leaf_form, refusal_form, frontier_form;
static bool native_control;
static bool native_wf;
static unsigned native_kind, spawn_depth, requested;
static void require(bool ok, const char *why) {
    if (!ok) { fprintf(stderr, "quadrature: %s\n", why); exit(1); }
}
static uint64_t now(void) {
    struct timespec t;
    require(!clock_gettime(CLOCK_MONOTONIC_RAW, &t), "clock");
    return (uint64_t)t.tv_sec * UINT64_C(1000000000) + (uint64_t)t.tv_nsec;
}
static uint64_t bits(double x) { uint64_t v; memcpy(&v, &x, sizeof(v)); return v; }
static uint64_t cpu_us(struct timeval t) { return (uint64_t)t.tv_sec*1000000+(uint64_t)t.tv_usec; }
static uint64_t cpu_clock_ns(clockid_t clock) {
    struct timespec t;
    require(!clock_gettime(clock,&t),"batch CPU clock");
    return (uint64_t)t.tv_sec*UINT64_C(1000000000)+(uint64_t)t.tv_nsec;
}
#if defined(WF_COMPUTE_EVENTS)
static unsigned long event_sum(unsigned event) {
    unsigned long count=0;
    for(unsigned lane=0;lane<4;++lane)count+=wf_compute_event(lane,event);
    return count;
}
#endif
typedef struct { const char *name; double a,b,center,width,tolerance; unsigned depth; bool converged; } Input;
static const Input cases[] = {
    {"smooth",0,1,0.5,1,0x1p-30,24,true},
    {"center-peak",0,1,0.5,0x1p-6,0x1p-42,24,true},
    {"left-peak",0,1,0x1p-5,0x1p-6,0x1p-42,24,true},
    {"right-peak",0,1,1-0x1p-5,0x1p-6,0x1p-42,24,true},
    {"outside-peak",0,1,1.25,0x1p-4,0x1p-42,24,true},
    {"loose",0,1,0.5,0.25,0x1p-8,24,true},
    {"depth-zero",0,1,0.5,0x1p-6,0,0,false},
    {"depth-cap",0,1,0x1p-5,0x1p-6,0,12,false},
    {"empty",0.5,0.5,0.5,0.25,0x1p-30,24,true},
    {"reverse",1,0,0.5,0.25,0x1p-30,24,true}
};
/* Native recursion is an algorithm/kernel reference, not a scheduler ceiling. */
static double density(double x, double center, double width) {
    double z=(x-center)/width; return 1/(1+z*z);
}
static double simpson(double a,double b,double fa,double fm,double fb) {
    return ((b-a)/6)*((fa+4*fm)+fb);
}
static double adaptive(double a,double b,double c,double w,double fa,double fm,double fb,
                       double whole,double tolerance,unsigned depth) {
    double m=(a+b)*0.5,fl=density((a+m)*0.5,c,w),fr=density((m+b)*0.5,c,w);
    double left=simpson(a,m,fa,fl,fm),right=simpson(m,b,fm,fr,fb);
    double combined=left+right,delta=combined-whole;
    if (!depth || fabs(delta)<=15*tolerance) return combined+delta/15;
    double l=adaptive(a,m,c,w,fa,fl,fm,left,tolerance*0.5,depth-1);
    double r=adaptive(m,b,c,w,fm,fr,fb,right,tolerance*0.5,depth-1);
    return l+r;
}
static double native(const Input *p) {
    double fa=density(p->a,p->center,p->width),fm=density((p->a+p->b)*0.5,p->center,p->width);
    double fb=density(p->b,p->center,p->width),whole=simpson(p->a,p->b,fa,fm,fb);
    return adaptive(p->a,p->b,p->center,p->width,fa,fm,fb,whole,p->tolerance,p->depth);
}
/* Independent explicit-stack traversal preserves the declared left/right sum
 * order. Volatile temporaries force every binary64 rounding outside timing.
 * The analytic antiderivative separately anchors converged cases. */
static double oracle_density(double x,const Input *p) {
    volatile double d=x-p->center,z=d/p->width,zz=z*z,den=1+zz,value=1/den;
    return value;
}
static double oracle_simpson(double a,double b,double fa,double fm,double fb) {
    volatile double span=b-a,scale=span/6,weighted=4*fm,partial=fa+weighted,sum=partial+fb,value=scale*sum;
    return value;
}
typedef struct { double a,b,fa,fm,fb,whole,tol,m,fl,fr,left,right,result; unsigned depth,state; bool right_spine; } Node;
typedef struct { double value; uint64_t nodes,leaves,capped,forks,refusal_forks; unsigned deepest; } Reference;
static Reference reference(const Input *p) {
    Node stack[25]={0}; unsigned top=0; Reference out={0};
    volatile double endpoints=p->a+p->b,m=endpoints*0.5;
    double fa=oracle_density(p->a,p),fm=oracle_density(m,p),fb=oracle_density(p->b,p);
    stack[0]=(Node){.a=p->a,.b=p->b,.fa=fa,.fm=fm,.fb=fb,
        .whole=oracle_simpson(p->a,p->b,fa,fm,fb),.tol=p->tolerance,.depth=p->depth,.right_spine=true};
    for (;;) {
        Node *n=&stack[top];
        if (!n->state) {
            ++out.nodes; if(top>out.deepest)out.deepest=top;
            volatile double sum=n->a+n->b,mid=sum*0.5,ls=n->a+mid,lm=ls*0.5,rs=mid+n->b,rm=rs*0.5;
            n->m=mid;n->fl=oracle_density(lm,p);n->fr=oracle_density(rm,p);
            n->left=oracle_simpson(n->a,n->m,n->fa,n->fl,n->fm);
            n->right=oracle_simpson(n->m,n->b,n->fm,n->fr,n->fb);
            volatile double combined=n->left+n->right,delta=combined-n->whole,threshold=15*n->tol;
            if (!n->depth || fabs(delta)<=threshold) {
                volatile double correction=delta/15,value=combined+correction;
                n->result=value;n->state=3;++out.leaves;
                if (!n->depth && fabs(delta)>threshold)++out.capped;
            } else {
                if(top<spawn_depth)++out.forks;
                if(n->right_spine)++out.refusal_forks;
                n->state=1;require(top<24,"oracle stack domain");
                volatile double tol=n->tol*0.5;
                Node child={.a=n->a,.b=n->m,.fa=n->fa,.fm=n->fl,.fb=n->fm,
                    .whole=n->left,.tol=tol,.depth=n->depth-1};
                stack[++top]=child;continue;
            }
        }
        if(n->state==3) {
            double value=n->result;
            if(!top){out.value=value;return out;}
            --top;Node *parent=&stack[top];
            if(parent->state==1) {
                parent->result=value;parent->state=2;
                volatile double tol=parent->tol*0.5;
                Node child={.a=parent->m,.b=parent->b,.fa=parent->fm,.fm=parent->fr,.fb=parent->fb,
                    .whole=parent->right,.tol=tol,.depth=parent->depth-1,.right_spine=parent->right_spine};
                stack[++top]=child;
            } else {
                require(parent->state==2,"oracle traversal");
                volatile double sum=parent->result+value;parent->result=sum;parent->state=3;
            }
        }
    }
}
static double run(const Input *p,const char *form) {
    if(!strcmp(form,"native"))return native(p);
    if(native_control)return quadrature_native_run(native_kind,requested,spawn_depth,
        p->a,p->b,p->center,p->width,p->tolerance,p->depth);
    return generated_run(p->a,p->b,p->center,p->width,p->tolerance,p->depth,parallel_form);
}
/* One opaque dispatch shared by every sustained form prevents a visible pure
 * native kernel from being hoisted out of the repeated-input loop. Read once
 * before warmup; the timed loop pays one common indirect call per result. */
static double (*volatile batch_dispatch)(const Input *,const char *)=run;
static unsigned environment_number(const char *name,unsigned minimum,unsigned maximum) {
    const char *text=getenv(name);char *end;
    require(text && text[0]>='0' && text[0]<='9',"batch numeric environment");
    errno=0;unsigned long value=strtoul(text,&end,10);
    require(!errno && !*end && value>=minimum && value<=maximum,"batch numeric domain");
    return (unsigned)value;
}
/* perf stat starts with inherited counters disabled. Its acknowledgement
 * brackets a sustained batch, excluding oracle construction and pool warmup.
 * Counts include this boundary protocol and resource/clock reads; they are
 * not instruction counts for an individual call or scheduler operation. */
static void perf_command(int control,int acknowledgement,const char *command) {
    if(control<0)return;
    size_t sent=0,length=strlen(command);
    while(sent<length) {
        ssize_t n=write(control,command+sent,length-sent);
        if(n<0 && errno==EINTR)continue;
        require(n>0,"perf control write");sent+=(size_t)n;
    }
    char ack[4];size_t received=0;bool skipped_nul=false;
    while(received<sizeof(ack)) {
        ssize_t n=read(acknowledgement,ack+received,1);
        if(n<0 && errno==EINTR)continue;
        require(n>0,"perf acknowledgement read");
        /* perf versions writing sizeof("ack\n") leave a trailing NUL
         * before the next acknowledgement. The documented line has none. */
        if(!received && !ack[0] && !skipped_nul){skipped_nul=true;continue;}
        received+=(size_t)n;
    }
    require(!memcmp(ack,"ack\n",sizeof(ack)),"perf acknowledgement value");
}
static void run_batch(const char *form) {
    const char *input=getenv("WF_QUADRATURE_INPUT");const Input *p=NULL;
    require(input!=NULL,"batch input environment");
    for(size_t i=0;i<sizeof(cases)/sizeof(cases[0]);++i)
        if(!strcmp(input,cases[i].name))p=&cases[i];
    require(p!=NULL,"batch input identity");
    unsigned repeats=environment_number("WF_QUADRATURE_REPEATS",1,65536);
    int control=-1,acknowledgement=-1;
    require((getenv("WF_PERF_CONTROL_FD")!=NULL)==(getenv("WF_PERF_ACK_FD")!=NULL),"paired perf descriptors");
    if(getenv("WF_PERF_CONTROL_FD")) {
        control=(int)environment_number("WF_PERF_CONTROL_FD",3,INT_MAX);
        acknowledgement=(int)environment_number("WF_PERF_ACK_FD",3,INT_MAX);
        require(control!=acknowledgement,"distinct perf descriptors");
    }
    Reference r=reference(p);uint64_t expected=bits(r.value),mismatch=0;
    double (*execute)(const Input *,const char *)=batch_dispatch;
    const unsigned warmup=8;
    for(unsigned i=0;i<warmup;++i)require(bits(execute(p,form))==expected,"batch warmup result");
    struct rusage before,after;
    perf_command(control,acknowledgement,"enable\n");
    require(!getrusage(RUSAGE_SELF,&before),"batch resources before");
    uint64_t process_start=cpu_clock_ns(CLOCK_PROCESS_CPUTIME_ID);
    uint64_t caller_start=cpu_clock_ns(CLOCK_THREAD_CPUTIME_ID);
    uint64_t start=now();
    for(unsigned i=0;i<repeats;++i)mismatch|=bits(execute(p,form))^expected;
    uint64_t elapsed=now()-start;
    uint64_t caller_cpu=cpu_clock_ns(CLOCK_THREAD_CPUTIME_ID)-caller_start;
    uint64_t process_cpu=cpu_clock_ns(CLOCK_PROCESS_CPUTIME_ID)-process_start;
    require(!getrusage(RUSAGE_SELF,&after),"batch resources after");
    perf_command(control,acknowledgement,"disable\n");
    require(!mismatch,"batch binary64 result");
    unsigned lanes=wf_compute_worker_count();
    bool offers=(parallel_form && (!leaf_form || r.nodes>1)) || (native_wf && r.forks);
    require(lanes==((offers && requested==4)?4:0),"batch WF pool width");
    puts("# quadrature batch v2: input form workers spawn_depth repeats warmup perf_control stats nodes_per_call wall_ns user_us system_us voluntary involuntary minor_faults major_faults wf_lanes process_cpu_ns caller_cpu_ns");
    printf("%s\t%s\t%u\t%u\t%u\t%u\t%d\t%d\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%ld\t%ld\t%ld\t%ld\t%u\t%" PRIu64 "\t%" PRIu64 "\n",
        p->name,form,requested,spawn_depth,repeats,warmup,control>=0,WF_COMPUTE_STATS,r.nodes,elapsed,
        cpu_us(after.ru_utime)-cpu_us(before.ru_utime),cpu_us(after.ru_stime)-cpu_us(before.ru_stime),
        after.ru_nvcsw-before.ru_nvcsw,after.ru_nivcsw-before.ru_nivcsw,
        after.ru_minflt-before.ru_minflt,after.ru_majflt-before.ru_majflt,lanes,process_cpu,caller_cpu);
    printf("# quadrature batch PASS: outputs=%u expected=%a mismatch=0\n",repeats+warmup,r.value);
}
int wf__main_body(int argc,char **argv) {
    require(argc==3 || argc==4,"usage: quadrature check|bench|exhaust|batch form [spawn-depth]");
    bool bench=!strcmp(argv[1],"bench"),exhaust=!strcmp(argv[1],"exhaust"),batch=!strcmp(argv[1],"batch");
    require(bench || exhaust || batch || !strcmp(argv[1],"check"),"mode");
    const char *form=argv[2];
    frontier_form=!strcmp(form,"wf-frontier") || !strcmp(form,"wf-frontier-seq");
    refusal_form=!strcmp(form,"wf-refusal") || !strcmp(form,"wf-refusal-seq");
    leaf_form=!strcmp(form,"wf-leaf") || !strcmp(form,"wf-leaf-seq") || refusal_form || frontier_form;
    parallel_form=!strcmp(form,"wf-auto") || !strcmp(form,"wf-leaf") || !strcmp(form,"wf-refusal") || !strcmp(form,"wf-frontier");
    native_wf=!strcmp(form,"wf-native") || !strcmp(form,"wf-value") || !strcmp(form,"wf-value-right");
    native_control=!strcmp(form,"cpp-seq") || !strcmp(form,"tbb") || !strcmp(form,"parlay") || !strcmp(form,"parlay-left") || native_wf || !strcmp(form,"rayon") || !strcmp(form,"rayon-left") || !strcmp(form,"rust-seq");
    native_kind=!strcmp(form,"tbb")?1:!strcmp(form,"parlay")?2:!strcmp(form,"wf-native")?3:
        !strcmp(form,"wf-value")?4:!strcmp(form,"parlay-left")?5:!strcmp(form,"wf-value-right")?6:!strcmp(form,"rayon")?7:!strcmp(form,"rayon-left")?8:!strcmp(form,"rust-seq")?9:0;
    require(!strcmp(form,"native") || !strcmp(form,"wf-seq") || parallel_form || leaf_form || native_control,"form");
    if((native_kind && native_kind!=9) || frontier_form) {
        require(argc==4 && argv[3][0]>='0' && argv[3][0]<='9',"explicit spawn depth");
        char *end;errno=0;unsigned long parsed=strtoul(argv[3],&end,10);
        require(!errno && !*end && parsed<=24,"spawn depth domain");spawn_depth=(unsigned)parsed;
        require(!frontier_form || spawn_depth==8,"compiled frontier depth");
    } else require(argc==3,"spawn depth only for depth-controlled forms");
    generated_run=frontier_form?wf_research_quadrature_frontier:refusal_form?wf_research_quadrature_refusal:leaf_form?wf_research_quadrature_leaf:wf_research_quadrature;
    const char *workers=getenv("WF_WORKERS");
    require(workers && (!strcmp(workers,"1") || !strcmp(workers,"4")),"explicit worker count");
    requested=!strcmp(workers,"4")?4:1;
    if(batch) {
        run_batch(form);quadrature_native_stop();
        require(!fflush(stdout),"batch report flush");return 0;
    }
    void **held=NULL;unsigned reserved=0;
    if(exhaust) {
        require(requested==4 && ((native_wf && spawn_depth==24) || ((refusal_form || frontier_form) && parallel_form)),"exhaustion control arguments");
        reserved=wf_compute_slot_capacity();held=calloc(reserved,sizeof(*held));
        require(held!=NULL && reserved>0,"exhaustion reservation array");
        for(unsigned i=0;i<reserved;++i) {
            held[i]=wf__par_acquire_lane(8);require(held[i]!=NULL,"exhaustion slot reservation");
        }
    }
    unsigned calls=bench?9:2;uint64_t outputs=0,total_steals=0,total_migrated=0;
    printf("# quadrature mode=%s form=%s workers=%u stats=%d events=%d spawn_depth=%u\n",argv[1],form,requested,WF_COMPUTE_STATS,
#if defined(WF_COMPUTE_EVENTS)
        1
#else
        0
#endif
        ,spawn_depth
    );
    puts("# columns=input form call phase wall_ns user_us system_us voluntary involuntary steals pool_lanes publishes local_pops runs joins slot_refusals native_nodes native_forks migrated_branches");
    for(size_t i=0;i<sizeof(cases)/sizeof(cases[0]);++i) {
        const Input *p=&cases[i];Reference r=reference(p);
        if(p->converged) {
            long double exact=(long double)p->width*(atanl(((long double)p->b-p->center)/p->width)-atanl(((long double)p->a-p->center)/p->width));
            require(r.capped==0,"unexpected depth exhaustion");
            require(fabsl((long double)r.value-exact)<=8*p->tolerance+0x1p-48L,"analytic integral");
        }
        printf("# input=%s nodes=%" PRIu64 " leaves=%" PRIu64 " capped=%" PRIu64 " deepest=%u evaluations=%" PRIu64 " expected=%a forks=%" PRIu64 "\n",p->name,r.nodes,r.leaves,r.capped,r.deepest,3+2*r.nodes,r.value,r.forks);
        for(unsigned call=0;call<calls;++call) {
            struct rusage before,after;require(!getrusage(RUSAGE_SELF,&before),"resources before");
#if WF_COMPUTE_STATS
            unsigned long steals=wf__par_grants();
#endif
#if defined(WF_COMPUTE_EVENTS)
            unsigned long events[WF_EVENT_COUNT];
            for(unsigned e=0;e<WF_EVENT_COUNT;++e)events[e]=event_sum(e);
#endif
            uint64_t start=now();double value=run(p,form);uint64_t elapsed=now()-start;
#if WF_COMPUTE_STATS
            steals=wf__par_grants()-steals;total_steals+=steals;
#else
            unsigned long steals=0;
#endif
            require(!getrusage(RUSAGE_SELF,&after),"resources after");
            require(bits(value)==bits(r.value),"binary64 result");++outputs;
            QuadratureObservation observed=quadrature_native_observation();
#if WF_COMPUTE_STATS
            if(native_control) {
                require(observed.nodes==r.nodes && observed.forks==r.forks,"native work conservation");
                require(observed.migrated<=2*observed.forks,"native branch count");
                if(native_wf)require(observed.migrated==steals,"native WF steal witness");
                if(requested==1)require(!observed.migrated,"native single-worker exclusion");
            } else require(!observed.nodes && !observed.forks && !observed.migrated,"native exclusion");
#else
            require(!observed.nodes && !observed.forks && !observed.migrated,"disabled native counters");
#endif
            total_migrated+=observed.migrated;
            unsigned long publishes=0,pops=0,runs=0,joins=0,refusals=0;
#if defined(WF_COMPUTE_EVENTS)
            for(unsigned e=0;e<WF_EVENT_COUNT;++e)events[e]=event_sum(e)-events[e];
            publishes=events[WF_EVENT_PUBLISH];pops=events[WF_EVENT_LOCAL_POP];
            runs=events[WF_EVENT_RUN_END];joins=events[WF_EVENT_JOIN];refusals=events[WF_EVENT_SLOT_REFUSAL];
            require(publishes==pops+events[WF_EVENT_STEAL_SUCCESS] && publishes==runs &&
                    publishes==events[WF_EVENT_RUN_BEGIN] && publishes==joins,"joined task conservation");
            if((parallel_form || (native_wf && spawn_depth)) && requested==4) {
                require(wf_compute_worker_count()==4,"four-worker startup");
                uint64_t opportunities=(native_wf || frontier_form)?r.forks:leaf_form?r.nodes-r.leaves:3*r.nodes-r.leaves+2;
                if(refusal_form) {
                    /* Descendants entered through a sequential clone attempt no
                     * acquisition. Full refusal follows only the right spine,
                     * counted independently by the explicit-stack oracle. */
                    if(exhaust)opportunities=r.refusal_forks;
                    else {
                        require(publishes+refusals>=r.refusal_forks && publishes+refusals<=opportunities,
                            "refusal subtree opportunity bounds");
                        if(!refusals)require(publishes==opportunities,"unrefused subtree opportunities");
                    }
                }
                if(!refusal_form || exhaust)
                    require(publishes+refusals==opportunities,"recursive publication opportunities");
                if(exhaust)require(!publishes && refusals==opportunities,"full owner-pool fallback");
            } else require(publishes==0,"sequential publication exclusion");
#endif
            printf("%s\t%s\t%u\t%s\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%ld\t%ld\t%lu\t%u\t%lu\t%lu\t%lu\t%lu\t%lu\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\n",
                p->name,form,call,call?"warm":"first",elapsed,
                cpu_us(after.ru_utime)-cpu_us(before.ru_utime),cpu_us(after.ru_stime)-cpu_us(before.ru_stime),
                after.ru_nvcsw-before.ru_nvcsw,after.ru_nivcsw-before.ru_nivcsw,steals,wf_compute_worker_count(),
                publishes,pops,runs,joins,refusals,observed.nodes,observed.forks,observed.migrated);
        }
    }
#if WF_COMPUTE_STATS
    if(parallel_form && requested==4 && !exhaust)require(total_steals>0,"parallel actualization");
    if((native_kind==7 || native_kind==8) && requested==4 && spawn_depth)
        require(total_migrated>0,"Rayon branch migration witness");
    if(!parallel_form && !native_wf)require(total_steals==0,"sequential task exclusion");
#else
    if((parallel_form || (native_wf && spawn_depth)) && requested==4)require(wf_compute_worker_count()==4,"four-worker startup");
#endif
    printf("# quadrature PASS: outputs=%" PRIu64 " stats=%d steals=%" PRIu64 " migrated=%" PRIu64 "\n",outputs,WF_COMPUTE_STATS,total_steals,total_migrated);
    for(unsigned i=0;i<reserved;++i)wf__par_release(held[i]);
    free(held);
    quadrature_native_stop();
    require(!fflush(stdout),"report flush");return 0;
}
int main(int argc,char **argv) { return wf__floor_run(argc,argv); }
