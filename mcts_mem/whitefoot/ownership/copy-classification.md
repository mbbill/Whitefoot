- Values are classified copy or affine by type: primitives, shared borrows, shared slice views, and tag-only enums copy on use; exclusive views and the remaining owned composites are affine.
- Copy values are used bare in the written body. A generic body's consuming spelling is checked against its declared bound and is not rejected again when a concrete argument is copy.
- Copy values are exempt from the affine consumption rules everywhere they bind: return, error propagation, owned-argument passing, and match-scrutinee consumption all skip the copy class.

## Facts

- 2026-07-07 statement: the primitive-and-shared-borrow core of the classification is Featherweight Rust Def 3.6 verbatim ("T has copy semantics when T = int or T = &w"); tag-only enums are the recorded extension beyond FR. (sourced)
- 2026-07-10 measurement: after the amendment, the chunk-summary wc classifier rewritten in i1 dataflow reached 134 ms, matching C (132.6 ms) and safe Rust (133.9 ms) at one thread, from 220–242 ms before — the workload's language-level penalty was entirely the affine-Bool workaround. (sourced)

- 2026-09-09 correction: OWN-1 and VIEW-1 include shared slice views in the copy class. The older enum experiment does not establish the comparative merits of this later view classification. (sourced)
- 2026-09-09 correction: FN-2 checks generic consuming spellings at the template bound rather than rejecting an admitted move after copy substitution. Rechecking that spelling per concrete type would contradict the selected generic authoring policy. (sourced)

## Moves

- 2026-07-10 (c658bc34) replaced [[uniform-affine-enums]]: affine classification of resource-free tag-only values bought zero safety while taxing every boolean — an owned Bool could not be loop-carried state or flow through band/bnot dataflow, forcing integer-flag workarounds whose i64 recurrence vectorized at width 2x4 instead of 16, a measured 1.6–1.8x kernel loss (sourced)
