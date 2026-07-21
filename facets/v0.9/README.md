# Exact-v0.9 facet foundation

This directory is bound to `spec/kernel-spec-v0.9.md`, SHA-256
`bdfb461d1901f610633c5cbcd2477d24df3c77ca90599b9580c8289e50b82b68`.
It is compiler-independent. The v0.8 directory remains an immutable historical
snapshot and is not rewritten or selected by these tools.

`source.json` is the generated structural index. It records 92 rules, 62
syntax productions (58 fenced and four inline), 44 operation rows, four report
rows, and two exact-byte fences. Together these form 204 exact source atoms.
These counts establish source coverage only; they are not implementation or
release claims.

The 22 authored files in `decomposition/` split the specification by cohesive
semantic responsibility. They tile the 92 rules into 420 exact clauses, name
679 semantic facets, and cover all 204 source atoms. The normalized
decomposition SHA-256 is
`81cc67795feb9dfb9458df7987da44663b8d5ea034921a1c56322e2771e4310c`.
Only three exact-byte exclusions are admitted: the reviewed deferred spans in
FORM-5 and LEX-1 and the reviewed non-normative OWN-9 span. Marker words do not
create exclusions.

The canonical generated catalog is 596,390 bytes with SHA-256
`3ff82e48fc860c4a414e8e1a16a652426b7505d7b74beedf057e418533151aae`.
No generated catalog file is checked in. `static-catalog.sha256` pins its exact
identity, and the compiler carries the same identity-only lock.

`open-discrepancies.json` is a generated, non-authorizing projection of seven
closed-registry predicates. Its SHA-256 is
`39a1d6f869df08ef08f4a2a58ad07bf4c503f61a235601847ebdd716caf8bc3d`.
Every record blocks affected facet closure and release. No record waives a
language obligation or grants compiler behavior. The sidecar is bound to the
exact v0.9 specification, source index, catalog, and approved guard baseline.

Verify the foundation with:

```sh
python3 -B tools/facet_catalog.py check
python3 -B tools/test_facet_catalog.py
python3 -B tools/semantic_catalog.py check
python3 -B tools/test_semantic_catalog.py
python3 -B tools/catalog_identity.py check
python3 -B tools/test_catalog_identity.py
python3 -B tools/facet_discrepancies.py check
python3 -B tools/test_facet_discrepancies.py
```

The tools select only the explicit v0.9 paths and exact identities. They never
select a numerically latest specification, reinterpret the specification, or
edit protected conformance and governance material.
