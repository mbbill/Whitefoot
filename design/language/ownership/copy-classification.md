Decision: Values are copy or affine by type, with primitives, shared borrows, shared slice views, and tag-only enums copying on use and exclusive views and owned composites affine, because affine classification of resource-free tag-only values bought no safety while taxing every boolean: an owned Bool could not be loop-carried state or flow through boolean dataflow, forcing integer-flag workarounds whose recurrence vectorized at width two by four instead of sixteen, a measured 1.6 to 1.8 times kernel loss, instead of every enum affine regardless of payload.

Decision: A generic body's consuming spelling is checked against its declared bound and not rejected again when a concrete argument is copy, because rechecking the spelling per concrete type would contradict authoring a generic body once against its bound, instead of per-instance rechecking.

Rejected:
- Every enum affine regardless of payload, including Bool: rejected because it bought zero safety and forced integer-flag workarounds that cost a measured 1.6 to 1.8 times on scanner kernels.
