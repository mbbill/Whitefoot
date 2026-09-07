/* Portable operation-trace qualification for the optional client observer.
 * The Linux client/stream checks separately exercise real socket outcomes. */
#include "netload_observe.h"
#include <assert.h>

int main(void) {
    struct netload_observation count = {0};
    /* Two complete 64 KiB exchanges; one send/receive is fragmented. Empty
     * probes carry requested bytes, but do not increase transferred bytes. */
    netload_observe_transfer(&count.send, 65536, 65536);
    netload_observe_transfer(&count.send, 65536, 8192);
    netload_observe_transfer(&count.send, 57344, -1);
    netload_observe_transfer(&count.send, 57344, 57344);
    netload_observe_transfer(&count.receive, 65536, -1);
    netload_observe_transfer(&count.receive, 65536, 32768);
    netload_observe_transfer(&count.receive, 32768, 32768);
    netload_observe_transfer(&count.receive, 65536, 65536);
    netload_observe_poll(&count, 2);
    netload_observe_poll(&count, 0);
    netload_observe_poll(&count, -1);
    count.input = 1;
    count.output = 2;
    count.both = 1;
    count.skipped = 1;
    count.dispatched = 1;
    count.verified = 2;
    count.verified_bytes = 131072;
    assert(netload_observation_conserved(&count, 2, 131072));
    assert(count.send.calls == 4 && count.send.positive == 3 && count.send.short_count == 1);
    assert(count.send.requested == 245760 && count.send.bytes == 131072);
    assert(count.send.sizes[4] == 1 && count.send.sizes[7] == 2);
    assert(count.receive.calls == 4 && count.receive.short_count == 1);
    assert(count.receive.sizes[6] == 2 && count.receive.sizes[7] == 1);
    assert(count.ready == 1 && count.batches[1] == 1 && count.maximum_batch == 2);

    /* Corrupt each independent conservation edge; none may certify a run. */
    struct netload_observation bad = count;
    bad.send.bytes--;
    assert(!netload_observation_conserved(&bad, 2, 131072));
    bad = count; bad.receive.sizes[6]--;
    assert(!netload_observation_conserved(&bad, 2, 131072));
    bad = count; bad.verified--;
    assert(!netload_observation_conserved(&bad, 2, 131072));
    bad = count; bad.dispatched--;
    assert(!netload_observation_conserved(&bad, 2, 131072));
    bad = count; bad.batches[1]--;
    assert(!netload_observation_conserved(&bad, 2, 131072));
    bad = count; bad.both = 2;
    assert(!netload_observation_conserved(&bad, 2, 131072));
    netload_observe_transfer(&bad.receive, 1, 0);
    assert(bad.receive.zero == 1 && netload_transfer_conserved(&bad.receive));
    netload_observe_transfer(&bad.send, 1, -2);
    assert(bad.send.error == 1 && netload_transfer_conserved(&bad.send));
    netload_observe_poll(&bad, -2);
    assert(bad.poll_errors == 1);
    assert(!netload_observation_conserved(&bad, 2, 131072));

    struct netload_transfer boundaries = {0};
    const uint64_t sizes[] = {1, 2, 64, 65, 512, 513, 4096, 4097, 8192,
        8193, 16384, 16385, 32768, 32769, 65536, 65537};
    for (unsigned at = 0; at < sizeof sizes / sizeof sizes[0]; at++)
        netload_observe_transfer(&boundaries, sizes[at], (int64_t)sizes[at]);
    assert(netload_transfer_conserved(&boundaries));
    for (unsigned at = 0; at < NETLOAD_SIZE_BUCKETS; at++)
        assert(boundaries.sizes[at] == (at == 0 || at == 8 ? 1u : 2u));
    puts("netload observer: fragmented/full transfers, probes, errors and conservation PASS");
    return 0;
}
