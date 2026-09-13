Decision: Every top-level function signature is visible throughout the closed compilation unit while every other declaration keeps its lexical visibility point, because source-order visibility made function item order semantically significant and obstructed direct mutual recursion, and whole-unit signature visibility admits recursion without making order semantic, instead of declaration-before-use for functions.

Decision: Resolution starts from one complete declaration inventory and then resolves each grammar-selected use role in its own domain and scope, with inventory knowledge never granting visibility outside the rule selected for that declaration class, because a single inventory keeps resolution deterministic while per-class visibility rules keep every non-function declaration's meaning local, instead of inventory-wide visibility.

Rejected:
- A top-level function visible only after its declaration: rejected because item order became semantically significant and one edge of direct mutual recursion was unwritable without reordering or indirection.
