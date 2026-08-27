#ifndef WHITEFOOT_NATIVE_COMPLETION_API_H
#define WHITEFOOT_NATIVE_COMPLETION_API_H

/*
 * Target-adapter view of the completion core.
 *
 * The POSIX core currently exposes pthread-backed storage in contract.h.
 * Native Windows adapters do not need that storage layout: they retain only
 * an opaque runtime pointer and call this target-publication ABI.  When the
 * full contract has already been included, its definitions remain canonical.
 * Otherwise these declarations preserve the same ABI without importing a
 * POSIX threading API into a Windows translation unit.
 */

#include <stddef.h>
#include <stdint.h>

#if !defined(WHITEFOOT_COMPLETION_CONTRACT_H)

#define WF_COMPLETION_RESULT_CAPACITY 256u

enum wf_completion_milestone {
    WF_COMPLETION_RESULT_READY = 1u << 0,
    WF_COMPLETION_PAYLOAD_RELEASED = 1u << 1,
    WF_COMPLETION_RESOURCE_RELEASED = 1u << 2,
    WF_COMPLETION_TERMINAL = 1u << 3
};

#define WF_COMPLETION_OWNERSHIP_COMPLETE                                      \
    (WF_COMPLETION_RESULT_READY | WF_COMPLETION_PAYLOAD_RELEASED               \
     | WF_COMPLETION_RESOURCE_RELEASED | WF_COMPLETION_TERMINAL)

typedef struct wf_completion_runtime wf_completion_runtime;

typedef struct wf_completion_token {
    uint32_t slot;
    uint64_t generation;
} wf_completion_token;

enum wf_completion_transition_result {
    WF_COMPLETION_TRANSITIONED = 0,
    WF_COMPLETION_TRANSITION_STALE = 1,
    WF_COMPLETION_TRANSITION_INVALID_STATE = 2
};

enum wf_completion_publish_result {
    WF_COMPLETION_PUBLISHED = 0,
    WF_COMPLETION_PUBLISH_STALE = 1,
    WF_COMPLETION_PUBLISH_DUPLICATE_TERMINAL = 2,
    WF_COMPLETION_PUBLISH_INVALID_STATE = 3,
    WF_COMPLETION_PUBLISH_RESULT_TOO_LARGE = 4,
    WF_COMPLETION_PUBLISH_INCOMPLETE_TERMINAL = 5,
    WF_COMPLETION_PUBLISH_INVALID_ARGUMENT = 6
};

typedef struct wf_completion_publication {
    uint32_t milestones;
    uint32_t terminal_kind;
    const void *result;
    size_t result_size;
} wf_completion_publication;

enum wf_completion_transition_result wf_completion_begin_submit(
    wf_completion_runtime *runtime,
    wf_completion_token token
);

enum wf_completion_transition_result wf_completion_mark_wait_capacity(
    wf_completion_runtime *runtime,
    wf_completion_token token
);

enum wf_completion_transition_result wf_completion_target_accepted(
    wf_completion_runtime *runtime,
    wf_completion_token token
);

enum wf_completion_transition_result wf_completion_set_adapter_tag(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    uint32_t adapter_tag
);

enum wf_completion_publish_result wf_completion_publish_terminal(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    const wf_completion_publication *publication
);

enum wf_completion_publish_result wf_completion_publish_inline_terminal(
    wf_completion_runtime *runtime,
    wf_completion_token token,
    const wf_completion_publication *publication
);

void wf_completion_notify_capacity(wf_completion_runtime *runtime);

#endif

#endif
