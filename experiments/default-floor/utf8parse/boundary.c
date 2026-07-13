#define _DARWIN_C_SOURCE

#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <sys/wait.h>
#include <unistd.h>

typedef struct {
  uint8_t *p;
  int64_t n;
} Buf;

typedef uint64_t (*parse_fn)(Buf, Buf);

extern uint64_t xlang_parse_facts(Buf out, Buf src);
extern uint64_t xlang_parse_nofacts(Buf out, Buf src);

enum check_result {
  CHECK_OK = 0,
  CHECK_CANDIDATE_FAILURE = 1,
  CHECK_HARNESS_FAILURE = 2,
};

enum {
  INVALID_EVENT = 0x00110000,
  OUTPUT_WORDS = 1024,
  OUTPUT_GUARD_WORDS = 32,
  SOURCE_STORAGE_BYTES = 4096,
  CONTROL_STORAGE_BYTES = 4096,
};

static const uint32_t OUTPUT_CANARY = UINT32_C(0xA5A5A5A5);

struct oracle_state {
  uint8_t pending[4];
  size_t pending_len;
  size_t expected_len;
};

static void oracle_reset(struct oracle_state *state) {
  state->pending_len = 0;
  state->expected_len = 0;
}

static int continuation(uint8_t byte, uint8_t low, uint8_t high) {
  return byte >= low && byte <= high;
}

static void oracle_event(uint32_t *out, size_t *produced, uint32_t event) {
  out[*produced] = event;
  *produced += 1;
}

/* Independent pending-byte oracle; it does not share the crate's state table. */
static size_t oracle_parse(uint32_t *out, const uint8_t *src, size_t len) {
  struct oracle_state state = {{0, 0, 0, 0}, 0, 0};
  size_t produced = 0;
  for (size_t input = 0; input < len; ++input) {
    uint8_t byte = src[input];
    if (state.pending_len == 0) {
      if (byte <= 0x7F) {
        oracle_event(out, &produced, byte);
      } else if (byte >= 0xC2 && byte <= 0xDF) {
        state.pending[0] = byte;
        state.pending_len = 1;
        state.expected_len = 2;
      } else if (byte >= 0xE0 && byte <= 0xEF) {
        state.pending[0] = byte;
        state.pending_len = 1;
        state.expected_len = 3;
      } else if (byte >= 0xF0 && byte <= 0xF4) {
        state.pending[0] = byte;
        state.pending_len = 1;
        state.expected_len = 4;
      } else {
        oracle_event(out, &produced, INVALID_EVENT);
      }
      continue;
    }

    uint8_t lead = state.pending[0];
    int acceptable;
    if (state.pending_len != 1) {
      acceptable = continuation(byte, 0x80, 0xBF);
    } else if (lead == 0xE0) {
      acceptable = continuation(byte, 0xA0, 0xBF);
    } else if (lead == 0xED) {
      acceptable = continuation(byte, 0x80, 0x9F);
    } else if (lead == 0xF0) {
      acceptable = continuation(byte, 0x90, 0xBF);
    } else if (lead == 0xF4) {
      acceptable = continuation(byte, 0x80, 0x8F);
    } else {
      acceptable = continuation(byte, 0x80, 0xBF);
    }

    if (!acceptable) {
      oracle_reset(&state);
      oracle_event(out, &produced, INVALID_EVENT);
      continue;
    }

    state.pending[state.pending_len++] = byte;
    if (state.pending_len != state.expected_len) {
      continue;
    }

    uint32_t point;
    if (state.expected_len == 2) {
      point = ((uint32_t)(state.pending[0] & 0x1F) << 6) |
              (uint32_t)(state.pending[1] & 0x3F);
    } else if (state.expected_len == 3) {
      point = ((uint32_t)(state.pending[0] & 0x0F) << 12) |
              ((uint32_t)(state.pending[1] & 0x3F) << 6) |
              (uint32_t)(state.pending[2] & 0x3F);
    } else {
      point = ((uint32_t)(state.pending[0] & 0x07) << 18) |
              ((uint32_t)(state.pending[1] & 0x3F) << 12) |
              ((uint32_t)(state.pending[2] & 0x3F) << 6) |
              (uint32_t)(state.pending[3] & 0x3F);
    }
    oracle_reset(&state);
    oracle_event(out, &produced, point);
  }
  return produced;
}

static void print_bytes(const uint8_t *bytes, size_t len) {
  for (size_t i = 0; i < len; ++i) {
    fprintf(stderr, "%02x", bytes[i]);
  }
}

static void print_events(const uint32_t *events, size_t len) {
  for (size_t i = 0; i < len; ++i) {
    fprintf(stderr, "%s%06x", i == 0 ? "" : ",", events[i]);
  }
}

static int harness_failure(const char *operation) {
  fprintf(stderr, "HARNESS: capacity subprocess %s failed\n", operation);
  return CHECK_HARNESS_FAILURE;
}

static void print_actual_termination(int status) {
  if (WIFSIGNALED(status)) {
    fprintf(stderr, "signal-%d", WTERMSIG(status));
  } else if (WIFEXITED(status)) {
    fprintf(stderr, "exit-%d", WEXITSTATUS(status));
  } else {
    fprintf(stderr, "unknown-process-status");
  }
}

static int run_child(const char *parser_name, parse_fn parse,
                     unsigned case_index, const uint8_t *src_data,
                     int64_t src_len, int64_t out_len, int expect_success) {
  const size_t output_storage_bytes = OUTPUT_WORDS * sizeof(uint32_t);
  const size_t mapping_len = output_storage_bytes + SOURCE_STORAGE_BYTES +
                             CONTROL_STORAGE_BYTES;
  uint32_t expected_output[32];
  uint32_t actual_prefix[32];
  if (src_len < 0 || (size_t)src_len > 32) {
    return harness_failure("locked source size");
  }
  size_t expected_len =
      oracle_parse(expected_output, src_data, (size_t)src_len);

  uint8_t *mapping = mmap(NULL, mapping_len, PROT_READ | PROT_WRITE,
                          MAP_SHARED | MAP_ANON, -1, 0);
  if (mapping == MAP_FAILED) {
    return harness_failure("mmap");
  }
  uint32_t *output_storage = (uint32_t *)mapping;
  uint32_t *out = output_storage + OUTPUT_GUARD_WORDS;
  uint8_t *source = mapping + output_storage_bytes;
  uint8_t *control = source + SOURCE_STORAGE_BYTES;
  uint64_t *returned = (uint64_t *)control;
  uint8_t *returned_set = control + sizeof(*returned);
  for (size_t i = 0; i < OUTPUT_WORDS; ++i) {
    output_storage[i] = OUTPUT_CANARY;
  }
  memset(source, 0x5A, SOURCE_STORAGE_BYTES);
  memset(control, 0, CONTROL_STORAGE_BYTES);
  memcpy(source, src_data, (size_t)src_len);

  pid_t child = fork();
  if (child < 0) {
    (void)munmap(mapping, mapping_len);
    return harness_failure("fork");
  }
  if (child == 0) {
    Buf src = {source, src_len};
    Buf output = {(uint8_t *)out, out_len};
    *returned = parse(output, src);
    *returned_set = 1;
    _exit(expect_success ? 0 : 99);
  }

  int status = 0;
  pid_t waited;
  do {
    waited = waitpid(child, &status, 0);
  } while (waited < 0 && errno == EINTR);
  if (waited != child) {
    (void)munmap(mapping, mapping_len);
    return harness_failure("waitpid");
  }

  int sentinel_unchanged = 1;
  int source_unchanged = 1;
  int prefix_matches = 1;
  int returned_matches = 1;
  uint64_t returned_value = *returned;
  int return_available = *returned_set != 0;
  if (expect_success) {
    memcpy(actual_prefix, out, expected_len * sizeof(uint32_t));
    prefix_matches =
        memcmp(out, expected_output, expected_len * sizeof(uint32_t)) == 0;
    returned_matches =
        return_available && returned_value == (uint64_t)expected_len;
    for (size_t i = 0; i < OUTPUT_GUARD_WORDS; ++i) {
      if (output_storage[i] != OUTPUT_CANARY) {
        sentinel_unchanged = 0;
      }
    }
    for (size_t i = OUTPUT_GUARD_WORDS + expected_len; i < OUTPUT_WORDS; ++i) {
      if (output_storage[i] != OUTPUT_CANARY) {
        sentinel_unchanged = 0;
      }
    }
  } else {
    for (size_t i = 0; i < OUTPUT_WORDS; ++i) {
      if (output_storage[i] != OUTPUT_CANARY) {
        sentinel_unchanged = 0;
      }
    }
  }
  for (size_t i = 0; i < SOURCE_STORAGE_BYTES; ++i) {
    uint8_t expected =
        i < (size_t)src_len ? src_data[i] : (uint8_t)0x5A;
    if (source[i] != expected) {
      source_unchanged = 0;
    }
  }

  if (munmap(mapping, mapping_len) != 0) {
    return harness_failure("munmap");
  }

  int termination_matches = expect_success
                                ? WIFEXITED(status) && WEXITSTATUS(status) == 0
                                : WIFSIGNALED(status);
  if (!termination_matches || !sentinel_unchanged || !source_unchanged ||
      (expect_success && (!returned_matches || !prefix_matches))) {
    fprintf(stderr, "%s/capacity: case=%u input=", parser_name, case_index);
    print_bytes(src_data, (size_t)src_len);
    fprintf(stderr, " output_capacity=%lld expected=%s actual=",
            (long long)out_len,
            expect_success ? "success" : "trapped-before-write");
    print_actual_termination(status);
    if (return_available) {
      fprintf(stderr, " returned=%llu", (unsigned long long)returned_value);
    } else {
      fprintf(stderr, " returned=unavailable");
    }
    if (expect_success) {
      fprintf(stderr, " expected_length=%zu expected_events=", expected_len);
      print_events(expected_output, expected_len);
      fprintf(stderr, " actual_prefix=");
      print_events(actual_prefix, expected_len);
      fprintf(stderr, " prefix_matches=%s",
              prefix_matches ? "true" : "false");
    }
    fprintf(stderr, " sentinel_unchanged=%s source_unchanged=%s\n",
            sentinel_unchanged ? "true" : "false",
            source_unchanged ? "true" : "false");
    return CHECK_CANDIDATE_FAILURE;
  }
  return CHECK_OK;
}

static int check_source(const char *parser_name, parse_fn parse,
                        unsigned case_index, const uint8_t *src, int64_t len) {
  for (int64_t capacity = 0; capacity < len; ++capacity) {
    int rejected = run_child(parser_name, parse, case_index, src, len,
                             capacity, 0);
    if (rejected != CHECK_OK) {
      return rejected;
    }
  }
  return run_child(parser_name, parse, case_index, src, len, len, 1);
}

static int check_parser(const char *parser_name, parse_fn parse) {
  static const uint8_t s0[] = {0};
  static const uint8_t s1[] = {'A'};
  static const uint8_t s2[] = {0xC2, 0xA2};
  static const uint8_t s3[] = {0xF0, 0x9F, 0x92, 0xA9};
  static const uint8_t s4[] = {0x80};
  static const uint8_t s5[] = {0xC2, 'A'};
  static const uint8_t s6[] = {0xE0, 0x9F, 'A'};
  static const uint8_t s7[] = {'A', 0xF0, 0x90, 0x80};
  static const uint8_t s8[] = {'A', 0xC2, 0xA2, 0xFF, 'Z'};
  struct {
    const uint8_t *src;
    int64_t len;
  } cases[] = {
      {s0, 0},          {s1, (int64_t)sizeof(s1)},
      {s2, (int64_t)sizeof(s2)}, {s3, (int64_t)sizeof(s3)},
      {s4, (int64_t)sizeof(s4)}, {s5, (int64_t)sizeof(s5)},
      {s6, (int64_t)sizeof(s6)}, {s7, (int64_t)sizeof(s7)},
      {s8, (int64_t)sizeof(s8)},
  };
  for (unsigned i = 0; i < sizeof(cases) / sizeof(cases[0]); ++i) {
    int status = check_source(parser_name, parse, i, cases[i].src,
                              cases[i].len);
    if (status != CHECK_OK) {
      return status;
    }
  }
  return CHECK_OK;
}

int main(void) {
  int facts = check_parser("facts-on", xlang_parse_facts);
  if (facts != CHECK_OK) {
    return facts;
  }
  int nofacts = check_parser("facts-off", xlang_parse_nofacts);
  if (nofacts != CHECK_OK) {
    return nofacts;
  }
  return CHECK_OK;
}
