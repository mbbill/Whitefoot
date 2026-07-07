# democ — demo compiler (temporary; endgame is self-hosting)

Compiles a micro-subset of kernel-spec v0.3: `fn` with region params, `own`/`&'r`/`&uniq 'r` i32 params, `let`/`set`/`return`, `deref(p)` places, `iadd.wrap|trap|checked`-subset ops, borrow exprs.

    python3 democ.py examples/twice_read.xl --asm     # -> .ll + clang -O2 .s
    python3 democ.py examples/twice_read.xl --no-facts # A/B: omit noalias facts
    python3 democ.py examples/dangle.xl                # exits 1: OWN-10 rejection

Pipeline: parse -> ownership checker (../checker) -> LLVM IR. `&uniq` emits `noalias`, `&` emits `noalias readonly`; measured effect: 1 load vs 2 at -O2 (see decision-gates "Demo compiler online").

## ex1 (M1 increment)

    python3 democ.py examples/ex1.xl --run   # compiles enums/match/check/calls; runs natively, exit 0
