#include "records_native.h"

uint64_t records_reference(const uint8_t *s, size_t n) {
    size_t i = 0;
    uint64_t count = 0;
    while (i < n) {
        uint8_t b = s[i++];
        uint32_t v;
        unsigned more;
        uint32_t minimum;
        if (b < 128) { ++count; continue; }
        if ((b & 0xe0) == 0xc0) { v = b & 0x1f; more = 1; minimum = 0x80; }
        else if ((b & 0xf0) == 0xe0) { v = b & 0x0f; more = 2; minimum = 0x800; }
        else if ((b & 0xf8) == 0xf0) { v = b & 0x07; more = 3; minimum = 0x10000; }
        else return UINT64_MAX;
        if (n - i < more) return UINT64_MAX;
        for (unsigned j = 0; j < more; ++j) {
            b = s[i++];
            if ((b & 0xc0) != 0x80) return UINT64_MAX;
            v = (v << 6) | (b & 0x3f);
        }
        if (v < minimum || v > 0x10ffff || (v >= 0xd800 && v <= 0xdfff)) return UINT64_MAX;
        ++count;
    }
    return count;
}
