# Discriminating programs for the ownership redesign

Research date: 2026-09-16. These programs are the fixed test set every debate
in the requirement-first ownership redesign derives its candidates against;
verdicts in [OPTIONS.md](OPTIONS.md)'s successor files cite them as P1..P10.
Pseudocode only; each program's required properties are fixed, the notation
is whatever a candidate needs. Supersede in place; remove with the
investigation.

program is accepted, rejected, or made parallel has not been evaluated.

## P1 Container split with a runtime index

```text
v: growable vector of u64, len n
(part_i, part_j) = take two elements i and j out of v for exclusive writing
write through part_i; write through part_j          // must be allowed, and may run in parallel
read v[k] for a runtime k                            // legal iff k is provably not i and not j; else the writer must branch
put both parts back; push(v, x)                      // must be allowed only after both are back
```
Required: sequential legality by proof, not by borrow scope; parallel permission
for the two writes; a diagnostic at the read of `v[k]` that names the missing
fact; no runtime check.

## P2 Arena with independently released blocks

```text
arena A with metadata M and byte storage S
b1 = alloc(A, 64); b2 = alloc(A, 64); b3 = alloc(A, 64)
fill(b1) ∥ fill(b2) ∥ fill(b3)                        // must be permitted to overlap
free(A, b2); b4 = alloc(A, 32)                        // reuse of b2's bytes; every old pointer into b2 must be refused afterwards
read(b1); free(A, b1); free(A, b3); free(A, b4)
```
Required: blocks are exclusively owned; metadata is written only at alloc and
free; nobody holds A exclusively between operations; a pointer into freed b2
is refused even though its address is reused by b4.

## P3 Graph with back edges in a pool

```text
nodes: pool of Node { next: handle-or-pointer, prev: handle-or-pointer, data: u64 }
insert node n between a and b (four link writes)
remove node m; the storage of m is reused for a later insert
traverse from any node following next, reading data
parallel map over all nodes writing data              // must be permitted: nodes are distinct
```
Required: link updates through several paths to one node are legal
sequentially; removal invalidates every path to m; parallel data writes are
permitted by distinctness of nodes; the invariant "next/prev point to live
nodes" is stated once and reused.

## P4 Cursor over a growing vector

```text
c = Cursor { at: pointer-or-handle to v, i: 0 }
a = c.read()                                          // needs v initialized and i < len
push(v, 5)                                            // may or may not reallocate
b = c.read()                                          // legal iff the candidate can carry "still valid" across push
free(v); c.read()                                     // must be refused
```
Required: a stored pointer or handle to a container in a struct; validity
decided by state and proof, not by a borrow that blocks push.

## P5 Struct-of-arrays kernel

```text
struct Cols { a, b, c, d, e, f, g, h: buffer<u64> }
for i in 0..n { a[i] = f(a[i], c[i], d[i]); b[i] = g(b[i], e[i], f[i], g[i], h[i]) }
```
Required: the eight columns are provably distinct storages inside the loop so
the backend can be told so (per-load alias facts, no runtime guards); PAR-2
iteration overlap permitted; obvious shape is the fast shape.

## P6 Range partition helper

```text
out: buffer of n bytes; k workers
for w in 0..k { helper(slice_of(out, w*s .. (w+1)*s), input) }   // helper writes its slice only
```
Required: iteration overlap permitted from the arithmetic of the ranges; the
helper's signature alone tells the caller what it writes; no worker count in
the source beyond k.

## P7 Take, put, replace through aliases

```text
p, q point to the same slot A holding an affine value
v = take(p); read(q)                                  // refused: hole
put(q, move v); read(p)                               // allowed
old = replace(p, new); read(q)                        // allowed, reads new
free A; read(p)                                       // refused forever
```
Required: state is a property of the storage, not of the pointer; take leaves
a hole that another alias may fill; free is permanent.

## P8 Conditional release and loop exits (CASES.md B10-B13, L04-L06)

```text
if c { free(a) } else { free(b) }; then use the survivor; then free it
loop { if stop { break }; take(p); ...; put(p) }; free(p) after the loop
```
Required: no runtime drop flag; the writer's own branches carry the state;
rejection names the path whose state differs.

## P9 A callee that changes storage state

```text
fn drain(x) leaves x's slot empty
fn reserve(v, m) may replace v's backing when cap < m
fn pick(c, x, y) returns one of two pointers with its permission
```
Required: each effect is visible in the signature; the caller judges by the
signature alone; the callee body is checked once against it.

## P10 Two holders, occasional writes, across threads (future)

```text
counter shared by two threads; each increments occasionally; a third reads
```
Required: synchronization cost only at the write; what the checker knows
before and after acquire; whether determinism is promised.

## Existing corpus programs to preserve

wfgrep, zlib-core-kernels, compute-bench, the accumulators snapshot cases:
whatever the current design permits to overlap under PAR-1 and PAR-2 must
remain permitted or the loss must be stated.
