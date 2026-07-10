#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>
#include <time.h>

extern unsigned long xcrc32(unsigned long, const unsigned char *, unsigned int);

static double now(void) { struct timespec t; clock_gettime(CLOCK_MONOTONIC, &t); return t.tv_sec + t.tv_nsec * 1e-9; }

int main(void) {
  void *z = dlopen("/usr/lib/libz.dylib", RTLD_NOW);
  unsigned long (*zcrc)(unsigned long, const unsigned char *, unsigned int) =
      (unsigned long (*)(unsigned long, const unsigned char *, unsigned int))dlsym(z, "crc32");
  unsigned int n = 64u << 20;
  unsigned char *buf = malloc(n);
  for (unsigned int i = 0; i < n; i++) buf[i] = (unsigned char)(i * 2654435761u >> 24);
  unsigned long sink = 0;
  sink ^= xcrc32(0, buf, n); sink ^= zcrc(0, buf, n);   /* warm */
  double t0 = now(); for (int r = 0; r < 8; r++) sink ^= xcrc32(0, buf, n); double tx = (now() - t0) / 8;
  t0 = now(); for (int r = 0; r < 8; r++) sink ^= zcrc(0, buf, n); double tz = (now() - t0) / 8;
  printf("xlang crc32: %.2f GB/s\nsystem zlib: %.2f GB/s\n(sink %lx)\n", n / tx / 1e9, n / tz / 1e9, sink);
  return 0;
}
