/* A batch host for the decoder's Slice/MutSlice API, independent of the CLI's
 * deliberately smaller input buffer. Lengths and results use little endian. */
#include <stdint.h>
#include <stdio.h>
#include <string.h>
typedef struct { void *data; uint64_t length; } View;
extern uint64_t wf_decode_case(View input, View output);
extern int wf__floor_run(int, char **);
static unsigned char input[70000], output[70002];
static uint32_t word(const unsigned char *p) {
    return (uint32_t)p[0] | ((uint32_t)p[1] << 8)
        | ((uint32_t)p[2] << 16) | ((uint32_t)p[3] << 24);
}
int wf__main_body(int argc, char **argv) {
    (void)argc; (void)argv;
    unsigned char header[8];
    for (;;) {
        size_t got = fread(header, 1, sizeof header, stdin);
        if (!got && feof(stdin)) return 0;
        if (got != sizeof header) return 81;
        uint32_t length = word(header), capacity = word(header + 4);
        if (length > sizeof input || capacity > sizeof output - 2) return 82;
        if (fread(input, 1, length, stdin) != length) return 83;
        memset(output, 0xa5, sizeof output);
        uint64_t result = wf_decode_case((View){input, length}, (View){output + 1, capacity});
        if (output[0] != 0xa5 || output[capacity + 1] != 0xa5) return 84;
        if (result > capacity && result < UINT64_MAX - 6) return 85;
        unsigned char record[8];
        for (unsigned at = 0; at < 8; ++at) record[at] = (unsigned char)(result >> (at * 8));
        if (fwrite(record, 1, sizeof record, stdout) != sizeof record) return 86;
        if (result <= capacity && fwrite(output + 1, 1, (size_t)result, stdout) != result) return 87;
    }
}
int main(int argc, char **argv) { return wf__floor_run(argc, argv); }
