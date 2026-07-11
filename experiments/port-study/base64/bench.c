/* encode-only throughput: no file I/O, no output write — isolates the kernel */
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <time.h>
typedef struct { uint8_t *p; int64_t n; } Buf;
extern uint64_t encode(Buf out, Buf src);
int main(int argc, char **argv) {
  long n = argc > 1 ? atol(argv[1]) : 384000000;
  Buf src; src.p = malloc(n); src.n = n;
  for (long i = 0; i < n; i++) src.p[i] = (uint8_t)(i * 131 + 7);
  Buf out; out.n = ((n+2)/3)*4 + 16; out.p = malloc(out.n);
  for (int w = 0; w < 2; w++) encode(out, src);
  struct timespec t0,t1; clock_gettime(CLOCK_MONOTONIC,&t0);
  uint64_t len = 0;
  for (int r = 0; r < 5; r++) len ^= encode(out, src);
  clock_gettime(CLOCK_MONOTONIC,&t1);
  double ns = (t1.tv_sec-t0.tv_sec)*1e9 + (t1.tv_nsec-t0.tv_nsec);
  printf("encode: %.3f GB/s (%.1f ms/pass, sink=%llu)\n",
         n*5/ (ns), ns/5e6, (unsigned long long)len);
  return 0;
}
