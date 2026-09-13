- Every compilation unit has exactly one `command fn main`; the former unlabelled source-callable main does not derive.
- Main may request any ordered subset, including the empty subset, of the four labelled command inputs. Omitting a capability remains the least-authority form for a program that does not need it.
- Main has a mandatory named `own ExitStatus` result, carries no generic or region parameters and no contract, and cannot be called from source.
- Program start performs target qualification and the command lifecycle setup, transfers each requested owner exactly once, calls main exactly once, and maps its returned status. It executes no source contract and owns no language trap condition.

## Facts

- 2026-08-19 (55a75434) statement: the owner selected one parameterized command form and explicitly removed migration cost from the design criterion; allowing zero selected inputs preserves capability minimality while the named ExitStatus result remains mandatory. (sourced)

## Moves

- 2026-08-19 (55a75434) replaced [[dual-entry-forms]]: two main forms make one declaration serve incompatible ordinary-call and process-entry roles and require a special executable requirement boundary; one source-uncallable command entry removes both ambiguity and the only contract-owned runtime trap exception (sourced)
- 2026-09-12 (d695f385) replaced by [[ordinary-entry]]: Selected C1 decision 3 and C2 remove the command kind and labelled inputs; an ordinary callable unit can have no entry, and an ordinary build caller supplies arguments without changing source contracts or ownership. (sourced)
