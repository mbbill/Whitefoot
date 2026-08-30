/* Native Windows namespace evidence for a future Whitefoot target row.
 *
 * The operation under test receives only a directory HANDLE and one explicit
 * UTF-16 component.  Fixture setup and cleanup use absolute Win32 paths, but
 * the tested open never receives one: OBJECT_ATTRIBUTES.RootDirectory is the
 * sole namespace root passed to NtCreateFile.  The probe opens the process cwd
 * as a directory HANDLE and then moves the ambient cwd to an empty sibling
 * before any tested open or enumeration.  A successful result therefore
 * cannot have come from prefix concatenation or the ambient current directory.
 */

#define WIN32_LEAN_AND_MEAN
#ifndef _WIN32_WINNT
#define _WIN32_WINNT 0x0602
#endif

#include <windows.h>
#include <winternl.h>

#include <limits.h>
#include <stddef.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <wchar.h>

#if !defined(_WIN32)
#error "windows_namespace_probe.c requires a Windows target"
#endif

#ifndef OBJ_CASE_INSENSITIVE
#define OBJ_CASE_INSENSITIVE 0x00000040UL
#endif
#ifndef FILE_OPEN
#define FILE_OPEN 0x00000001UL
#endif
#ifndef FILE_DIRECTORY_FILE
#define FILE_DIRECTORY_FILE 0x00000001UL
#endif
#ifndef FILE_SYNCHRONOUS_IO_NONALERT
#define FILE_SYNCHRONOUS_IO_NONALERT 0x00000020UL
#endif
#ifndef FILE_NON_DIRECTORY_FILE
#define FILE_NON_DIRECTORY_FILE 0x00000040UL
#endif
#ifndef FILE_OPEN_REPARSE_POINT
#define FILE_OPEN_REPARSE_POINT 0x00200000UL
#endif
#ifndef SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE
#define SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE 0x00000002UL
#endif
#ifndef ERROR_MR_MID_NOT_FOUND
#define ERROR_MR_MID_NOT_FOUND 317UL
#endif

#define WF_PATH_CAPACITY 1024u
#define WF_DIRECTORY_BUFFER_BYTES 8192u
#define WF_FILE_NAMES_INFORMATION_CLASS ((FILE_INFORMATION_CLASS)12)
#define WF_STATUS_NO_MORE_FILES ((NTSTATUS)(LONG)0x80000006UL)
#define WF_ARRAY_COUNT(array) (sizeof(array) / sizeof((array)[0]))

typedef NTSTATUS(NTAPI *wf_nt_create_file_fn)(
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

typedef NTSTATUS(NTAPI *wf_nt_query_directory_file_fn)(
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

typedef ULONG(NTAPI *wf_rtl_nt_status_to_dos_error_fn)(NTSTATUS);

typedef struct wf_nt_api {
    wf_nt_create_file_fn create_file;
    wf_nt_query_directory_file_fn query_directory_file;
    wf_rtl_nt_status_to_dos_error_fn status_to_dos_error;
} wf_nt_api;

typedef struct wf_tracked_handle {
    HANDLE value;
    unsigned acquired;
    unsigned closes;
} wf_tracked_handle;

typedef struct wf_file_names_information {
    ULONG next_entry_offset;
    ULONG file_index;
    ULONG file_name_length;
    WCHAR file_name[1];
} wf_file_names_information;

typedef union wf_directory_buffer {
    ULONG_PTR alignment;
    unsigned char bytes[WF_DIRECTORY_BUFFER_BYTES];
} wf_directory_buffer;

_Static_assert(sizeof(WCHAR) == 2, "the probe requires native UTF-16 code units");
_Static_assert(sizeof(NTSTATUS) == 4, "NTSTATUS must retain its 32-bit detail");
_Static_assert(
    offsetof(wf_file_names_information, file_name) == 12,
    "FILE_NAMES_INFORMATION prefix layout changed"
);
_Static_assert(
    sizeof(wf_nt_create_file_fn) == sizeof(FARPROC)
        && sizeof(wf_nt_query_directory_file_fn) == sizeof(FARPROC)
        && sizeof(wf_rtl_nt_status_to_dos_error_fn) == sizeof(FARPROC),
    "GetProcAddress result does not fit the native procedure pointers"
);

enum wf_component_verdict {
    WF_COMPONENT_ACCEPTED = 0,
    WF_COMPONENT_EMPTY,
    WF_COMPONENT_DOT,
    WF_COMPONENT_DOT_DOT,
    WF_COMPONENT_NUL,
    WF_COMPONENT_SEPARATOR,
    WF_COMPONENT_DRIVE_PREFIX,
    WF_COMPONENT_UNC_PREFIX,
    WF_COMPONENT_TOO_LONG
};

typedef struct wf_component_open_result {
    enum wf_component_verdict verdict;
    NTSTATUS status;
    HANDLE handle;
    unsigned native_called;
} wf_component_open_result;

typedef struct wf_probe_failure {
    unsigned check;
    DWORD win32_error;
    NTSTATUS nt_status;
} wf_probe_failure;

static WCHAR wf_regular_name[] = L"regular-\x4e2d.bin";
static WCHAR wf_directory_name[] = L"child-\x76ee\x5f55";
static WCHAR wf_link_name[] = L"link-\x94fe\x63a5";
static WCHAR wf_missing_name[] = L"missing-\x4e0d\x5b58\x5728";

static int wf_nt_success(NTSTATUS status) {
    return status >= 0;
}

static int wf_handle_live(const wf_tracked_handle *tracked) {
    return tracked->acquired != 0 && tracked->closes == 0
        && tracked->value != NULL && tracked->value != INVALID_HANDLE_VALUE;
}

static int wf_adopt_handle(wf_tracked_handle *tracked, HANDLE value) {
    if (tracked == NULL || tracked->acquired != 0 || value == NULL
        || value == INVALID_HANDLE_VALUE) {
        return 0;
    }
    tracked->value = value;
    tracked->acquired = 1;
    return 1;
}

static int wf_close_handle_once(wf_tracked_handle *tracked) {
    if (!wf_handle_live(tracked)) {
        return 0;
    }
    if (CloseHandle(tracked->value) == FALSE) {
        return 0;
    }
    tracked->value = INVALID_HANDLE_VALUE;
    tracked->closes += 1;
    return 1;
}

static int wf_closed_exactly_once(const wf_tracked_handle *tracked) {
    return tracked->acquired == 0
        || (tracked->closes == 1 && tracked->value == INVALID_HANDLE_VALUE);
}

static int wf_fixture_join_path(
    WCHAR *destination,
    size_t capacity,
    const WCHAR *directory,
    const WCHAR *name
) {
    size_t directory_length;
    size_t name_length;
    int needs_separator;
    if (destination == NULL || directory == NULL || name == NULL
        || capacity == 0) {
        return 0;
    }
    directory_length = wcslen(directory);
    name_length = wcslen(name);
    needs_separator = directory_length != 0
        && directory[directory_length - 1] != L'\\'
        && directory[directory_length - 1] != L'/';
    if (directory_length + (size_t)needs_separator + name_length + 1
        > capacity) {
        return 0;
    }
    memcpy(
        destination,
        directory,
        directory_length * sizeof(WCHAR)
    );
    if (needs_separator != 0) {
        destination[directory_length] = L'\\';
        directory_length += 1;
    }
    memcpy(
        destination + directory_length,
        name,
        (name_length + 1) * sizeof(WCHAR)
    );
    return 1;
}

static int wf_resolve_procedure(
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

static int wf_resolve_nt_api(wf_nt_api *api) {
    HMODULE module = GetModuleHandleW(L"ntdll.dll");
    if (api == NULL || module == NULL) {
        return 0;
    }
    memset(api, 0, sizeof(*api));
    return wf_resolve_procedure(
               module,
               "NtCreateFile",
               &api->create_file,
               sizeof(api->create_file)
           )
        && wf_resolve_procedure(
               module,
               "NtQueryDirectoryFile",
               &api->query_directory_file,
               sizeof(api->query_directory_file)
           )
        && wf_resolve_procedure(
               module,
               "RtlNtStatusToDosError",
               &api->status_to_dos_error,
               sizeof(api->status_to_dos_error)
           );
}

static int wf_is_separator(WCHAR unit) {
    return unit == L'\\' || unit == L'/';
}

static int wf_is_ascii_letter(WCHAR unit) {
    return (unit >= L'A' && unit <= L'Z')
        || (unit >= L'a' && unit <= L'z');
}

static enum wf_component_verdict wf_validate_component(
    const WCHAR *units,
    size_t count
) {
    size_t index;
    if (units == NULL || count == 0) {
        return WF_COMPONENT_EMPTY;
    }
    if (count > (size_t)USHRT_MAX / sizeof(WCHAR)) {
        return WF_COMPONENT_TOO_LONG;
    }
    for (index = 0; index < count; ++index) {
        if (units[index] == L'\0') {
            return WF_COMPONENT_NUL;
        }
    }
    if (count == 1 && units[0] == L'.') {
        return WF_COMPONENT_DOT;
    }
    if (count == 2 && units[0] == L'.' && units[1] == L'.') {
        return WF_COMPONENT_DOT_DOT;
    }
    if (count >= 2 && wf_is_separator(units[0])
        && wf_is_separator(units[1])) {
        return WF_COMPONENT_UNC_PREFIX;
    }
    if (count >= 2 && wf_is_ascii_letter(units[0]) && units[1] == L':') {
        return WF_COMPONENT_DRIVE_PREFIX;
    }
    for (index = 0; index < count; ++index) {
        if (wf_is_separator(units[index])) {
            return WF_COMPONENT_SEPARATOR;
        }
    }
    return WF_COMPONENT_ACCEPTED;
}

static wf_component_open_result wf_open_component(
    const wf_nt_api *api,
    HANDLE root,
    WCHAR *units,
    size_t count,
    ULONG create_options
) {
    wf_component_open_result result;
    UNICODE_STRING name;
    OBJECT_ATTRIBUTES attributes;
    IO_STATUS_BLOCK io_status;
    result.verdict = wf_validate_component(units, count);
    result.status = 0;
    result.handle = INVALID_HANDLE_VALUE;
    result.native_called = 0;
    if (result.verdict != WF_COMPONENT_ACCEPTED) {
        return result;
    }
    memset(&name, 0, sizeof(name));
    name.Length = (USHORT)(count * sizeof(WCHAR));
    name.MaximumLength = name.Length;
    name.Buffer = units;
    memset(&attributes, 0, sizeof(attributes));
    attributes.Length = (ULONG)sizeof(attributes);
    attributes.RootDirectory = root;
    attributes.ObjectName = &name;
    attributes.Attributes = OBJ_CASE_INSENSITIVE;
    memset(&io_status, 0, sizeof(io_status));
    result.native_called = 1;
    result.status = api->create_file(
        &result.handle,
        FILE_READ_ATTRIBUTES | SYNCHRONIZE,
        &attributes,
        &io_status,
        NULL,
        0,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        FILE_OPEN,
        create_options | FILE_SYNCHRONOUS_IO_NONALERT,
        NULL,
        0
    );
    if (!wf_nt_success(result.status)) {
        result.handle = INVALID_HANDLE_VALUE;
    }
    return result;
}

static int wf_name_equal(
    const WCHAR *left,
    size_t left_count,
    const WCHAR *right
) {
    size_t right_count = wcslen(right);
    return left_count == right_count
        && memcmp(left, right, left_count * sizeof(WCHAR)) == 0;
}

static int wf_enumerate_directory_handle(
    const wf_nt_api *api,
    HANDLE directory,
    int *found_regular,
    int *found_directory,
    NTSTATUS *failure_status
) {
    wf_directory_buffer buffer;
    BOOLEAN restart = TRUE;
    unsigned calls = 0;
    *found_regular = 0;
    *found_directory = 0;
    *failure_status = 0;
    for (;;) {
        IO_STATUS_BLOCK io_status;
        NTSTATUS status;
        size_t offset = 0;
        size_t transferred;
        memset(&io_status, 0, sizeof(io_status));
        status = api->query_directory_file(
            directory,
            NULL,
            NULL,
            NULL,
            &io_status,
            buffer.bytes,
            (ULONG)sizeof(buffer.bytes),
            WF_FILE_NAMES_INFORMATION_CLASS,
            FALSE,
            NULL,
            restart
        );
        restart = FALSE;
        calls += 1;
        if (status == WF_STATUS_NO_MORE_FILES) {
            return calls != 0 && *found_regular != 0 && *found_directory != 0;
        }
        if (!wf_nt_success(status)) {
            *failure_status = status;
            return 0;
        }
        transferred = (size_t)io_status.Information;
        if (transferred == 0) {
            if (calls >= 64) {
                return 0;
            }
            continue;
        }
        if (transferred > sizeof(buffer.bytes)
            || transferred < offsetof(wf_file_names_information, file_name)) {
            return 0;
        }
        for (;;) {
            const wf_file_names_information *entry;
            size_t header = offsetof(wf_file_names_information, file_name);
            size_t remaining = transferred - offset;
            size_t name_units;
            if (remaining < header) {
                return 0;
            }
            entry = (const wf_file_names_information *)(buffer.bytes + offset);
            if ((entry->file_name_length % sizeof(WCHAR)) != 0
                || entry->file_name_length > remaining - header) {
                return 0;
            }
            name_units = entry->file_name_length / sizeof(WCHAR);
            if (wf_name_equal(entry->file_name, name_units, wf_regular_name)) {
                *found_regular = 1;
            }
            if (wf_name_equal(entry->file_name, name_units, wf_directory_name)) {
                *found_directory = 1;
            }
            if (entry->next_entry_offset == 0) {
                break;
            }
            if (entry->next_entry_offset < header
                || entry->next_entry_offset > remaining) {
                return 0;
            }
            offset += entry->next_entry_offset;
        }
        if (calls >= 64) {
            return 0;
        }
    }
}

static int wf_handle_attributes(
    HANDLE handle,
    DWORD *attributes,
    DWORD *reparse_tag
) {
    FILE_ATTRIBUTE_TAG_INFO information;
    if (GetFileInformationByHandleEx(
            handle,
            FileAttributeTagInfo,
            &information,
            (DWORD)sizeof(information)
        ) == FALSE) {
        return 0;
    }
    *attributes = information.FileAttributes;
    *reparse_tag = information.ReparseTag;
    return 1;
}

static int wf_reparse_creation_unavailable(DWORD error) {
    return error == ERROR_PRIVILEGE_NOT_HELD || error == ERROR_NOT_SUPPORTED
        || error == ERROR_INVALID_FUNCTION;
}

static void wf_record_win32_failure(
    wf_probe_failure *failure,
    unsigned check
) {
    failure->check = check;
    failure->win32_error = GetLastError();
}

int main(void) {
    static WCHAR empty_component[] = L"";
    static WCHAR dot_component[] = L".";
    static WCHAR dot_dot_component[] = L"..";
    static WCHAR slash_component[] = L"left/right";
    static WCHAR backslash_component[] = L"left\\right";
    static WCHAR drive_component[] = L"C:relative";
    static WCHAR unc_component[] = L"\\\\server\\share";
    static WCHAR nul_component[] = {L'a', L'\0', L'b'};
    struct wf_rejection_case {
        WCHAR *units;
        size_t count;
        enum wf_component_verdict expected;
    };
    static const struct wf_rejection_case rejection_cases[] = {
        {empty_component, 0, WF_COMPONENT_EMPTY},
        {dot_component, WF_ARRAY_COUNT(dot_component) - 1, WF_COMPONENT_DOT},
        {
            dot_dot_component,
            WF_ARRAY_COUNT(dot_dot_component) - 1,
            WF_COMPONENT_DOT_DOT
        },
        {
            slash_component,
            WF_ARRAY_COUNT(slash_component) - 1,
            WF_COMPONENT_SEPARATOR
        },
        {
            backslash_component,
            WF_ARRAY_COUNT(backslash_component) - 1,
            WF_COMPONENT_SEPARATOR
        },
        {
            drive_component,
            WF_ARRAY_COUNT(drive_component) - 1,
            WF_COMPONENT_DRIVE_PREFIX
        },
        {
            unc_component,
            WF_ARRAY_COUNT(unc_component) - 1,
            WF_COMPONENT_UNC_PREFIX
        },
        {nul_component, WF_ARRAY_COUNT(nul_component), WF_COMPONENT_NUL}
    };
    wf_nt_api nt_api;
    wf_probe_failure failure = {0, 0, 0};
    wf_tracked_handle fixture_file = {INVALID_HANDLE_VALUE, 0, 0};
    wf_tracked_handle cwd_directory = {INVALID_HANDLE_VALUE, 0, 0};
    wf_tracked_handle opened_file = {INVALID_HANDLE_VALUE, 0, 0};
    wf_tracked_handle opened_directory = {INVALID_HANDLE_VALUE, 0, 0};
    wf_tracked_handle nofollow_link = {INVALID_HANDLE_VALUE, 0, 0};
    wf_tracked_handle followed_link = {INVALID_HANDLE_VALUE, 0, 0};
    WCHAR original_cwd[WF_PATH_CAPACITY];
    WCHAR temporary_directory[WF_PATH_CAPACITY];
    WCHAR base_path[WF_PATH_CAPACITY];
    WCHAR root_path[WF_PATH_CAPACITY];
    WCHAR ambient_path[WF_PATH_CAPACITY];
    WCHAR regular_path[WF_PATH_CAPACITY];
    WCHAR directory_path[WF_PATH_CAPACITY];
    WCHAR link_path[WF_PATH_CAPACITY];
    int temporary_file_exists = 0;
    int base_exists = 0;
    int root_exists = 0;
    int ambient_exists = 0;
    int regular_exists = 0;
    int directory_exists = 0;
    int link_exists = 0;
    int cwd_changed = 0;
    int reparse_skipped = 0;
    DWORD reparse_skip_error = 0;
    int outcome = 1;
    size_t index;

#define WF_REQUIRE(condition, check_number)                                    \
    do {                                                                        \
        if (!(condition)) {                                                     \
            failure.check = (check_number);                                     \
            goto cleanup;                                                       \
        }                                                                       \
    } while (0)

    WF_REQUIRE(wf_resolve_nt_api(&nt_api), 10);
    {
        DWORD length = GetCurrentDirectoryW(
            (DWORD)WF_ARRAY_COUNT(original_cwd),
            original_cwd
        );
        if (length == 0 || length >= WF_ARRAY_COUNT(original_cwd)) {
            wf_record_win32_failure(&failure, 11);
            goto cleanup;
        }
    }
    {
        DWORD length = GetTempPathW(
            (DWORD)WF_ARRAY_COUNT(temporary_directory),
            temporary_directory
        );
        if (length == 0 || length >= WF_ARRAY_COUNT(temporary_directory)) {
            wf_record_win32_failure(&failure, 12);
            goto cleanup;
        }
    }
    if (GetTempFileNameW(
            temporary_directory,
            L"wfn",
            0,
            base_path
        ) == 0) {
        wf_record_win32_failure(&failure, 13);
        goto cleanup;
    }
    temporary_file_exists = 1;
    if (DeleteFileW(base_path) == FALSE) {
        wf_record_win32_failure(&failure, 14);
        goto cleanup;
    }
    temporary_file_exists = 0;
    if (CreateDirectoryW(base_path, NULL) == FALSE) {
        wf_record_win32_failure(&failure, 15);
        goto cleanup;
    }
    base_exists = 1;
    WF_REQUIRE(
        wf_fixture_join_path(
            root_path,
            WF_ARRAY_COUNT(root_path),
            base_path,
            L"root"
        ),
        16
    );
    WF_REQUIRE(
        wf_fixture_join_path(
            ambient_path,
            WF_ARRAY_COUNT(ambient_path),
            base_path,
            L"ambient"
        ),
        17
    );
    if (CreateDirectoryW(root_path, NULL) == FALSE) {
        wf_record_win32_failure(&failure, 18);
        goto cleanup;
    }
    root_exists = 1;
    if (CreateDirectoryW(ambient_path, NULL) == FALSE) {
        wf_record_win32_failure(&failure, 19);
        goto cleanup;
    }
    ambient_exists = 1;
    WF_REQUIRE(
        wf_fixture_join_path(
            regular_path,
            WF_ARRAY_COUNT(regular_path),
            root_path,
            wf_regular_name
        ),
        20
    );
    WF_REQUIRE(
        wf_fixture_join_path(
            directory_path,
            WF_ARRAY_COUNT(directory_path),
            root_path,
            wf_directory_name
        ),
        21
    );
    WF_REQUIRE(
        wf_fixture_join_path(
            link_path,
            WF_ARRAY_COUNT(link_path),
            root_path,
            wf_link_name
        ),
        22
    );
    {
        HANDLE handle = CreateFileW(
            regular_path,
            GENERIC_WRITE | FILE_READ_ATTRIBUTES,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            NULL,
            CREATE_NEW,
            FILE_ATTRIBUTE_NORMAL,
            NULL
        );
        if (!wf_adopt_handle(&fixture_file, handle)) {
            wf_record_win32_failure(&failure, 23);
            goto cleanup;
        }
        regular_exists = 1;
    }
    if (CreateDirectoryW(directory_path, NULL) == FALSE) {
        wf_record_win32_failure(&failure, 24);
        goto cleanup;
    }
    directory_exists = 1;
    WF_REQUIRE(wf_close_handle_once(&fixture_file), 25);

    if (SetCurrentDirectoryW(root_path) == FALSE) {
        wf_record_win32_failure(&failure, 26);
        goto cleanup;
    }
    cwd_changed = 1;
    {
        HANDLE handle = CreateFileW(
            L".",
            FILE_LIST_DIRECTORY | FILE_TRAVERSE | FILE_READ_ATTRIBUTES
                | SYNCHRONIZE,
            FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
            NULL,
            OPEN_EXISTING,
            FILE_FLAG_BACKUP_SEMANTICS | FILE_FLAG_OPEN_REPARSE_POINT,
            NULL
        );
        if (!wf_adopt_handle(&cwd_directory, handle)) {
            wf_record_win32_failure(&failure, 27);
            goto cleanup;
        }
    }
    if (SetCurrentDirectoryW(ambient_path) == FALSE) {
        wf_record_win32_failure(&failure, 28);
        goto cleanup;
    }
    WF_REQUIRE(
        GetFileAttributesW(wf_regular_name) == INVALID_FILE_ATTRIBUTES
            && GetLastError() == ERROR_FILE_NOT_FOUND,
        29
    );

    for (index = 0; index < WF_ARRAY_COUNT(rejection_cases); ++index) {
        wf_component_open_result rejected = wf_open_component(
            &nt_api,
            cwd_directory.value,
            rejection_cases[index].units,
            rejection_cases[index].count,
            FILE_OPEN_REPARSE_POINT
        );
        WF_REQUIRE(
            rejected.verdict == rejection_cases[index].expected
                && rejected.native_called == 0
                && rejected.handle == INVALID_HANDLE_VALUE,
            (unsigned)(40 + index)
        );
    }

    {
        wf_component_open_result opened = wf_open_component(
            &nt_api,
            cwd_directory.value,
            wf_regular_name,
            WF_ARRAY_COUNT(wf_regular_name) - 1,
            FILE_NON_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT
        );
        DWORD attributes;
        DWORD tag;
        failure.nt_status = opened.status;
        WF_REQUIRE(opened.native_called != 0 && wf_nt_success(opened.status), 50);
        WF_REQUIRE(wf_adopt_handle(&opened_file, opened.handle), 51);
        if (!wf_handle_attributes(opened_file.value, &attributes, &tag)) {
            wf_record_win32_failure(&failure, 52);
            goto cleanup;
        }
        WF_REQUIRE(
            (attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT))
                    == 0
                && tag == 0,
            53
        );
    }
    {
        wf_component_open_result opened = wf_open_component(
            &nt_api,
            cwd_directory.value,
            wf_directory_name,
            WF_ARRAY_COUNT(wf_directory_name) - 1,
            FILE_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT
        );
        DWORD attributes;
        DWORD tag;
        failure.nt_status = opened.status;
        WF_REQUIRE(opened.native_called != 0 && wf_nt_success(opened.status), 54);
        WF_REQUIRE(wf_adopt_handle(&opened_directory, opened.handle), 55);
        if (!wf_handle_attributes(opened_directory.value, &attributes, &tag)) {
            wf_record_win32_failure(&failure, 56);
            goto cleanup;
        }
        WF_REQUIRE(
            (attributes & FILE_ATTRIBUTE_DIRECTORY) != 0
                && (attributes & FILE_ATTRIBUTE_REPARSE_POINT) == 0
                && tag == 0,
            57
        );
    }
    {
        int found_regular;
        int found_directory;
        NTSTATUS enumeration_status;
        WF_REQUIRE(
            wf_enumerate_directory_handle(
                &nt_api,
                cwd_directory.value,
                &found_regular,
                &found_directory,
                &enumeration_status
            ),
            58
        );
        failure.nt_status = enumeration_status;
        WF_REQUIRE(found_regular != 0 && found_directory != 0, 59);
    }
    {
        wf_component_open_result missing = wf_open_component(
            &nt_api,
            cwd_directory.value,
            wf_missing_name,
            WF_ARRAY_COUNT(wf_missing_name) - 1,
            FILE_NON_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT
        );
        ULONG mapped;
        failure.nt_status = missing.status;
        WF_REQUIRE(missing.native_called != 0 && !wf_nt_success(missing.status), 60);
        mapped = nt_api.status_to_dos_error(missing.status);
        WF_REQUIRE(mapped != ERROR_SUCCESS && mapped != ERROR_MR_MID_NOT_FOUND, 61);
    }

    if (CreateSymbolicLinkW(
            link_path,
            regular_path,
            SYMBOLIC_LINK_FLAG_ALLOW_UNPRIVILEGED_CREATE
        ) == FALSE) {
        DWORD first_error = GetLastError();
        if (first_error == ERROR_INVALID_PARAMETER
            && CreateSymbolicLinkW(link_path, regular_path, 0) != FALSE) {
            link_exists = 1;
        } else {
            DWORD final_error = first_error == ERROR_INVALID_PARAMETER
                ? GetLastError()
                : first_error;
            if (wf_reparse_creation_unavailable(final_error)) {
                reparse_skipped = 1;
                reparse_skip_error = final_error;
                outcome = 77;
                goto cleanup;
            }
            failure.check = 62;
            failure.win32_error = final_error;
            goto cleanup;
        }
    } else {
        link_exists = 1;
    }
    {
        wf_component_open_result opened = wf_open_component(
            &nt_api,
            cwd_directory.value,
            wf_link_name,
            WF_ARRAY_COUNT(wf_link_name) - 1,
            FILE_NON_DIRECTORY_FILE | FILE_OPEN_REPARSE_POINT
        );
        DWORD attributes;
        DWORD tag;
        failure.nt_status = opened.status;
        WF_REQUIRE(opened.native_called != 0 && wf_nt_success(opened.status), 63);
        WF_REQUIRE(wf_adopt_handle(&nofollow_link, opened.handle), 64);
        if (!wf_handle_attributes(nofollow_link.value, &attributes, &tag)) {
            wf_record_win32_failure(&failure, 65);
            goto cleanup;
        }
        WF_REQUIRE(
            (attributes & FILE_ATTRIBUTE_REPARSE_POINT) != 0
                && tag == IO_REPARSE_TAG_SYMLINK,
            66
        );
    }
    {
        wf_component_open_result opened = wf_open_component(
            &nt_api,
            cwd_directory.value,
            wf_link_name,
            WF_ARRAY_COUNT(wf_link_name) - 1,
            FILE_NON_DIRECTORY_FILE
        );
        DWORD attributes;
        DWORD tag;
        failure.nt_status = opened.status;
        WF_REQUIRE(opened.native_called != 0 && wf_nt_success(opened.status), 67);
        WF_REQUIRE(wf_adopt_handle(&followed_link, opened.handle), 68);
        if (!wf_handle_attributes(followed_link.value, &attributes, &tag)) {
            wf_record_win32_failure(&failure, 69);
            goto cleanup;
        }
        WF_REQUIRE(
            (attributes & (FILE_ATTRIBUTE_DIRECTORY | FILE_ATTRIBUTE_REPARSE_POINT))
                    == 0
                && tag == 0,
            70
        );
    }
    outcome = 0;

cleanup:
    if (cwd_changed != 0 && SetCurrentDirectoryW(original_cwd) == FALSE
        && failure.check == 0) {
        wf_record_win32_failure(&failure, 80);
        outcome = 1;
    }
    if (wf_handle_live(&followed_link)
        && !wf_close_handle_once(&followed_link) && failure.check == 0) {
        wf_record_win32_failure(&failure, 81);
        outcome = 1;
    }
    if (wf_handle_live(&nofollow_link)
        && !wf_close_handle_once(&nofollow_link) && failure.check == 0) {
        wf_record_win32_failure(&failure, 82);
        outcome = 1;
    }
    if (wf_handle_live(&opened_directory)
        && !wf_close_handle_once(&opened_directory) && failure.check == 0) {
        wf_record_win32_failure(&failure, 83);
        outcome = 1;
    }
    if (wf_handle_live(&opened_file)
        && !wf_close_handle_once(&opened_file) && failure.check == 0) {
        wf_record_win32_failure(&failure, 84);
        outcome = 1;
    }
    if (wf_handle_live(&cwd_directory)
        && !wf_close_handle_once(&cwd_directory) && failure.check == 0) {
        wf_record_win32_failure(&failure, 85);
        outcome = 1;
    }
    if (wf_handle_live(&fixture_file)
        && !wf_close_handle_once(&fixture_file) && failure.check == 0) {
        wf_record_win32_failure(&failure, 86);
        outcome = 1;
    }
    if (!wf_closed_exactly_once(&followed_link)
        || !wf_closed_exactly_once(&nofollow_link)
        || !wf_closed_exactly_once(&opened_directory)
        || !wf_closed_exactly_once(&opened_file)
        || !wf_closed_exactly_once(&cwd_directory)
        || !wf_closed_exactly_once(&fixture_file)) {
        if (failure.check == 0) {
            failure.check = 87;
        }
        outcome = 1;
    }
    if (link_exists != 0 && DeleteFileW(link_path) == FALSE
        && failure.check == 0) {
        wf_record_win32_failure(&failure, 88);
        outcome = 1;
    }
    if (regular_exists != 0 && DeleteFileW(regular_path) == FALSE
        && failure.check == 0) {
        wf_record_win32_failure(&failure, 89);
        outcome = 1;
    }
    if (directory_exists != 0 && RemoveDirectoryW(directory_path) == FALSE
        && failure.check == 0) {
        wf_record_win32_failure(&failure, 90);
        outcome = 1;
    }
    if (ambient_exists != 0 && RemoveDirectoryW(ambient_path) == FALSE
        && failure.check == 0) {
        wf_record_win32_failure(&failure, 91);
        outcome = 1;
    }
    if (root_exists != 0 && RemoveDirectoryW(root_path) == FALSE
        && failure.check == 0) {
        wf_record_win32_failure(&failure, 92);
        outcome = 1;
    }
    if (base_exists != 0 && RemoveDirectoryW(base_path) == FALSE
        && failure.check == 0) {
        wf_record_win32_failure(&failure, 93);
        outcome = 1;
    }
    if (temporary_file_exists != 0 && DeleteFileW(base_path) == FALSE
        && failure.check == 0) {
        wf_record_win32_failure(&failure, 94);
        outcome = 1;
    }

    if (failure.check != 0 || outcome == 1) {
        fprintf(
            stderr,
            "windows-native-namespace-probe status=fail check=%u "
            "win32=%lu ntstatus=0x%08lx\n",
            failure.check,
            (unsigned long)failure.win32_error,
            (unsigned long)(uint32_t)failure.nt_status
        );
        return 1;
    }
    if (reparse_skipped != 0 || outcome == 77) {
        printf(
            "windows-native-namespace-probe status=skip "
            "reason=reparse-create-unavailable win32=%lu\n",
            (unsigned long)reparse_skip_error
        );
        return 77;
    }
    printf("windows-native-namespace-probe status=pass\n");
    return 0;

#undef WF_REQUIRE
}
