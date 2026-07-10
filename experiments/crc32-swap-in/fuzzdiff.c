/* Bit-identical fuzz-diff: xlang crc32 vs the system zlib, random buffers,
   random incremental chaining splits. */
#include <stdio.h>
#include <stdlib.h>
#include <dlfcn.h>

extern unsigned long xcrc32(unsigned long, const unsigned char *, unsigned int);

int main(void) {
  void *z = dlopen("/usr/lib/libz.dylib", RTLD_NOW);
  if (!z) { fprintf(stderr, "no libz\n"); return 2; }
  unsigned long (*zcrc)(unsigned long, const unsigned char *, unsigned int) =
      (unsigned long (*)(unsigned long, const unsigned char *, unsigned int))dlsym(z, "crc32");
  srandom(42);
  static unsigned char buf[1 << 20];
  for (int t = 0; t < 2000; t++) {
    unsigned int len = (unsigned int)(random() % (t < 1000 ? 4096 : sizeof buf));
    for (unsigned int i = 0; i < len; i++) buf[i] = (unsigned char)random();
    unsigned long a = xcrc32(0, buf, len);
    unsigned long b = zcrc(0, buf, len);
    /* incremental chaining at a random split */
    unsigned int cut = len ? (unsigned int)(random() % len) : 0;
    unsigned long a2 = xcrc32(xcrc32(0, buf, cut), buf + cut, len - cut);
    if (a != b || a2 != b) {
      fprintf(stderr, "MISMATCH t=%d len=%u: x=%lx z=%lx inc=%lx\n", t, len, a, b, a2);
      return 1;
    }
  }
  printf("fuzz-diff: 2000 cases + incremental chaining, bit-identical with system zlib\n");
  return 0;
}
