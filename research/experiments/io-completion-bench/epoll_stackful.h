/* Sequential handlers for the owner-local epoll representation experiment.
 * Included by epoll_echo.c only with WF_BENCH_STACKFUL. The listener, event
 * polling, FIFO, buffers and compute protocol are the reference's unchanged
 * engine; automatic locals and nested calls replace its explicit continuation
 * fields. Retire this variant when the representation comparison is settled.
 *
 * A connection never migrates. No callback can resume it until its owner has
 * returned from the switch; no descriptor is closed while its stack runs. */

static void connection_wait(struct connection *link) {
#if defined(WF_BENCH_OBSERVE)
    link->owner->waits++;
#endif
    wf_switch_raw(&link->saved_sp, link->owner->saved_sp);
}

#if defined(WF_BENCH_QUANTUM)
static void connection_yield(struct connection *link) {
    enqueue(link->owner, link->descriptor);
#if defined(WF_BENCH_OBSERVE)
    link->owner->yields++;
#endif
    wf_switch_raw(&link->saved_sp, link->owner->saved_sp);
}
#endif

/* A receive's offset is an ordinary local spanning any number of waits. */
#if defined(WF_BENCH_COMPUTE)
static int receive_request(struct connection *link) {
    unsigned received = 0;
    while (received < COMPUTE_BYTES) {
        ssize_t taken = recv(link->descriptor, link->pending + received,
                             COMPUTE_BYTES - received, 0);
        if (taken > 0) {
            received += (unsigned)taken;
        } else if (taken < 0 && errno == EINTR) {
            continue;
        } else if (taken < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
            connection_wait(link);
        } else {
            if (taken == 0 && received != 0) mark_failed();
            return 0;
        }
    }
    return 1;
}
#endif

/* With shared receive scratch, copy the unsent remainder before yielding so
 * another connection may reuse it, just as the manual reference does. Private
 * receive storage is already link->pending and needs no copy across a wait. */
static int send_response(struct connection *link, unsigned char *source, size_t length) {
    size_t offset = 0;
    while (offset < length) {
        ssize_t moved = send(link->descriptor, source + offset, length - offset, MSG_NOSIGNAL);
        if (moved > 0) {
            offset += (size_t)moved;
        } else if (moved < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
#if defined(WF_BENCH_OBSERVE)
            link->owner->send_waits++;
#endif
            if (source != link->pending) {
                length -= offset;
                memcpy(link->pending, source + offset, length);
                source = link->pending;
                offset = 0;
            }
            connection_wait(link);
        } else {
            return 0;
        }
    }
    return 1;
}

static void connection_main(void *raw) {
    struct connection *link = (struct connection *)raw;
    for (;;) {
#if defined(WF_BENCH_COMPUTE)
        if (!receive_request(link)) break;
#if defined(WF_BENCH_QUANTUM)
        uint64_t value = compute_decode(link->pending);
        uint64_t remaining = compute_decode(link->pending + 8);
        if (remaining > COMPUTE_MAX_ROUNDS) {
            mark_failed();
            break;
        }
        while (remaining != 0) {
            /* The manual handler queues a nonzero request before its first
             * chunk and after each unfinished chunk. Keep those same turns. */
            connection_yield(link);
            uint64_t steps = remaining < option_quantum ? remaining : option_quantum;
            value = compute_churn(value, steps);
            remaining -= steps;
        }
        compute_encode(link->pending, value);
#else
        if (!compute_response(link->pending)) {
            mark_failed();
            break;
        }
#endif
        if (!send_response(link, link->pending, COMPUTE_BYTES)) break;
#if defined(WF_BENCH_QUANTUM)
        /* A service turn owns this budget; every resume resets it, including
         * resumes inside receive_request and send_response. This is the
         * manual handler's per-invocation local reply counter. */
        if (++link->owner->replies == 8u) connection_yield(link);
#endif
#else
        ssize_t taken = recv(link->descriptor, WF_BENCH_RECEIVE_BUFFER(link->owner, link), TRANSFER_BYTES, 0);
        if (taken > 0) {
            if (!send_response(link, WF_BENCH_RECEIVE_BUFFER(link->owner, link), (size_t)taken)) break;
        } else if (taken < 0 && errno == EINTR) {
            continue;
        } else if (taken < 0 && (errno == EAGAIN || errno == EWOULDBLOCK)) {
            connection_wait(link);
        } else {
            break;
        }
#endif
    }
    link->done = 1;
    wf_switch_raw(&link->saved_sp, link->owner->saved_sp);
    /* The owner closes only after returning from this final switch. */
    abort();
}

static void service(struct worker *worker, int descriptor) {
    struct connection *link = &table[descriptor];
#if defined(WF_BENCH_QUANTUM)
    worker->replies = 0;
#endif
#if defined(WF_BENCH_OBSERVE)
    worker->resumes++;
#endif
    wf_switch_raw(&worker->saved_sp, link->saved_sp);
    if (link->done) close_connection(worker, descriptor);
}
