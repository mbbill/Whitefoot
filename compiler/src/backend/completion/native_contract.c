#include "native_contract.h"

wf_completion_target_contract wf_completion_target_contract_for(
    enum wf_completion_target_adapter_kind kind
) {
    switch (kind) {
    case WF_TARGET_DARWIN_FILE_FALLBACK:
        return (wf_completion_target_contract){
            .name = "darwin-bounded-file-fallback",
#if defined(__APPLE__)
            .implemented = 1,
#else
            .implemented = 0,
#endif
            .native_completion = 0,
            .may_use_blocking_helpers = 1,
            .supports_scheduler_progress = 1,
        };
    case WF_TARGET_LINUX_IO_URING:
        return (wf_completion_target_contract){
            .name = "linux-io-uring",
#if defined(__linux__)
            .implemented = 1,
#else
            .implemented = 0,
#endif
            .native_completion = 1,
            .may_use_blocking_helpers = 0,
            .supports_scheduler_progress = 1,
        };
    case WF_TARGET_WINDOWS_IOCP:
        return (wf_completion_target_contract){
            .name = "windows-iocp",
            /* The Win32-native core and adapter cross-link against the real
             * IOCP API, but no Windows execution runner has qualified their
             * combined behavior and the wake packet path is not yet bounded
             * persistent broadcast for every announced waiter. Keep target
             * inventory fail-closed until both conditions close. */
            .implemented = 0,
            .native_completion = 1,
            .may_use_blocking_helpers = 0,
            .supports_scheduler_progress = 1,
        };
    default:
        return (wf_completion_target_contract){
            .name = "invalid",
            .implemented = 0,
            .native_completion = 0,
            .may_use_blocking_helpers = 0,
            .supports_scheduler_progress = 0,
        };
    }
}
