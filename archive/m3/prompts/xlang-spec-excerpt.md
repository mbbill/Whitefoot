# xlang Writer's Excerpt (kernel v0.6 subset)

You are writing xlang: a checked systems language with no operator syntax, no
`if`/`while`/`for`, explicit types everywhere, and value semantics with
ownership. The checker rejects anything outside these rules with a rule-cited
diagnostic. Everything below is exact; there are no other forms.

## Program shape

A program is a sequence of declarations separated by one blank line:
`struct`, `enum`, `contract`, `conform`, and `fn`. Exactly one
`fn main () -> own unit <effects> { ... }`. Two-space indentation per `{`
level, one statement per line, no comments (use `doc "...";` as the first
statement of a body if needed).

```
struct Pair {
  a: buffer<u64>;
  b: u64;
}

enum ParseError {
  Empty();
  BadDigit();
}
```

Enum variant names are globally unique across the whole program (do not reuse
`Overflow`, `DivError`, `Ok`, `Err`, `Some`, `None`, `True`, `False`).

## Functions

```
fn name ['r] (p1: own u64, p2: &uniq 'r Pair, p3: &'r u64) -> own u64 <effects> {
  ...
  return expr;
}
```

- Parameter modes: `own T` (value, consumed), `&'r T` (shared borrow, read-only),
  `&uniq 'r T` (exclusive borrow, writable). Region parameters `['r]` are
  declared when borrows are taken.
- Effect row (comma-separated, in this order, or `pure`):
  `reads('r)`, `writes('r)`, `allocates(heap)`, `traps`.
  The row must both cover and be exhibited by the body: `index`, `iadd.trap`,
  `buffer_new` (size overflow), and `check ... else trap` all exhibit `traps`;
  `buffer_new` exhibits `allocates(heap)`; reading/writing through a borrow
  exhibits `reads('r)`/`writes('r)`. Declaring an effect the body does not
  exhibit is an error, and vice versa.
- Calls name every argument in declared order: `f(x: a, y: b)`. No positional
  arguments. Passing an affine value (buffer, struct, enum value) uses
  `move p`; primitives copy.
- Borrow arguments are written inline: `f(s: &uniq 'r p)` inside
  `region 'r { ... }`.

## Statements

```
let x: own u64 = expr;
set place = expr;
loop @l { ... break @l; ... }
match expr { Variant(field: binder) => { ... } Other() => { ... } }
check cond_expr else trap "message";
region 'r { ... }
return expr;
```

- `let` always declares mode and type. No shadowing, no reassignment via let;
  use `set` for mutation.
- Everything is three-address form: an operation's arguments are atoms
  (literals, places, `move place`, borrows) — never nested calls. Bind
  intermediates with `let`.
- Conditionals are `match` on Bool: `match ilt<u64>(i, n) { True() => { ... } False() => { ... } }`
  with both arms present. A conditional VALUE uses a let-initializer match
  where each arm ends `give expr;`:

```
let y: own u64 = match iadd.checked<u64>(a, b) {
  Ok(value: v) => {
    give v;
  }
  Err(error: e) => {
    give 0_u64;
  }
}
```

- Loops: only `loop @label { ... }` + `break @label;`. The idiomatic while:

```
loop @l {
  match ige<u64>(i, n) {
    True() => {
      break @l;
    }
    False() => {
    }
  }
  ...body...
  set i = iadd.trap<u64>(i, 1_u64);
}
```

## Types and literals

Primitives: `i8 i16 i32 i64 u8 u16 u32 u64`, `Bool`, `unit`. Every integer
literal carries its type suffix: `0_u64`, `-1_i64`, `42_i32`. No bare
integers, no leading zeros, no `-0`. `unit` is the unit value.

`buffer<T>`: heap array of primitives, fixed length at allocation.
- `buffer_new<u64>(n, fill)` -> `own buffer<u64>` (effects: `allocates(heap), traps`)
- `index<u64>(b, i)` is a PLACE (read `let x: own u64 = index<u64>(b, i);`,
  write `set index<u64>(b, i) = v;`); bounds-checked, traps out-of-bounds.
- `len<u64>(b)` -> `own u64`.
- Buffers are affine: pass with `move b`, or operate through a borrowed
  struct field: `index<u64>(deref(s).a, i)` where `s: &uniq 'r Pair`.

`Result<T, E>` / `Option<T>` are prelude enums with variants
`Ok(value: T)` / `Err(error: E)` and `Some(value: T)` / `None()`.

## Operations (closed table; no operators)

Comparisons -> Bool: `ieq ine ilt ile igt ige` as `ilt<u64>(a, b)`.
Arithmetic carries an explicit overflow mode:
- `iadd.wrap<T>(a, b)` two's-complement wrap (total)
- `iadd.trap<T>(a, b)` traps on overflow
- `iadd.checked<T>(a, b)` -> `Result<T, Overflow>` (match it)
- `iadd.sat<T>(a, b)` clamps to T's range
Same axis for `isub`, `imul`. Division: `idiv.trap` / `idiv.checked` only.
Bitwise (dotless, total): `iand ior ixor inot imin imax`.
Bool ops: `band bor bxor bnot`.
`deref(p)` reads through a borrow; write with `set deref(p) = v;`.

## Errors and propagation

Recoverable errors are values: return `own Result<T, E>`. Construct with
`Ok(value: x)` / `Err(error: e)` where `e` is an enum value like
`BadDigit()` (error variants carry no payload in this subset). Propagate with `try`:

```
let v: own u64 = try step_a(x: x);
```

`try` unwraps `Ok` or immediately returns the `Err` — legal only when the
enclosing function's error type is EXACTLY the same `E`. Non-recoverable
violations use `check cond else trap "msg";` (aborts).

## Worked example

```
fn parse_pair (b: own buffer<u8>) -> own Result<u64,ParseError> traps {
  let n: own u64 = len<u8>(b);
  match ieq<u64>(n, 0_u64) {
    True() => {
      return Err(error: Empty());
    }
    False() => {
    }
  }
  let d: own u8 = index<u8>(b, 0_u64);
  let dv: own u64 = cvt<u8, u64>(d);
  return Ok(value: dv);
}
```

Success is exit code 0: `main` returns `unit` after all `check` statements
pass; any trap aborts with nonzero exit.
