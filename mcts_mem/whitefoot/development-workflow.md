- Keep one living, in-place Direction Outline as the owner-facing map of goals, major directions, dependencies, evidence links, implementation state, and open research; an outline item schedules no work by itself.
- Keep one rolling Current Plan as the sole source of plan-derived execution authority and sequencing. `PROPOSED` authorizes nothing; `ACTIVE` names one owner-selected milestone, its smallest end-to-end slice, validation, acceptance conditions, and stop conditions; [[work-authority-and-tracking]] governs non-authorizing task status and integration.
- Let candidate projects decide when outline directions matter. Derive each proposed milestone backward from a concrete project pressure and cite the exact outline items and dependencies it consumes.
- Treat the first project attempt as a diagnostic of the current language and compiler; a workaround that violates the frozen behavior, boundary, resource or asymptotic contract, or measured performance band remains a blocker even when it runs.
- Stop widening a project when the current slice has a structurally unavoidable or measured-and-attributed material performance blocker; resolve its owning layer generally and rerun the same frozen slice before adding downstream behavior.
- Close a blocker only when project-independent evidence and the same frozen project slice both pass.
- Attribute a performance gap through comparable work and the relevant generated-code layers before an optimizer mechanism enters a plan.
- Keep language and specification blockers cumulative across every Current Plan for one project milestone; a second independent gap defaults to reframe or park unless the owner explicitly overrides it.
- Treat an umbrella-project selection as revisitable at milestone closure; the next state may continue it, park or deselect it and reopen candidate selection, or leave no active plan.
- Permit bounded parallel research only when an `ACTIVE` plan or separate owner approval names its question, the decision it can change, required evidence, and stop condition.
- Keep ordinary project delivery and language change distinct: already-specified implementation, defect repair, project adaptation, and approved research stay on the main path, while only a measured language gap enters the guarded specification-change branch in `docs/WORKFLOW.md`.

## Facts

- 2026-08-03 owner ruling: the project needs two views, not a phase program — a compact living outline for reviewing every meaningful direction and one AI-executable plan derived from candidate-project pressure; unrelated research may proceed only when explicitly bounded and authorized. (sourced)
- 2026-08-03 (36273e48) implementation: `docs/roadmap.md` became Direction Outline revision 1, `docs/current-plan.md` became the sole status-bearing plan and remains `PROPOSED`, and root `WORKFLOW.md` now defines project gates plus the guarded specification branch. (code)
- 2026-08-03 owner ruling: SQLite is selected to expose general language defects and quantify proof or lowering advantages; making a port run by violating its frozen behavior, boundary, or cost contract is not milestone success, and a performance advantage needs generated-binary attribution rather than timing alone. (sourced)
- 2026-08-04 owner ruling: ripgrep 15.2.0 replaces SQLite as the umbrella target; the product objective is a fair 2x end-to-end ripgrep result, bounded milestones may expose smaller blockers without shrinking that objective, and optimization opportunities are discovered and attributed iteratively rather than preselected. (sourced)
- 2026-08-04 owner ruling: performance is first in the project-driven loop; do not force-write downstream `wfgrep` code on top of a missing performance capability. Each runnable slice pairs its correctness oracle with a cost-shape or measured performance gate, and an attributed material blocker stops expansion until its owning layer is resolved and the same slice passes again. (sourced)

## Moves

- 2026-08-03 (36273e48) replaced [[phase-led-roadmap]]: the phase sequence and experiment-first trunk mixed long-term ideas, implementation history, and current authorization in one roadmap; a living direction outline plus one rolling project-derived plan keeps state reviewable while preserving exact specification-change gates (sourced)
