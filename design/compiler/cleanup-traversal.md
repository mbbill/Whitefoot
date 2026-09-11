Decision: A compiler-derived drop whose type's cleanup can reach that type again runs on an explicit worklist bounded by the structure being dismantled, decided by strongly connected components over the cleanup graph, while every other drop keeps its straight-line expansion, because a generated drop that recursed over the value descended the machine stack in code the writer never wrote, could not instrument, and could not bound, and it ran after the program had already spent its stack, instead of recursive cleanup expansion.

Decision: A buffer takes one worklist entry per live element plus one for the block, because an entry per buffer would have to carry a cursor and a length to resume the element walk, while one entry per element keeps an entry two words and yields the fixed ascending-index-then-release order directly out of the last-in first-out discipline, instead of one entry per buffer.

Rejected:
- Recursive expansion of a compiler-derived drop, with the depth of destruction equal to the depth of the value: rejected because it descended the machine stack in unwritten, uninstrumentable, unbounded code after the program had already spent its stack.
