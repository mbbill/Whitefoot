/* Compiler-owned Windows v0.39 direct host runtime.
 *
 * Writer-visible resources remain CRT i32 descriptors. Native HANDLE values
 * are recovered only inside these target-selected operations; relative opens
 * are rooted by NtCreateFile's OBJECT_ATTRIBUTES.RootDirectory and never by
 * ambient-path concatenation.
 */

#define WIN32_LEAN_AND_MEAN
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

#include "windows_runtime.h"

#include <windows.h>
#include <winternl.h>

#include <errno.h>
#include <fcntl.h>
#include <io.h>
#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <string.h>

#if !defined(_WIN32)
#error "windows_runtime.c requires a Windows target"
#endif

#ifndef OBJ_CASE_INSENSITIVE
#define OBJ_CASE_INSENSITIVE 0x00000040UL
#endif
#ifndef FILE_OPEN
#define FILE_OPEN 0x00000001UL
#endif
#ifndef FILE_SYNCHRONOUS_IO_NONALERT
#define FILE_SYNCHRONOUS_IO_NONALERT 0x00000020UL
#endif
#ifndef FILE_OPEN_REPARSE_POINT
#define FILE_OPEN_REPARSE_POINT 0x00200000UL
#endif
#ifndef ERROR_MR_MID_NOT_FOUND
#define ERROR_MR_MID_NOT_FOUND 317UL
#endif

#define WF_WINDOWS_NO_FOLLOW 1
#define WF_WINDOWS_EXPECT_ANY 0u
#define WF_WINDOWS_EXPECT_REGULAR 1u
#define WF_WINDOWS_EXPECT_DIRECTORY 2u

#define WF_WINDOWS_OPEN_SUCCEEDED 0u
#define WF_WINDOWS_OPEN_FAILED 1u
#define WF_WINDOWS_OPEN_STATUS_FAILED 2u
#define WF_WINDOWS_OPEN_IS_DIRECTORY 3u
#define WF_WINDOWS_OPEN_OTHER_KIND 4u

#define WF_WINDOWS_MODE_DIRECTORY UINT16_C(0x4000)
#define WF_WINDOWS_MODE_REGULAR UINT16_C(0x8000)
#define WF_WINDOWS_MODE_SYMLINK UINT16_C(0xa000)

#define WF_WINDOWS_DIRECTORY_NATIVE_HEADER 64u
#define WF_WINDOWS_DIRECTORY_RAW_HEADER 5u
#define WF_WINDOWS_COMPONENT_MAX_BYTES 510u
#define WF_WINDOWS_DIRECTORY_SCRATCH_BYTES 8192u
#define WF_WINDOWS_FILE_DIRECTORY_INFORMATION_CLASS \
    ((FILE_INFORMATION_CLASS)1)
#define WF_WINDOWS_STATUS_NO_MORE_FILES \
    ((NTSTATUS)(LONG)0x80000006UL)
#define WF_WINDOWS_STATUS_PENDING ((NTSTATUS)(LONG)0x00000103UL)

typedef NTSTATUS(NTAPI *wf_windows_nt_create_file_fn)(
    PHANDLE,
    ACCESS_MASK,
    POBJECT_ATTRIBUTES,
    PIO_STATUS_BLOCK,
    PLARGE_INTEGER,
    ULONG,
    ULONG,
    ULONG,
    ULONG,
    PVOID,
    ULONG
);

typedef NTSTATUS(NTAPI *wf_windows_nt_query_directory_file_fn)(
    HANDLE,
    HANDLE,
    PIO_APC_ROUTINE,
    PVOID,
    PIO_STATUS_BLOCK,
    PVOID,
    ULONG,
    FILE_INFORMATION_CLASS,
    BOOLEAN,
    PUNICODE_STRING,
    BOOLEAN
);

typedef ULONG(NTAPI *wf_windows_rtl_status_to_dos_error_fn)(NTSTATUS);

typedef struct wf_windows_nt_api {
    wf_windows_nt_create_file_fn create_file;
    wf_windows_nt_query_directory_file_fn query_directory_file;
    wf_windows_rtl_status_to_dos_error_fn status_to_dos_error;
} wf_windows_nt_api;

typedef struct wf_windows_file_directory_information {
    ULONG next_entry_offset;
    ULONG file_index;
    LARGE_INTEGER creation_time;
    LARGE_INTEGER last_access_time;
    LARGE_INTEGER last_write_time;
    LARGE_INTEGER change_time;
    LARGE_INTEGER end_of_file;
    LARGE_INTEGER allocation_size;
    ULONG file_attributes;
    ULONG file_name_length;
    WCHAR file_name[1];
} wf_windows_file_directory_information;

typedef union wf_windows_directory_scratch {
    ULONG_PTR alignment;
    unsigned char bytes[WF_WINDOWS_DIRECTORY_SCRATCH_BYTES];
} wf_windows_directory_scratch;

typedef struct wf_windows_status_cell {
    uint16_t mode;
    uint16_t reserved;
    uint32_t attributes;
} wf_windows_status_cell;

_Static_assert(CHAR_BIT == 8, "the runtime ABI requires eight-bit bytes");
_Static_assert(sizeof(int) == 4, "the runtime ABI requires i32 int");
_Static_assert(sizeof(size_t) == 8, "the target row requires 64-bit size_t");
_Static_assert(sizeof(uint16_t) == 2, "UTF-16 code units must be 16-bit");
_Static_assert(sizeof(WCHAR) == 2, "Windows WCHAR must be UTF-16-sized");
_Static_assert(sizeof(NTSTATUS) == 4, "NTSTATUS detail must remain 32-bit");
_Static_assert(
    sizeof(wf_windows_status_cell) == 8
        && offsetof(wf_windows_status_cell, mode) == 0,
    "the compiler-owned status cell layout changed"
);
_Static_assert(
    offsetof(wf_windows_file_directory_information, file_name)
        == WF_WINDOWS_DIRECTORY_NATIVE_HEADER,
    "FILE_DIRECTORY_INFORMATION layout changed"
);
_Static_assert(
    sizeof(wf_windows_nt_create_file_fn) == sizeof(FARPROC)
        && sizeof(wf_windows_nt_query_directory_file_fn) == sizeof(FARPROC)
        && sizeof(wf_windows_rtl_status_to_dos_error_fn) == sizeof(FARPROC),
    "GetProcAddress does not fit the native procedure pointers"
);

static _Thread_local int wf_windows_error_code;

static void wf_windows_record_error(DWORD error_code) {
    DWORD recorded = error_code;
    if (recorded == ERROR_SUCCESS || recorded > (DWORD)INT_MAX) {
        recorded = ERROR_GEN_FAILURE;
    }
    wf_windows_error_code = (int)recorded;
}

static DWORD wf_windows_error_from_errno(int error_code) {
    switch (error_code) {
    case 0:
        return ERROR_GEN_FAILURE;
    case EACCES:
        return ERROR_ACCESS_DENIED;
    case EBADF:
        return ERROR_INVALID_HANDLE;
    case EINVAL:
        return ERROR_INVALID_PARAMETER;
    case EMFILE:
        return ERROR_TOO_MANY_OPEN_FILES;
    case ENOENT:
        return ERROR_FILE_NOT_FOUND;
    case ENOMEM:
        return ERROR_NOT_ENOUGH_MEMORY;
    default:
        return ERROR_GEN_FAILURE;
    }
}

static int wf_windows_nt_success(NTSTATUS status) {
    return status >= 0;
}

static int wf_windows_resolve_procedure(
    HMODULE module,
    const char *name,
    void *destination,
    size_t destination_size
) {
    FARPROC address;
    if (module == NULL || name == NULL || destination == NULL) {
        return 0;
    }
    address = GetProcAddress(module, name);
    if (address == NULL || destination_size != sizeof(address)) {
        return 0;
    }
    memcpy(destination, &address, sizeof(address));
    return 1;
}

static int wf_windows_resolve_nt_api(wf_windows_nt_api *api) {
    HMODULE module = GetModuleHandleW(L"ntdll.dll");
    if (api == NULL || module == NULL) {
        return 0;
    }
    memset(api, 0, sizeof(*api));
    return wf_windows_resolve_procedure(
               module,
               "NtCreateFile",
               &api->create_file,
               sizeof(api->create_file)
           )
        && wf_windows_resolve_procedure(
               module,
               "NtQueryDirectoryFile",
               &api->query_directory_file,
               sizeof(api->query_directory_file)
           )
        && wf_windows_resolve_procedure(
               module,
               "RtlNtStatusToDosError",
               &api->status_to_dos_error,
               sizeof(api->status_to_dos_error)
           );
}

static DWORD wf_windows_nt_error(
    const wf_windows_nt_api *api,
    NTSTATUS status
) {
    ULONG mapped = api->status_to_dos_error(status);
    return mapped == ERROR_SUCCESS || mapped == ERROR_MR_MID_NOT_FOUND
        ? ERROR_GEN_FAILURE
        : (DWORD)mapped;
}

static int wf_windows_descriptor_handle(int descriptor, HANDLE *handle) {
    intptr_t native;
    int saved_errno;
    if (handle == NULL || descriptor < 0) {
        wf_windows_record_error(ERROR_INVALID_HANDLE);
        return 0;
    }
    errno = 0;
    native = _get_osfhandle(descriptor);
    saved_errno = errno;
    if (native == (intptr_t)-1) {
        wf_windows_record_error(wf_windows_error_from_errno(saved_errno));
        return 0;
    }
    *handle = (HANDLE)native;
    if (*handle == NULL || *handle == INVALID_HANDLE_VALUE) {
        wf_windows_record_error(ERROR_INVALID_HANDLE);
        return 0;
    }
    return 1;
}

static int wf_windows_adopt_handle(HANDLE handle, int open_flags) {
    int descriptor;
    int saved_errno;
    errno = 0;
    descriptor = _open_osfhandle((intptr_t)handle, open_flags);
    saved_errno = errno;
    if (descriptor < 0) {
        DWORD mapped = wf_windows_error_from_errno(saved_errno);
        (void)CloseHandle(handle);
        wf_windows_record_error(mapped);
        return -1;
    }
    return descriptor;
}

static uint16_t wf_windows_mode_from_attributes(DWORD attributes) {
    if ((attributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0) {
        return WF_WINDOWS_MODE_SYMLINK;
    }
    if ((attributes & FILE_ATTRIBUTE_DIRECTORY) != 0) {
        return WF_WINDOWS_MODE_DIRECTORY;
    }
    if ((attributes & FILE_ATTRIBUTE_DEVICE) != 0) {
        return 0;
    }
    return WF_WINDOWS_MODE_REGULAR;
}

static unsigned wf_windows_kind_outcome(
    unsigned expected_kind,
    DWORD attributes
) {
    uint16_t mode = wf_windows_mode_from_attributes(attributes);
    if (expected_kind == WF_WINDOWS_EXPECT_ANY
        || (expected_kind == WF_WINDOWS_EXPECT_REGULAR
            && mode == WF_WINDOWS_MODE_REGULAR)
        || (expected_kind == WF_WINDOWS_EXPECT_DIRECTORY
            && mode == WF_WINDOWS_MODE_DIRECTORY)) {
        return WF_WINDOWS_OPEN_SUCCEEDED;
    }
    return mode == WF_WINDOWS_MODE_DIRECTORY
        ? WF_WINDOWS_OPEN_IS_DIRECTORY
        : WF_WINDOWS_OPEN_OTHER_KIND;
}

static int wf_windows_bounded_wcslen(
    const uint16_t *text,
    size_t *unit_count
) {
    size_t index;
    size_t maximum = (size_t)USHRT_MAX / sizeof(uint16_t);
    if (text == NULL || unit_count == NULL) {
        return 0;
    }
    for (index = 0; index <= maximum; ++index) {
        if (text[index] == 0) {
            *unit_count = index;
            return 1;
        }
    }
    return 0;
}

uint64_t wf__windows_wcslen(const uint16_t *text) {
    uint64_t length = 0;
    if (text == NULL) {
        return 0;
    }
    while (text[length] != 0) {
        length += 1;
    }
    return length;
}

int wf__windows_utf8_measure(
    const uint16_t *text,
    uint64_t unit_count,
    uint64_t *encoded_length
) {
    uint64_t index = 0;
    uint64_t measured = 0;
    if (encoded_length == NULL || (text == NULL && unit_count != 0)) {
        return 0;
    }
    while (index < unit_count) {
        uint16_t unit = text[index];
        uint64_t addition;
        if (unit >= UINT16_C(0xd800) && unit <= UINT16_C(0xdbff)) {
            uint16_t low;
            if (index + 1 >= unit_count) {
                return 0;
            }
            low = text[index + 1];
            if (low < UINT16_C(0xdc00) || low > UINT16_C(0xdfff)) {
                return 0;
            }
            addition = 4;
            index += 2;
        } else {
            if (unit >= UINT16_C(0xdc00) && unit <= UINT16_C(0xdfff)) {
                return 0;
            }
            addition = unit <= UINT16_C(0x007f)
                ? 1
                : (unit <= UINT16_C(0x07ff) ? 2 : 3);
            index += 1;
        }
        if (measured > UINT64_MAX - addition) {
            return 0;
        }
        measured += addition;
    }
    *encoded_length = measured;
    return 1;
}

int wf__windows_utf8_copy(
    const uint16_t *text,
    uint64_t unit_count,
    uint8_t *destination,
    uint64_t destination_capacity
) {
    uint64_t required;
    uint64_t source = 0;
    uint64_t target = 0;
    if (!wf__windows_utf8_measure(text, unit_count, &required)
        || required > destination_capacity
        || (destination == NULL && required != 0)) {
        return 0;
    }
    while (source < unit_count) {
        uint32_t scalar = text[source];
        if (scalar >= UINT32_C(0xd800) && scalar <= UINT32_C(0xdbff)) {
            uint32_t low = text[source + 1];
            scalar = UINT32_C(0x10000)
                + ((scalar - UINT32_C(0xd800)) << 10)
                + (low - UINT32_C(0xdc00));
            source += 2;
        } else {
            source += 1;
        }
        if (scalar <= UINT32_C(0x7f)) {
            destination[target] = (uint8_t)scalar;
            target += 1;
        } else if (scalar <= UINT32_C(0x7ff)) {
            destination[target] = (uint8_t)(UINT32_C(0xc0) | (scalar >> 6));
            destination[target + 1] =
                (uint8_t)(UINT32_C(0x80) | (scalar & UINT32_C(0x3f)));
            target += 2;
        } else if (scalar <= UINT32_C(0xffff)) {
            destination[target] = (uint8_t)(UINT32_C(0xe0) | (scalar >> 12));
            destination[target + 1] = (uint8_t)(
                UINT32_C(0x80) | ((scalar >> 6) & UINT32_C(0x3f))
            );
            destination[target + 2] =
                (uint8_t)(UINT32_C(0x80) | (scalar & UINT32_C(0x3f)));
            target += 3;
        } else {
            destination[target] = (uint8_t)(UINT32_C(0xf0) | (scalar >> 18));
            destination[target + 1] = (uint8_t)(
                UINT32_C(0x80) | ((scalar >> 12) & UINT32_C(0x3f))
            );
            destination[target + 2] = (uint8_t)(
                UINT32_C(0x80) | ((scalar >> 6) & UINT32_C(0x3f))
            );
            destination[target + 3] =
                (uint8_t)(UINT32_C(0x80) | (scalar & UINT32_C(0x3f)));
            target += 4;
        }
    }
    return target == required ? 1 : 0;
}

int wf__windows_relative_path_valid(
    const uint16_t *text,
    uint64_t unit_count
) {
    uint64_t index;
    if (unit_count == 0) {
        return 1;
    }
    if (text == NULL) {
        return 0;
    }
    if (text[0] == UINT16_C('/') || text[0] == UINT16_C('\\')) {
        return 0;
    }
    if (unit_count >= 2
        && ((text[0] >= UINT16_C('A') && text[0] <= UINT16_C('Z'))
            || (text[0] >= UINT16_C('a') && text[0] <= UINT16_C('z')))
        && text[1] == UINT16_C(':')) {
        return 0;
    }
    for (index = 0; index < unit_count; ++index) {
        if (text[index] == 0) {
            return 0;
        }
    }
    return 1;
}

int *wf__windows_error_location(void) {
    return &wf_windows_error_code;
}

int wf__windows_open_cwd(const void *unused_path, int flags, ...) {
    HANDLE handle;
    (void)unused_path;
    if (flags != 0) {
        wf_windows_record_error(ERROR_INVALID_PARAMETER);
        return -1;
    }
    handle = CreateFileW(
        L".",
        FILE_LIST_DIRECTORY | FILE_TRAVERSE | FILE_READ_ATTRIBUTES
            | SYNCHRONIZE,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        NULL,
        OPEN_EXISTING,
        FILE_FLAG_BACKUP_SEMANTICS,
        NULL
    );
    if (handle == INVALID_HANDLE_VALUE) {
        wf_windows_record_error(GetLastError());
        return -1;
    }
    return wf_windows_adopt_handle(handle, _O_RDONLY | _O_BINARY);
}

static int wf_windows_duplicate_descriptor(int descriptor) {
    int duplicate;
    int saved_errno;
    errno = 0;
    duplicate = _dup(descriptor);
    saved_errno = errno;
    if (duplicate < 0) {
        wf_windows_record_error(wf_windows_error_from_errno(saved_errno));
    }
    return duplicate;
}

int wf__windows_stdout_descriptor(void) {
    return wf_windows_duplicate_descriptor(1);
}

int wf__windows_stderr_descriptor(void) {
    return wf_windows_duplicate_descriptor(2);
}

int64_t wf__windows_diagnostic_write(const void *bytes, uint64_t length) {
    HANDLE handle;
    DWORD written = 0;
    if (length == 0) {
        return 0;
    }
    if (bytes == NULL || length > (uint64_t)MAXDWORD) {
        wf_windows_record_error(ERROR_INVALID_PARAMETER);
        return -1;
    }
    handle = GetStdHandle(STD_ERROR_HANDLE);
    if (handle == NULL || handle == INVALID_HANDLE_VALUE) {
        DWORD error = GetLastError();
        wf_windows_record_error(
            error == ERROR_SUCCESS ? ERROR_INVALID_HANDLE : error
        );
        return -1;
    }
    if (WriteFile(handle, bytes, (DWORD)length, &written, NULL) == FALSE) {
        wf_windows_record_error(GetLastError());
        return -1;
    }
    return (int64_t)written;
}

int wf__completion_file_open_at_direct(
    int directory,
    const char *path,
    int flags,
    unsigned mode,
    unsigned has_mode,
    unsigned expected_kind,
    int *error_code,
    unsigned *open_outcome
) {
    const uint16_t *units = (const uint16_t *)(const void *)path;
    wf_windows_nt_api api;
    HANDLE root;
    HANDLE opened = INVALID_HANDLE_VALUE;
    size_t unit_count;
    UNICODE_STRING name;
    OBJECT_ATTRIBUTES attributes;
    IO_STATUS_BLOCK io_status;
    FILE_ATTRIBUTE_TAG_INFO tag_information;
    ACCESS_MASK desired_access;
    ULONG create_options = 0;
    NTSTATUS status;
    unsigned outcome;
    int descriptor;
    (void)mode;

    if (error_code != NULL) {
        *error_code = 0;
    }
    if (open_outcome != NULL) {
        *open_outcome = WF_WINDOWS_OPEN_FAILED;
    }
    if (path == NULL || error_code == NULL || open_outcome == NULL
        || (flags != 0 && flags != WF_WINDOWS_NO_FOLLOW)
        || has_mode > 1u || has_mode != 0u
        || expected_kind > WF_WINDOWS_EXPECT_DIRECTORY) {
        wf_windows_record_error(ERROR_INVALID_PARAMETER);
        if (error_code != NULL) {
            *error_code = ERROR_INVALID_PARAMETER;
        }
        return -1;
    }
    if (!wf_windows_bounded_wcslen(units, &unit_count)) {
        wf_windows_record_error(ERROR_FILENAME_EXCED_RANGE);
        *error_code = ERROR_FILENAME_EXCED_RANGE;
        return -1;
    }
    if (!wf__windows_relative_path_valid(units, (uint64_t)unit_count)) {
        wf_windows_record_error(ERROR_INVALID_NAME);
        *error_code = ERROR_INVALID_NAME;
        return -1;
    }
    if (!wf_windows_descriptor_handle(directory, &root)) {
        *error_code = wf_windows_error_code;
        return -1;
    }
    if (!wf_windows_resolve_nt_api(&api)) {
        DWORD error = GetLastError();
        error = error == ERROR_SUCCESS ? ERROR_PROC_NOT_FOUND : error;
        wf_windows_record_error(error);
        *error_code = (int)error;
        return -1;
    }

    memset(&name, 0, sizeof(name));
    name.Length = (USHORT)(unit_count * sizeof(uint16_t));
    name.MaximumLength = name.Length;
    name.Buffer = (PWSTR)(void *)units;
    memset(&attributes, 0, sizeof(attributes));
    attributes.Length = (ULONG)sizeof(attributes);
    attributes.RootDirectory = root;
    attributes.ObjectName = &name;
    attributes.Attributes = OBJ_CASE_INSENSITIVE;
    memset(&io_status, 0, sizeof(io_status));

    desired_access = FILE_READ_ATTRIBUTES | SYNCHRONIZE;
    if (expected_kind == WF_WINDOWS_EXPECT_REGULAR) {
        desired_access |= FILE_READ_DATA;
    } else if (expected_kind == WF_WINDOWS_EXPECT_DIRECTORY) {
        desired_access |= FILE_LIST_DIRECTORY | FILE_TRAVERSE;
        create_options |= FILE_SYNCHRONOUS_IO_NONALERT;
    }
    if (flags == WF_WINDOWS_NO_FOLLOW) {
        create_options |= FILE_OPEN_REPARSE_POINT;
    }
    status = api.create_file(
        &opened,
        desired_access,
        &attributes,
        &io_status,
        NULL,
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        FILE_OPEN,
        create_options,
        NULL,
        0
    );
    if (!wf_windows_nt_success(status)) {
        DWORD error = wf_windows_nt_error(&api, status);
        wf_windows_record_error(error);
        *error_code = (int)error;
        return -1;
    }
    if (GetFileInformationByHandleEx(
            opened,
            FileAttributeTagInfo,
            &tag_information,
            (DWORD)sizeof(tag_information)
        ) == FALSE) {
        DWORD error = GetLastError();
        (void)CloseHandle(opened);
        wf_windows_record_error(error);
        *error_code = (int)error;
        *open_outcome = WF_WINDOWS_OPEN_STATUS_FAILED;
        return -1;
    }
    outcome = wf_windows_kind_outcome(
        expected_kind,
        tag_information.FileAttributes
    );
    if (outcome != WF_WINDOWS_OPEN_SUCCEEDED) {
        (void)CloseHandle(opened);
        *open_outcome = outcome;
        return -1;
    }
    descriptor = wf_windows_adopt_handle(opened, _O_RDONLY | _O_BINARY);
    if (descriptor < 0) {
        *error_code = wf_windows_error_code;
        return -1;
    }
    if (expected_kind == WF_WINDOWS_EXPECT_REGULAR) {
        int association_error =
            wf__windows_completion_associate_descriptor(descriptor);
        if (association_error != 0) {
            (void)_close(descriptor);
            wf_windows_record_error((DWORD)association_error);
            *error_code = wf_windows_error_code;
            return -1;
        }
    }
    *open_outcome = WF_WINDOWS_OPEN_SUCCEEDED;
    return descriptor;
}

int wf__completion_file_status_direct(
    int descriptor,
    void *status,
    uint64_t status_capacity
) {
    HANDLE handle;
    FILE_ATTRIBUTE_TAG_INFO tag_information;
    wf_windows_status_cell cell;
    if (status == NULL) {
        wf_windows_record_error(ERROR_INVALID_PARAMETER);
        return -1;
    }
    if (status_capacity < sizeof(cell)) {
        wf_windows_record_error(ERROR_INSUFFICIENT_BUFFER);
        return -1;
    }
    if (!wf_windows_descriptor_handle(descriptor, &handle)) {
        return -1;
    }
    if (GetFileInformationByHandleEx(
            handle,
            FileAttributeTagInfo,
            &tag_information,
            (DWORD)sizeof(tag_information)
        ) == FALSE) {
        wf_windows_record_error(GetLastError());
        return -1;
    }
    memset(&cell, 0, sizeof(cell));
    cell.mode = wf_windows_mode_from_attributes(
        tag_information.FileAttributes
    );
    cell.attributes = (uint32_t)tag_information.FileAttributes;
    memcpy(status, &cell, sizeof(cell));
    return 0;
}

int wf__completion_file_close_direct(int descriptor) {
    int result;
    int saved_errno;
    wf__windows_completion_forget_descriptor(descriptor);
    errno = 0;
    result = _close(descriptor);
    saved_errno = errno;
    if (result != 0) {
        wf_windows_record_error(wf_windows_error_from_errno(saved_errno));
        return -1;
    }
    return 0;
}

int64_t wf__completion_file_pread_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t file_offset
) {
    HANDLE handle;
    HANDLE event;
    OVERLAPPED overlapped;
    ULARGE_INTEGER offset;
    DWORD transferred = 0;
    BOOL started;
    DWORD error;
    if (count == 0) {
        return 0;
    }
    if (buffer == NULL || count > (uint64_t)MAXDWORD || file_offset < 0
        || count > (uint64_t)INT64_MAX - (uint64_t)file_offset) {
        wf_windows_record_error(ERROR_INVALID_PARAMETER);
        return -1;
    }
    if (!wf_windows_descriptor_handle(descriptor, &handle)) {
        return -1;
    }
    event = CreateEventW(NULL, TRUE, FALSE, NULL);
    if (event == NULL) {
        wf_windows_record_error(GetLastError());
        return -1;
    }
    memset(&overlapped, 0, sizeof(overlapped));
    offset.QuadPart = (uint64_t)file_offset;
    overlapped.Offset = offset.LowPart;
    overlapped.OffsetHigh = offset.HighPart;
    /* This handle is already associated with the shared completion port. The
     * low event bit is Windows' per-operation instruction not to enqueue an
     * IOCP packet: this direct call waits for its own event and its OVERLAPPED
     * lives only on this stack. Without the bit, the adapter could later
     * dequeue that expired address as though it were one of its own entries. */
    overlapped.hEvent = (HANDLE)((uintptr_t)event | (uintptr_t)1u);
    started = ReadFile(
        handle,
        buffer,
        (DWORD)count,
        &transferred,
        &overlapped
    );
    if (started == FALSE) {
        error = GetLastError();
        if (error == ERROR_HANDLE_EOF) {
            (void)CloseHandle(event);
            return 0;
        }
        if (error != ERROR_IO_PENDING) {
            (void)CloseHandle(event);
            wf_windows_record_error(error);
            return -1;
        }
        if (WaitForSingleObject(event, INFINITE) != WAIT_OBJECT_0) {
            error = GetLastError();
            (void)CancelIoEx(handle, &overlapped);
            (void)GetOverlappedResult(handle, &overlapped, &transferred, TRUE);
            (void)CloseHandle(event);
            wf_windows_record_error(
                error == ERROR_SUCCESS ? ERROR_GEN_FAILURE : error
            );
            return -1;
        }
        if (GetOverlappedResult(
                handle,
                &overlapped,
                &transferred,
                FALSE
            ) == FALSE) {
            error = GetLastError();
            (void)CloseHandle(event);
            if (error == ERROR_HANDLE_EOF) {
                return 0;
            }
            wf_windows_record_error(error);
            return -1;
        }
    }
    (void)CloseHandle(event);
    return (int64_t)transferred;
}

int64_t wf__completion_file_write_direct(
    int descriptor,
    const void *buffer,
    uint64_t count
) {
    HANDLE handle;
    DWORD written = 0;
    if (count == 0) {
        return 0;
    }
    if (buffer == NULL || count > (uint64_t)MAXDWORD) {
        wf_windows_record_error(ERROR_INVALID_PARAMETER);
        return -1;
    }
    if (!wf_windows_descriptor_handle(descriptor, &handle)) {
        return -1;
    }
    if (WriteFile(handle, buffer, (DWORD)count, &written, NULL) == FALSE) {
        wf_windows_record_error(GetLastError());
        return -1;
    }
    return (int64_t)written;
}

static int wf_windows_directory_record_valid(
    const unsigned char *record_bytes,
    size_t remaining,
    size_t *native_extent,
    size_t *raw_extent
) {
    const wf_windows_file_directory_information *record;
    size_t minimum;
    size_t name_bytes;
    size_t index;
    if (remaining < WF_WINDOWS_DIRECTORY_NATIVE_HEADER
        || record_bytes == NULL || native_extent == NULL
        || raw_extent == NULL) {
        return 0;
    }
    record = (const wf_windows_file_directory_information *)record_bytes;
    name_bytes = (size_t)record->file_name_length;
    if (name_bytes == 0 || name_bytes > WF_WINDOWS_COMPONENT_MAX_BYTES
        || (name_bytes & 1u) != 0
        || name_bytes > remaining - WF_WINDOWS_DIRECTORY_NATIVE_HEADER) {
        return 0;
    }
    minimum = WF_WINDOWS_DIRECTORY_NATIVE_HEADER + name_bytes;
    if (record->next_entry_offset == 0) {
        *native_extent = remaining;
    } else {
        *native_extent = (size_t)record->next_entry_offset;
        if (*native_extent < minimum || *native_extent > remaining
            || (*native_extent & (sizeof(ULONG_PTR) - 1u)) != 0) {
            return 0;
        }
    }
    for (index = 0; index < name_bytes; index += sizeof(uint16_t)) {
        uint16_t unit;
        memcpy(
            &unit,
            record_bytes + WF_WINDOWS_DIRECTORY_NATIVE_HEADER + index,
            sizeof(unit)
        );
        if (unit == 0 || unit == UINT16_C('/') || unit == UINT16_C('\\')) {
            return 0;
        }
    }
    *raw_extent = WF_WINDOWS_DIRECTORY_RAW_HEADER + name_bytes;
    return *raw_extent <= (size_t)USHRT_MAX;
}

static uint8_t wf_windows_directory_kind(DWORD attributes) {
    if ((attributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0) {
        return UINT8_C(3);
    }
    if ((attributes & FILE_ATTRIBUTE_DIRECTORY) != 0) {
        return UINT8_C(2);
    }
    if ((attributes & FILE_ATTRIBUTE_DEVICE) != 0) {
        return UINT8_C(0);
    }
    return UINT8_C(1);
}

static int64_t wf_windows_directory_batch_worker(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position
) {
    wf_windows_nt_api api;
    wf_windows_directory_scratch scratch;
    HANDLE directory;
    HANDLE event;
    IO_STATUS_BLOCK io_status;
    ULONG query_capacity;
    NTSTATUS status;
    size_t transferred;
    size_t source;
    size_t required;
    unsigned char *destination = (unsigned char *)buffer;
    (void)position;

    if (position == NULL || (buffer == NULL && count != 0)) {
        wf_windows_record_error(ERROR_INVALID_PARAMETER);
        return -1;
    }
    if (count == 0) {
        return 0;
    }
    if (count < WF_WINDOWS_DIRECTORY_NATIVE_HEADER
            + WF_WINDOWS_COMPONENT_MAX_BYTES
        || count > (uint64_t)INT64_MAX) {
        wf_windows_record_error(
            count > (uint64_t)INT64_MAX
                ? ERROR_INVALID_PARAMETER
                : ERROR_INSUFFICIENT_BUFFER
        );
        return -1;
    }
    if (!wf_windows_descriptor_handle(descriptor, &directory)) {
        return -1;
    }
    if (!wf_windows_resolve_nt_api(&api)) {
        DWORD error = GetLastError();
        wf_windows_record_error(
            error == ERROR_SUCCESS ? ERROR_PROC_NOT_FOUND : error
        );
        return -1;
    }
    query_capacity = count < WF_WINDOWS_DIRECTORY_SCRATCH_BYTES
        ? (ULONG)count
        : (ULONG)WF_WINDOWS_DIRECTORY_SCRATCH_BYTES;
    event = CreateEventW(NULL, TRUE, FALSE, NULL);
    if (event == NULL) {
        wf_windows_record_error(GetLastError());
        return -1;
    }
    memset(&io_status, 0, sizeof(io_status));
    status = api.query_directory_file(
        directory,
        event,
        NULL,
        NULL,
        &io_status,
        scratch.bytes,
        query_capacity,
        WF_WINDOWS_FILE_DIRECTORY_INFORMATION_CLASS,
        FALSE,
        NULL,
        FALSE
    );
    if (status == WF_WINDOWS_STATUS_PENDING) {
        if (WaitForSingleObject(event, INFINITE) != WAIT_OBJECT_0) {
            DWORD error = GetLastError();
            (void)CloseHandle(event);
            wf_windows_record_error(
                error == ERROR_SUCCESS ? ERROR_GEN_FAILURE : error
            );
            return -1;
        }
        status = io_status.Status;
    }
    (void)CloseHandle(event);
    if (status == WF_WINDOWS_STATUS_NO_MORE_FILES) {
        return 0;
    }
    if (!wf_windows_nt_success(status)) {
        wf_windows_record_error(wf_windows_nt_error(&api, status));
        return -1;
    }
    transferred = (size_t)io_status.Information;
    if (transferred == 0) {
        return 0;
    }
    if (transferred > (size_t)query_capacity) {
        wf_windows_record_error(ERROR_INVALID_DATA);
        return -1;
    }

    source = 0;
    required = 0;
    for (;;) {
        size_t native_extent;
        size_t raw_extent;
        const wf_windows_file_directory_information *record;
        if (!wf_windows_directory_record_valid(
                scratch.bytes + source,
                transferred - source,
                &native_extent,
                &raw_extent
            ) || required > (size_t)count - raw_extent) {
            wf_windows_record_error(ERROR_INVALID_DATA);
            return -1;
        }
        record = (const wf_windows_file_directory_information *)(
            scratch.bytes + source
        );
        required += raw_extent;
        if (record->next_entry_offset == 0) {
            break;
        }
        source += native_extent;
    }

    source = 0;
    required = 0;
    for (;;) {
        const wf_windows_file_directory_information *record =
            (const wf_windows_file_directory_information *)(
                scratch.bytes + source
            );
        size_t native_extent;
        size_t raw_extent;
        size_t name_bytes = (size_t)record->file_name_length;
        size_t index;
        (void)wf_windows_directory_record_valid(
            scratch.bytes + source,
            transferred - source,
            &native_extent,
            &raw_extent
        );
        destination[required] = (uint8_t)(raw_extent & 0xffu);
        destination[required + 1] = (uint8_t)((raw_extent >> 8) & 0xffu);
        destination[required + 2] = (uint8_t)(name_bytes & 0xffu);
        destination[required + 3] =
            (uint8_t)((name_bytes >> 8) & 0xffu);
        destination[required + 4] = wf_windows_directory_kind(
            record->file_attributes
        );
        for (index = 0; index < name_bytes; index += sizeof(uint16_t)) {
            uint16_t unit;
            memcpy(
                &unit,
                scratch.bytes + source
                    + WF_WINDOWS_DIRECTORY_NATIVE_HEADER + index,
                sizeof(unit)
            );
            destination[required + WF_WINDOWS_DIRECTORY_RAW_HEADER + index] =
                (uint8_t)(unit & UINT16_C(0xff));
            destination[
                required + WF_WINDOWS_DIRECTORY_RAW_HEADER + index + 1
            ] = (uint8_t)(unit >> 8);
        }
        required += raw_extent;
        if (record->next_entry_offset == 0) {
            break;
        }
        source += native_extent;
    }
    return (int64_t)required;
}

int64_t wf__completion_directory_next_direct(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position
) {
    return wf_windows_directory_batch_worker(
        descriptor,
        buffer,
        count,
        position
    );
}

int64_t wf__windows_directory_batch(
    int descriptor,
    void *buffer,
    uint64_t count,
    int64_t *position
) {
    return wf_windows_directory_batch_worker(
        descriptor,
        buffer,
        count,
        position
    );
}
