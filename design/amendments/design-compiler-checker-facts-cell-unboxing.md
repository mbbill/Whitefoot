Node: compiler/checker-facts

Decision: `let n = move b.inner;` produces a checked node of its own, distinct from the node an ordinary content read produces, because [TYPE-9] makes the spelling consume the `Box`, yield its content and free the cell, while the read leaves the cell standing, and lowering must emit the load-and-release operation for one and the plain load for the other; the two cannot be told apart from the node's shape, since both name the same field step through the same cell, instead of marking the read node with a flag, which would leave every consumer of the read node responsible for testing it.

Decision: The consume kills the whole binding the cell is rooted in and takes no residual release, because [WIN-3] says a move out of a field "consumes the whole owner: the owner ceases to exist, its other affine parts take their compiler-derived release", and a cell declares exactly one field, so there is no other part; the content is the value the expression hands back and the cell is freed with it.

Decision: The spelling is admitted only where the written base is an owned place: a base written `deref(p)` stays [OWN-1]'s refusal, because a consume is admitted only for a place rooted in a live own-mode binding and the storage a reference names is not this function's to carry away.

Decision: A runtime-capacity content is refused at the same site citing [TYPE-9], because the rule names that refusal and its restructuring explicitly, and the window's block is the cell's heap object: taking it out would put a `Slots<T>` value in a position the rule admits nowhere.

Rejected:
- Reaching the consume through the retired destructuring `let Box(inner: n) = move cell;`: rejected because [TYPE-2] refuses a destructuring `let_stmt` whose TYPEID names an opaque struct, and [PRE-1] declares `Box` as one.
- Admitting the consume for a path with field steps above the cell, such as `move outer.cell.inner`: rejected because the owner that ceases to exist is then `outer`, whose other fields owe their compiler-derived release at that same statement, and this node computes no residual release; such a path keeps its existing [WIN-3] refusal, which is a wrong rejection and never a wrong acceptance.
