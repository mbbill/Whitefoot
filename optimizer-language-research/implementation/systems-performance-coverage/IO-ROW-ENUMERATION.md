# io-file row enumeration + UDP consumer pinning

Date: 2026-07-16

Status: research draft closing two open flags from `MEMBER-AUDIT-THREADS-IO.md`
(the "'plus what wal needs' must stop being a phrase" flag and the "UDP rows
must be pinned to numbered 51-map scenarios before ratification" flag). Not a
normative language document; input to the decision-gates line that must precede
wal ratification and to net-unit ratification. Sources: `CATALOG-V1-RECUT.md`
(io-file, line 73), `MEMBER-AUDIT-THREADS-IO.md`, `SCENARIO-DEMAND-MAP.md`
(the 51-scenario map).

---

## 1. Explicit io-file row enumeration (17 rows)

This replaces the two load-bearing phrases with a closed list:
- the re-cut's `stat/len` is **row 12 (fstat)**; the six audit fs/os rows
  (fstat, stat_path, readdir, mkdir, remove, ftruncate) are rows 11, 12,
  14-17; and **"plus what wal needs"** expands to rows **1 (O_DIRECT open),
  4/5 (pread/pwrite), 7 (fdatasync), 8 (fsync), 9 (dir-fsync), 10 (rename),
  11 (ftruncate), 13 (block-size query)**.
- Of these, the only row not already in the re-cut's line-73 list is
  **ftruncate**; the four fs-namespace rows (stat_path, readdir, mkdir,
  remove) are the seam the audit's FATAL exposed (never priced in any pocket).

**Darwin durability pin (applies to rows 7, 8, 9 — read before the table).**
On macOS, plain `fsync(2)`/`fdatasync(2)` return once the write reaches the
drive but do **not** force the drive's cache to stable media; the **durable
spelling is `fcntl(fd, F_FULLFSYNC)`**. Rows 7/8/9 MUST lower to `F_FULLFSYNC`
on Darwin to satisfy the WAL commit contract. `F_BARRIERFSYNC` gives ordering
only, not full durability, and is **not** sufficient for a commit. Until this
lowering is pinned, no macOS-dev-machine WAL-durability test counts as
evidence (audit open flag, `MEMBER-AUDIT` line 340).

| # | row | syscall(s) — Linux / macOS | contract one-liner (failure channel + teardown) | named consumer (scenario / wal / program) | platform divergence (durable spelling in **bold**) |
|---|---|---|---|---|---|
| 1 | **open** (O_DIRECT / alignment-honoring) | `openat` (`O_CLOEXEC` [`\|O_DIRECT`]) / `open` (`O_CLOEXEC`) + `fcntl(F_NOCACHE,1)` | `open(path, mode, flags) -> io_file_handle \| err(ENOENT/EACCES/EISDIR/EMFILE)`; teardown = close on handle drop (row 6) | #45 buffered file IO; #49 durability/wal; #32/#48 mmap open leg; wal (O_DIRECT leg) | **MATERIAL: macOS has no `O_DIRECT` flag.** Unbuffered = `fcntl(F_NOCACHE)` (advisory "don't cache", weaker than Linux `O_DIRECT`'s strict page-cache bypass); app-aligned buffers required either way; durability still needs row 7's F_FULLFSYNC. See §3(a). |
| 2 | **read** | `read` / `read` | `read(&uniq h, buf) -> 0..n` (0 = EOF/peer-FIN); EINTR retried; errno-class error; no teardown | #45; #46 (merged `net_recv`); #22/#23 scan-into-buffer; os.stdio fd0 | Low. `read(2)` portable; socket handles widen the errno set (ECONNRESET; SO_RCVTIMEO expiry -> timeout error value). |
| 3 | **write** | `write` / `write` | `write(&uniq h, bytes) -> n consumed` (partial; caller short-write loop); EPIPE-as-error; EINTR retried; no teardown | #45; #46 (merged `net_send`); #49 wal; os.stdio fd1/2 | Low, but depends on the **unconditional startup `SIGPIPE=IGN` invariant** (else socket write raises SIGPIPE on macOS); socket errno widens (EPIPE/ECONNRESET; SO_SNDTIMEO -> timeout). See §3(d). |
| 4 | **pread** | `pread` / `pread` | `pread(&h, buf, off) -> 0..n` at explicit offset, no file-offset mutation; 0/short = EOF; errno error; no teardown | #49 wal recovery (pread-to-EOF loop); #8 DB B-tree/SSTable page reads; #45 O_DIRECT reads | Low; `O_APPEND` interaction with pwrite noted on row 5. |
| 5 | **pwrite** | `pwrite` / `pwrite` | `pwrite(&h, bytes, off) -> n` at explicit offset; durability of bytes needs row 7; errno error; no teardown | #49 wal append (pwrite + fdatasync); #8 DB | Low; POSIX leaves `pwrite` + `O_APPEND` offset undefined — row opens WAL segments without `O_APPEND` so offset is authoritative. |
| 6 | **close** | `close` / `close` | `close(h)` consumes the linear handle exactly once; EINTR-on-close policy = fd is dead either way (no retry); merged `net_close` | all handle consumers; #46 sockets | Low. Linux frees the fd even on EINTR-return; row's no-retry policy owns the POSIX ambiguity; `SO_LINGER` deliberately unexposed. |
| 7 | **fdatasync** | `fdatasync` / **`fcntl(F_FULLFSYNC)`** | `fdatasync(&h) -> ok \| err(EIO)`; flushes data (+size metadata) to stable media; no teardown | #49 durability (dominant cost); wal commit; DB checkpoint | **MATERIAL — the pin.** macOS `fdatasync` does NOT flush the drive cache; durable spelling is **`fcntl(fd, F_FULLFSYNC)`**. See §3(b). |
| 8 | **fsync** | `fsync` / **`fcntl(F_FULLFSYNC)`** | `fsync(&h) -> ok \| err(EIO)`; flushes data + all inode metadata to stable media; no teardown | #49 checkpoint new-file fsync; build-tool atomic output publish; wal | **MATERIAL — same F_FULLFSYNC pin as row 7.** Linux `fsync` durable; macOS plain `fsync` is not. |
| 9 | **fsync-on-directory-handle** (dir-fsync) | `fsync` on `O_RDONLY\|O_DIRECTORY` fd / **`fcntl(F_FULLFSYNC)`** on dir fd | `dir_fsync(&dir_h) -> ok \| err`; makes prior namespace ops (rename/create) in that dir durable; no teardown | #49 checkpoint (rename-then-dir-fsync); build-tool publish; wal | **MATERIAL.** Linux requires explicit parent-dir fsync to make a rename durable; on APFS the durable spelling is **`F_FULLFSYNC` on the directory fd**. Plain dir `fsync` on macOS is not a durability barrier. See §3(b). |
| 10 | **rename** | `renameat2`/`rename` / `rename`/`renamex_np` | `rename(from, to) -> ok \| err(EXDEV/ENOENT/EACCES)`; atomic within one filesystem; durability requires row 9 after; no teardown | #49 atomic file replace; build-tool atomic publish; package-manager install | Low-material. Atomic-replace POSIX on both; no-clobber is `renameat2(RENAME_NOREPLACE)` (Linux) vs `renamex_np(RENAME_EXCL)` (macOS), sealed; cross-device is EXDEV on both. |
| 11 | **ftruncate** | `ftruncate` / `ftruncate` | `ftruncate(&write-h, len) -> ok \| err(EINVAL/EIO/EFBIG)`; shrink discards, extend reads back logical zeros; size-change durability needs row 7; no physical-alloc claim; no teardown | wal segment recycling; KV compaction space reclaim (correctness — bounded disk); build clean | Low. Both support shrink+extend; APFS vs ext4 sparse/preallocation differ but the contract is logical-content-only, so no divergence surfaces. |
| 12 | **fstat** (handle-based; the re-cut's `stat/len`) | `fstat`/`statx` / `fstat` | `fstat(&h) -> {kind, size:u64, mtime_ns} \| err(EIO)`; EBADF unreachable by handle linearity; size/mtime are ordinary **values, not optimizer facts**; no teardown | #32/#48 mmap len source (`fstat`-then-`map_file_ro` idiom); #45/#46 static-file Content-Length; build post-open verify | Low. `st_mtim` (Linux) vs `st_mtimespec` (macOS) sealed; mtime granularity is filesystem-dependent — ns field exposed with a granularity caveat, no cross-fs equality claim. |
| 13 | **logical-block-size query** | `ioctl(BLKSSZGET)` / `statx(STATX_DIOALIGN)` / `fstatfs f_bsize` — `fcntl(F_LOG2PHYS_EXT)` / `ioctl(DKIOCGETBLOCKSIZE)` / `fstatfs f_iosize` | `block_size(&h) -> {logical, physical} bytes \| err`; used to align O_DIRECT buffers/offsets (the 4Kn/512e gap); no teardown | #45 O_DIRECT aligned IO; #49 wal O_DIRECT leg | MATERIAL implementation divergence: different call per platform; the returned **value** unifies, but 4Kn-vs-512e correctness is a mandatory per-platform battery item. See §3(c). |
| 14 | **stat_path** (stat / lstat) | `newfstatat`/`statx` (±`AT_SYMLINK_NOFOLLOW`) / `stat`/`lstat`/`fstatat` | `stat(path, follow) -> {kind:{file\|dir\|symlink\|other}, size, mtime_ns} \| err(ENOENT/EACCES/ELOOP)`; `follow=false` = lstat; **inherently TOCTOU-racy, advisory snapshot**; correctness-critical reads must open-then-fstat; no teardown | build-tool incremental staleness at make/ninja par (1 syscall/path); recursive-grep entry classification + symlink-loop avoidance | Low. Same struct field divergence as fstat, sealed; ELOOP limits differ — row exposes the error, not the limit. |
| 15 | **readdir** | `openat`+`getdents64` / `fdopendir`+`getdirentries` | `opendir(path) -> DirStream \| err`; `next(&uniq DirStream) -> Option<{name, kind_hint}>`; `.`/`..` excluded; **no ordering**; concurrent-mod entries may be missed/duplicated (POSIX); `kind_hint` advisory (confirm via row 14); teardown = closedir on drop, infallible | recursive-grep tree walk; build-tool source globbing | Low. `getdents64` vs `getdirentries64` sealed; `d_type` population differs by filesystem on both — hence the `unknown` fallback in the contract. |
| 16 | **mkdir** | `mkdirat` / `mkdir` | `mkdir(path) -> ok \| err(EEXIST/ENOENT/EACCES)`; mode `0o777 & ~umask`; **single level** (mkdir-p is a checked-lib loop, EEXIST surfaced for race-tolerance); no teardown | build-tool output-directory creation | None material; identical on both targets. |
| 17 | **remove** (unlink + rmdir-empty, typed kind) | `unlinkat` (±`AT_REMOVEDIR`) / `unlink`/`rmdir` | `remove(path, kind:{file, empty_dir}) -> ok \| err(ENOENT/EACCES/ENOTEMPTY/kind-mismatch)`; POSIX unlink-while-open keeps data reachable until last close (compaction can unlink still-mapped files); recursive tree removal is a checked-lib walk over rows 15+17; no teardown | KV compaction reclaiming dead SSTable/WAL-segment files (correctness — else disk grows unbounded); build clean step | Low. unlink-while-open POSIX on both; kind-mismatch errno differs (EISDIR vs EPERM) — normalized by the row. |

Notes:
- **Row count: 17.** Byte/handle core (1-6), durability (7-11), introspection
  (12-13), fs-namespace (14-17).
- `filemap.map_file_ro` is **not** an io-file row (separate filemap pocket) but
  is row 12's headline consumer — `fstat`-then-`map_file_ro` is the stated
  idiom that closes the audit's unsatisfiable-len hole.
- The four fs-namespace rows (14-17) do not map to a single numbered F1-F51
  scenario — the demand map is structure/IO-shape centric and never enumerates
  "directory traversal." Their consumers are the audit's canonical composite
  programs (build tool, recursive grep, KV compaction); that they were invisible
  to the per-scenario map is exactly why the composition attack found them (the
  FATAL). This is honest scope, not a gap in this enumeration.

---

## 2. UDP row consumer pinning — verdict: **DEFER (all three)**

The three UDP rows are `udp_bind`, `udp_sendto`, `udp_recvfrom`. Walking the
full 51-scenario map for a numbered scenario that genuinely needs UDP **at par
in v1**:

- **#46 "Network sockets: TCP/UDP request paths" is the only UDP-naming
  scenario, and it does not need these rows at par.** Its contract is
  connection-oriented ("accept connections ... tens of thousands of concurrent
  connections"), its par targets are all TCP (redis ~100-200k GET/s/core,
  nginx, io_uring echo), and it explicitly classes UDP/QUIC as a **"distinct
  sub-shape [that] needs batching syscalls (recvmmsg, GSO)."** The three kept
  rows are plain **blocking single-datagram** ops — they are not the batching
  sub-shape, and the batching path rides the deferred readiness/completion
  machinery (#47 / evring, already deferred). So #46's UDP-at-par need is
  unmet by these rows and lives in deferred territory; #46's v1-served, par-
  bearing traffic is TCP, covered by the kept `tcp_connect`/`tcp_listen`/
  `tcp_accept` rows.
- **No other numbered scenario is UDP.** #47 (multiplexing) is deferred; #51
  (process/time/entropy/signals) has no socket surface. F1-F7 are storage /
  parsing / memory / concurrency.
- **The audit's own named UDP consumers are not numbered scenarios.** "Engines'
  realtime channels" and "telemetry" appear nowhere in the 51-map; "DNS-shaped
  services" resolve to `dns_resolve`, which the audit itself **DEFERs**, and the
  "stub-resolver over the kept UDP rows" is on that same deferred path.

Every UDP consumer the audit could name is either a non-numbered audience
hand-wave or an already-deferred capability. This is the loosest D16 second-
prong naming in the net unit (the audit flags it, line 344). Pinning would
require stretching a justification, which the task forbids.

**Recommendation: DEFER `udp_bind` / `udp_sendto` / `udp_recvfrom`,** aligning
them with the net unit's other consumer-less DEFERs (`readiness_wait`,
`dns_resolve`, `sendfile`). This is a change from the audit's conditional KEEP,
warranted because scenario-pinning — the audit's own ratification precondition
— fails.

**Trigger (reinstate when any fires):**
1. `dns_resolve` fires (a kept scenario must resolve names at runtime) **and**
   the chosen route is the checked-library stub resolver over UDP — that route
   needs `udp_bind`/`sendto`/`recvfrom` directly; reinstate together, at par-if-
   possible. (This is the audit's own stated stub-resolver path, line 330.)
2. A numbered demand-map scenario with a concrete UDP par target is admitted —
   e.g. #46's UDP/QUIC sub-shape is prioritized for v1 (which also un-defers the
   batching/`recvmmsg`+GSO path and the #47/evring machinery), or a new
   engine-realtime-channel / telemetry scenario lands with a measured par
   number. Plain blocking rows admit first; batching admits with #47.

Note: this defers only the **UDP leg**. The net unit's TCP leg
(`tcp_connect`/`tcp_listen`/`tcp_accept`/`sockopt_set`/`net_shutdown`/`net_sendv`
and the io-merged read/write/close) stays KEEP, pinned to #46 server scenarios.

---

## 3. Rows where Linux/macOS semantics genuinely diverge (flag, don't paper over)

Three genuine divergences; the first two are reconcilable **only** via a pinned
Darwin lowering (state which spelling is durable), the third only at the value
level.

**(a) Row 1 — O_DIRECT open has no clean unified contract.** Linux `O_DIRECT`
is strict page-cache bypass with hard alignment requirements; macOS has **no
`O_DIRECT` flag** — the nearest is `fcntl(fd, F_NOCACHE, 1)`, an *advisory*
"don't keep pages cached" hint that is semantically weaker (it does not
guarantee the DMA-direct, no-copy path Linux does, and does not by itself make
writes durable). A single contract can promise only the **intersection**:
"caching-avoidance requested, app-aligned buffers required, durability still via
row 7's F_FULLFSYNC." The row must state the macOS path is F_NOCACHE and must
not imply Linux-O_DIRECT guarantees on Darwin. **Flag: cannot be unified as one
strong contract; unifies only as the weaker intersection.**

**(b) Rows 7/8/9 — fsync/fdatasync/dir-fsync durability.** Reconcilable under
one contract **only** by pinning the Darwin lowering to **`fcntl(F_FULLFSYNC)`**
(the durable spelling). Plain macOS `fsync`/`fdatasync` return before the drive
cache is flushed, so a naive one-syscall-name mapping produces a contract that
is *falsely* unified — the dev machine can go green on exactly the platform
where semantics differ most (audit line 340). `F_BARRIERFSYNC` is ordering-only
and does **not** satisfy the commit contract. **Flag: unifiable, but only with
the F_FULLFSYNC lowering pinned and battery-verified before any macOS WAL-
durability result counts; this is the load-bearing pin of the whole io-file /
wal enumeration.**

**(c) Row 13 — logical-block-size query.** Different syscall per platform
(`ioctl(BLKSSZGET)`/`statx(STATX_DIOALIGN)` vs `ioctl(DKIOCGETBLOCKSIZE)`/
`fcntl(F_LOG2PHYS_EXT)`). The returned **value** (logical/physical block size)
unifies, so the contract is one row returning a number — but the 4Kn-vs-512e
correctness (whether the reported size matches what O_DIRECT alignment actually
demands) is a **mandatory per-platform battery item**, not something the
contract can assert portably. **Flag: unifies at the value level only;
correctness needs per-platform verification.**

Everything else unifies cleanly at the contract level (close EINTR policy,
fstat field naming/granularity, readdir d_type unknown-fallback, remove kind-
mismatch errno, rename no-clobber flag) — divergences owned inside the row, no
flag needed.
