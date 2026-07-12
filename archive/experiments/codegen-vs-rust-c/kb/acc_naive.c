#include <stdint.h>
#define K 0x9E3779B97F4A7C15ULL
void accumulate(uint64_t *acc, const uint64_t *addend, uint64_t n){
  for(uint64_t i=0;i<n;i++){ *acc = (*acc ^ *addend) * K; } }
