/* Experiment 58 client qualification. The trace executes netload's actual
 * exchange loop against a strict syscall sequence; the socket fixture runs
 * the separately built client against adversarial, byte-checked echo peers.
 * Owned by client-readiness-check; retire with that client policy control. */
#define _GNU_SOURCE
#include <sys/epoll.h>
#include <sys/socket.h>
#include <sys/types.h>
#include <time.h>
#include <signal.h>
static ssize_t trace_send(int, const void *, size_t, int);
static ssize_t trace_recv(int, void *, size_t, int);
static int trace_poll(int, struct epoll_event *, int, int);
static int trace_pwait(int, struct epoll_event *, int, const struct timespec *, const sigset_t *);
static int trace_clock(clockid_t, struct timespec *);
#define send trace_send
#define recv trace_recv
#define epoll_wait trace_poll
#define epoll_pwait2 trace_pwait
#define clock_gettime trace_clock
#define main unused_netload_main
#include "netload.c"
#undef main
#undef epoll_wait
#undef epoll_pwait2
#undef clock_gettime
#undef recv
#undef send
#include <signal.h>
#include <sys/wait.h>

enum operation { SEND, RECEIVE, POLL };
struct step { enum operation operation; int result; unsigned peer; };
static const struct step *steps;
static size_t step_count, step_at, wire_sent[9], wire_received[9];
static unsigned step_peer, trace_peers;
static int trace_paced;
static struct timespec trace_now;
static unsigned char wire[9][64];
static struct connection *traced_link;
static volatile sig_atomic_t client_pid;
#define ALL_READABLE 0x40000000

static void require(int condition, const char *message) {
    if (!condition) {
        fprintf(stderr, "netload_check: %s\n", message);
        if (client_pid > 0) kill((pid_t)client_pid, SIGKILL);
        exit(2);
    }
}

static int next(enum operation operation) {
    require(step_at < step_count, "unexpected operation after trace ended (possibly lost edge)");
    require(steps[step_at].operation == operation, "wrong syscall order (possibly speculative receive or lost edge)");
    step_peer = steps[step_at].peer;
    return steps[step_at++].result;
}

static ssize_t trace_send(int descriptor, const void *buffer, size_t bytes, int flags) {
    require(flags == MSG_NOSIGNAL, "send flags changed");
    int count = next(SEND);
    require((unsigned)descriptor == step_peer, "send to wrong traced peer");
    if (count < 0) { errno = -count; return -1; }
    if (wire_received[step_peer] == sizeof wire[0]) wire_received[step_peer] = wire_sent[step_peer] = 0;
    require(count > 0 && (size_t)count <= bytes && wire_sent[step_peer] + (size_t)count <= sizeof wire[0],
            "invalid scripted send");
    memcpy(wire[step_peer] + wire_sent[step_peer], buffer, (size_t)count);
    wire_sent[step_peer] += (size_t)count;
    return count;
}

static ssize_t trace_recv(int descriptor, void *buffer, size_t bytes, int flags) {
    require(flags == 0, "receive flags changed");
    int count = next(RECEIVE);
    require((unsigned)descriptor == step_peer, "receive from wrong traced peer");
    if (count < 0) { errno = -count; return -1; }
    require((size_t)count <= bytes && wire_received[step_peer] + (size_t)count <= wire_sent[step_peer],
            "trace received bytes that were never sent");
    memcpy(buffer, wire[step_peer] + wire_received[step_peer], (size_t)count);
    wire_received[step_peer] += (size_t)count;
    return count;
}

static int trace_poll(int descriptor, struct epoll_event *events, int maximum, int timeout) {
    (void)descriptor;
    (void)timeout;
    require(maximum == 256, "event batch capacity changed");
    int flags = next(POLL);
    if (trace_paced && (flags & EPOLLRDHUP)) trace_now.tv_nsec = 2000000;
    if (flags == ALL_READABLE) {
        for (unsigned at=0; at<trace_peers; at++) {
            events[at].events = EPOLLIN;
            events[at].data.ptr = &traced_link[at];
        }
        return (int)trace_peers;
    }
    events[0].events = (uint32_t)flags;
    events[0].data.ptr = &traced_link[step_peer];
    return 1;
}

static int trace_pwait(int descriptor, struct epoll_event *events, int maximum,
                       const struct timespec *timeout, const sigset_t *mask) {
    (void)timeout;
    require(mask==NULL,"unexpected trace signal mask");
    return trace_poll(descriptor,events,maximum,0);
}

static int trace_clock(clockid_t clock, struct timespec *now) {
    require(clock==CLOCK_MONOTONIC,"trace clock changed");
    *now=trace_now;
    return 0;
}

static void trace_case(const char *name, const struct step *script, size_t count,
                       unsigned rounds, int admission, int expected, unsigned peers) {
    fflush(NULL);
    pid_t child = fork();
    require(child >= 0, "trace fork failed");
    if (!child) {
        unsigned char outgoing[9][64], incoming[9][64];
        uint32_t latencies[9*4] = {0};
        uint32_t dispatches[9*4] = {0};
        struct connection links[9] = {0};
        struct client owner = {.first=links, .count=peers};
        require(peers<=9 && rounds<=4,"trace capacity exceeded");
        steps = script;
        step_count = count;
        step_at = 0;
        memset(wire_sent,0,sizeof wire_sent);
        memset(wire_received,0,sizeof wire_received);
        traced_link = links;
        trace_peers = peers;
        trace_now = (struct timespec){10,0};
        exchange_origin = trace_now;
        if (trace_paced) option_light_per_second=1000;
        option_bytes = 64;
        option_roundtrips = rounds;
        latency_us = latencies;
        dispatch_us = dispatches;
        for (unsigned at=0; at<peers; at++) {
            links[at].outgoing=outgoing[at];
            links[at].incoming=incoming[at];
            links[at].index=at;
            links[at].descriptor=(int)at;
            links[at].planned=rounds;
            fill_message(&links[at]);
        }
        if (admission) {
            owner.admitting = 1;
            exchange(&owner);
            require(links[0].read_events & EPOLLRDHUP, "terminal readiness lost across complete admission");
            owner.admitting = 0;
        }
        exchange(&owner);
        require(step_at == step_count && links[0].round == rounds && !owner.outstanding,
                "trace stopped before all frames were verified");
        _exit(0);
    }
    int status;
    require(waitpid(child, &status, 0) == child && WIFEXITED(status) && WEXITSTATUS(status) == expected,
            "trace exit status mismatch");
    printf("netload trace: %s budget=%u PASS\n", name, (unsigned)WF_NETLOAD_SERVICE_ROUNDS);
}

#define TRACE(name, script, rounds, admission, expected) \
    trace_case(name, script, sizeof(script)/sizeof((script)[0]), rounds, admission, expected, 1)
static void trace_checks(void) {
    const struct step full[] = {
        {SEND,64,0},{POLL,EPOLLOUT,0},{POLL,EPOLLIN|EPOLLOUT,0},{RECEIVE,64,0},
        {SEND,64,0},{POLL,EPOLLIN,0},{RECEIVE,64,0}
    };
    TRACE("out-only event does not cause speculative receive",full,2,0,0);
    const struct step write_completion[] = {
        {SEND,16,0},{SEND,-EAGAIN,0},{POLL,EPOLLIN|EPOLLOUT,0},{SEND,48,0},{RECEIVE,64,0},
        {SEND,64,0},{POLL,EPOLLIN,0},{RECEIVE,64,0}
    };
    TRACE("read edge already delivered when partial send finishes",write_completion,2,0,0);
    const struct step fragmented[] = {
        {SEND,16,0},{SEND,-EAGAIN,0},{POLL,EPOLLIN|EPOLLOUT,0},
        {SEND,-EAGAIN,0},{RECEIVE,11,0},{RECEIVE,-EAGAIN,0},
        {POLL,EPOLLOUT,0},{SEND,48,0},{POLL,EPOLLIN,0},{RECEIVE,53,0},
        {SEND,64,0},{POLL,EPOLLIN,0},{RECEIVE,64,0}
    };
    TRACE("bidirectional partial progress and a new edge after EAGAIN",fragmented,2,0,0);
    const struct step eof[] = {{SEND,64,0},{POLL,EPOLLHUP,0},{RECEIVE,0,0}};
    TRACE("EOF without EPOLLIN",eof,1,0,1);
    const struct step reset[] = {{SEND,64,0},{POLL,EPOLLERR,0},{RECEIVE,-ECONNRESET,0}};
    TRACE("reset without EPOLLIN",reset,1,0,1);
    const struct step truncated[] = {{SEND,64,0},{POLL,EPOLLIN|EPOLLRDHUP,0},{RECEIVE,11,0},{RECEIVE,0,0}};
    TRACE("truncated response",truncated,1,0,1);
    const struct step admitted_eof[] = {
        {SEND,64,0},{POLL,EPOLLIN|EPOLLRDHUP,0},{RECEIVE,64,0},{SEND,64,0},{RECEIVE,0,0}
    };
    TRACE("buffered admission then sticky EOF",admitted_eof,1,1,1);
    const struct step late_terminal[] = {
        {SEND,64,0},{SEND,64,1},{POLL,EPOLLIN,0},{RECEIVE,64,0},
        {POLL,EPOLLRDHUP,0},{POLL,EPOLLIN,1},{RECEIVE,64,1},{SEND,64,0},{RECEIVE,0,0}
    };
    trace_case("terminal event on finished admission peer",late_terminal,
               sizeof late_terminal/sizeof late_terminal[0],1,1,1,2);
#if WF_NETLOAD_SERVICE_ROUNDS == 1
    /* Nine independently ready peers leave one next request queued after the
     * eight-item FIFO drain. A later epoll call delivers its terminal edge. */
    struct step queued[32];
    size_t count=0;
    for (unsigned peer=0; peer<9; peer++) queued[count++]=(struct step){SEND,64,peer};
    queued[count++]=(struct step){POLL,ALL_READABLE,0};
    for (unsigned peer=0; peer<9; peer++) queued[count++]=(struct step){RECEIVE,64,peer};
    for (unsigned peer=0; peer<8; peer++) queued[count++]=(struct step){SEND,64,peer};
    queued[count++]=(struct step){POLL,EPOLLRDHUP,8};
    queued[count++]=(struct step){SEND,64,8};
    queued[count++]=(struct step){RECEIVE,0,8};
    trace_case("terminal edge on queued ninth peer",queued,count,2,0,1,9);
#endif
    const struct step waiting[] = {
        {SEND,64,0},{SEND,64,1},{POLL,EPOLLIN,1},{RECEIVE,64,1},
        {POLL,EPOLLRDHUP,1},{SEND,64,1},{RECEIVE,0,1}
    };
    trace_paced=1;
    trace_case("terminal edge while next arrival is waiting",waiting,
               sizeof waiting/sizeof waiting[0],2,0,1,2);
    trace_paced=0;

}
#undef TRACE

static void timed_out(int number) {
    (void)number;
    if (client_pid > 0) kill((pid_t)client_pid, SIGKILL);
    _exit(124);
}

static void write_all(int descriptor, const unsigned char *buffer, size_t count) {
    while (count) {
        ssize_t written = send(descriptor, buffer, count, MSG_NOSIGNAL);
        if (written < 0 && errno == EINTR) continue;
        require(written > 0, "fixture send failed");
        buffer += written;
        count -= (size_t)written;
    }
}

static uint64_t observation_field(const char *line, const char *name) {
    char key[96];
    int size=snprintf(key,sizeof key," %s=",name);
    require(size>0 && (size_t)size<sizeof key,"observation key overflow");
    const char *at=strstr(line,key);
    require(at!=NULL,"missing observer field");
    return strtoull(at+size,NULL,10);
}

static void socket_case(const char *binary, const char *mode, unsigned bytes, int admit, int observed) {
    enum { ROUNDS = 3 };
    int listener = socket(AF_INET, SOCK_STREAM, 0);
    require(listener >= 0, "fixture socket failed");
    struct sockaddr_in address = {.sin_family=AF_INET, .sin_addr={htonl(INADDR_LOOPBACK)}};
    require(bind(listener,(struct sockaddr *)&address,sizeof address)==0 && listen(listener,1)==0,
            "fixture listen failed");
    socklen_t length = sizeof address;
    require(getsockname(listener,(struct sockaddr *)&address,&length)==0, "fixture address failed");
    FILE *output = tmpfile(), *diagnostics = tmpfile();
    require(output && diagnostics, "fixture capture failed");
    fflush(NULL);
    pid_t child = fork();
    require(child >= 0, "fixture fork failed");
    if (!child) {
        close(listener);
        require(dup2(fileno(output),STDOUT_FILENO)>=0 && dup2(fileno(diagnostics),STDERR_FILENO)>=0,
                "fixture redirection failed");
        char port[16], size[32];
        snprintf(port,sizeof port,"%u",ntohs(address.sin_port));
        snprintf(size,sizeof size,"%u",bytes);
        if (admit) execl(binary,binary,port,"1","3",size,"--admit",(char *)NULL);
        else execl(binary,binary,port,"1","3",size,(char *)NULL);
        _exit(2);
    }
    client_pid = child;
    int peer = accept(listener,NULL,NULL);
    require(peer>=0, "fixture accept failed");
    close(listener);
    int one=1, buffer_bytes=4096;
    require(setsockopt(peer,IPPROTO_TCP,TCP_NODELAY,&one,sizeof one)==0 &&
            setsockopt(peer,SOL_SOCKET,SO_SNDBUF,&buffer_bytes,sizeof buffer_bytes)==0,
            "fixture socket policy failed");
    int expected = strcmp(mode,"echo") && strcmp(mode,"fragmented");
    for (unsigned round=0; round < ROUNDS+(unsigned)admit; round++) {
        unsigned offset=0;
        unsigned char first_byte=0;
        unsigned char buffer[16384];
        while (offset < bytes) {
            size_t wanted = bytes-offset;
            size_t fragment = strcmp(mode,"fragmented")==0 ? (offset==0 ? 1u : 8191u) : sizeof buffer;
            if (wanted>fragment) wanted=fragment;
            ssize_t moved = recv(peer,buffer,wanted,0);
            if (moved<0 && errno==EINTR) continue;
            require(moved>0, "fixture request ended early");
            for (size_t at=0; at<(size_t)moved; at++) {
                unsigned position=offset+(unsigned)at;
                unsigned char value=position ? (unsigned char)(position*7u+11u)
                    : admit && !round ? 250u : (unsigned char)((round-(unsigned)admit)*17u+3u);
                require(buffer[at]==value,"fixture request byte mismatch");
            }
            if (!offset) first_byte=buffer[0];
            if (!strcmp(mode,"corrupt") && !offset) buffer[0]^=1u;
            if (!expected || !strcmp(mode,"corrupt")) write_all(peer,buffer,(size_t)moved);
            offset+=(unsigned)moved;
        }
        if (expected) {
            /* Drain the complete request first. Closing with unread bytes
             * could turn an intended EOF/truncation oracle into a reset. */
            if (!strcmp(mode,"truncated")) write_all(peer,&first_byte,1);
            if (!strcmp(mode,"reset")) {
                struct linger reset={1,0};
                require(setsockopt(peer,SOL_SOCKET,SO_LINGER,&reset,sizeof reset)==0,"fixture reset failed");
            } else require(shutdown(peer,SHUT_WR)==0,"fixture EOF failed");
            goto finished;
        }
    }
finished:
    close(peer);
    int status;
    require(waitpid(child,&status,0)==child && WIFEXITED(status) && WEXITSTATUS(status)==expected,
            "socket client exit status mismatch");
    client_pid=0;
    require(fseek(output,0,SEEK_END)==0,"fixture output seek failed");
    long size=ftell(output);
    require(size>=0 && (expected ? size==0 : size>0),"failed client published a measurement or successful client did not");
    rewind(output);
    char line[4096];
    if (!expected) {
        require(fgets(line,sizeof line,output)!=NULL && strstr(line,"roundtrips=3\t")!=NULL,
                "socket fixture round count mismatch");
    }
    require(fseek(diagnostics,0,SEEK_END)==0,"fixture diagnostics seek failed");
    long errors=ftell(diagnostics);
    require(errors>=0 && (expected || observed ? errors>0 : errors==0),"socket fixture diagnostic mismatch");
    if (expected) {
        rewind(diagnostics);
        require(fgets(line,sizeof line,diagnostics)!=NULL,"missing client error");
        const char *reason=!strcmp(mode,"corrupt") ? "round 0: byte 0 is "
            : !strcmp(mode,"truncated") ? "the peer closed after echoing 1 of "
            : !strcmp(mode,"eof") ? "the peer closed after echoing 0 of " : "receive failed:";
        require(strstr(line,reason)!=NULL,"client failed for the wrong reason");
    }
    if (!expected && observed) {
        rewind(diagnostics);
        require(fgets(line,sizeof line,diagnostics)!=NULL && !strncmp(line,"netload-worker ",15),"missing observer worker");
        for (int phase=0; phase<=admit; phase++) {
            require(fgets(line,sizeof line,diagnostics)!=NULL && strstr(line,phase ? "phase=admission " : "phase=exchange "),
                    "missing separate observer phase");
            uint64_t frames=phase ? 1u : ROUNDS;
            require(observation_field(line,"verified")==frames && observation_field(line,"verified_bytes")==frames*bytes &&
                    observation_field(line,"send_bytes")==frames*bytes && observation_field(line,"recv_bytes")==frames*bytes,
                    "socket observer byte conservation");
            if (!strcmp(mode,"fragmented")) {
                require(observation_field(line,"send_again")>0 && observation_field(line,"send_short")>0 &&
                        observation_field(line,"recv_short")>0,"fixture did not exercise actual partial writes/reads and backpressure");
            }
            printf("netload socket counter: mode=%s bytes=%u %s",mode,bytes,line);
        }
        require(fgetc(diagnostics)==EOF,"unexpected observer diagnostics");
    }
    fclose(output);
    fclose(diagnostics);
    printf("netload socket: mode=%s bytes=%u admission=%d PASS\n",mode,bytes,admit);
}

int main(int argc, char **argv) {
    signal(SIGALRM,timed_out);
    alarm(90);
    if (argc==1) trace_checks();
    else {
        require(argc==2 || (argc==3 && !strcmp(argv[2],"observed")),"usage: netload_check [CLIENT [observed]]");
        int observed=argc==3;
        socket_case(argv[1],"echo",64,0,observed);
        socket_case(argv[1],"echo",65536,0,observed);
        socket_case(argv[1],"echo",65536,1,observed);
        socket_case(argv[1],"fragmented",8u*1024u*1024u,1,observed);
        socket_case(argv[1],"eof",64,0,observed);
        socket_case(argv[1],"reset",65536,0,observed);
        socket_case(argv[1],"truncated",65536,0,observed);
        socket_case(argv[1],"corrupt",64,0,observed);
    }
    alarm(0);
    return 0;
}
