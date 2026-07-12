#include <stdint.h>
void accumulate(uint64_t *acc, const uint64_t *addend, uint64_t n){
  for(uint64_t i=0;i<n;i++){ *acc += *addend; } }
