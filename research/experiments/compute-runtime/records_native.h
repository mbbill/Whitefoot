#ifndef WHITEFOOT_RECORDS_NATIVE_H
#define WHITEFOOT_RECORDS_NATIVE_H
#include <stddef.h>
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
/* Valid UTF-8 returns its Unicode scalar count. Invalid input returns
 * UINT64_MAX. Inputs may be empty and need no padding or alignment. */
typedef uint64_t (*RecordKernel)(const uint8_t *, size_t);
uint64_t records_state(const uint8_t *, size_t);
uint64_t records_word(const uint8_t *, size_t);
uint64_t records_reference(const uint8_t *, size_t);
#ifdef __cplusplus
}
#endif
#endif
