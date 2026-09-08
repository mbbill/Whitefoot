/* Escape-time workload for compute scheduling. The host checks every output
 * against an independently expressed, explicitly rounded recurrence. */
#include "records_scheduler.h"
#include "runtime.h"
#include <errno.h>
#include <inttypes.h>
#include <math.h>
#include <stdbool.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/resource.h>
#include <time.h>
extern int wf__floor_run(int, char **);
extern uint64_t wf_research_escape(double, double, uint64_t);
extern void wf_research_mandelbrot(const double *, const double *, uint64_t, uint64_t,
                                   bool, uint64_t **, uint64_t *);
extern void wf_research_mandelbrot_release(uint64_t *, uint64_t);
extern void records_scheduler_select(const char *);
static const char *backend, *policy;
static unsigned width;
static bool generated, clear_native;
static void require(bool ok, const char *why) {
    if (!ok) { fprintf(stderr,"mandelbrot: %s\n",why); exit(1); }
}
static uint64_t number(const char *s) {
    require(s && *s,"missing number");
    for (const char *p=s; *p; ++p) require(*p>='0' && *p<='9',"invalid number");
    errno=0; char *end; uint64_t v=strtoull(s,&end,10);
    require(!errno && !*end,"integer overflow"); return v;
}
static uint64_t now(void) {
    struct timespec t; require(!clock_gettime(CLOCK_MONOTONIC_RAW,&t),"clock");
    return (uint64_t)t.tv_sec*UINT64_C(1000000000)+(uint64_t)t.tv_nsec;
}
static uint64_t cpu_us(struct timeval t) { return (uint64_t)t.tv_sec*1000000+(uint64_t)t.tv_usec; }
/* Volatile temporaries force each specified binary64 rounding in the oracle.
 * This is outside timing. Known orbits below anchor iteration numbering. */
static uint64_t reference(double real, double imaginary, uint64_t limit) {
    volatile double x=0, y=0;
    for (uint64_t i=0; i<limit; ++i) {
        volatile double xx=x*x, yy=y*y, magnitude=xx+yy;
        if (magnitude>4) return i;
        volatile double xy=x*y, twice=xy+xy, difference=xx-yy;
        y=twice+imaginary; x=difference+real;
    }
    return limit;
}
static uint64_t native(double real, double imaginary, uint64_t limit) {
    double x=0, y=0, xx=0, yy=0;
    for (uint64_t i=0; i<limit; ++i) {
        if (xx+yy>4) return i;
        double xy=x*y;
        y=(xy+xy)+imaginary; x=(xx-yy)+real;
        xx=x*x; yy=y*y;
    }
    return limit;
}
typedef struct { double *x,*y,*held_x,*held_y; uint64_t *expected,*output; size_t n,grain; uint64_t limit,iterations; bool poison; } Work;
static uint32_t random_next(uint32_t *s) { *s^=*s<<13; *s^=*s>>17; *s^=*s<<5; return *s; }
static Work input(size_t n, uint64_t limit, const char *shape, uint32_t seed, size_t grain) {
    require(n<=1048576 && limit<=65536 && grain>=1 && grain<=1048576,"input domain");
    Work w={0}; w.n=n;w.limit=limit;w.grain=grain;
    size_t bytes=(n?n:1)*sizeof(double);
    w.x=malloc(bytes);w.y=malloc(bytes);w.held_x=malloc(bytes);w.held_y=malloc(bytes);
    w.expected=malloc((n?n:1)*sizeof(uint64_t));
    require(w.x && w.y && w.held_x && w.held_y && w.expected,"input allocation");
    int kind=!strcmp(shape,"plane")?0:!strcmp(shape,"boundary")?1:!strcmp(shape,"clustered")?2:
             !strcmp(shape,"interleaved")?3:!strcmp(shape,"interior")?4:!strcmp(shape,"exterior")?5:
             !strcmp(shape,"trailing")?6:-1;
    require(kind>=0,"input shape");
    for(size_t i=0;i<n;++i) {
        if(kind<2) {
            double a=(double)(random_next(&seed)&65535)/65536;
            double b=(double)(random_next(&seed)&65535)/65536;
            w.x[i]=kind==0 ? -2+3*a : -0.75+(a-0.5)/32;
            w.y[i]=kind==0 ? -1.5+3*b : 0.125+(b-0.5)/32;
        } else {
            bool inside=kind==4 || (kind==2 && i<(n+3)/4) || (kind==3 && i%4==0) ||
                        (kind==6 && i>=n-(n+3)/4);
            w.x[i]=inside?0:3;w.y[i]=0;
        }
        w.expected[i]=reference(w.x[i],w.y[i],limit);w.iterations+=w.expected[i];
    }
    memcpy(w.held_x,w.x,n*sizeof(double));memcpy(w.held_y,w.y,n*sizeof(double));
    return w;
}
static void native_chunk(void *opaque,size_t chunk) {
    Work *w=opaque;size_t first=chunk*w->grain,end=first+w->grain;if(end>w->n)end=w->n;
    for(size_t i=first;i<end;++i)w->output[i]=native(w->x[i],w->y[i],w->limit);
}
static void run(Work *w) {
    if(generated) {
        uint64_t length=UINT64_MAX;
        wf_research_mandelbrot(w->x,w->y,w->n,w->limit,strcmp(backend,"wf-seq")!=0,&w->output,&length);
        require(length==w->n,"generated length");
    } else {
        w->output=malloc((w->n?w->n:1)*sizeof(uint64_t));require(w->output!=NULL,"output allocation");
        if(clear_native && w->poison)for(size_t i=0;i<w->n;++i)w->output[i]=UINT64_MAX;
        if(clear_native)memset(w->output,0,w->n*sizeof(uint64_t));
        if(clear_native && w->poison) {
            volatile const uint64_t *observed=w->output;
            for(size_t i=0;i<w->n;++i)require(observed[i]==0,"native output clearing");
        }
        if(w->poison)for(size_t i=0;i<w->n;++i)w->output[i]=UINT64_MAX;
        records_scheduler_run(width,(w->n+w->grain-1)/w->grain,native_chunk,w);
    }
}
static void verify(Work *w) {
    require(w->output || !w->n,"output pointer");
    for(size_t i=0;i<w->n;++i)if(w->output[i]!=w->expected[i]) {
        fprintf(stderr,"mandelbrot: point=%zu expected=%" PRIu64 " actual=%" PRIu64 "\n",i,w->expected[i],w->output[i]);exit(1);
    }
    require(!memcmp(w->x,w->held_x,w->n*sizeof(double)) && !memcmp(w->y,w->held_y,w->n*sizeof(double)),"input immutability");
}
static void release(Work *w) {
    if(generated)wf_research_mandelbrot_release(w->output,w->n);else free(w->output);
    w->output=NULL;
}
/* Check the research request outside timing, including spans too short for
 * its full depth. This does not claim that all terminal ranges run in parallel. */
static void check_requested_chunks(size_t n) {
    if (!wf_compute_requested_chunks) return;
    unsigned long depth=wf__par_split_budget(n,1);
    require(depth<64,"split depth domain");
    unsigned long chunks=1ul<<depth;
    size_t bound=n<wf_compute_requested_chunks?n:wf_compute_requested_chunks;
    require(width>1 && (bound<2 ? depth==0 : chunks<=bound && chunks>bound/2),"requested split budget");
}
static void destroy(Work *w) { free(w->x);free(w->y);free(w->held_x);free(w->held_y);free(w->expected); }
static int qualify(void) {
    const double known[][3]={{0,0,64},{-1,0,64},{-2,0,64},{0,1,64},{2,0,2},{3,0,1},{0,3,1}};
    uint64_t leaves=0,outputs=0,calls=0;
    for(size_t i=0;i<sizeof(known)/sizeof(known[0]);++i) {
        require(reference(known[i][0],known[i][1],64)==(uint64_t)known[i][2],"known orbit");
        uint64_t expected=known[i][2]==64 ? 65536 : (uint64_t)known[i][2];
        require(reference(known[i][0],known[i][1],65536)==expected &&
                native(known[i][0],known[i][1],65536)==expected &&
                wf_research_escape(known[i][0],known[i][1],65536)==expected,"maximum iteration orbit");
        ++leaves;
    }
    const double special[]={-INFINITY,-3,-2,-1,-0.0,0,1,2,3,INFINITY,NAN};
    const uint64_t limits[]={0,1,2,3,16,64};
    for(size_t i=0;i<11;++i)for(size_t j=0;j<11;++j)for(size_t k=0;k<6;++k) {
        uint64_t expected=reference(special[i],special[j],limits[k]);
        require(native(special[i],special[j],limits[k])==expected && wf_research_escape(special[i],special[j],limits[k])==expected,"special point");++leaves;
    }
    const char *shapes[]={"plane","boundary","clustered","interleaved","interior","exterior","trailing"};
    const size_t sizes[]={0,1,3,33,257,4097};
    for(size_t s=0;s<7;++s)for(size_t n=0;n<6;++n)for(size_t k=0;k<6;++k) {
        Work w=input(sizes[n],limits[k],shapes[s],828219,16);
        check_requested_chunks(w.n);
        w.poison=true;
        for(size_t i=0;i<w.n;++i) {
            require(native(w.x[i],w.y[i],w.limit)==w.expected[i] && wf_research_escape(w.x[i],w.y[i],w.limit)==w.expected[i],"leaf");++leaves;
        }
        run(&w);verify(&w);release(&w);++calls;outputs+=w.n;destroy(&w);
    }
    printf("# Mandelbrot qualification PASS: leaves=%" PRIu64 " calls=%" PRIu64 " outputs=%" PRIu64 "\n",leaves,calls,outputs);
    return 0;
}
static int body(int argc,char **argv) {
    if(argc==4 && !strcmp(argv[3],"check"))return qualify();
    require(argc==10,"usage: mandelbrot BACKEND WIDTH SHAPE POINTS LIMIT GRAIN REPS SEED PASS");
    const char *shape=argv[3];size_t n=number(argv[4]);uint64_t limit=number(argv[5]);size_t grain=number(argv[6]);
    uint64_t reps=number(argv[7]),seed=number(argv[8]),pass=number(argv[9]);
    require(reps>=1 && reps<=256 && seed<=UINT32_MAX,"measurement domain");
    require(generated ? grain==0 : grain>=1 && grain<=1048576,"native grain or automatic split");
    Work w=input(n,limit,shape,(uint32_t)seed,generated?1:grain);
    check_requested_chunks(n);
    uint64_t *sink=malloc((n?n:1)*sizeof(uint64_t));require(sink!=NULL,"result sink");
    printf("# backend=%s width=%u policy=%s shape=%s points=%zu limit=%" PRIu64 " grain=%zu reps=%" PRIu64 " seed=%" PRIu64 " pass=%" PRIu64 " iterations=%" PRIu64 "\n",backend,width,policy,shape,n,limit,grain,reps,seed,pass,w.iterations);
    puts("call\tphase\tcore_ns\tcycle_ns");
    uint64_t gaps[256],previous=0;
    struct rusage before,after;require(!getrusage(RUSAGE_SELF,&before),"resource start");uint64_t start=now();
    for(uint64_t call=0;call<=reps;++call) {
        uint64_t a=now();if(call)gaps[call-1]=a-previous;run(&w);uint64_t b=now();
        if(n)memcpy(sink,w.output,n*sizeof(uint64_t));
        release(&w);uint64_t c=now();previous=c;
        w.output=sink;verify(&w);w.output=NULL;
        printf("%" PRIu64 "\t%s\t%" PRIu64 "\t%" PRIu64 "\n",call,call?"warm":"first",b-a,c-a);
    }
    uint64_t finish=now();require(!getrusage(RUSAGE_SELF,&after),"resource end");
    uint64_t rss=(uint64_t)after.ru_maxrss;
#if !defined(__APPLE__)
    rss*=1024;
#endif
    printf("# batch_ns=%" PRIu64 " user_us=%" PRIu64 " system_us=%" PRIu64 " voluntary=%ld involuntary=%ld wf_pool_lanes=%u maxrss_bytes=%" PRIu64 "\n",finish-start,cpu_us(after.ru_utime)-cpu_us(before.ru_utime),cpu_us(after.ru_stime)-cpu_us(before.ru_stime),after.ru_nvcsw-before.ru_nvcsw,after.ru_nivcsw-before.ru_nivcsw,wf_compute_worker_count(),rss);
    for(uint64_t call=1;call<=reps;++call)printf("# gap call=%" PRIu64 " ns=%" PRIu64 "\n",call,gaps[call-1]);
    printf("# Mandelbrot PASS: calls=%" PRIu64 " outputs=%" PRIu64 "\n",reps+1,(reps+1)*n);free(sink);destroy(&w);return 0;
}
int wf__main_body(int argc,char **argv) {
    int status=body(argc,argv);
    if(!generated)(void)records_scheduler_stop();
    require(fflush(stdout)==0,"flush completed report");
    return status;
}
int main(int argc,char **argv) {
    require(argc>=3,"backend and width required");backend=argv[1];uint64_t requested=number(argv[2]);
    require(requested==1 || requested==2 || requested==4,"width domain");width=(unsigned)requested;
    require(getenv("WF_WORKERS") && number(getenv("WF_WORKERS"))==width,"matching worker setting");
    generated=!strcmp(backend,"wf-auto") || !strcmp(backend,"wf-seq");
    require(strcmp(backend,"wf-seq") || width==1,"sequential width");
    policy=getenv("WF_BUDGET_CONTROL");if(!policy)policy="cost";
    require(!strcmp(policy,"cost") || !strcmp(policy,"team") || !strcmp(policy,"capacity") ||
            !strcmp(policy,"chunks16") || !strcmp(policy,"chunks256") || !strcmp(policy,"zero"),"experiment policy");
    clear_native=!strcmp(policy,"zero");
    require(generated ? !clear_native : (!strcmp(policy,"cost") || clear_native),"policy applies to selected form");
    wf_compute_capacity_budget=!strcmp(policy,"team")?2:!strcmp(policy,"capacity");
    if(!strcmp(policy,"chunks16") || !strcmp(policy,"chunks256")) {
        require(generated && !strcmp(backend,"wf-auto") && width==4,"explicit chunk control");
        wf_compute_capacity_budget=1;
        wf_compute_requested_chunks=!strcmp(policy,"chunks16")?16:256;
    }
    if(!generated)records_scheduler_select(backend);
    return wf__floor_run(argc,argv);
}
