# E0.1 pre-prototype baseline

Date: 2026-07-13.  No E0.1 implementation existed when these facts were
recorded.

## Verification

At Git commit `58baa71fb4c36a4728dd42aea6b05ce4be7aa0b1`, the full root `make
check` completed green before the protocol edits.  Among its reported gates:

- checker model check: 10,000 cases, zero soundness violations;
- codegen parity: zero failures;
- conformance: 231 pass, 13 skip, all 90 rules covered;
- self parse: 1,029,044 bytes, 211,374 tokens, 105,550 AST nodes;
- semantic unit: 477 total, 15 clean, 462 legal-unsupported, zero rejected.

This is a correctness baseline, not a performance result.

Before any stage-0 prototype edit, the unchanged-source generated-IR pins were:

| Fixture | Facts | Raw LLVM bytes | SHA-256 |
|---|---|---:|---|
| canonical full compiler unit | off | 1,860,733 | `23baa6cce795a7c8c21b66af2c2c01dbbeade8e40b5fe7dda64966db9f8e615a` |
| canonical full compiler unit | on | 2,229,127 | `0cde7c30e63ea4e60277ed76fb50940012b9b900d04abfc4385ceb4816e95001` |
| `prototype/democ/examples/soa_kernel.xl` | off | 3,069 | `dfe27e6ac18799b2ac5e4d6f382fea3f979be04bd269ef64fe224c3c73d42d7c` |
| `prototype/democ/examples/soa_kernel.xl` | on | 3,779 | `fa80c462223036a8a2b67d0aabee4e232d326ee6c3327a8a8d6118cdbec20f5d` |

These are hard byte-identity gates for codegen changes, not benchmark values.
The pre-prototype `prototype/democ/democ.py` SHA-256 was
`211f7caee393ac5822df71cdd0777a2c9b77dc2fafd47d4f3fd4d0aeed9c5336`.

## Current allocation and utilization

`frontend_unit_new` creates 30 fixed buffers.  `xlc_frontend_run` uses
`source_length + 1`, or `1,029,045` elements for the canonical compiler unit,
as every column's capacity.

Current stage-0 element widths are:

| Tape | SoA bytes per index | Natural AoS stride | Used elements |
|---|---:|---:|---:|
| Token: kind + start + end | 20 | 24 | 211,374 |
| AST: kind + six `u64` fields | 52 | 56 | 105,550 |

Consequences for this source:

- all 30 fixed columns request `214,041,360` bytes (`204.13 MiB`), excluding
  allocator metadata;
- fixed Token+AST SoA requests `74,091,240` bytes (`70.66 MiB`);
- their semantically used prefixes contain `9,716,080` bytes (`9.27 MiB`);
- fixed Token+AST AoS would request `82,323,600` bytes (`78.51 MiB`), 11.1%
  above the current Token+AST SoA because of record padding;
- changing only those two tapes to fixed AoS would increase the 30-column
  requested total by 3.85%, before allocator-count or locality effects.

Token and AST capacity utilization is therefore 20.54% and 10.26%.  There are
two independent possible effects: SoA avoids record padding and favors
single-column consumers; initialized-prefix storage avoids eagerly initializing
unused capacity.  The protocol does not attribute the latter to AoS.

## Existing implementation blocker

The stage-0 backend cannot safely be enabled by relaxing a checker rule.
`prototype/democ/democ.py` currently falls back to four-byte/i32 treatment for
unknown aggregate buffer elements, while buffer allocation, indexing, fill, and
local binding paths assume scalar elements.  A premature `buffer<Record>` would
mis-size allocations and generate incorrect addressing.

E0.1a therefore requires end-to-end record size/alignment, aggregate element
addressing, field projection, fill/copy behavior, alias metadata, and
cross-target layout tests before any benchmark arm is valid.

## Baseline interpretation

Parser production writes Token/Ast rows together, so AoS may improve producer
locality and reduce capacity checks.  Later compiler passes frequently project
kind, span, or child-link columns, so SoA may reduce fetched bytes and preserve
unit-stride/scoped-alias vectorization.  Neither representation dominates by
construction.  The present SoA remains the default until the complete frozen
workload proves otherwise.
