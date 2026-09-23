Node: language/system-interface/declaration-home

Decision: Types and signatures supplied by the host are ordinary prelude declarations with the same naming, type, ownership, effect and proof rules as source declarations, because implementation origin cannot justify a separate acceptance path, instead of a distinct system declaration domain, semantic operation identities or visibility selected by a program kind.

Decision: Each host handle type is written in the prelude as an ordinary opaque struct declaration with no fields, the same source form a writer may write, so that its empty release, its `nodrop` modifier where it stands for a resource that must be closed and its `nocopy` modifier otherwise (`language/ownership/copy-classification`), and the refusal of its constructor all follow from a declaration the reader can see, because a prelude declaration earns that name only when the writer could have written it, and a catalog of types named in prose alone is this node's refused separate system declaration domain under another name, instead of listing the host handles as opaque nominal types in notation no source syntax expresses.

Rejected:
- Mandatory capability bounds on every prelude type parameter: rejected because the prelude follows ordinary generic syntax, where an absent bound already requires no capability.
- Listing the host handle types as a table of opaque nominals that source syntax cannot express: rejected because it left the writer with types whose declaration form was unavailable to source, distinguishing host types from ordinary ones by implementation origin alone, while the opaque struct modifier states the same withholding of construction in the writer's own form (`language/data-model/opaque-struct`).
- A compiler-owned system declaration domain separate from the prelude: rejected because it distinguished otherwise ordinary values and functions solely by implementation origin.
- A source callable restricted to a WF body rather than a prelude signature: rejected because native binding is a link choice and supplies no reason to exclude the same ordinary signature from a function-formal argument.
