Decision: Every compilation unit has exactly one `command fn main` that requests any ordered subset of the labelled command inputs, has a named `ExitStatus` result, carries no contract, and cannot be called from source, because two main forms made one declaration serve incompatible ordinary-call and process-entry roles and required a special executable requirement boundary, and one source-uncallable entry removes both the ambiguity and the only contract-owned runtime trap exception, instead of a second unlabelled source-callable main.

Rejected:
- Two main forms, an unlabelled source-callable function and a command function, selected per unit: rejected because one declaration then serves incompatible roles and needs a special executable requirement boundary at program start.
