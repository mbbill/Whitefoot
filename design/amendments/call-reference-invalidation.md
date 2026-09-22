Node: language/effects

Decision: A callee's declared `writes`, after its paths are substituted with the actual argument paths at the call, kills exactly the caller facts whose storage support overlaps those paths and applies the ordinary storage-validity rule to every live reference, including actual arguments, before the callee's verified `ensures` publishes replacement facts, because preserving a fact across a write to its support is unsound and argument membership cannot keep removed storage alive, while killing anything the write cannot reach destroys modular proofs, instead of killing by the syntax of an actual argument, killing the whole argument, exempting every actual reference, or leaving an unknown replacement unchanged.

Rejected:
- Retain the seventh decision's restriction to live references outside the call: rejected because one actual's destructive ancestor write can remove the storage selected by another unused actual, as tests/conformance/cases/ref2-neg-wildcard-ancestor-call.wf demonstrates; replace that decision with the proposal above, retaining ordinary preservation for content writes at or below the captured target under spec/kernel-spec.md REF-2 and EFF-5.
