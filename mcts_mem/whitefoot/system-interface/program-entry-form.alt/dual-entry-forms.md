- A unit contains exactly one `main`, selected from two source forms: an unlabelled no-input function returning `unit`, or a kind-declaring `command` function returning `ExitStatus` and optionally receiving labelled standard inputs.
- The unlabelled form is also an ordinary source-callable function and sees no system declaration domain; the command form is invoked only by program start and cannot be called from source.
- A program-start wrapper evaluates an entry requirement when present, then calls the selected main once; command input owners transfer from the host and the returned status maps to the process status.

## Moves

- 2026-08-19 (55a75434) replaced by [[program-entry-form]]: two main forms make one declaration serve incompatible ordinary-call and process-entry roles and require a special executable requirement boundary; one source-uncallable command entry removes both ambiguity and the only contract-owned runtime trap exception (sourced)
