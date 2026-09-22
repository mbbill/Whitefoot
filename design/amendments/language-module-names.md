Node: language/name-resolution

Decision: Every top-level function signature is visible throughout its own module while other local declarations retain their lexical visibility points, and imported declarations are selected from a completed export table by an explicit module-qualified name, because forward calls and mutual recursion remain useful while module-local names prevent unrelated library declarations from colliding, instead of one whole-program spelling inventory or declaration-before-use for functions.

Decision: Module identity and dependency selection are explicit build inputs, source headers state membership and imports, and declarations and struct fields are private unless exported with the single pub form, because independent checking needs a closed naming boundary that does not depend on directory search or implicit transitive imports, instead of source-path-derived namespaces, wildcard imports or manually maintained interface copies.

Decision: Source privacy does not hide verified contract projections, physical layouts or implementation bodies from the compiler, and public enums expose their complete variant schema while struct construction and destructuring respect private fields and the existing opaque rule, because ordinary proof transport, exhaustive matching, release checking and optimized by-value representation still need those descriptions, instead of granting source access from optimizer visibility or making privacy require boxing or dynamic dispatch.

Rejected:
- Replaced decisions that top-level functions are visible throughout the closed compilation unit and that resolution begins with one complete declaration inventory: rejected because explicitly selected module inventories and export tables preserve the declaration-class visibility rules without whole-program name collisions.
- Treating module imports as ordinary function calls or trusted theorem imports: rejected because membership and visibility grant no postcondition evidence.
- Hiding a private field from compiler contract substitution merely because client source cannot select it: rejected because a verified accessor can publish a relation over that field without granting source access or adding a new fact constructor.
