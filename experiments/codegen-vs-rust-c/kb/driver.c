#include <stdint.h>
#include <stdio.h>
extern void accumulate(uint64_t*, const uint64_t*, uint64_t);
#ifndef NITER
#define NITER 1000000000ULL
#endif
int main(void){ uint64_t a=1,b=3; accumulate(&a,&b,NITER);
  printf("%llu\n",(unsigned long long)a); return (int)(a & 0xFF); }
