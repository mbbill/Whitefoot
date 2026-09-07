#include "records_native.h"
#include <string.h>

uint64_t records_state(const uint8_t *s, size_t n) {
    uint64_t count = 0;
    unsigned remaining = 0, lower = 128, upper = 191;
    for (size_t i = 0; i < n; ++i) {
        unsigned b = s[i];
        if (remaining == 0) {
            ++count;
            if (b < 128) continue;
            if (b < 194) return UINT64_MAX;
            if (b <= 223) remaining = 1;
            else if (b <= 239) {
                remaining = 2;
                if (b == 224) lower = 160;
                if (b == 237) upper = 159;
            } else if (b <= 244) {
                remaining = 3;
                if (b == 240) lower = 144;
                if (b == 244) upper = 143;
            } else return UINT64_MAX;
        } else {
            if (b < lower || b > upper) return UINT64_MAX;
            --remaining;
            lower = 128; upper = 191;
        }
    }
    return remaining ? UINT64_MAX : count;
}

static int continuation(unsigned b) { return b >= 128 && b <= 191; }

uint64_t records_word(const uint8_t *s, size_t n) {
    size_t i = 0;
    uint64_t count = 0;
    while (i < n) {
        /* The word path starts at a code-point boundary. memcpy admits
         * unaligned input, and the remaining-length test excludes overread. */
        if (n - i >= 16) {
            uint64_t a, b;
            memcpy(&a, s + i, sizeof(a));
            memcpy(&b, s + i + 8, sizeof(b));
            if (((a | b) & UINT64_C(0x8080808080808080)) == 0) {
                i += 16; count += 16; continue;
            }
        }
        unsigned b = s[i];
        if (b < 128) { ++i; ++count; continue; }
        if (b < 194 || b > 244) return UINT64_MAX;
        if (b <= 223) {
            if (n - i < 2 || !continuation(s[i+1])) return UINT64_MAX;
            i += 2;
        } else if (b <= 239) {
            if (n - i < 3 || !continuation(s[i+1]) || !continuation(s[i+2])) return UINT64_MAX;
            if ((b == 224 && s[i+1] < 160) || (b == 237 && s[i+1] > 159)) return UINT64_MAX;
            i += 3;
        } else {
            if (n - i < 4 || !continuation(s[i+1]) || !continuation(s[i+2]) || !continuation(s[i+3])) return UINT64_MAX;
            if ((b == 240 && s[i+1] < 144) || (b == 244 && s[i+1] > 143)) return UINT64_MAX;
            i += 4;
        }
        ++count;
    }
    return count;
}
