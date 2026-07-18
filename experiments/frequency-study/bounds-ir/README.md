# Surviving bounds-check finder

This is a small, throwaway corpus-audit helper. It scans optimized textual
LLVM IR for first-party functions that still directly call or invoke Rust's
`panic_bounds_check`, then emits one deterministic JSON document:

```sh
python3 -B experiments/frequency-study/bounds-ir/collect.py \
  --source-root /path/to/project /path/to/llvm-ir --pretty
```

`/path/to/llvm-ir` may be one or more `.ll` files or directories. Directories
are searched recursively. Debug locations under `--source-root` establish
first-party attribution; hits outside it or without usable attribution are
reported separately as `unattributed_hits`. Any empty, unreadable, malformed,
or non-IR input aborts the whole run instead of producing clean-zero evidence.

Every result is only a `heuristic-surviving-bounds-candidate`. A surviving
panic edge may be cold, unavoidable, introduced by expansion or inlining, or
unrelated to anything current Whitefoot can eliminate. Manually inspect and
profile promising hits before treating them as evidence of an advantage.

Run the stdlib-only tests with:

```sh
python3 -B -m unittest discover \
  -s experiments/frequency-study/bounds-ir -p 'test_*.py'
```
