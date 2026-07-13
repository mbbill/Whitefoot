# E0.1 baseline-harness amendments

No scored timing existed when this file was created.  These amendments govern
experiment infrastructure only; they do not amend the language, checker,
compiler, specification, or production runtime.

## A1 — macOS native-main stack reservation (2026-07-13)

The fresh-process `F-SOA` harness compiled the byte-exact pre-prototype
facts-off IR: 1,860,733 bytes, SHA-256
`23baa6cce795a7c8c21b66af2c2c01dbbeade8e40b5fe7dda64966db9f8e615a`.
With the default 8 MiB Mach-O main stack, it deterministically reached the stack
guard in `lexer_scan_one`; the crash backtrace continued through `lexer_run`,
`frontend_analyze_parts`, and `xlc_frontend_run`.

A non-timed threshold probe relinked the identical IR and driver.  A 9 MiB
reservation still failed; 10, 12, and 16 MiB returned the frozen report and
correctness digest.  The harness therefore uses the deliberately conservative
64 MiB Mach-O reservation `-Wl,-stack_size,0x4000000` rather than tuning near
the observed threshold.

The exact same option is mandatory for every future native comparison arm and
is recorded in each build manifest and score binding.  It may not be varied per
candidate.  The option changes neither source nor generated IR, optimization,
target CPU, or the timed wrapper boundary.  It reserves virtual address
capacity; future memory work must measure touched pages/physical footprint
separately rather than treating 64 MiB as live tape memory.

The v1 harness fails closed on non-Darwin hosts.  A Linux runner must first
record and equalize its process stack limit across all arms.
