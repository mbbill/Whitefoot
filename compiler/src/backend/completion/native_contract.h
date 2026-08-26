#ifndef WHITEFOOT_COMPLETION_NATIVE_CONTRACT_H
#define WHITEFOOT_COMPLETION_NATIVE_CONTRACT_H

/* Compile-time inventory of target adapters.  Unimplemented native adapters
 * are explicit contract stubs, not aliases for the POSIX helper fallback and
 * not claims that a wake primitive performs I/O completion. */

enum wf_completion_target_adapter_kind {
    WF_TARGET_DARWIN_FILE_FALLBACK = 0,
    WF_TARGET_LINUX_IO_URING = 1,
    WF_TARGET_WINDOWS_IOCP = 2
};

typedef struct wf_completion_target_contract {
    const char *name;
    unsigned implemented;
    unsigned native_completion;
    unsigned may_use_blocking_helpers;
    unsigned supports_scheduler_progress;
} wf_completion_target_contract;

wf_completion_target_contract wf_completion_target_contract_for(
    enum wf_completion_target_adapter_kind kind
);

#endif
