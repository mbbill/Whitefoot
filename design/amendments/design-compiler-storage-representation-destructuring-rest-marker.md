Node: compiler/storage-representation

Decision: A destructuring `let` binder carries the field ordinal it receives in the checked statement, rather than receiving the ordinal its own position in the binder list gives it, because [GRAM-4]'s `TYPEID "(" ( fieldbind_list ("," "..")? | ".." )? ")"` lets a final `..` cover fields between and after the ones the statement names, so `let Inputs(cwd: root, handles: files, ..) = move inputs;` binds ordinals one and four; lowering, the [MSR-3] destructuring placement and the binder-type walk all read that ordinal, instead of keeping the positional reading, which silently bound the wrong field of every struct whose covered fields are not a suffix.

Decision: The checked statement also carries the compiler-derived release of every field the rest marker covers, in declaration order and rooted at the consumed value, and lowering runs those releases at the statement before the named binders read their fields, because [WIN-3] says the owner ceases to exist there and its other affine parts take their [STOR-3] release, instead of leaving the covered fields to a scope exit that has no binding to reach them through.

Decision: A covered field whose type is linear is a hard error citing WIN-3 at the complete consumed `place`, because [WIN-3] states that refusal for a remaining linear part of a consumed owner and offers the restructuring that names the field in the same destructuring, instead of citing the PROV-6 partial-consume refusal, whose subject is a consume of a proper sub-place and whose mechanical fix is the whole-value destructuring this statement already is.

Rejected:
- Binding every declared field to a compiler-owned binder and letting the covered ones take an ordinary scope-exit release: rejected because the covered fields would then outlive the statement that consumed their owner, and a covered linear field would reach [PROV-6]'s scope-exit refusal at the wrong edge instead of [WIN-3]'s refusal at the consumed place.
