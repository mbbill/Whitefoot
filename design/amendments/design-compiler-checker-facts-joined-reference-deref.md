Node: compiler/checker-facts

Decision: A `deref` carries the complete resolved path set [REF-1] beside one exact-target identity: a singleton set uses its unique member, so distinct reference holders known to name the same place satisfy [OP-12]'s exact-target judgment, while a set with more than one member uses the reference holder's symbolic identity because no member is statically selected; effects, const and readonly provenance, writability, liveness, overlap and reference invalidation range over every member, because [REF-1] requires every check to hold for every member and neither choosing one member nor replacing different roots by a common prefix preserves that requirement.

Decision: The lowering of that `deref` reads the reference's own runtime value and consults neither the exact-target identity nor any member of the set, because the reference carries one address and a read through it is a read at that address [REF-1, TYPE-7], so the checker metadata never materializes a selector or duplicates the read once per member.

Decision: A value binder every arm of which delivers a reference carries that reference's representation -- the address its arms delivered -- through the join, and the checked value-initializer statement states the binder's derived mode beside its type, because [REF-1] makes such a binder a reference variable while [TYPE-8] makes the type it carries the *referent's*, so the type alone cannot say whether the join carries an address or a value, instead of deriving the representation from the type, which lowers a delivered address at the referent's type and stops as a malformed checked program.

Rejected:
- Picking the first member of the set: rejected because the members differ exactly when the two incoming edges name different storage, so the one the checker picked is the wrong one on the other edge, and the error is silent.
- Refusing the program at the join: rejected because [REF-1] states the union rule as an admission and gives it its own check discipline, so a refusal would reject a program the language accepts on a representation ground.
- Replacing the set by a common prefix: rejected because different roots have no common resolved place, and a same-root prefix loses the distinct member identities that CONST-2, EFF-2, OWN-7 and REF-2 judge.
- Treating any overlapping member as an exact target: rejected because a joined holder may select another member at runtime; exact [SET-1]/[OP-12] identity is the unique member only when the set is a singleton.
