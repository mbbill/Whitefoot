#include "kernel.h"

#if defined(WF_FULL_SCAN)

uint64_t wf_scan(WfBuffer input) {
    uint64_t lines = 0;
    uint64_t candidates = 0;
    for (uint64_t offset = 0; offset < input.length; ++offset) {
        const uint8_t byte = input.data[offset];
        lines += byte == 10;
        candidates += byte == 80;
    }
    return lines + candidates * UINT64_C(1000000007);
}

#elif defined(WF_EARLY_SCAN)

static uint64_t find_byte(WfBuffer input, uint8_t needle) {
    for (uint64_t offset = 0; offset < input.length; ++offset) {
        if (input.data[offset] == needle) {
            return offset;
        }
    }
    return input.length;
}

uint64_t wf_scan(WfBuffer input) {
    const uint64_t quarter = find_byte(input, 251);
    const uint64_t half = find_byte(input, 252);
    const uint64_t end = find_byte(input, 253);
    const uint64_t missing = find_byte(input, 254);
    return quarter + half + end + missing;
}

#else
#error "define exactly one scan shape"
#endif
