# Containers over the x1 language

This investigation assumes the owner-decided framework of PR #70 is fully
implemented. Its pinned rule/source baseline is
`efd6ebc9efc3012735f4ddedf6979617d321d3a6`, kernel v0.60. Implementation
failures of that revision are recorded separately from language limits; fixing
or completing PR #70 is outside this investigation. The active
[specification](../../../spec/kernel-spec.md) remains the language authority.

The question is whether full system-container operations can be expressed at
competitive cost using the existing framework, and which implementation or
measurement should come next. One global heap, local nonescaping references,
structural copy/drop capabilities, ordinary owned values, and monomorphized
`interface`/`binding` behavior are premises, not alternatives reopened here.
Older provider/refusal and region contracts in [REASSESSMENT.md](REASSESSMENT.md)
and [FOUNDATION.md](FOUNDATION.md) are dated evidence, not requirements of this
library. A checked branch is acceptable when its cost meets the operation's
requirements; inability to erase it is not itself a language gap.

## Evidence and comparison criteria

The [external workload traces](EXTERNAL-WORKLOADS.md) supply application
contracts from pinned Rust, C++ and Go programs, including the later Linux,
Redis and SQLite extension. They do not measure prevalence. Application needs
must be separated from the host language's chosen representation: e.g. stable
logical identity does not by itself require pointer-linked nodes.

Use these labels throughout the assessment:

- **Rule deduction:** follows from the cited v0.60 rule, not a compiler run.
- **Source observation:** present in pinned source or generated lowering.
- **Checked experiment:** an identified program passed an identified binary.
- **Measured control:** an identified native model, not WF performance.
- **Candidate:** an interface/representation awaiting implementation evidence.

Before any new selection experiment, keep the operation contract and its
observable result fixed. For each representation record construction, lookup,
mutation, growth and cleanup; occupied and reserved bytes; allocations;
element transfers; and the facts required across helpers. Test copy, owning
droppable and must-consume elements separately where relevant. No failed
program may be called a language limit when it contradicts the specification.

The initial discriminators are:

1. Complexity: drain must move O(n) elements, ordinary probing must terminate
   after at most capacity probes even with hostile hash/equality, and rehash
   must account for its new allocation plus old live elements. Reject an
   implementation that meets only a small fixed-size timing while violating
   the promised complexity.
2. Sparse storage: compare one backing of valid enum slots with boxed payloads
   and with a dense payload store plus sparse indexes. Count all metadata and
   identity-repair work; do not charge a tag only to one side or assume niche
   encoding in a WF layout.
3. References: distinguish stable slot identity, stable physical addresses and
   retained membership. Compare the same removal/invalidation contract.
4. Construction: retain helper boundaries and separate the irreducible move
   between different storage from avoidable temporary/result transfers.
5. New proof or storage machinery needs a concrete consumer and evidence of
   the remaining cost after the best ordinary representation. No universal
   percentage threshold is invented for all workloads.

## Scope of the resulting comparison

The common families are growable vector, ring deque, generational slab,
owning hash map, priority queue, and an ordered tree. The ceiling challenges
are multiple indexes retaining one object, large-value construction and
compact variable-sized records. A composite in-memory index workload will
connect the families without being described as a production database.

This note is the current home for this comparison, with experimental evidence
linked at its source. It will retain its pinned conditions when subsequent
implementation replaces individual candidate conclusions.
