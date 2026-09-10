- Owning run insertion retains complete supplier coverage without recording
  the logical positions occupied by individual supplied elements.
- Owning extraction returns unlocated supplier bounds for both the remaining
  run and the extracted value.
- The owner-routing image carries no independent logical run-length facts.

## Moves

- 2026-09-10 (19771851) replaced by [[known-run-endpoints]]: Unconditionally forgetting owning run endpoints lost the last appended owner's position before unrelated replacements; separate known-length facts retain that position and the extracted remainder without treating owner identity as a length contract. (code)
