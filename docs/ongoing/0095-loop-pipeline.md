# 0095 — staged loop pipeline, Stage A prerequisites

Branch `batch/0095-loop-pipeline`. The design is
`research/investigations/io-model/LOOP-PIPELINE.md` §1, §3, §7 "Batch 2" and
its §9 probe results, with `research/investigations/io-model/FIRST-PRINCIPLES.md`
§13-16 as the ownership contract the runtime work has to keep.

Stage A lands the four prerequisites §3.9 and §7 name. **None of them changes a
permission verdict or a published byte.** Stage B lands the judgment, the
lowering, and the measurement, and this record moves to `docs/done/` then.

## Items

1. `wf__completion_window(span, slot_bytes, ceiling)` — the runtime's own
   answer to how many iterations of one loop may be in flight, asked once per
   loop entry. Plus a weak fallback returning 1 so a link without the
   completion unit is sequential.
2. Deferred io_uring doorbell — `wf_linux_kick_locked` off the submission path,
   flushed on the first join, when the submission queue fills, and before any
   blocking direct host call.
3. Retire-and-retry on `ResourceExhausted` (design §2.10) — an open refused for
   want of a host descriptor while the pipeline holds more than the
   source-order footprint is retried once, after the older operations retire.
4. Back-edge-tolerant joins — the emitter can carry outstanding operations
   across a loop back edge under an explicit opt-in Stage B sets. Off, the
   emitted IR is byte-identical to `main`.

## Status

- [ ] item 1
- [ ] item 2
- [ ] item 3
- [ ] item 4
- [ ] `make check`, `completion-test`, `completion-sanitize`, TSan
- [ ] CI green (gate + io-hosts)
