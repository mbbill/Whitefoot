#ifndef WHITEFOOT_COMPLETION_FILE_POSIX_H
#define WHITEFOOT_COMPLETION_FILE_POSIX_H

/*
 * The POSIX file leaf's own vocabulary: the rules that read a host `stat` mode
 * and a host open flag, and the WF_IO_NOCACHE policy's one host call.
 *
 * These were inline in `file_adapter.h` while that header was POSIX's alone.
 * They are here now because they name `<sys/stat.h>` and `<fcntl.h>`, and the
 * adapter's own header is shared with a platform that has neither.  Their two
 * readers are both POSIX leaves: `file_posix.c`, the adapter's per-operation
 * host switch, and `linux_io_uring.c`, which decides a ring-completed open's
 * kind with the same rule so that one open states one contract whichever
 * engine ran it.
 */

#include "contract.h"

#include <arpa/inet.h>
#include <fcntl.h>
#include <netinet/in.h>
#include <stdatomic.h>
#include <stdlib.h>
#include <string.h>
#include <sys/socket.h>
#include <sys/stat.h>

#if defined(__cplusplus)
extern "C" {
#endif

/* The one rule deciding whether an opened descriptor is the kind the
 * operation asked for.  Every POSIX engine answers with this function, so a
 * FIFO is refused identically whether the open ran on a helper thread, on the
 * joining thread itself, or on a kernel completion ring.  `file_mode` is the
 * host mode word of the descriptor that was actually opened, never of a path
 * inspected a second time. */
static inline enum wf_file_open_outcome wf_file_kind_outcome(
    enum wf_file_expected_kind expected,
    unsigned int file_mode
) {
    if (expected == WF_FILE_EXPECT_ANY
        || (expected == WF_FILE_EXPECT_REGULAR && S_ISREG(file_mode))
        || (expected == WF_FILE_EXPECT_DIRECTORY && S_ISDIR(file_mode))) {
        return WF_FILE_OPEN_SUCCEEDED;
    }
    return S_ISDIR(file_mode) ? WF_FILE_OPEN_IS_DIRECTORY
                              : WF_FILE_OPEN_OTHER_KIND;
}

/* The extra open flag a kind-checked open needs so that opening a waiting
 * facility such as a FIFO cannot block before the kind is known. */
static inline int wf_file_open_kind_flags(enum wf_file_expected_kind expected) {
    return expected == WF_FILE_EXPECT_REGULAR ? O_NONBLOCK : 0;
}

/* ------------------------------------------------------------- addresses */

/* The two host calls a socket request makes beyond the transfer ones need one
 * address in the host's own record and one back, and both POSIX engines make
 * them: `file_posix.c` for the queued and inline routes, `linux_io_uring.c`
 * for the ring's connect and accept.  So the two conversions live here, with
 * the kind rule above, and one address states one contract whichever engine
 * ran the operation.
 *
 * `socklen_t` is the host's own length type and the record carries `unsigned`,
 * because the record is one ABI on every platform and `<sys/socket.h>` does
 * not reach it.  The assertion is what keeps the two the same fact; the ring
 * hands the record's field straight to the kernel as a `socklen_t *`. */
_Static_assert(
    sizeof(socklen_t) == sizeof(unsigned),
    "the record's address length must be the host's own socklen_t"
);
_Static_assert(
    sizeof(struct sockaddr_in6) <= sizeof(wf_socket_native_address),
    "the record's native address storage must hold the larger of the two families"
);
_Static_assert(
    sizeof(struct sockaddr_in) <= sizeof(wf_socket_native_address),
    "the record's native address storage must hold an IPv4 address"
);

/* The byte of one portable address at `index`, by the rule the emitted
 * constructors write it with: byte `i` occupies bits `8 * (i % 8)` of word
 * `i / 8`, which reads the same on either endianness. */
static inline unsigned char wf_socket_address_byte(
    const wf_socket_address *address,
    unsigned index
) {
    return (unsigned char)((address->words[index / 8u] >> (8u * (index % 8u)))
                           & 0xffu);
}

static inline void wf_socket_address_set_byte(
    wf_socket_address *address,
    unsigned index,
    unsigned char value
) {
    unsigned shift = 8u * (index % 8u);
    address->words[index / 8u] &= ~((uint64_t)0xffu << shift);
    address->words[index / 8u] |= (uint64_t)value << shift;
}

/* Builds the host's own record from one portable address, and reports its
 * length.  The family is one bit of the value, so there are exactly two
 * records to build and every address builds one.
 *
 * The port is carried in host order in the portable value and put into
 * network order here, which is the one place either representation is
 * converted. */
static inline unsigned wf_socket_native_from_address(
    const wf_socket_address *address,
    wf_socket_native_address *native
) {
    unsigned port = address->port_and_family & WF_SOCKET_PORT_MASK;
    memset(native, 0, sizeof(*native));
    if ((address->port_and_family & WF_SOCKET_FAMILY_V6) != 0u) {
        struct sockaddr_in6 host;
        unsigned index;
        memset(&host, 0, sizeof(host));
        host.sin6_family = AF_INET6;
        host.sin6_port = htons((unsigned short)port);
        for (index = 0; index < 16u; index += 1u) {
            host.sin6_addr.s6_addr[index] =
                wf_socket_address_byte(address, index);
        }
        memcpy(native->bytes, &host, sizeof(host));
        return (unsigned)sizeof(host);
    } else {
        struct sockaddr_in host;
        uint32_t packed = 0;
        unsigned index;
        memset(&host, 0, sizeof(host));
        host.sin_family = AF_INET;
        host.sin_port = htons((unsigned short)port);
        /* The four bytes are the address in its conventional order, so byte 0
         * is the first written octet; `s_addr` is that sequence in network
         * order, which is the same sequence read big-endian. */
        for (index = 0; index < 4u; index += 1u) {
            packed = (packed << 8) | wf_socket_address_byte(address, index);
        }
        host.sin_addr.s_addr = htonl(packed);
        memcpy(native->bytes, &host, sizeof(host));
        return (unsigned)sizeof(host);
    }
}

/* The reverse: one portable address from the host's own record.  A record of
 * neither family, or one shorter than its family needs, answers the all-zero
 * address rather than reading a byte the host did not write; the target
 * reported the peer and a target that reports nothing usable is not a program
 * outcome [SYS-17]. */
static inline void wf_socket_address_from_native(
    const wf_socket_native_address *native,
    unsigned length,
    wf_socket_address *address
) {
    struct sockaddr head;
    unsigned family;
    memset(address, 0, sizeof(*address));
    if (length < sizeof(head)) {
        return;
    }
    /* The family is read through `struct sockaddr`'s own member and never as
     * a number at the record's first byte: on Linux `sa_family` is a
     * sixteen-bit field at offset zero, on the BSD family and Darwin it is one
     * byte at offset one behind `sa_len`, and only the host's own record type
     * knows which. The copy is because the record's storage is bytes. */
    memcpy(&head, native->bytes, sizeof(head));
    family = head.sa_family;
    if (family == AF_INET6 && length >= sizeof(struct sockaddr_in6)) {
        struct sockaddr_in6 host;
        unsigned index;
        memcpy(&host, native->bytes, sizeof(host));
        for (index = 0; index < 16u; index += 1u) {
            wf_socket_address_set_byte(
                address,
                index,
                host.sin6_addr.s6_addr[index]
            );
        }
        address->port_and_family =
            (unsigned)ntohs(host.sin6_port) | WF_SOCKET_FAMILY_V6;
        return;
    }
    if (family == AF_INET && length >= sizeof(struct sockaddr_in)) {
        struct sockaddr_in host;
        uint32_t packed;
        unsigned index;
        memcpy(&host, native->bytes, sizeof(host));
        packed = ntohl(host.sin_addr.s_addr);
        for (index = 0; index < 4u; index += 1u) {
            wf_socket_address_set_byte(
                address,
                index,
                (unsigned char)((packed >> (8u * (3u - index))) & 0xffu)
            );
        }
        address->port_and_family = (unsigned)ntohs(host.sin_port);
    }
}

/* Rewrites one completed accept's peer record, in place, as the portable form
 * the accept join publishes.  Both POSIX engines call it on the completion
 * path, so the join reads one form whichever of them ran the accept. */
static inline void wf_socket_publish_peer(wf_file_request *request) {
    wf_socket_native_address native = request->operation.accept.peer.native;
    wf_socket_address_from_native(
        &native,
        request->operation.accept.peer_length,
        &request->operation.accept.peer.portable
    );
}

/* One socket of the family one portable address names, closed on exec.
 *
 * SO_REUSEADDR is deliberately not set.  It exists to let a program bind a
 * port a previous connection still holds in TIME_WAIT, which is a decision
 * about what the program's own bind means, and [SYS-17] already fixes that
 * meaning: two binds of one port are the program's own source-order conflict
 * and `AddressInUse` is the host's answer to the second.  Setting it would
 * make that answer depend on a runtime option no source names. */
static inline int wf_socket_open(const wf_socket_address *address) {
    int family = (address->port_and_family & WF_SOCKET_FAMILY_V6) != 0u
        ? AF_INET6
        : AF_INET;
#if defined(SOCK_CLOEXEC)
    return socket(family, SOCK_STREAM | SOCK_CLOEXEC, 0);
#else
    int descriptor = socket(family, SOCK_STREAM, 0);
    if (descriptor >= 0) {
        (void)fcntl(descriptor, F_SETFD, FD_CLOEXEC);
    }
    return descriptor;
#endif
}

/* The flags a send carries.  A write to a connection whose peer has gone is
 * an ordinary `BrokenPipe` outcome [SYS-8], and the bootstrap already ignores
 * the write-to-closed-pipe signal once before the entry [QUAL-3]; where the
 * host offers the per-call form as well, it is used, so a runtime unit with no
 * bootstrap in front of it -- a probe -- answers the same way. */
#if defined(MSG_NOSIGNAL)
#define WF_SOCKET_SEND_FLAGS MSG_NOSIGNAL
#else
#define WF_SOCKET_SEND_FLAGS 0
#endif

/* WF_IO_NOCACHE: a target policy that makes a program's reads genuinely wait.
 *
 * This is runtime policy of the same class as WF_IO_HELPERS and WF_WORKERS,
 * not a language surface. No Whitefoot source names it, no accepted program
 * changes meaning under it, and no byte any read produces differs with it
 * set: it asks the host to keep this file's pages out of its cache, which
 * changes only how long a read takes. Absent — and any value other than the
 * exact text "1" — is today's behaviour exactly, with no host call made at
 * all.
 *
 * It exists because every file measurement in docs/done/0084 and 0086 ran
 * against a warm page cache, where a read is a memory copy and a completion
 * model that overlaps waits has nothing to overlap. Measuring the completion
 * framework needs reads that wait on a device.
 *
 * Darwin gets F_NOCACHE, which is a mode of the descriptor: every read
 * through it bypasses the unified buffer cache for the whole life of the
 * open. Linux has no such per-descriptor mode, so it gets one
 * POSIX_FADV_DONTNEED of the whole file, which evicts what is cached at the
 * moment of the open. O_DIRECT is deliberately not used: it constrains
 * buffer address, offset, and length alignment, which would change the
 * program's own buffers and so change what is being measured.
 *
 * The setting is read once per process and cached. Two threads that race here
 * compute the same answer from the same environment, so the race is benign. */
static inline int wf_file_uncached_reads_requested(void) {
    /* 0 not yet decided, 1 asked for, 2 not asked for. */
    static _Atomic int decided;
    int state = atomic_load_explicit(&decided, memory_order_relaxed);
    if (state == 0) {
        const char *text = getenv("WF_IO_NOCACHE");
        state = (text != NULL && text[0] == '1' && text[1] == 0) ? 1 : 2;
        atomic_store_explicit(&decided, state, memory_order_relaxed);
    }
    return state == 1;
}

/* The one host call the policy makes, in one place so both POSIX engines make
 * the same one. A build may name a different function here to observe it; the
 * observing build is expected to perform the same host call. */
#if !defined(WF_FILE_UNCACHED_APPLY)
#define WF_FILE_UNCACHED_APPLY wf_file_uncached_apply_host
#else
extern void WF_FILE_UNCACHED_APPLY(int);
#endif

static inline void wf_file_uncached_apply_host(int descriptor) {
#if defined(__APPLE__)
    (void)fcntl(descriptor, F_NOCACHE, 1);
#elif defined(__linux__)
    (void)posix_fadvise(descriptor, 0, 0, POSIX_FADV_DONTNEED);
#else
    (void)descriptor;
#endif
}

/* Applied exactly once to each descriptor an open hands back, by whichever
 * engine produced it. A descriptor the kind check refuses never reaches here,
 * so the count of applications equals the count of successful opens. */
static inline void wf_file_apply_uncached_reads(int descriptor) {
    if (descriptor < 0 || !wf_file_uncached_reads_requested()) {
        return;
    }
    WF_FILE_UNCACHED_APPLY(descriptor);
}

#if defined(__cplusplus)
}
#endif

#endif
