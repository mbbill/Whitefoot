/* Whole-process measurements for ordinary compiler-produced commands and
 * native controls. No WF private ABI. Retire with these command benchmarks. */
#define _DEFAULT_SOURCE
#define _DARWIN_C_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#ifdef _WIN32
#define WIN32_LEAN_AND_MEAN
#include <windows.h>
#include <psapi.h>
#include <wchar.h>

static uint64_t ticks(FILETIME time) {
    return ((uint64_t)time.dwHighDateTime<<32)|time.dwLowDateTime;
}

int wmain(int argc,wchar_t **argv) {
    if(argc<2)return 2;
    /* Windows' argv quoting, with a bounded CreateProcess command line. The
     * child is invoked directly; no shell interprets metacharacters. */
    size_t room=1;
    for(int i=1;i<argc;++i) {
        size_t length=wcslen(argv[i]);
        if(length>16000 || room>32767-(2*length+3))return 2;
        room+=2*length+3;
    }
    wchar_t *command=calloc(room,sizeof(*command));
    if(!command)return 2;
    size_t at=0;
    for(int i=1;i<argc;++i) {
        if(i>1)command[at++]=L' ';
        command[at++]=L'"';
        const wchar_t *p=argv[i];
        while(*p) {
            size_t slashes=0;
            while(*p==L'\\'){++slashes;++p;}
            size_t copies=slashes;
            if(!*p || *p==L'"')copies*=2;
            while(copies--)command[at++]=L'\\';
            if(!*p)break;
            if(*p==L'"')command[at++]=L'\\';
            command[at++]=*p++;
        }
        command[at++]=L'"';
    }
    STARTUPINFOW startup={0};
    PROCESS_INFORMATION process={0};
    startup.cb=sizeof(startup);
    LARGE_INTEGER frequency,start,stop;
    if(!QueryPerformanceFrequency(&frequency) || !QueryPerformanceCounter(&start)) {
        free(command);return 2;
    }
    if(!CreateProcessW(argv[1],command,NULL,NULL,TRUE,0,NULL,NULL,&startup,&process)) {
        fprintf(stderr,"command runner: CreateProcess error %lu\n",GetLastError());
        free(command);return 2;
    }
    free(command);
    DWORD waited=WaitForSingleObject(process.hProcess,INFINITE),status;
    FILETIME created,exited,kernel,user;
    PROCESS_MEMORY_COUNTERS memory={0};
    memory.cb=sizeof(memory);
    if(waited!=WAIT_OBJECT_0 || !QueryPerformanceCounter(&stop) ||
       !GetExitCodeProcess(process.hProcess,&status) ||
       !GetProcessTimes(process.hProcess,&created,&exited,&kernel,&user) ||
       !GetProcessMemoryInfo(process.hProcess,&memory,sizeof(memory))) {
        fprintf(stderr,"command runner: process measurement error %lu\n",GetLastError());
        CloseHandle(process.hThread);CloseHandle(process.hProcess);return 2;
    }
    uint64_t delta=(uint64_t)(stop.QuadPart-start.QuadPart),hz=(uint64_t)frequency.QuadPart;
    uint64_t wall=(delta/hz)*UINT64_C(1000000000)+(delta%hz)*UINT64_C(1000000000)/hz;
    printf("%llu\t%llu\t%llu\t%llu\tNA\tNA\t%lu\n",
        (unsigned long long)wall,(unsigned long long)(ticks(user)*100),
        (unsigned long long)(ticks(kernel)*100),(unsigned long long)memory.PeakWorkingSetSize,status);
    CloseHandle(process.hThread);CloseHandle(process.hProcess);
    return status==0?0:1;
}
#else
#include <errno.h>
#include <inttypes.h>
#include <sys/resource.h>
#include <sys/wait.h>
#include <time.h>
#include <unistd.h>

static uint64_t now(void) {
    struct timespec time;
    if(clock_gettime(CLOCK_MONOTONIC,&time)){perror("command runner clock");exit(2);}
    return (uint64_t)time.tv_sec*UINT64_C(1000000000)+(uint64_t)time.tv_nsec;
}
static uint64_t cpu_ns(struct timeval time) {
    return (uint64_t)time.tv_sec*UINT64_C(1000000000)+(uint64_t)time.tv_usec*1000;
}
int main(int argc,char **argv) {
    if(argc<2)return 2;
    uint64_t start=now();
    pid_t child=fork();
    if(child<0){perror("command runner fork");return 2;}
    if(child==0){execv(argv[1],argv+1);perror("command runner exec");_exit(127);}
    struct rusage usage;
    int status;
    pid_t waited;
    do {waited=wait4(child,&status,0,&usage);}while(waited<0 && errno==EINTR);
    if(waited!=child){perror("command runner wait");return 2;}
    uint64_t wall=now()-start;
    unsigned code=WIFEXITED(status)?(unsigned)WEXITSTATUS(status):128u+(unsigned)WTERMSIG(status);
    uint64_t rss=(uint64_t)usage.ru_maxrss;
#ifndef __APPLE__
    rss*=1024;
#endif
    printf("%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%" PRIu64 "\t%ld\t%ld\t%u\n",
        wall,cpu_ns(usage.ru_utime),cpu_ns(usage.ru_stime),rss,usage.ru_nvcsw,usage.ru_nivcsw,code);
    return code==0?0:1;
}
#endif
