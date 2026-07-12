#include <stdint.h>
void accumulate(uint64_t *restrict acc, const uint64_t *restrict addend, uint64_t n){
  for(uint64_t i=0;i<n;i++){ *acc += *addend; } }
