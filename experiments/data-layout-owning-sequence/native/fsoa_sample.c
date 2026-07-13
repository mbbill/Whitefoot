#define _POSIX_C_SOURCE 200809L

#include <errno.h>
#include <inttypes.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <time.h>
#include <unistd.h>

typedef struct {
  void *data;
  uint64_t length;
} Buffer;

typedef struct {
  int32_t stage;
  uint64_t status;
  uint64_t error_start;
  uint64_t error_end;
  uint64_t node;
  uint64_t related;
  uint64_t token_count;
  uint64_t node_count;
  uint64_t type_count;
  uint64_t symbol_count;
  uint64_t function_count;
} FrontendReport;

extern void xlc_frontend_run(Buffer source, FrontendReport *report);

typedef struct {
  uint32_t state[8];
  uint64_t bit_count;
  uint8_t block[64];
  size_t used;
} Sha256;

static const uint32_t sha256_round_constants[64] = {
    0x428a2f98U, 0x71374491U, 0xb5c0fbcfU, 0xe9b5dba5U,
    0x3956c25bU, 0x59f111f1U, 0x923f82a4U, 0xab1c5ed5U,
    0xd807aa98U, 0x12835b01U, 0x243185beU, 0x550c7dc3U,
    0x72be5d74U, 0x80deb1feU, 0x9bdc06a7U, 0xc19bf174U,
    0xe49b69c1U, 0xefbe4786U, 0x0fc19dc6U, 0x240ca1ccU,
    0x2de92c6fU, 0x4a7484aaU, 0x5cb0a9dcU, 0x76f988daU,
    0x983e5152U, 0xa831c66dU, 0xb00327c8U, 0xbf597fc7U,
    0xc6e00bf3U, 0xd5a79147U, 0x06ca6351U, 0x14292967U,
    0x27b70a85U, 0x2e1b2138U, 0x4d2c6dfcU, 0x53380d13U,
    0x650a7354U, 0x766a0abbU, 0x81c2c92eU, 0x92722c85U,
    0xa2bfe8a1U, 0xa81a664bU, 0xc24b8b70U, 0xc76c51a3U,
    0xd192e819U, 0xd6990624U, 0xf40e3585U, 0x106aa070U,
    0x19a4c116U, 0x1e376c08U, 0x2748774cU, 0x34b0bcb5U,
    0x391c0cb3U, 0x4ed8aa4aU, 0x5b9cca4fU, 0x682e6ff3U,
    0x748f82eeU, 0x78a5636fU, 0x84c87814U, 0x8cc70208U,
    0x90befffaU, 0xa4506cebU, 0xbef9a3f7U, 0xc67178f2U,
};

static uint32_t rotate_right(uint32_t value, unsigned amount) {
  return (value >> amount) | (value << (32U - amount));
}

static uint32_t load_be32(const uint8_t *bytes) {
  return ((uint32_t)bytes[0] << 24U) | ((uint32_t)bytes[1] << 16U) |
         ((uint32_t)bytes[2] << 8U) | (uint32_t)bytes[3];
}

static void store_be32(uint8_t *bytes, uint32_t value) {
  bytes[0] = (uint8_t)(value >> 24U);
  bytes[1] = (uint8_t)(value >> 16U);
  bytes[2] = (uint8_t)(value >> 8U);
  bytes[3] = (uint8_t)value;
}

static void sha256_transform(Sha256 *sha, const uint8_t block[64]) {
  uint32_t words[64];
  for (size_t index = 0; index < 16; index += 1) {
    words[index] = load_be32(block + index * 4U);
  }
  for (size_t index = 16; index < 64; index += 1) {
    uint32_t x = words[index - 15];
    uint32_t y = words[index - 2];
    uint32_t s0 = rotate_right(x, 7U) ^ rotate_right(x, 18U) ^ (x >> 3U);
    uint32_t s1 = rotate_right(y, 17U) ^ rotate_right(y, 19U) ^ (y >> 10U);
    words[index] = words[index - 16] + s0 + words[index - 7] + s1;
  }

  uint32_t a = sha->state[0];
  uint32_t b = sha->state[1];
  uint32_t c = sha->state[2];
  uint32_t d = sha->state[3];
  uint32_t e = sha->state[4];
  uint32_t f = sha->state[5];
  uint32_t g = sha->state[6];
  uint32_t h = sha->state[7];

  for (size_t index = 0; index < 64; index += 1) {
    uint32_t sum1 = rotate_right(e, 6U) ^ rotate_right(e, 11U) ^
                    rotate_right(e, 25U);
    uint32_t choose = (e & f) ^ ((~e) & g);
    uint32_t temporary1 =
        h + sum1 + choose + sha256_round_constants[index] + words[index];
    uint32_t sum0 = rotate_right(a, 2U) ^ rotate_right(a, 13U) ^
                    rotate_right(a, 22U);
    uint32_t majority = (a & b) ^ (a & c) ^ (b & c);
    uint32_t temporary2 = sum0 + majority;
    h = g;
    g = f;
    f = e;
    e = d + temporary1;
    d = c;
    c = b;
    b = a;
    a = temporary1 + temporary2;
  }

  sha->state[0] += a;
  sha->state[1] += b;
  sha->state[2] += c;
  sha->state[3] += d;
  sha->state[4] += e;
  sha->state[5] += f;
  sha->state[6] += g;
  sha->state[7] += h;
}

static void sha256_init(Sha256 *sha) {
  static const uint32_t initial[8] = {
      0x6a09e667U, 0xbb67ae85U, 0x3c6ef372U, 0xa54ff53aU,
      0x510e527fU, 0x9b05688cU, 0x1f83d9abU, 0x5be0cd19U,
  };
  memcpy(sha->state, initial, sizeof(initial));
  sha->bit_count = 0;
  sha->used = 0;
}

static void sha256_update(Sha256 *sha, const uint8_t *bytes, size_t length) {
  if (length > UINT64_MAX / 8U || sha->bit_count > UINT64_MAX - length * 8U) {
    fputs("sha256 input too large\n", stderr);
    exit(2);
  }
  sha->bit_count += (uint64_t)length * 8U;
  while (length != 0) {
    size_t available = sizeof(sha->block) - sha->used;
    size_t take = length < available ? length : available;
    memcpy(sha->block + sha->used, bytes, take);
    sha->used += take;
    bytes += take;
    length -= take;
    if (sha->used == sizeof(sha->block)) {
      sha256_transform(sha, sha->block);
      sha->used = 0;
    }
  }
}

static void sha256_final(Sha256 *sha, uint8_t digest[32]) {
  uint64_t original_bits = sha->bit_count;
  sha->block[sha->used++] = 0x80U;
  if (sha->used > 56U) {
    memset(sha->block + sha->used, 0, sizeof(sha->block) - sha->used);
    sha256_transform(sha, sha->block);
    sha->used = 0;
  }
  memset(sha->block + sha->used, 0, 56U - sha->used);
  for (size_t index = 0; index < 8; index += 1) {
    sha->block[63U - index] = (uint8_t)(original_bits >> (index * 8U));
  }
  sha256_transform(sha, sha->block);
  for (size_t index = 0; index < 8; index += 1) {
    store_be32(digest + index * 4U, sha->state[index]);
  }
}

static void sha256_bytes(const uint8_t *bytes, size_t length,
                         uint8_t digest[32]) {
  Sha256 sha;
  sha256_init(&sha);
  sha256_update(&sha, bytes, length);
  sha256_final(&sha, digest);
}

static void digest_hex(const uint8_t digest[32], char output[65]) {
  static const char digits[] = "0123456789abcdef";
  for (size_t index = 0; index < 32; index += 1) {
    output[index * 2U] = digits[digest[index] >> 4U];
    output[index * 2U + 1U] = digits[digest[index] & 15U];
  }
  output[64] = '\0';
}

static void store_le64(uint8_t output[8], uint64_t value) {
  for (size_t index = 0; index < 8; index += 1) {
    output[index] = (uint8_t)(value >> (index * 8U));
  }
}

static void report_digest(const FrontendReport *report, uint8_t digest[32]) {
  static const uint8_t prefix[] = "xlang-fsoa-frontend-report-v1\0";
  uint64_t fields[11] = {
      (uint64_t)(uint32_t)report->stage,
      report->status,
      report->error_start,
      report->error_end,
      report->node,
      report->related,
      report->token_count,
      report->node_count,
      report->type_count,
      report->symbol_count,
      report->function_count,
  };
  Sha256 sha;
  sha256_init(&sha);
  sha256_update(&sha, prefix, sizeof(prefix) - 1U);
  for (size_t index = 0; index < 11; index += 1) {
    uint8_t encoded[8];
    store_le64(encoded, fields[index]);
    sha256_update(&sha, encoded, sizeof(encoded));
  }
  sha256_final(&sha, digest);
}

static uint8_t *read_file(const char *path, size_t *length_out) {
  FILE *file = fopen(path, "rb");
  if (file == NULL) {
    fprintf(stderr, "could not open %s: %s\n", path, strerror(errno));
    return NULL;
  }
  if (fseek(file, 0, SEEK_END) != 0) {
    fprintf(stderr, "could not seek %s\n", path);
    fclose(file);
    return NULL;
  }
  long end = ftell(file);
  if (end < 0 || fseek(file, 0, SEEK_SET) != 0) {
    fprintf(stderr, "could not size %s\n", path);
    fclose(file);
    return NULL;
  }
  size_t length = (size_t)end;
  if ((long)length != end) {
    fprintf(stderr, "file does not fit size_t: %s\n", path);
    fclose(file);
    return NULL;
  }
  uint8_t *bytes = malloc(length == 0 ? 1U : length);
  if (bytes == NULL) {
    fputs("could not allocate file buffer\n", stderr);
    fclose(file);
    return NULL;
  }
  if (length != 0 && fread(bytes, 1, length, file) != length) {
    fprintf(stderr, "could not read %s\n", path);
    free(bytes);
    fclose(file);
    return NULL;
  }
  if (fclose(file) != 0) {
    fprintf(stderr, "could not close %s\n", path);
    free(bytes);
    return NULL;
  }
  *length_out = length;
  return bytes;
}

static int parse_sample_index(const char *text, uint64_t *value_out) {
  if (*text == '\0' || *text == '-') {
    return 0;
  }
  errno = 0;
  char *end = NULL;
  unsigned long long value = strtoull(text, &end, 10);
  if (errno != 0 || end == text || *end != '\0') {
    return 0;
  }
  *value_out = (uint64_t)value;
  return 1;
}

static uint64_t elapsed_ns(struct timespec before, struct timespec after) {
  uint64_t seconds = (uint64_t)(after.tv_sec - before.tv_sec);
  int64_t nanoseconds = after.tv_nsec - before.tv_nsec;
  if (nanoseconds < 0) {
    seconds -= 1U;
    nanoseconds += 1000000000LL;
  }
  return seconds * 1000000000ULL + (uint64_t)nanoseconds;
}

#ifdef CLOCK_MONOTONIC_RAW
#define SAMPLE_CLOCK CLOCK_MONOTONIC_RAW
#define SAMPLE_CLOCK_NAME "CLOCK_MONOTONIC_RAW"
#else
#define SAMPLE_CLOCK CLOCK_MONOTONIC
#define SAMPLE_CLOCK_NAME "CLOCK_MONOTONIC"
#endif

int main(int argc, char **argv) {
  if (argc != 11 || strcmp(argv[1], "--corpus") != 0 ||
      strcmp(argv[3], "--expected-source-sha256") != 0 ||
      strcmp(argv[5], "--expected-executable-sha256") != 0 ||
      strcmp(argv[7], "--mode") != 0 ||
      strcmp(argv[9], "--sample-index") != 0) {
    fputs("usage: fsoa_sample --corpus PATH --expected-source-sha256 HEX "
          "--expected-executable-sha256 HEX --mode smoke|score "
          "--sample-index N\n",
          stderr);
    return 2;
  }
  const char *corpus_path = argv[2];
  const char *expected_source_sha256 = argv[4];
  const char *expected_executable_sha256 = argv[6];
  const char *mode = argv[8];
  uint64_t sample_index = 0;
  if (strlen(expected_source_sha256) != 64U ||
      strlen(expected_executable_sha256) != 64U ||
      (strcmp(mode, "smoke") != 0 && strcmp(mode, "score") != 0) ||
      !parse_sample_index(argv[10], &sample_index)) {
    fputs("invalid driver argument\n", stderr);
    return 2;
  }

  size_t executable_length = 0;
  uint8_t *executable_bytes = read_file(argv[0], &executable_length);
  if (executable_bytes == NULL) {
    return 2;
  }
  uint8_t executable_digest[32];
  char executable_hex[65];
  sha256_bytes(executable_bytes, executable_length, executable_digest);
  digest_hex(executable_digest, executable_hex);
  free(executable_bytes);
  if (strcmp(executable_hex, expected_executable_sha256) != 0) {
    fprintf(stderr, "executable hash mismatch: expected %s observed %s\n",
            expected_executable_sha256, executable_hex);
    return 3;
  }

  size_t corpus_length = 0;
  uint8_t *corpus = read_file(corpus_path, &corpus_length);
  if (corpus == NULL) {
    return 2;
  }
  uint8_t source_digest[32];
  char source_hex[65];
  sha256_bytes(corpus, corpus_length, source_digest);
  digest_hex(source_digest, source_hex);
  if (strcmp(source_hex, expected_source_sha256) != 0) {
    fprintf(stderr, "source hash mismatch: expected %s observed %s\n",
            expected_source_sha256, source_hex);
    free(corpus);
    return 3;
  }

  Buffer source = {corpus, (uint64_t)corpus_length};
  FrontendReport report;
  memset(&report, 0x5a, sizeof(report));
  struct timespec before;
  struct timespec after;
  if (clock_gettime(SAMPLE_CLOCK, &before) != 0) {
    fputs("clock_gettime before failed\n", stderr);
    free(corpus);
    return 2;
  }
  xlc_frontend_run(source, &report);
  if (clock_gettime(SAMPLE_CLOCK, &after) != 0) {
    fputs("clock_gettime after failed\n", stderr);
    free(corpus);
    return 2;
  }
  uint64_t duration = elapsed_ns(before, after);

  uint8_t correctness_digest[32];
  char correctness_hex[65];
  report_digest(&report, correctness_digest);
  digest_hex(correctness_digest, correctness_hex);

  int not_a_score = strcmp(mode, "smoke") == 0;
  printf(
      "{\"schema_version\":1,\"kind\":\"f-soa-baseline-sample\","
      "\"variant\":\"F-SOA\",\"phase\":\"cold-wrapper\","
      "\"mode\":\"%s\",\"not_a_score\":%s,\"sample_index\":%" PRIu64
      ",\"pid\":%ld,\"clock\":\"%s\",\"corpus_bytes\":%zu,"
      "\"corpus_sha256\":\"%s\",\"executable_sha256\":\"%s\","
      "\"elapsed_ns\":%" PRIu64
      ",\"report\":{\"stage\":%d,\"status\":%" PRIu64
      ",\"error_start\":%" PRIu64 ",\"error_end\":%" PRIu64
      ",\"node\":%" PRIu64 ",\"related\":%" PRIu64
      ",\"token_count\":%" PRIu64 ",\"node_count\":%" PRIu64
      ",\"type_count\":%" PRIu64 ",\"symbol_count\":%" PRIu64
      ",\"function_count\":%" PRIu64
      "},\"correctness_schema\":\"frontend-report-le-v1\","
      "\"correctness_sha256\":\"%s\"}\n",
      mode, not_a_score ? "true" : "false", sample_index, (long)getpid(),
      SAMPLE_CLOCK_NAME, corpus_length, source_hex, executable_hex, duration,
      report.stage, report.status, report.error_start, report.error_end,
      report.node, report.related, report.token_count, report.node_count,
      report.type_count, report.symbol_count, report.function_count,
      correctness_hex);
  if (fflush(stdout) != 0) {
    fputs("stdout flush failed\n", stderr);
    free(corpus);
    return 2;
  }

  free(corpus);
  return 0;
}
