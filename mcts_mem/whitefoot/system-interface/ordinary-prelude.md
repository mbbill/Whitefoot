- Host-facing types are ordinary opaque or transparent nominal declarations in the prelude. They obey the ordinary naming, linearity, type argument and construction rules.
- Host-facing operations are ordinary prelude signatures. Their supplied bodies use the same callable representation as source definitions; implementation selection is a build and link concern.
- Declaration origin supplies no semantic operation identity, qualification judgment, native effect category or additional diagnostic class.

## Moves

- 2026-09-12 (d695f385) replaced [[declaration-home]]: C1 DESIGN and selected decision 6 require one ordinary declaration and call model; the always-visible system domain adds a source distinction based only on implementation origin, while ordinary prelude signatures retain the same named arguments and contracts. (sourced)
