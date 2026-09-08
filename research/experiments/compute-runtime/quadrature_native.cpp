// Recursive scalar controls for quadrature; retire with the owning experiment.
#include "quadrature_native.h"
#include "quadrature-rayon-binding.h"
extern "C" double quadrature_rayon(unsigned, unsigned, unsigned,
    double, double, double, double, double, unsigned,
    void (*)(uint64_t, uint64_t, uint64_t, uint64_t, uint64_t, uint64_t, uint64_t)) __asm__(QUADRATURE_RAYON_SYMBOL);
extern "C" {
#include "runtime.h"
}
#include <cstddef>
#include <cmath>
#include <cstring>
#include <cstdio>
#include <cstdlib>
#include <thread>
#include <type_traits>
#include <oneapi/tbb/global_control.h>
#include <oneapi/tbb/parallel_invoke.h>
#include <oneapi/tbb/task_arena.h>
#include <oneapi/tbb/version.h>
#if defined(PARLAY_CILKPLUS) || defined(PARLAY_OPENCILK) || defined(PARLAY_OPENMP) || \
    defined(PARLAY_TBB) || defined(PARLAY_SEQUENTIAL)
#error "quadrature requires the native Parlay scheduler"
#endif
#include <parlay/parallel.h>
#ifndef PARLAY_USING_PARLAY_SCHEDULER
#error "quadrature did not select the native Parlay scheduler"
#endif
static_assert(TBB_VERSION_MAJOR == 2023 && TBB_VERSION_MINOR == 1 && TBB_VERSION_PATCH == 0);
static_assert(PARLAY_ELASTIC_PARALLELISM && PARLAY_ELASTIC_STEAL_TIMEOUT == 10000);
#ifndef WF_COMPUTE_STATS
#error "explicit instrumentation selection required"
#endif
namespace {
struct Result {
    double value;
#if WF_COMPUTE_STATS
    uint64_t nodes = 1, forks = 0, migrated = 0;
#endif
};
QuadratureObservation observation{};
void observe_rayon(uint64_t nodes, uint64_t forks, uint64_t migrated,
    uint64_t w0, uint64_t w1, uint64_t w2, uint64_t w3) {
    observation={nodes,forks,migrated,{w0,w1,w2,w3}};
}
using Pool = parlay::internal::scheduler_type;
Pool *parlay_pool = nullptr;
unsigned selected_width = 0;
[[noreturn]] void fail(const char *why) {
    std::fprintf(stderr, "quadrature native: %s\n", why); std::abort();
}
double density(double x, double c, double w) {
    double z = (x-c)/w; return 1/(1+z*z);
}
double simpson(double a,double b,double fa,double fm,double fb) {
    return ((b-a)/6)*((fa+4*fm)+fb);
}
enum class Kind { Serial, Tbb, Parlay, WfBorrow, WfValue, ParlayLeft, WfValueRight };
struct ValueFrame {
    double a,b,c,w,fa,fm,fb,whole,tolerance;
    unsigned depth,budget;
    Result result;
#if WF_COMPUTE_STATS
    std::thread::id owner;
#endif
};
static_assert(std::is_trivially_copyable_v<ValueFrame>);
#if !WF_COMPUTE_STATS
static_assert(sizeof(ValueFrame)==88, "scalar by-value frame size");
#endif
template<Kind kind>
Result adaptive(double a,double b,double c,double w,double fa,double fm,double fb,
                double whole,double tolerance,unsigned depth,unsigned budget) {
    // Below the selected frontier use a specialization with no scheduler path.
    if constexpr (kind != Kind::Serial) {
        if (!budget) return adaptive<Kind::Serial>(a,b,c,w,fa,fm,fb,whole,tolerance,depth,0);
    }
    double m=(a+b)*0.5,fl=density((a+m)*0.5,c,w),fr=density((m+b)*0.5,c,w);
    double left=simpson(a,m,fa,fl,fm),right=simpson(m,b,fm,fr,fb);
    double combined=left+right,delta=combined-whole;
    if (!depth || std::fabs(delta)<=15*tolerance) return Result{combined+delta/15};
    Result l{},r{};
    auto run_left=[&] { l=adaptive<kind>(a,m,c,w,fa,fl,fm,left,tolerance*0.5,depth-1,
                                       kind==Kind::Serial?0:budget-1); };
    auto run_right=[&] { r=adaptive<kind>(m,b,c,w,fm,fr,fb,right,tolerance*0.5,depth-1,
                                        kind==Kind::Serial?0:budget-1); };
#if WF_COMPUTE_STATS
    uint64_t moved_left=0,moved_right=0;
#endif
    if constexpr (kind == Kind::Serial) { run_left(); run_right(); }
    else {
#if WF_COMPUTE_STATS
        const auto owner=std::this_thread::get_id();
        auto l_task=[&] { moved_left=std::this_thread::get_id()!=owner; run_left(); };
        auto r_task=[&] { moved_right=std::this_thread::get_id()!=owner; run_right(); };
#else
        auto &l_task=run_left;
        auto &r_task=run_right;
#endif
        if constexpr (kind == Kind::Tbb) oneapi::tbb::parallel_invoke(l_task,r_task);
        else if constexpr (kind == Kind::Parlay) parlay::par_do(l_task,r_task);
        else if constexpr (kind == Kind::ParlayLeft) parlay::par_do(r_task,l_task);
        else if constexpr (kind == Kind::WfBorrow) {
            // The parent's closure and result outlive the joined task. Only a
            // pointer crosses the runtime frame; this is a native control, not
            // a new borrowing rule for generated WF.
            auto *fn=&l_task;
            void *task=wf__par_acquire_lane(sizeof(fn));
            if (task) {
                std::memcpy(task,&fn,sizeof(fn));
                wf__par_publish(task,[](void *opaque) {
                    decltype(fn) callable;
                    std::memcpy(&callable,opaque,sizeof(callable)); (*callable)();
                });
                r_task(); wf__par_join(task); wf__par_release(task);
            } else { l_task(); r_task(); }
        } else {
            // Match the emitted scalar payload size while retaining the same
            // C++ kernel/grain. Raw runtime bytes are accessed through memcpy;
            // no C++ object lifetime is assumed in the C-owned storage.
            void *task=wf__par_acquire_lane(sizeof(ValueFrame));
            if (task) {
                constexpr bool send_right=kind==Kind::WfValueRight;
                ValueFrame frame{send_right?m:a,send_right?b:m,c,w,
                    send_right?fm:fa,send_right?fr:fl,send_right?fb:fm,
                    send_right?right:left,tolerance*0.5,depth-1,budget-1,{}
#if WF_COMPUTE_STATS
                    ,owner
#endif
                };
                std::memcpy(task,&frame,sizeof(frame));
                wf__par_publish(task,[](void *opaque) {
                    ValueFrame copy;
                    std::memcpy(&copy,opaque,sizeof(copy));
                    Result value=adaptive<kind>(copy.a,copy.b,copy.c,copy.w,
                        copy.fa,copy.fm,copy.fb,copy.whole,copy.tolerance,copy.depth,copy.budget);
#if WF_COMPUTE_STATS
                    value.migrated+=std::this_thread::get_id()!=copy.owner;
#endif
                    std::memcpy(static_cast<unsigned char *>(opaque)+offsetof(ValueFrame,result),&value,sizeof(value));
                });
                if constexpr (send_right) l_task(); else r_task();
                wf__par_join(task);
                std::memcpy(send_right?&r:&l,static_cast<unsigned char *>(task)+offsetof(ValueFrame,result),sizeof(l));
                wf__par_release(task);
            } else { l_task(); r_task(); }
        }
    }
    Result result{l.value+r.value};
#if WF_COMPUTE_STATS
    result.nodes+=l.nodes+r.nodes;
    result.forks=l.forks+r.forks+(kind!=Kind::Serial);
    result.migrated=l.migrated+r.migrated+moved_left+moved_right;
#endif
    return result;
}
template<Kind kind>
Result integrate(double a,double b,double c,double w,double tolerance,unsigned depth,unsigned budget) {
    double fa=density(a,c,w),fm=density((a+b)*0.5,c,w),fb=density(b,c,w);
    double whole=simpson(a,b,fa,fm,fb);
    return adaptive<kind>(a,b,c,w,fa,fm,fb,whole,tolerance,depth,budget);
}
struct TbbPool {
    oneapi::tbb::global_control control;
    oneapi::tbb::task_arena arena;
    explicit TbbPool(unsigned width)
        : control(oneapi::tbb::global_control::max_allowed_parallelism,width),
          arena(static_cast<int>(width),1) { arena.initialize(); }
};
}
extern "C" double quadrature_native_run(unsigned kind,unsigned workers,unsigned budget,
    double a,double b,double c,double w,double tolerance,unsigned depth) {
    if (kind>9 || (workers!=1 && workers!=4) || budget>24 || depth>24) fail("arguments");
    if (selected_width && selected_width!=workers) fail("worker width changed");
    if (!selected_width && (kind==3 || kind==4 || kind==6)) {
        const char *configured=std::getenv("WF_WORKERS");
        if (!configured || std::strcmp(configured,workers==1?"1":"4")) fail("WF worker budget");
    }
    selected_width=workers;
    if (kind>=7) return quadrature_rayon(kind==9?0:kind-6,workers,budget,
        a,b,c,w,tolerance,depth,observe_rayon);
    Result result{};
    try {
        if (!kind) result=integrate<Kind::Serial>(a,b,c,w,tolerance,depth,0);
        else if (kind==1) {
            static TbbPool pool(workers);
            result=pool.arena.execute([&] { return integrate<Kind::Tbb>(a,b,c,w,tolerance,depth,budget); });
        } else if (kind==2 || kind==5) {
            if (!parlay_pool) {
                if (Pool::get_current_scheduler()) fail("existing Parlay scheduler");
                parlay_pool=new Pool(workers);
            }
            if (Pool::get_current_scheduler()!=parlay_pool) fail("Parlay owner changed");
            if (kind==2) result=integrate<Kind::Parlay>(a,b,c,w,tolerance,depth,budget);
            else result=integrate<Kind::ParlayLeft>(a,b,c,w,tolerance,depth,budget);
        } else {
            if (kind==3) result=integrate<Kind::WfBorrow>(a,b,c,w,tolerance,depth,budget);
            else if (kind==4) result=integrate<Kind::WfValue>(a,b,c,w,tolerance,depth,budget);
            else result=integrate<Kind::WfValueRight>(a,b,c,w,tolerance,depth,budget);
        }
    } catch (...) { fail("scheduler exception"); }
#if WF_COMPUTE_STATS
    observation={result.nodes,result.forks,result.migrated,{}};
#endif
    return result.value;
}
extern "C" QuadratureObservation quadrature_native_observation(void) { return observation; }
extern "C" void quadrature_native_stop(void) {
    // Joined runs precede destruction; Parlay joins helpers and restores owner.
    delete parlay_pool; parlay_pool=nullptr;
}
