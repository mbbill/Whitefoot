#include "kernels.h"

uint64_t walk_checkfree(const Node *slab, uint32_t start, size_t steps) {
    uint64_t sum = 0;
    uint32_t idx = start;
    for (size_t k = 0; k < steps; k++) {
        const Node *n = &slab[idx];      /* base + idx*16, no check */
        sum += n->val;
        idx = n->next_idx;
    }
    return sum;
}

uint64_t walk_checked(const Node *slab, size_t count, uint32_t start, size_t steps) {
    uint64_t sum = 0;
    uint32_t idx = start;
    for (size_t k = 0; k < steps; k++) {
        if (idx >= count) __builtin_trap();   /* per-deref bound; idx is data-dependent */
        const Node *n = &slab[idx];
        sum += n->val;
        idx = n->next_idx;
    }
    return sum;
}

uint64_t walk_pointer(const PNode *start, size_t steps) {
    uint64_t sum = 0;
    const PNode *n = start;
    for (size_t k = 0; k < steps; k++) {
        sum += n->val;
        n = n->next;
    }
    return sum;
}
