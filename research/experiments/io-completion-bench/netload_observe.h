/* Observer-only client accounting, owned by io-model experiment 56.
 * No clocks, allocation or system calls occur while recording an operation.
 * Remove with the client diagnostic when the generator limit is resolved. */
#ifndef NETLOAD_OBSERVE_H
#define NETLOAD_OBSERVE_H

#include <stdint.h>
#include <stdio.h>

enum { NETLOAD_SIZE_BUCKETS = 9, NETLOAD_BATCH_BUCKETS = 4 };
static const uint64_t netload_size_limits[NETLOAD_SIZE_BUCKETS - 1] = {
    1, 64, 512, 4096, 8192, 16384, 32768, 65536
};

struct netload_transfer {
    uint64_t calls, positive, short_count, again, zero, error;
    uint64_t requested, bytes, maximum, sizes[NETLOAD_SIZE_BUCKETS];
};

struct netload_observation {
    struct netload_transfer send, receive;
    uint64_t pumps, verified, verified_bytes;
    uint64_t polls, ready, empty, interrupted, poll_errors;
    uint64_t events, input, output, both, errors, hangups, read_hangups;
    uint64_t skipped, dispatched, maximum_batch, batches[NETLOAD_BATCH_BUCKETS];
};

/* outcome: positive byte count, zero, -1 for EAGAIN, -2 for another error. */
static inline void netload_observe_transfer(struct netload_transfer *count,
                                           uint64_t requested, int64_t outcome) {
    count->calls++;
    count->requested += requested;
    if (outcome > 0) {
        uint64_t bytes = (uint64_t)outcome;
        count->positive++;
        count->bytes += bytes;
        count->short_count += bytes < requested;
        if (bytes > count->maximum) count->maximum = bytes;
        unsigned bucket = 0;
        while (bucket + 1 < NETLOAD_SIZE_BUCKETS && bytes > netload_size_limits[bucket]) bucket++;
        count->sizes[bucket]++;
    } else if (outcome == 0) count->zero++;
    else if (outcome == -1) count->again++;
    else count->error++;
}

/* result: epoll's nonnegative event count, -1 for EINTR, -2 for another error. */
static inline void netload_observe_poll(struct netload_observation *count, int result) {
    count->polls++;
    if (result > 0) {
        uint64_t batch = (uint64_t)result;
        count->ready++;
        count->events += batch;
        if (batch > count->maximum_batch) count->maximum_batch = batch;
        count->batches[batch == 1 ? 0 : batch <= 8 ? 1 : batch <= 64 ? 2 : 3]++;
    } else if (result == 0) count->empty++;
    else if (result == -1) count->interrupted++;
    else count->poll_errors++;
}

static inline int netload_transfer_conserved(const struct netload_transfer *count) {
    uint64_t sizes = 0;
    for (unsigned at = 0; at < NETLOAD_SIZE_BUCKETS; at++) sizes += count->sizes[at];
    return count->calls == count->positive + count->again + count->zero + count->error &&
        sizes == count->positive && count->short_count <= count->positive &&
        count->requested >= count->bytes && count->maximum <= count->bytes;
}

static inline int netload_observation_conserved(const struct netload_observation *count,
                                               uint64_t rounds, uint64_t bytes) {
    uint64_t batches = 0;
    for (unsigned at = 0; at < NETLOAD_BATCH_BUCKETS; at++) batches += count->batches[at];
    return netload_transfer_conserved(&count->send) && netload_transfer_conserved(&count->receive) &&
        !count->send.zero && !count->send.error && !count->receive.zero && !count->receive.error &&
        count->send.bytes == bytes && count->receive.bytes == bytes &&
        count->verified == rounds && count->verified_bytes == bytes &&
        count->polls == count->ready + count->empty + count->interrupted + count->poll_errors &&
        !count->poll_errors && batches == count->ready && count->maximum_batch <= count->events &&
        count->skipped + count->dispatched == count->events &&
        count->both <= count->input && count->both <= count->output &&
        count->input + count->output - count->both <= count->events &&
        count->errors <= count->events && count->hangups <= count->events &&
        count->read_hangups <= count->events;
}

static inline void netload_report_transfer(FILE *file, const char *name,
                                          const struct netload_transfer *count) {
#define NETLOAD_TRANSFER_FIELD(label, member) \
    fprintf(file, " %s_" label "=%llu", name, (unsigned long long)count->member)
    NETLOAD_TRANSFER_FIELD("calls", calls);
    NETLOAD_TRANSFER_FIELD("positive", positive);
    NETLOAD_TRANSFER_FIELD("short", short_count);
    NETLOAD_TRANSFER_FIELD("again", again);
    NETLOAD_TRANSFER_FIELD("zero", zero);
    NETLOAD_TRANSFER_FIELD("error", error);
    NETLOAD_TRANSFER_FIELD("requested", requested);
    NETLOAD_TRANSFER_FIELD("bytes", bytes);
    NETLOAD_TRANSFER_FIELD("maximum", maximum);
#undef NETLOAD_TRANSFER_FIELD
    for (unsigned at = 0; at < NETLOAD_SIZE_BUCKETS; at++)
        fprintf(file, " %s_size%u=%llu", name, at, (unsigned long long)count->sizes[at]);
}

static inline void netload_report_observation(FILE *file, uint64_t worker, const char *phase,
                                             const struct netload_observation *count) {
    fprintf(file, "netload-observe worker=%llu phase=%s", (unsigned long long)worker, phase);
    netload_report_transfer(file, "send", &count->send);
    netload_report_transfer(file, "recv", &count->receive);
#define NETLOAD_OBSERVATION_FIELD(member) \
    fprintf(file, " " #member "=%llu", (unsigned long long)count->member)
    NETLOAD_OBSERVATION_FIELD(pumps);
    NETLOAD_OBSERVATION_FIELD(verified);
    NETLOAD_OBSERVATION_FIELD(verified_bytes);
    NETLOAD_OBSERVATION_FIELD(polls);
    NETLOAD_OBSERVATION_FIELD(ready);
    NETLOAD_OBSERVATION_FIELD(empty);
    NETLOAD_OBSERVATION_FIELD(interrupted);
    NETLOAD_OBSERVATION_FIELD(poll_errors);
    NETLOAD_OBSERVATION_FIELD(events);
    NETLOAD_OBSERVATION_FIELD(input);
    NETLOAD_OBSERVATION_FIELD(output);
    NETLOAD_OBSERVATION_FIELD(both);
    NETLOAD_OBSERVATION_FIELD(errors);
    NETLOAD_OBSERVATION_FIELD(hangups);
    NETLOAD_OBSERVATION_FIELD(read_hangups);
    NETLOAD_OBSERVATION_FIELD(skipped);
    NETLOAD_OBSERVATION_FIELD(dispatched);
    NETLOAD_OBSERVATION_FIELD(maximum_batch);
#undef NETLOAD_OBSERVATION_FIELD
    for (unsigned at = 0; at < NETLOAD_BATCH_BUCKETS; at++)
        fprintf(file, " batch%u=%llu", at, (unsigned long long)count->batches[at]);
    fputc('\n', file);
}
#endif
