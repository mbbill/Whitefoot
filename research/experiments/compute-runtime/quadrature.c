#define _GNU_SOURCE
#define _DARWIN_C_SOURCE
/* Adaptive Lorentz-profile integration: correctness and initial generated-code
 * attribution. Retire with the owning quadrature experiment. No SIMD or FMA. */
#include "runtime.h"
#include "runtime_events.h"
#include <errno.h>
#include <inttypes.h>
#include <math.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <time.h>

extern int wf__floor_run(int, char **);
extern double wf_research_quadrature(double, double, double, double, double, uint64_t, bool);
extern double wf_research_quadrature_leaf(double, double, double, double, double, uint64_t, bool);
static double (*generated_run)(double,double,double,double,double,uint64_t,bool);
static bool parallel_form, leaf_form;
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
typedef struct { double a,b,fa,fm,fb,whole,tol,m,fl,fr,left,right,result; unsigned depth,state; } Node;
typedef struct { double value; uint64_t nodes,leaves,capped; unsigned deepest; } Reference;
static Reference reference(const Input *p) {
    Node stack[25]={0}; unsigned top=0; Reference out={0};
    volatile double endpoints=p->a+p->b,m=endpoints*0.5;
    double fa=oracle_density(p->a,p),fm=oracle_density(m,p),fb=oracle_density(p->b,p);
    stack[0]=(Node){.a=p->a,.b=p->b,.fa=fa,.fm=fm,.fb=fb,
        .whole=oracle_simpson(p->a,p->b,fa,fm,fb),.tol=p->tolerance,.depth=p->depth};
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
                    .whole=parent->right,.tol=tol,.depth=parent->depth-1};
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
    return generated_run(p->a,p->b,p->center,p->width,p->tolerance,p->depth,parallel_form);
}
int wf__main_body(int argc,char **argv) {
    require(argc==3,"usage: quadrature check|bench native|wf-seq|wf-auto|wf-leaf-seq|wf-leaf");
    bool bench=!strcmp(argv[1],"bench");require(bench || !strcmp(argv[1],"check"),"mode");
    const char *form=argv[2];
    leaf_form=!strcmp(form,"wf-leaf") || !strcmp(form,"wf-leaf-seq");
    parallel_form=!strcmp(form,"wf-auto") || !strcmp(form,"wf-leaf");
    require(!strcmp(form,"native") || !strcmp(form,"wf-seq") || parallel_form || leaf_form,"form");
    generated_run=leaf_form?wf_research_quadrature_leaf:wf_research_quadrature;
    const char *workers=getenv("WF_WORKERS");
    require(workers && (!strcmp(workers,"1") || !strcmp(workers,"4")),"explicit worker count");
    unsigned requested=!strcmp(workers,"4")?4:1;
    unsigned calls=bench?9:2;uint64_t outputs=0,total_steals=0;
    printf("# quadrature mode=%s form=%s workers=%u stats=%d events=%d\n",argv[1],form,requested,WF_COMPUTE_STATS,
#if defined(WF_COMPUTE_EVENTS)
        1
#else
        0
#endif
    );
    puts("# columns=input form call phase wall_ns user_us system_us voluntary involuntary steals pool_lanes publishes local_pops runs joins slot_refusals");
    for(size_t i=0;i<sizeof(cases)/sizeof(cases[0]);++i) {
        const Input *p=&cases[i];Reference r=reference(p);
        if(p->converged) {
            long double exact=(long double)p->width*(atanl(((long double)p->b-p->center)/p->width)-atanl(((long double)p->a-p->center)/p->width));
            require(r.capped==0,"unexpected depth exhaustion");
            require(fabsl((long double)r.value-exact)<=8*p->tolerance+0x1p-48L,"analytic integral");
        }
        printf("# input=%s nodes=%" PRIu64 " leaves=%" PRIu64 " capped=%" PRIu64 " deepest=%u evaluations=%" PRIu64 " expected=%a\n",p->name,r.nodes,r.leaves,r.capped,r.deepest,3+2*r.nodes,r.value);
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
            unsigned long publishes=0,pops=0,runs=0,joins=0,refusals=0;
#if defined(WF_COMPUTE_EVENTS)
            for(unsigned e=0;e<WF_EVENT_COUNT;++e)events[e]=event_sum(e)-events[e];
            publishes=events[WF_EVENT_PUBLISH];pops=events[WF_EVENT_LOCAL_POP];
            runs=events[WF_EVENT_RUN_END];joins=events[WF_EVENT_JOIN];refusals=events[WF_EVENT_SLOT_REFUSAL];
            require(publishes==pops+events[WF_EVENT_STEAL_SUCCESS] && publishes==runs &&
                    publishes==events[WF_EVENT_RUN_BEGIN] && publishes==joins,"joined task conservation");
            if(parallel_form && requested==4) {
                require(wf_compute_worker_count()==4,"four-worker startup");
                uint64_t opportunities=leaf_form?r.nodes-r.leaves:3*r.nodes-r.leaves+2;
                require(publishes+refusals==opportunities,"recursive publication opportunities");
            } else require(publishes==0,"sequential publication exclusion");
#endif
            printf("%s\t%s\t%u\t%s\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%ld\t%ld\t%lu\t%u\t%lu\t%lu\t%lu\t%lu\t%lu\n",
                p->name,form,call,call?"warm":"first",elapsed,
                cpu_us(after.ru_utime)-cpu_us(before.ru_utime),cpu_us(after.ru_stime)-cpu_us(before.ru_stime),
                after.ru_nvcsw-before.ru_nvcsw,after.ru_nivcsw-before.ru_nivcsw,steals,wf_compute_worker_count(),
                publishes,pops,runs,joins,refusals);
        }
    }
#if WF_COMPUTE_STATS
    if(parallel_form && requested==4)require(total_steals>0,"parallel actualization");
    if(!parallel_form)require(total_steals==0,"sequential task exclusion");
#else
    if(parallel_form && requested==4)require(wf_compute_worker_count()==4,"four-worker startup");
#endif
    printf("# quadrature PASS: outputs=%" PRIu64 " stats=%d steals=%" PRIu64 "\n",outputs,WF_COMPUTE_STATS,total_steals);
    require(!fflush(stdout),"report flush");return 0;
}
int main(int argc,char **argv) { return wf__floor_run(argc,argv); }
