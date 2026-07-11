#include <stdint.h>

typedef struct { uint8_t *p; int64_t n; } Buf;
typedef struct { uint64_t lines, words, bytes, first_space, last_space; } Summary;

static int is_space(uint8_t byte) {
  return (byte >= 9 && byte <= 13) || byte == 32;
}

void summarize(Summary *out, Buf b) {
  uint64_t lines = 0, words = 0;
  int previous_space = 1;
  for (int64_t i = 0; i < b.n; i++) {
    uint8_t byte = b.p[i];
    lines += byte == 10;
    int space = is_space(byte);
    words += !space && previous_space;
    previous_space = space;
  }
  out->lines = lines;
  out->words = words;
  out->bytes = (uint64_t)b.n;
  out->first_space = b.n == 0 || is_space(b.p[0]);
  out->last_space = previous_space;
}

void combine(Summary *out, const Summary *a, const Summary *b) {
  if (a->bytes == 0) { *out = *b; return; }
  if (b->bytes == 0) { *out = *a; return; }
  *out = (Summary){
    a->lines + b->lines,
    a->words + b->words - (!a->last_space && !b->first_space),
    a->bytes + b->bytes,
    a->first_space,
    b->last_space,
  };
}
