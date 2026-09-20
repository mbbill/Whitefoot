Node: compiler/verification

Decision: The conformance corpus carries no toolchain-readiness axis: `runnable` is the only manifest status, any other value is a manifest error, and the native adapter's outcomes are pass and fail alone, because a case whose declared verdict this toolchain does not reach is a defect of the toolchain and the corpus is what states the language's required behavior, instead of the `pending` and `xfail` statuses and their `Skip`, `Xfail` and `Xpass` outcomes, which let a case sit outside the gate indefinitely while the manifest still claimed to declare its verdict.

Decision: A case that stops as unsupported without declaring `unsupported` fails, because the corpus has no status for a capability gap now, so an unsupported stop is an ordinary failure rather than a verdict comparison, instead of reporting it as a distinct outcome, which would reintroduce the readiness axis under another name.

Rejected:
- Keeping `xfail` for a case whose expectation is the correct specification behavior the toolchain does not yet produce: rejected because that is exactly the defect a gate exists to report, and a non-failing report of it is a gate that stays green while the compiler is wrong.
