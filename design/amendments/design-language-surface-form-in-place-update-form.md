Node: language/surface-form/in-place-update-form

Decision: Replacing the value of an owned place by a function of that same old value is written as an ordinary assignment whose right-hand side is a call taking that place as a by-value argument, and the language gives the operation no name of its own, because that byte sequence has no competing meaning, since handing the place to a function by value in any other position would consume the whole owner the place sits in, so one spelling says exactly what happens, the old value enters the function and the function's result becomes the place's new value, instead of a named built-in update or modify operation taking a reference to the place together with a function argument.

Rejected:
- A named built-in operation taking a reference to the place and the function to apply: rejected because it would give one semantic construct a second spelling beside the assignment that already denotes it, and the assignment is the form in which the writer can see which place is being updated and by what.
