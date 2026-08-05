#ifndef WF_LITERAL_LINE_FLOOR_KERNEL_H
#define WF_LITERAL_LINE_FLOOR_KERNEL_H

#include <stdint.h>

typedef struct {
    const uint8_t *data;
    uint64_t length;
} WfBuffer;

uint64_t wf_literal_line(WfBuffer haystack, WfBuffer needle);

#endif
