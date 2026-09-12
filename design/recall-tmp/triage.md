# Triage of audit findings

The owner's rulings on the items the module audits listed under "choices
without a node", recorded as they were given. A ruling of "no decision
needed" means the item is implementation and stays out of the tree; a ruling
of "tree" names the node that now carries it.

## audit/lexer-and-syntax.md, section 3

1. Two-pass, count-then-allocate lexer: no decision needed.
2. Every resource ceiling mandatory with no default: no decision needed. The
   owner's reading: such limits are a concern of the future self-hosted
   compiler, not of this research compiler.
3. Iterative task-stack machines for derivation, the diagnostic re-walk, and
   shape re-verification: no decision needed, with the same reading.
4. The frontend never panics and each stage re-derives what the previous one
   guarantees: the re-verification half is refused. Tree: `compiler` root,
   rejected alternative on hardening and re-verification inside this compiler.
   The no-panic half is ordinary engineering quality and needs no decision.
5. A curated subset of syntax diagnostics carries a mechanical fix: no
   decision needed.
6. Flat postorder parse array finalized by a second pass: no decision needed.

## audit/resolution.md, section 3

1. Historical [SYS-2] inventory states behind compile-time switches: the owner
   ruled them agent self-direction with no use. Deleted from the resolver;
   tree: `compiler` root, rejected alternative on superseded inventory states.
2. `set`-target-declares resolved by rebuilding to a fixpoint: no decision
   needed.
3. Diagnostic wording inside the resolution payload: no decision needed.
4. Contract `define` binder sharing the `let` carrier role: a defect, fixed
   (`ReservedDeclarationRole::ContractDefinition`).
5. `fn_bind` lookup failure citing FN-4: a defect, fixed (cites FN-3).

## main, pull requests #36 to #48 (proposed, awaiting the owner)

Choices carried into the tree for confirmation: the recursion budget clone
family and its runtime-derived default (`compiler/parallel-lowering/two-worlds`),
the idle window in time and the split grain (`compiler/parallel-lowering/parallel-runtime`),
the rejected `N`-layer frontier, alignment flags, and wake-path change.

No decision needed: the compute scoreboard and its harness, CPU column, A/B
twin, Darwin CPU source, and hosted workflows (research instruments); the
`compute-regression` required check and the scratch-root move (infrastructure);
the POSIX spin hint (a missing instruction, not a choice); the `WF_PAR_TRACE`
instrument compiled into no shipped build; the io-model scheduler findings and
the chunk-lease measurement (measurements, not decisions).
