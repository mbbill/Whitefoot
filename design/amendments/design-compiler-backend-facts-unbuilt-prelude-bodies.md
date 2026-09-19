Node: compiler/backend-facts

Decision: A [PRE-1] record the compiler itself owns -- a window operation [OP-10], `swap` [OP-11], a construction function [OP-13] or `free_empty` [OP-14] -- whose body this version does not build stops the compilation as an unimplemented capability at the point the body would be built, named by a closed list of those rows that a row leaves in the change that lowers it, because such a record is declared body-less exactly like a host row while no trusted-base object defines it, so letting it through emits a call to a symbol nothing defines and turns a missing compiler capability into a link failure that the conformance adapter can only report as a harness stop, instead of emitting the call and discovering the gap at link time, where the failure names a mangled instance symbol and stops the whole corpus run rather than one case.

Decision: That stop is reported as an unsupported capability and never as a lowering-invariant failure or a source verdict, because an unimplemented feature is not a source-language rejection and the corpus verdict for such a case must be the toolchain gap it is, instead of the internal-lowering category, which would claim a trusted invariant had failed.

Rejected:
- Refusing the call in the checker instead, at the call site: rejected because a call-site refusal pre-empts the source rejections that later statements of the same function would raise, so a negative case would reach the capability stop instead of the rule it was written to exercise.
