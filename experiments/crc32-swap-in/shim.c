/* zlib-ABI shim over the xlang CRC-32 kernel (D4 FFI frame: the C side owns
   the boundary discipline — single-threaded init, buffers passed by {ptr,len}
   value, xlang treats each call's buffer as an owned affine value). */
#include <stddef.h>

typedef struct { void *p; long long n; } Buf;
extern Buf crc32_mktab(void);
extern unsigned int crc32_upd(Buf tab, unsigned int crc, Buf data);

static Buf tab;
static int ready;

unsigned long xcrc32(unsigned long crc, const unsigned char *buf, unsigned int len) {
  if (buf == NULL) return 0;                 /* zlib: crc32(x, Z_NULL, 0) == 0 */
  if (!ready) { tab = crc32_mktab(); ready = 1; }
  Buf d = { (void *)buf, (long long)len };
  return (unsigned long)crc32_upd(tab, (unsigned int)crc, d);
}
