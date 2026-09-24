Node: language/effects

Decision: Effect rows in public declarations, in function-kind formals and in code outside the declaring module name only accessible paths, and a public operation whose body writes private state declares the nearest accessible path covering it, with EFF-2 exactness unchanged, because one visibility rule serves every role and EFF-2 already counts a declared entry as exhibited by any access at or below its path, while an interface that needs finer independence publishes the finer field, instead of rows naming private fields, named footprint regions or upper-bound rows.

Decision: Mentioning a field in a proof or effect annotation grants no exhibited runtime access and creates no scheduling edge, because proof support determines fact validity while body operations determine effects, instead of using proof mentions to justify a padded row or treating permission to describe a path as permission to execute it; the existing pure and no-termination-promise decision remains unchanged.

Rejected:
- The named footprint proposal: rejected because the published field states the precise path directly; a named region would only keep client rows unchanged when a published representation changes, which the module goals do not require.
- Effect rows as upper bounds: rejected because every declared path still requires an ordinary exhibited access and actual structural overlap must decide independence.
