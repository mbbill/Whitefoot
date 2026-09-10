- Represent owner attribution as typed routes between destination subvalues and
  formal-input subvalues. Product fields, owning referents, selected variant
  payloads and literal elements have distinct selectors (CheckedStateOrigins).
- Keep whole-subvalue routes with relative subtree exclusions. An exact overwrite
  removes the selected old contribution and installs the replacement there;
  unselected parts retain their routes. A join keeps an exclusion only where
  both incoming images exclude the same contribution.
- Share projection, exclusion, join and call substitution between ordinary
  checking and normal-exit summaries. Substitute results and borrowed outputs
  against one entry image before installing outputs at exact actual places.
- Normalize coincident projected routes and use a no-change replacement shortcut
  only for structurally exact selected images. Equal supplier bounds do not
  establish subvalue identity. Scalar-only recursive changes retain their image.
- Keep finite transfers with unresolved placement distinct from wholly unknown
  sources through [[source-bounds]]. Selectors revisiting recursive types remain
  unknown. This finite path representation supplies no general recursive-content
  image, runtime metadata or loan authority.

## Facts

- 2026-09-09 (4bcb9961) pitfall: Expanding an unchanged recursive subvalue into ancestor and child routes grows the image despite no owner changing. Exact self-writeback and canonical projection preserve the existing recursive scalar-fold controls. (code)
- 2026-09-09 (4bcb9961) pitfall: Typed projections require matching producer and consumer selectors. Cell destructuring through a product-field selector discards referent origins; retaining old logical indices across front insertion can discard the shifted owner's origin. The latter remains a capability gap until its transfer is represented. (code)
- 2026-09-09 (e298ca67) pitfall: A loop's ownership permissions can agree while its current owner comes from different inputs on later iterations. Checking only the entry image misses those reads; stable entry/backedge origin headers and counted exhaustion expose the second supplier without relaxing liveness or loan equality. (code)
- 2026-09-09 (a577b7cb) pitfall: Address-evaluation access roots can name an enclosing aggregate rather than the transferred field. Adding those roots after projecting a captured owner image widens the field's effect to unrelated state. (code)

## Moves

- 2026-09-09 (4bcb9961) replaced [[whole-static-field-routes]]: Whole-owner and static-field routes could not retain an enclosing allocation and its displaced and installed children independently; typed subvalue routes with exclusions express that distinction. (code)
