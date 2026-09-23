Node: language/ownership/no-stored-references

Decision: Aggregates and generic instantiations contain only owned values, recursively and closed under wrapping, because a reference-free value can relocate by byte copy without leaving an interior name pointing at the old storage, instead of per-leaf provenance metadata or exceptions for particular hiding positions.

Decision: References are local names and call arguments, never aggregate contents, return values or stored or returned closure captures; a search returns an index or other owned data from which its caller forms a reference, because recomputing that address avoids provenance obligations on every escaping signature, instead of returning references with signature-tracked origins.

Rejected:
- Admitting a reference inside a generic argument or an option-like wrapper: rejected because one such position is enough to defeat relocation by byte copy, which is why the closure holds whatever the type argument is.
- A function value that captures a reference and is then stored or returned: rejected because it stores the reference under another name and carries the same obligations out of the forming function.
- Returning an index and treating a later use of it as a memory error when the storage has been reused: rejected because a stale index that is still in bounds names the current occupant of that slot, which is a logic error the program detects with its own data, such as a generation number, if it needs to.
