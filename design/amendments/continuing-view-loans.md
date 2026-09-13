Node: compiler/view-loans

Decision: The checker carries the identity of a continuing view loan separately from its possible storage origins, retaining its protected place, region, strength and parent holder through ordinary copies and results, because views over the same backing may carry different live loans and a moved incoming descriptor may have a live child without any local formation record, instead of identifying a loan solely by the storage origin or by a descriptor's local formation.

Rejected:
- Reconstructing continuing loans only from local view formations: rejected because an incoming exclusive view with a live shared child can otherwise be moved onward as though that child did not exist.
