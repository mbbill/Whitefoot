# Ownership reference model

This directory is an independent executable model of the ownership/effect
subset described at the top of `checker.py`. It is intentionally small and
uses a toy AST so that compiler and model bugs are unlikely to share an
implementation. It does not define the language, parse Whitefoot source, or
claim coverage of all v0.10 semantics.

During development, add a focused model judgment when an ownership rule would
benefit from a genuinely independent differential or bounded state search.
Do not grow this into a second compiler and do not copy compiler data
structures into it. The numbered specification remains authoritative when the
model, compiler, and spec disagree.

The oracle capability is permanent; this Python implementation is not. It may
move to `archive/` only after a replacement independent oracle:

- covers every still-relevant judgment and maps its regressions;
- retains bounded generated-state or property testing;
- includes mutation tests showing that wrong ownership behavior is detected;
  and
- leaves no unique rationale or counterexample only in this implementation.

Run it with `make reference` from the repository root.
