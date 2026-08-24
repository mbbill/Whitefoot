- A compiler-derived drop expands its type's cleanup edges in place, and a drop that can reach its own type again calls itself.
- The depth of a destruction is the depth of the value, and no source construct names, bounds, or instruments it.
- A type recursive through a buffer reaches the same expansion, and its glue closes a cycle through the buffer's element drop.

## Moves

- 2026-08-23 (dc8cf1a3) replaced by [[compiler-derived-cleanup-traversal]]: a compiler-generated drop that recursed over the value descended the machine stack in code the writer never wrote, cannot instrument, and cannot bound, and it ran after the program had already spent its stack (sourced)
