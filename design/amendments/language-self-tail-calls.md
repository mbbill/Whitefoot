Node: language/self-tail-calls

Decision: A call-site `musttail` marker requires a direct self call in the sole return position, reference actuals rooted only at incoming reference parameters, and no retained release after argument capture; ordinary calls and proof obligations remain unchanged, because an agent needs a checkable constant-stack transfer and an ordinary call leaves both optimization and a hidden cleanup continuation outside that guarantee, instead of an advisory optimization hint or an implicit promise for every recursive call. FN-10 states the complete conditions; mutual transfers remain a later design with their own ABI requirements.

Decision: Derive and perform the remaining ordinary releases after capturing arguments and before rebinding parameters, permitting a nonempty release only when no live valid reference binding names that owner, because STOR-3 provides only compiler-derived release and no user finalizer, so an unreferenced owner can be reclaimed without changing source-visible behavior; a referenced local cannot be carried into the replacement activation, instead of silently retaining cleanup after the call or rejecting every tail call with an unreferenced affine local.

Rejected:
- A function-wide tail annotation: rejected because a function can contain both tail and non-tail recursive edges, and the writer's obligation is at the selected call.
- Allowing references rooted at owned parameters or locals when their address happens to remain stable: rejected because frame replacement and ownership transfer do not preserve that activation's storage; the incoming-reference root is the portable source-level boundary.
