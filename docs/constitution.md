# The Whitefoot Constitution

Whitefoot is a programming language designed as a harness for AI agents.

Its purpose does not uniquely determine its objectives or mechanisms. The
objectives below are explicit choices; concrete designs require their own
grounds.

## Development objectives

Whitefoot must enable people to delegate implementation to AI agents while
retaining control over software objectives and key tradeoffs. The language must
enforce formalizable requirements through machine-checkable constraints,
reducing dependence on repeated human inspection of generated code.

The language must support construction, verification, maintenance, and evolution
at the scale of its target projects.
Whitefoot primarily targets large systems such as kernels, compilers, and
browsers, while also seeking to support small embedded systems.

Ease of manual source authorship and syntactic familiarity may be sacrificed
to serve the language's objectives. More verbose source and additional proof
work are acceptable costs when they serve those objectives and leave
development, compilation, and verification practically feasible.

## Performance

The language must provide the capabilities needed to meet its target projects'
performance requirements. Specific performance goals and their tradeoffs must
be determined by the project and use case.

Within the constraints of required safety and practical development
feasibility, prefer runtime performance over ease of writing or speed of
compilation. Additional writing, proof, and compilation work may be accepted,
and the compiler should be improved to reduce that work. Compilation and
proof-checking costs must permit practical iteration at the target project's
scale; guaranteed termination alone is insufficient.

Constraints and guidance for agents must serve these objectives. If a
restriction intended to guide writers excludes a better-performing
implementation that meets the safety and development-feasibility requirements,
revise or replace the guidance mechanism. Select concrete mechanisms using
the target project's requirements, technical arguments, and experimental
evidence.

## Safety

Accepted Whitefoot programs must exclude undefined behavior, memory corruption,
data races, uninitialized reads, silent overflow, and any other operation whose
required safety conditions have not been established by machine proof.

Required safety guarantees must be established by machine verification before
a program is accepted. Safety guarantees are constraints that performance and
other objectives must respect.

For uses with explicit resource budgets and applicable conditions, the language
must support machine-verifiable bounds on memory and other hardware resource
usage.

The language specification must define the execution model and external
conditions on which its safety guarantees depend. The compiler and runtime
must implement those guarantees; implementation defects do not change the
language's safety requirements. These guarantees do not by themselves establish
that requirements are correct or fully capture the intended behavior. Logic
errors, including unintended nontermination, may remain. Expected input and
environment failures must have defined program behavior.

## Compatibility and evolution

Backward compatibility may yield to the language's objectives and long-term
evolution. Once real projects have compatibility needs, weigh improvement
benefits, actual impact, and migration capability.

Reassess objectives, tradeoffs, and design choices as AI capabilities, evidence,
and expectations about future software development change their grounds.
Current choices, including constitutional principles, remain open to revision.
