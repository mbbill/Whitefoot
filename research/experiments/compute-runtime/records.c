#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>
#include <time.h>
#include <sys/resource.h>
#include <errno.h>
#include "records_native.h"
#ifdef WF_COMPUTE_CONTROL
#include "runtime.h"
#endif
extern uint64_t wf_research_record_summary(const uint8_t *, uint64_t, uint64_t, uint64_t);
extern int wf__floor_run(int, char **);
static uint64_t calls;
static void check(const uint8_t *s, size_t n, uint64_t known) {
    uint8_t held[272];
    if (n > 256) abort();
    memset(held, 0xff, sizeof(held));
    memcpy(held + 7, s, n);
    uint64_t expected = records_reference(s, n);
    if (known != UINT64_MAX - 1 && expected != known) abort();
    uint64_t actual = wf_research_record_summary(held, sizeof(held), 7, n + 7);
    if (records_state(s, n) != expected || records_word(s, n) != expected) abort();
    ++calls;
    if (expected != actual) {
        fprintf(stderr, "UTF8 mismatch call=%" PRIu64 " n=%zu expected=%" PRIu64 " actual=%" PRIu64 "\n", calls, n, expected, actual);
        exit(1);
    }
    for (size_t i = 0; i < sizeof(held); ++i)
        if (held[i] != (i >= 7 && i < n + 7 ? s[i - 7] : 0xff)) abort();
}
static size_t encode(uint32_t c, uint8_t *s) {
    if (c < 0x80) { s[0] = (uint8_t)c; return 1; }
    if (c < 0x800) { s[0] = (uint8_t)(0xc0 | c >> 6); s[1] = (uint8_t)(0x80 | (c & 63)); return 2; }
    if (c < 0x10000) { s[0] = (uint8_t)(0xe0 | c >> 12); s[1] = (uint8_t)(0x80 | ((c >> 6) & 63)); s[2] = (uint8_t)(0x80 | (c & 63)); return 3; }
    s[0] = (uint8_t)(0xf0 | c >> 18); s[1] = (uint8_t)(0x80 | ((c >> 12) & 63));
    s[2] = (uint8_t)(0x80 | ((c >> 6) & 63)); s[3] = (uint8_t)(0x80 | (c & 63)); return 4;
}
static int check_leaf(int argc, char **argv) {
    (void)argc; (void)argv;
    alarm(120);
    uint8_t s[256] = {0};
    check(s, 0, 0);
    for (unsigned a = 0; a < 256; ++a) {
        s[0] = (uint8_t)a;
        check(s, 1, UINT64_MAX - 1);
        for (unsigned b = 0; b < 256; ++b) {
            s[1] = (uint8_t)b;
            check(s, 2, UINT64_MAX - 1);
            s[2] = 0x80; check(s, 3, UINT64_MAX - 1);
            s[2] = 0xbf; s[3] = 0x80; check(s, 4, UINT64_MAX - 1);
        }
    }
    for (uint32_t c = 0; c <= 0x10ffff; ++c) {
        size_t n = encode(c, s);
        check(s, n, c >= 0xd800 && c <= 0xdfff ? UINT64_MAX : 1);
        for (size_t cut = 1; cut < n; ++cut) check(s, cut, UINT64_MAX);
    }
    uint32_t rng = 0x19e18ab3;
    for (unsigned j = 0; j < 10000; ++j) {
        for (size_t i = 0; i < sizeof(s); ++i) { rng ^= rng << 13; rng ^= rng >> 17; rng ^= rng << 5; s[i] = (uint8_t)rng; }
        check(s, j % 257, UINT64_MAX - 1);
    }
    memset(s, 0, sizeof(s)); check(s, sizeof(s), sizeof(s));
    for (size_t i = 0; i < sizeof(s); i += 4) { s[i] = 0xf4; s[i+1] = 0x8f; s[i+2] = 0xbf; s[i+3] = 0xbf; }
    check(s, sizeof(s), sizeof(s) / 4);
    if (calls != 4595603) abort();
    printf("UTF8 leaf qualification PASS: calls=%" PRIu64 " all-codepoints all-truncations two-byte-pairs\n", calls);
    alarm(0);
    return 0;
}
extern void wf_research_records_parallel(const uint8_t *, uint64_t, const uint64_t *, uint64_t,
                                        uint64_t, uint64_t, uint64_t **, uint64_t *);
extern void wf_research_records_sequential(const uint8_t *, uint64_t, const uint64_t *, uint64_t,
                                          uint64_t, uint64_t, uint64_t **, uint64_t *);
extern uint64_t wf_research_records_release(uint64_t *, uint64_t);
extern int wf__par_pool_active(void);
#ifdef WF_COMPUTE_CONTROL
#include "runtime.h"
#endif
typedef void (*Batch)(const uint8_t *, uint64_t, const uint64_t *, uint64_t, uint64_t, uint64_t,
                      uint64_t **, uint64_t *);
static uint64_t batch_calls, results;
static Batch batch;
static void check_batch(const uint8_t *input, size_t n, const uint64_t *offsets, size_t offset_count,
                        size_t first, size_t end) {
    uint8_t *copy = malloc(n ? n : 1);
    uint64_t *copy_offsets = malloc(offset_count * sizeof(uint64_t));
    if (!copy || !copy_offsets) abort();
    memcpy(copy, input, n); memcpy(copy_offsets, offsets, offset_count * sizeof(uint64_t));
    uint64_t *output = NULL, output_count = UINT64_MAX;
    batch(input, n, offsets, offset_count, first, end, &output, &output_count);
    if (output_count != end-first || (!output && output_count)) abort();
    for (size_t i = first; i < end; ++i) {
        uint64_t lo = offsets[i], hi = offsets[i+1];
        uint64_t expected = lo > hi || hi > n ? UINT64_MAX-1 : records_reference(input+lo, hi-lo);
        if (lo <= hi && hi <= n) {
            if (records_state(input+lo, hi-lo) != expected || records_word(input+lo, hi-lo) != expected) abort();
        }
        if (output[i-first] != expected) {
            fprintf(stderr, "batch mismatch call=%" PRIu64 " record=%zu expected=%" PRIu64 " actual=%" PRIu64 "\n",batch_calls,i,expected,output[i-first]);
            exit(1);
        }
        ++results;
    }
    if (memcmp(copy, input, n) || memcmp(copy_offsets, offsets, offset_count*sizeof(uint64_t))) abort();
    if (wf_research_records_release(output, output_count)) abort();
    ++batch_calls;
    free(copy); free(copy_offsets);
}
static int check_batches(int argc, char **argv) {
    (void)argc; (void)argv;
    alarm(120);
    int parallel = wf__par_pool_active() != 0;
    batch = parallel ? wf_research_records_parallel : wf_research_records_sequential;
    size_t cap = 2*1024*1024;
    uint8_t *data = malloc(cap);
    uint64_t *offsets = malloc((65536+1)*sizeof(uint64_t));
    if (!data || !offsets) abort();
    uint32_t random = 193827;
    const size_t counts[] = {0,1,2,3,7,15,16,17,31,32,33,257,4097,65536};
    for (unsigned form = 0; form < 6; ++form) {
        for (size_t ci = 0; ci < sizeof(counts)/sizeof(counts[0]); ++ci) {
            size_t count = counts[ci], n = 0;
            offsets[0] = 0;
            for (size_t j = 0; j < count; ++j) {
                random ^= random << 13; random ^= random >> 17; random ^= random << 5;
                size_t length = form==0 ? 0 : form==1 ? 1 : random%24;
                if (form==4 && j%4096==0) length=32768;
                if (n+length>cap) abort();
                for (size_t i = 0; i < length; ++i) data[n+i] = (uint8_t)(33+((j+i)%94));
                if (form==2) for (size_t i=0; i+3<length; i+=4) encode(0x10000+(j%0xfffff), data+n+i);
                if (form==3 && length) data[n+length/2]=0xff;
                n+=length; offsets[j+1]=n;
            }
            for (unsigned pass=0; pass<3; ++pass) check_batch(data,n,offsets,count+1,0,count);
            if (count>2) check_batch(data,n,offsets,count+1,1,count-1);
            if (form==5 && count>2) {
                offsets[count/2]=UINT64_MAX;
                check_batch(data,n,offsets,count+1,0,count);
                offsets[count/2]=0;
                check_batch(data,n,offsets,count+1,0,count);
            }
        }
    }
    /* Adjacent truncated records must not combine into one valid sequence. */
    data[0]=0xe2; data[1]=0x82; data[2]=0xac;
    offsets[0]=0; offsets[1]=1; offsets[2]=3;
    check_batch(data,3,offsets,3,0,2);
    offsets[1]=3; offsets[2]=3;
    check_batch(data,3,offsets,3,0,2);
#ifdef WF_COMPUTE_CONTROL
    printf("records control: actual=%u steals=%lu\n",wf_compute_worker_count(),wf__par_grants());
    if (parallel && wf_compute_worker_count()!=4) abort();
#endif
    if (batch_calls != 342 || results != 1821070) abort();
    printf("UTF8 batch qualification PASS: world=%s calls=%" PRIu64 " results=%" PRIu64 "\n", parallel?"parallel":"sequential",batch_calls,results);
    free(data); free(offsets); alarm(0); return 0;
}

#ifndef RECORD_RUNTIME
#define RECORD_RUNTIME "weak"
#endif
static uint64_t entry_start;
static uint64_t entry_duration;
static void require(int condition, const char *message) {
    if (!condition) { fprintf(stderr, "UTF8 records: %s\n", message); exit(1); }
}
static uint64_t number(const char *s) {
    require(s && *s, "missing number");
    for (const char *p=s; *p; ++p) require(*p>='0' && *p<='9', "invalid number");
    char *end;
    errno=0;
    unsigned long long v=strtoull(s,&end,10);
    require(!errno && !*end, "number overflow");
    return (uint64_t)v;
}
static uint64_t now(void) {
    struct timespec t;
    require(clock_gettime(CLOCK_MONOTONIC_RAW,&t)==0, "clock");
    return (uint64_t)t.tv_sec*UINT64_C(1000000000)+(uint64_t)t.tv_nsec;
}
static uint64_t cpu_us(struct timeval v) { return (uint64_t)v.tv_sec*1000000+(uint64_t)v.tv_usec; }
static int benchmark(int argc,char **argv) {
    require(argc==8, "usage: records wf|state|word RECORDS MAX_LENGTH SHAPE REPS SEED PASS");
    const char *kernel=argv[1], *shape=argv[4];
    size_t count=number(argv[2]), limit=number(argv[3]);
    uint64_t reps=number(argv[5]), seed=number(argv[6]), pass=number(argv[7]);
    uint64_t requested=getenv("WF_WORKERS") ? number(getenv("WF_WORKERS")) : 0;
    require(requested<=4 && count<=1048576 && limit<=1048576 && reps>=1 && reps<=256 && seed<=UINT32_MAX,
            "benchmark domain");
    unsigned workers=(unsigned)requested;
    require(count==0 || limit<=16777216/count, "input byte bound");
    RecordKernel native=NULL;
    if (!strcmp(kernel,"state")) native=records_state;
    else if (!strcmp(kernel,"word")) native=records_word;
    else require(!strcmp(kernel,"wf"), "unknown kernel");
    require(!native || workers==0, "native kernel requires one caller");
    int kind=!strcmp(shape,"ascii")?0:!strcmp(shape,"unicode")?1:!strcmp(shape,"error-first")?2:
             !strcmp(shape,"error-last")?3:!strcmp(shape,"skew")?4:-1;
    require(kind>=0, "unknown shape");
    int parallel=!native && wf__par_pool_active()!=0;
    batch=parallel?wf_research_records_parallel:wf_research_records_sequential;
    size_t capacity=count*limit;
    uint8_t *data=malloc(capacity?capacity:1), *copy=malloc(capacity?capacity:1);
    uint64_t *offsets=malloc((count+1)*sizeof(uint64_t)), *saved=malloc((count+1)*sizeof(uint64_t));
    uint64_t *expected=malloc((count?count:1)*sizeof(uint64_t)), *sink=malloc((count?count:1)*sizeof(uint64_t));
    require(data && copy && offsets && saved && expected && sink, "input allocation");
    uint32_t random=(uint32_t)seed;
    size_t n=0;
    offsets[0]=0;
    for (size_t j=0;j<count;++j) {
        random^=random<<13; random^=random>>17; random^=random<<5;
        size_t length=limit?random%(limit+1):0;
        if (kind==4) length=j%1024==0?limit:(limit?1:0);
        for (size_t i=0;i<length;++i) data[n+i]=(uint8_t)(32+((i+j)%95));
        if (kind==1) for (size_t i=0;i+3<length;i+=4) encode(0x10000+(j%0xfffff),data+n+i);
        if (kind==2 && length) data[n]=0xff;
        if (kind==3 && length) data[n+length-1]=0xff;
        expected[j]=records_reference(data+n,length);
        n+=length; offsets[j+1]=n;
    }
    memcpy(copy,data,n); memcpy(saved,offsets,(count+1)*sizeof(uint64_t));
    uint64_t floor=UINT64_MAX;
    for (unsigned i=0;i<1000;++i) { uint64_t t=now(); uint64_t d=now()-t; if(d<floor) floor=d; }
    printf("# runtime=%s kernel=%s workers=%u world=%s shape=%s records=%zu bytes=%zu max_length=%zu seed=%" PRIu64 " pass=%" PRIu64 " reps=%" PRIu64 " entry_ns=%" PRIu64 " clock_pair_min_ns=%" PRIu64 "\n",
           RECORD_RUNTIME,kernel,workers,parallel?"parallel":"sequential",shape,count,n,limit,seed,pass,reps,entry_duration,floor);
    puts("runtime\tkernel\tworkers\tshape\trecords\tbytes\tmax_length\tseed\tpass\tcall\tphase\tcore_ns\tcycle_ns");
    struct rusage before,after;
    require(getrusage(RUSAGE_SELF,&before)==0,"initial resource read");
    uint64_t begin=now();
    for (uint64_t call=0;call<=reps;++call) {
        uint64_t output_count=count, *output=NULL;
        uint64_t start=now();
        if (native) {
            output=malloc((count?count:1)*sizeof(uint64_t));
            require(output!=NULL,"native output allocation");
            for(size_t j=0;j<count;++j) output[j]=native(data+offsets[j],offsets[j+1]-offsets[j]);
        } else batch(data,n,offsets,count+1,0,count,&output,&output_count);
        uint64_t core_end=now();
        require((output || !count) && output_count==count,"timed result shape");
        if (count) memcpy(sink,output,count*sizeof(uint64_t));
        if(native) free(output);
        else require(wf_research_records_release(output,output_count)==0,"timed release");
        uint64_t end=now();
        require(memcmp(sink,expected,count*sizeof(uint64_t))==0,"timed full results");
        require(!memcmp(data,copy,n) && !memcmp(offsets,saved,(count+1)*sizeof(uint64_t)),"timed input immutability");
        printf("%s\t%s\t%u\t%s\t%zu\t%zu\t%zu\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%s\t%" PRIu64 "\t%" PRIu64 "\n",
               RECORD_RUNTIME,kernel,workers,shape,count,n,limit,seed,pass,call,call?"warm":"first",core_end-start,end-start);
    }
    uint64_t duration=now()-begin;
    require(getrusage(RUSAGE_SELF,&after)==0,"final resource read");
    uint64_t rss=(uint64_t)after.ru_maxrss;
#ifndef __APPLE__
    rss*=1024;
#endif
    printf("# batch_includes_checks=1 batch_ns=%" PRIu64 " user_us=%" PRIu64 " system_us=%" PRIu64 " maxrss_bytes=%" PRIu64 " voluntary_switches=%ld involuntary_switches=%ld\n",
           duration,cpu_us(after.ru_utime)-cpu_us(before.ru_utime),cpu_us(after.ru_stime)-cpu_us(before.ru_stime),rss,
           after.ru_nvcsw-before.ru_nvcsw,after.ru_nivcsw-before.ru_nivcsw);
#ifdef WF_COMPUTE_CONTROL
    unsigned actual=wf_compute_worker_count();
    printf("# actual_lanes=%u steals=%lu\n",actual,wf__par_grants());
    require(actual==0 || (!native && workers>=2 && actual==workers),"partial worker capacity");
#endif
    printf("# UTF8 records PASS: calls=%" PRIu64 " results=%" PRIu64 "\n",reps+1,(reps+1)*count);
    free(data);free(copy);free(offsets);free(saved);free(expected);free(sink);
    return 0;
}
int wf__main_body(int argc,char **argv) {
    entry_duration=now()-entry_start;
    if(argc==2 && !strcmp(argv[1],"check")) {
        require(check_leaf(argc,argv)==0,"leaf qualification");
        return check_batches(argc,argv);
    }
    return benchmark(argc,argv);
}
int main(int argc,char **argv) {
    entry_start=now();
    return wf__floor_run(argc,argv);
}
