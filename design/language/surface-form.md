Decision: Each semantic construct has exactly one spelling and one byte-level formatting, non-canonical input is rejected rather than reformatted, and whether an element is written is decided by its grammar class alone, because a rule keyed on whether the checker could infer an element at a site would force the writer to simulate the checker to know what is even writable and would grow the specification by one clause per relieved position, instead of per-position relief.

Decision: There are no comments; documentation lives in a declaration's doc field, because a comment is text the checker cannot see and a doc field is a checked part of the declaration, instead of free-form comments.

Decision: Computation is flat three-address form, one operation per expression with every intermediate named by a binding, because the form is performance-neutral by measurement and it removes precedence, nesting, and lookahead from the language, instead of nested expressions with precedence.

Decision: Whether a writer model happens to emit a form is never a criterion for a spelling decision, because model behavior is a motivation for looking at a class and not evidence about the class, instead of choosing spellings by what current models produce.

Decision: Internal corpus spelling counts are not evidence of real-use frequency, and corpus rewrite cost is not a ground for a spelling choice before real projects adopt the language, because the internal corpus describes only itself, instead of counting the corpus to select a keyword.

Rejected:
- Per-position relief, where an element is forbidden wherever the checker can reconstruct it and mandatory elsewhere: rejected because legality must be decidable from the grammar class alone, and a rule keyed on inference success at a site makes the writer simulate the checker and grows the specification by one conditional clause per position.
