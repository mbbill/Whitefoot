/* One fail-closed Windows process sample for io-completion-bench.
 *
 * PowerShell orchestrates the alternating paired schedule, but it does not
 * measure it. This runner launches one child on an explicit processor mask,
 * captures its complete stdout and stderr without pipe backpressure, checks
 * the exact published bytes and exit status, and reports QPC wall time plus
 * the child's user and kernel CPU time as one TSV row. A bad sample never
 * becomes a timing.
 */

#define WIN32_LEAN_AND_MEAN

#include <windows.h>

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <wchar.h>

#define WF_WINDOWS_RUNNER_CAPTURE_LIMIT (1024u * 1024u)
#define WF_WINDOWS_RUNNER_TIMEOUT_MS 120000u

static void wf_windows_error(const wchar_t *operation) {
    DWORD error = GetLastError();
    wchar_t *message = NULL;
    DWORD flags = FORMAT_MESSAGE_ALLOCATE_BUFFER
        | FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS;
    (void)FormatMessageW(
        flags,
        NULL,
        error,
        0,
        (wchar_t *)(void *)&message,
        0,
        NULL
    );
    fwprintf(
        stderr,
        L"windows-runner: %ls failed (%lu)%ls%ls\n",
        operation,
        (unsigned long)error,
        message == NULL ? L"" : L": ",
        message == NULL ? L"" : message
    );
    if (message != NULL) {
        LocalFree(message);
    }
}

static int wf_label_valid(const wchar_t *label) {
    const wchar_t *at;
    if (label == NULL || *label == L'\0') {
        return 0;
    }
    for (at = label; *at != L'\0'; ++at) {
        if (*at == L'\t' || *at == L'\r' || *at == L'\n') {
            return 0;
        }
    }
    return 1;
}

static int wf_read_file(
    const wchar_t *path,
    unsigned char **bytes_out,
    size_t *length_out
) {
    HANDLE file;
    LARGE_INTEGER size;
    unsigned char *bytes;
    DWORD read = 0;
    if (path == NULL || bytes_out == NULL || length_out == NULL) {
        SetLastError(ERROR_INVALID_PARAMETER);
        return 0;
    }
    *bytes_out = NULL;
    *length_out = 0;
    file = CreateFileW(
        path,
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE,
        NULL,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        NULL
    );
    if (file == INVALID_HANDLE_VALUE) {
        wf_windows_error(L"CreateFileW(capture)");
        return 0;
    }
    if (GetFileSizeEx(file, &size) == FALSE) {
        wf_windows_error(L"GetFileSizeEx(capture)");
        CloseHandle(file);
        return 0;
    }
    if (size.QuadPart < 0
        || (uint64_t)size.QuadPart > WF_WINDOWS_RUNNER_CAPTURE_LIMIT) {
        SetLastError(ERROR_FILE_TOO_LARGE);
        wf_windows_error(L"capture size");
        CloseHandle(file);
        return 0;
    }
    bytes = (unsigned char *)HeapAlloc(
        GetProcessHeap(),
        0,
        (size_t)size.QuadPart + 1u
    );
    if (bytes == NULL) {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        wf_windows_error(L"HeapAlloc(capture)");
        CloseHandle(file);
        return 0;
    }
    if (size.QuadPart != 0
        && (ReadFile(
                file,
                bytes,
                (DWORD)size.QuadPart,
                &read,
                NULL
            ) == FALSE
            || read != (DWORD)size.QuadPart)) {
        wf_windows_error(L"ReadFile(capture)");
        HeapFree(GetProcessHeap(), 0, bytes);
        CloseHandle(file);
        return 0;
    }
    if (CloseHandle(file) == FALSE) {
        wf_windows_error(L"CloseHandle(capture)");
        HeapFree(GetProcessHeap(), 0, bytes);
        return 0;
    }
    bytes[(size_t)size.QuadPart] = 0;
    *bytes_out = bytes;
    *length_out = (size_t)size.QuadPart;
    return 1;
}

static int wf_append_quoted_argument(
    wchar_t **cursor,
    size_t *remaining,
    const wchar_t *argument
) {
    const wchar_t *at = argument;
    size_t slashes = 0;
#define WF_APPEND(character)                                                   \
    do {                                                                       \
        if (*remaining <= 1u) {                                                \
            return 0;                                                          \
        }                                                                      \
        **cursor = (character);                                                \
        *cursor += 1;                                                          \
        *remaining -= 1u;                                                      \
    } while (0)
    WF_APPEND(L'"');
    for (;;) {
        if (*at == L'\\') {
            slashes += 1u;
            at += 1;
            continue;
        }
        if (*at == L'"') {
            size_t count;
            for (count = 0; count < slashes * 2u + 1u; ++count) {
                WF_APPEND(L'\\');
            }
            WF_APPEND(L'"');
            slashes = 0;
            at += 1;
            continue;
        }
        if (*at == L'\0') {
            size_t count;
            for (count = 0; count < slashes * 2u; ++count) {
                WF_APPEND(L'\\');
            }
            break;
        }
        while (slashes != 0) {
            WF_APPEND(L'\\');
            slashes -= 1u;
        }
        WF_APPEND(*at);
        at += 1;
    }
    WF_APPEND(L'"');
#undef WF_APPEND
    return 1;
}

static wchar_t *wf_command_line(int argc, wchar_t **argv) {
    size_t capacity = 1u;
    size_t remaining;
    wchar_t *line;
    wchar_t *cursor;
    int index;
    for (index = 5; index < argc; ++index) {
        size_t length = wcslen(argv[index]);
        if (length > (SIZE_MAX - capacity - 4u) / 2u) {
            SetLastError(ERROR_ARITHMETIC_OVERFLOW);
            return NULL;
        }
        capacity += length * 2u + 4u;
    }
    line = (wchar_t *)HeapAlloc(
        GetProcessHeap(),
        0,
        capacity * sizeof(wchar_t)
    );
    if (line == NULL) {
        SetLastError(ERROR_NOT_ENOUGH_MEMORY);
        return NULL;
    }
    cursor = line;
    remaining = capacity;
    for (index = 5; index < argc; ++index) {
        if (index != 5) {
            if (remaining <= 1u) {
                HeapFree(GetProcessHeap(), 0, line);
                SetLastError(ERROR_INSUFFICIENT_BUFFER);
                return NULL;
            }
            *cursor++ = L' ';
            remaining -= 1u;
        }
        if (!wf_append_quoted_argument(&cursor, &remaining, argv[index])) {
            HeapFree(GetProcessHeap(), 0, line);
            SetLastError(ERROR_INSUFFICIENT_BUFFER);
            return NULL;
        }
    }
    *cursor = L'\0';
    return line;
}

static uint64_t wf_file_time_100ns(FILETIME value) {
    ULARGE_INTEGER wide;
    wide.LowPart = value.dwLowDateTime;
    wide.HighPart = value.dwHighDateTime;
    return wide.QuadPart;
}

static void wf_remove_capture(const wchar_t *path) {
    if (path != NULL && *path != L'\0' && DeleteFileW(path) == FALSE) {
        wf_windows_error(L"DeleteFileW(capture)");
    }
}

int wmain(int argc, wchar_t **argv) {
    const wchar_t *label;
    const wchar_t *work_directory;
    const wchar_t *expected_path;
    const wchar_t *executable;
    wchar_t *end = NULL;
    unsigned long long parsed_mask;
    DWORD_PTR affinity_mask;
    DWORD_PTR process_mask = 0;
    DWORD_PTR system_mask = 0;
    wchar_t stdout_path[MAX_PATH] = {0};
    wchar_t stderr_path[MAX_PATH] = {0};
    SECURITY_ATTRIBUTES security;
    HANDLE stdout_file = INVALID_HANDLE_VALUE;
    HANDLE stderr_file = INVALID_HANDLE_VALUE;
    HANDLE stdin_file = INVALID_HANDLE_VALUE;
    STARTUPINFOW startup;
    PROCESS_INFORMATION process;
    wchar_t *command_line = NULL;
    LARGE_INTEGER frequency;
    LARGE_INTEGER started;
    LARGE_INTEGER finished;
    DWORD wait_result;
    DWORD exit_code = 0;
    FILETIME created;
    FILETIME exited;
    FILETIME kernel;
    FILETIME user;
    unsigned char *expected = NULL;
    unsigned char *actual = NULL;
    unsigned char *diagnostics = NULL;
    size_t expected_length = 0;
    size_t actual_length = 0;
    size_t diagnostics_length = 0;
    int passed = 0;
    if (argc < 6) {
        fputs(
            "usage: windows-runner LABEL WORKDIR EXPECTED AFFINITY_HEX "
            "EXECUTABLE [ARG ...]\n",
            stderr
        );
        return 2;
    }
    label = argv[1];
    work_directory = argv[2];
    expected_path = argv[3];
    executable = argv[5];
    if (!wf_label_valid(label)) {
        fputs("windows-runner: invalid label\n", stderr);
        return 2;
    }
    parsed_mask = wcstoull(argv[4], &end, 16);
    if (end == argv[4] || *end != L'\0' || parsed_mask == 0
        || parsed_mask > (unsigned long long)(DWORD_PTR)-1) {
        fputs("windows-runner: invalid affinity mask\n", stderr);
        return 2;
    }
    affinity_mask = (DWORD_PTR)parsed_mask;
    if (GetProcessAffinityMask(
            GetCurrentProcess(),
            &process_mask,
            &system_mask
        ) == FALSE) {
        wf_windows_error(L"GetProcessAffinityMask");
        return 2;
    }
    if ((affinity_mask & process_mask) != affinity_mask) {
        fputs(
            "windows-runner: affinity mask names an unavailable processor\n",
            stderr
        );
        return 2;
    }
    if (!wf_read_file(expected_path, &expected, &expected_length)) {
        return 2;
    }
    if (GetTempFileNameW(
            work_directory,
            L"wfo",
            0,
            stdout_path
        ) == 0
        || GetTempFileNameW(
               work_directory,
               L"wfe",
               0,
               stderr_path
           ) == 0) {
        wf_windows_error(L"GetTempFileNameW");
        goto cleanup;
    }
    security.nLength = sizeof(security);
    security.lpSecurityDescriptor = NULL;
    security.bInheritHandle = TRUE;
    stdout_file = CreateFileW(
        stdout_path,
        GENERIC_WRITE,
        FILE_SHARE_READ,
        &security,
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_TEMPORARY,
        NULL
    );
    stderr_file = CreateFileW(
        stderr_path,
        GENERIC_WRITE,
        FILE_SHARE_READ,
        &security,
        CREATE_ALWAYS,
        FILE_ATTRIBUTE_TEMPORARY,
        NULL
    );
    stdin_file = CreateFileW(
        L"NUL",
        GENERIC_READ,
        FILE_SHARE_READ | FILE_SHARE_WRITE,
        &security,
        OPEN_EXISTING,
        FILE_ATTRIBUTE_NORMAL,
        NULL
    );
    if (stdout_file == INVALID_HANDLE_VALUE
        || stderr_file == INVALID_HANDLE_VALUE
        || stdin_file == INVALID_HANDLE_VALUE) {
        wf_windows_error(L"CreateFileW(output capture)");
        goto cleanup;
    }
    command_line = wf_command_line(argc, argv);
    if (command_line == NULL) {
        wf_windows_error(L"build command line");
        goto cleanup;
    }
    ZeroMemory(&startup, sizeof(startup));
    startup.cb = sizeof(startup);
    startup.dwFlags = STARTF_USESTDHANDLES;
    startup.hStdInput = stdin_file;
    startup.hStdOutput = stdout_file;
    startup.hStdError = stderr_file;
    ZeroMemory(&process, sizeof(process));
    if (QueryPerformanceFrequency(&frequency) == FALSE) {
        wf_windows_error(L"QueryPerformanceFrequency");
        goto cleanup;
    }
    if (CreateProcessW(
            executable,
            command_line,
            NULL,
            NULL,
            TRUE,
            CREATE_NO_WINDOW | CREATE_SUSPENDED,
            NULL,
            work_directory,
            &startup,
            &process
        ) == FALSE) {
        wf_windows_error(L"CreateProcessW");
        goto cleanup;
    }
    if (CloseHandle(stdin_file) == FALSE) {
        stdin_file = INVALID_HANDLE_VALUE;
        wf_windows_error(L"CloseHandle(stdin)");
        TerminateProcess(process.hProcess, 123);
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        goto cleanup;
    }
    stdin_file = INVALID_HANDLE_VALUE;
    if (SetProcessAffinityMask(process.hProcess, affinity_mask) == FALSE) {
        wf_windows_error(L"SetProcessAffinityMask");
        TerminateProcess(process.hProcess, 120);
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        goto cleanup;
    }
    if (QueryPerformanceCounter(&started) == FALSE) {
        wf_windows_error(L"QueryPerformanceCounter(start)");
        TerminateProcess(process.hProcess, 122);
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        goto cleanup;
    }
    if (ResumeThread(process.hThread) == (DWORD)-1) {
        wf_windows_error(L"ResumeThread");
        TerminateProcess(process.hProcess, 121);
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        goto cleanup;
    }
    wait_result = WaitForSingleObject(
        process.hProcess,
        WF_WINDOWS_RUNNER_TIMEOUT_MS
    );
    if (wait_result == WAIT_TIMEOUT) {
        fwprintf(
            stderr,
            L"windows-runner: %ls exceeded the %lu ms sample timeout\n",
            label,
            (unsigned long)WF_WINDOWS_RUNNER_TIMEOUT_MS
        );
        (void)TerminateProcess(process.hProcess, 124);
        (void)WaitForSingleObject(process.hProcess, 5000u);
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        goto cleanup;
    }
    if (wait_result != WAIT_OBJECT_0) {
        wf_windows_error(L"WaitForSingleObject(sample)");
        (void)TerminateProcess(process.hProcess, 125);
        (void)WaitForSingleObject(process.hProcess, 5000u);
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        goto cleanup;
    }
    if (QueryPerformanceCounter(&finished) == FALSE) {
        wf_windows_error(L"QueryPerformanceCounter(finish)");
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        goto cleanup;
    }
    if (GetExitCodeProcess(process.hProcess, &exit_code) == FALSE
        || GetProcessTimes(
               process.hProcess,
               &created,
               &exited,
               &kernel,
               &user
           ) == FALSE) {
        wf_windows_error(L"wait for sampled process");
        CloseHandle(process.hThread);
        CloseHandle(process.hProcess);
        goto cleanup;
    }
    if (CloseHandle(process.hThread) == FALSE) {
        wf_windows_error(L"CloseHandle(process)");
        CloseHandle(process.hProcess);
        goto cleanup;
    }
    if (CloseHandle(process.hProcess) == FALSE) {
        wf_windows_error(L"CloseHandle(process)");
        goto cleanup;
    }
    if (CloseHandle(stdout_file) == FALSE) {
        stdout_file = INVALID_HANDLE_VALUE;
        wf_windows_error(L"CloseHandle(output capture)");
        CloseHandle(stderr_file);
        stderr_file = INVALID_HANDLE_VALUE;
        goto cleanup;
    }
    stdout_file = INVALID_HANDLE_VALUE;
    if (CloseHandle(stderr_file) == FALSE) {
        stderr_file = INVALID_HANDLE_VALUE;
        wf_windows_error(L"CloseHandle(output capture)");
        goto cleanup;
    }
    stderr_file = INVALID_HANDLE_VALUE;
    if (!wf_read_file(stdout_path, &actual, &actual_length)
        || !wf_read_file(
            stderr_path,
            &diagnostics,
            &diagnostics_length
        )) {
        goto cleanup;
    }
    if (exit_code != 0 || diagnostics_length != 0
        || actual_length != expected_length
        || memcmp(actual, expected, expected_length) != 0) {
        fwprintf(
            stderr,
            L"windows-runner: %ls invalid sample: exit=%lu stdout=%llu/%llu "
            L"stderr=%llu\n",
            label,
            (unsigned long)exit_code,
            (unsigned long long)actual_length,
            (unsigned long long)expected_length,
            (unsigned long long)diagnostics_length
        );
        if (diagnostics_length != 0) {
            fwrite(diagnostics, 1, diagnostics_length, stderr);
            if (diagnostics[diagnostics_length - 1u] != '\n') {
                fputc('\n', stderr);
            }
        }
        goto cleanup;
    }
    {
        double wall_ms = (double)(finished.QuadPart - started.QuadPart)
            * 1000.0 / (double)frequency.QuadPart;
        double user_ms = (double)wf_file_time_100ns(user) / 10000.0;
        double kernel_ms = (double)wf_file_time_100ns(kernel) / 10000.0;
        wprintf(
            L"%ls\t%.3f\t%.3f\t%.3f\t%lu\n",
            label,
            wall_ms,
            user_ms,
            kernel_ms,
            (unsigned long)exit_code
        );
    }
    passed = 1;

cleanup:
    if (stdout_file != INVALID_HANDLE_VALUE) {
        CloseHandle(stdout_file);
    }
    if (stderr_file != INVALID_HANDLE_VALUE) {
        CloseHandle(stderr_file);
    }
    if (stdin_file != INVALID_HANDLE_VALUE) {
        CloseHandle(stdin_file);
    }
    if (command_line != NULL) {
        HeapFree(GetProcessHeap(), 0, command_line);
    }
    if (expected != NULL) {
        HeapFree(GetProcessHeap(), 0, expected);
    }
    if (actual != NULL) {
        HeapFree(GetProcessHeap(), 0, actual);
    }
    if (diagnostics != NULL) {
        HeapFree(GetProcessHeap(), 0, diagnostics);
    }
    wf_remove_capture(stdout_path);
    wf_remove_capture(stderr_path);
    return passed ? 0 : 1;
}
