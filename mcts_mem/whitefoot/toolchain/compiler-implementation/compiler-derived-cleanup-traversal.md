- A compiler-derived drop whose type's cleanup can reach that type again runs on an explicit worklist; every other drop keeps the straight-line expansion whose depth its type bounds (`DropPlan`).
- Which drops those are is decided by strongly connected components over the cleanup graph, never by a name, a shape, a project, a corpus, or a source form.
- Only edges inside such a component are carried on the worklist; an edge that leaves one stays straight-line.
- Both owning indirections a cleanup cycle can close through have an arm: one entry names a whole heap block holding one content, and a buffer takes one entry per live element plus one for the block.
- Every pending entry names storage the traversal still holds, and the pending list is bounded by the structure being dismantled rather than by the depth reached.
- A reclamation order the language fixes is produced by the order entries are pushed, against a traversal that takes them last-in first-out.
- Growth of the pending list that the host refuses writes the same resource record as any other refused allocation.

## Facts

- 2026-08-23 (dc8cf1a3) measurement: on a boxed spine whose source contains no recursive function at all, the expansion form writes a stack resource record and aborts at 35,000,000 levels where the worklist completes at exit 0 in 2.25 GB of peak footprint, and still completes at 100,000,000. (sourced)
- 2026-08-23 (04f119ef) pitfall: the worklist first shipped with only its heap-block arm, on the stated ground that a nominal recursive through a buffer has no selected target layout and so no program can reach one. A heap block supplies the indirection the layout needs while the buffer stays inside the cycle, so the shape is four lines of source, and refusing it turned a program that had compiled and run into a bare compiler-invariant failure with no rule and no source coordinate. (code)
- 2026-08-23 rationale: a buffer arm that took one entry per buffer would have to resume the element walk, so the entry would carry a cursor and a length as well as a pointer. One entry per element instead keeps an entry two words and yields the fixed ascending-index-then-release order directly out of the last-in first-out discipline, with nothing to resume. (code)

## Moves

- 2026-08-23 (04f119ef) replaced [[recursive-cleanup-expansion]]: a compiler-generated drop that recursed over the value descended the machine stack in code the writer never wrote, cannot instrument, and cannot bound, and it ran after the program had already spent its stack (sourced)
