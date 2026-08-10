- Every invocation evaluated the complete requirement unconditionally in an executable callee-entry prologue.
- Ordinary call acceptance did not depend on caller proof; a false prologue condition trapped and a true condition supplied the body-entry fact.
- The prologue's reads and retained check contributed to the callee effect row.

## Moves

- 2026-08-10 (441cd5b8) replaced by [[requirement-enforcement]]: the unconditional ordinary-callee prologue let a helper hide a protected leaf behind a runtime trap; pre-transfer proof exposes the complete atomic requirement without narrowing the admitted predicate surface, while real process entries retain the checked boundary (sourced)
