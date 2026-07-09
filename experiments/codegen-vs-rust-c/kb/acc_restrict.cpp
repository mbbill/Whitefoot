#include <cstdint>
#define K 0x9E3779B97F4A7C15ULL
extern "C" void accumulate(uint64_t *__restrict acc, const uint64_t *__restrict addend, uint64_t n){
  for(uint64_t i=0;i<n;i++){ *acc = (*acc ^ *addend) * K; } }
