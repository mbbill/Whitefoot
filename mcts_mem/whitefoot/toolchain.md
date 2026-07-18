- The production compiler (wfc) is written in Whitefoot itself; the Python prototype is stage 0 only, compiling wfc until wfc compiles itself, with self-hosting defined as a byte-identical emitted-IR fixpoint across stages.
- wfc is built on fixed-capacity structure-of-arrays tapes backed by primitive buffers; token and node counts are bounded from source size, and the bootstrap needs no growable collections, no pool, and no general generics.
- Compilation is one whole-program unit in a deterministic declaration order; the target is LLVM IR text, and the external C compiler is driven by the build system, never spawned by the compiler.
- Two standing verification gates guard every change: the conformance/model-check/perf-pin gate, and the codegen-parity gate over earned IR, opcode, and proof-site properties, with audit-versus-gate maturity keeping known debt visible without becoming contract.
- Stage 0 builds wfc with optimizer facts disabled until wfc's own effect checking is complete.

## Facts

- 2026-07-07 rationale: owner ruling — the checker and compiler prototypes are temporary; the endgame is self-hosting, by the ladder host-language demo, then real compiler, then compiler in Whitefoot as the ultimate dogfood of the language's own claim to be worth writing in. (sourced)
- 2026-07-07 pitfall: the performance-regression pin's first run caught a real parser regression that the smoke check had missed by testing tool exit status only — regression checks must assert the artifact, not the tool's exit code. (sourced)
- 2026-07-12 (c6432014) pitfall: the parity gate broke silently during a repository restructure (pipeline exit-status masking — a trailing pipe swallowed the failure) and a live fixture was archived because a JSON manifest pin was missed; process rules recorded: never chain a commit after a piped gate, and grep every file type including JSON manifests before moving anything a gate might pin. (sourced)
- 2026-07-12 statement: exact opcode parity in the gate is intentionally sensitive to toolchain upgrades — on an upgrade the diff is inspected and the gate updated in the same change, never weakened merely to restore green; runtime measurements are deliberately excluded from the gate as noise-unstable. (code)

## Moves

- 2026-07-12 (e8c8eeb1) replaced [[pool-based-wfc-plan]]: fixed-capacity structure-of-arrays tapes with token and node counts bounded from source size let stage-0 democ bootstrap wfc without growable collections, pool, handle, or general generics — none of which stage 0 implements (sourced)
