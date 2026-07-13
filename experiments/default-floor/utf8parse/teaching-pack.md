# xlang kernel writer's pack

This is the complete compiler-supported subset needed for a caller-buffer
byte-to-codepoint kernel. Forms not listed here are unavailable.

## Lexical and file rules

- Use two spaces per brace nesting level, never tabs.
- Put one construct on each line.  Terminate simple statements such as `let`,
  `set`, `check`, `return`, `break`, `give`, and `doc` with `;`.  Do not add a
  semicolon after a closing brace for a function, `requires`, `loop`, `match`,
  `region`, or match initializer.
- Separate top-level declarations with one blank line.
- Integer literals use decimal digits and always have a type suffix, such as
  `0_u8`, `65_u32`, and `255_u64`. Bare integer literals and hexadecimal
  literal syntax are rejected.
- Line and block comments are unavailable.  An optional first body statement
  can be `doc "text";`.
- There are no operators, `if`, `while`, `for`, indexing brackets, casts with
  `as`, closures, iterators, slices, or methods.  Use only the forms below.

Primitive types available here are `u8`, `u32`, `u64`, `i8`, `i32`, `i64`,
`Bool`, and `unit`. A fixed-length owned byte buffer has type `buffer<u8>`;
a fixed-length owned codepoint/event buffer has type `buffer<u32>`.

## Functions, ownership, and effects

A function declaration has this shape (`T`, `R`, `EFFECTS`, and `STATEMENTS`
stand for concrete types, an effect row, and body statements):

```text
fn name ['r] (x: own T, out: &uniq 'r T) -> own R EFFECTS {
  STATEMENTS
}
```

Parameter modes are `own T`, `&'r T` for a shared borrow, and `&uniq 'r T` for
an exclusive writable borrow.  Declare a region parameter such as `['r]` when
the signature contains borrows.  Primitive owned values copy; buffers and
other aggregate values are affine.

An effect row is either `pure` or the exhibited effects, written in this order:
`reads('r)`, `writes('r)`, `allocates(heap)`, `traps`.  Reading through a borrow
exhibits `reads`, writing through one exhibits `writes`, and checked indexing or
a trapping check exhibits `traps`.  Do not declare an effect the function does
not exhibit, and do not omit an effect it does exhibit.

Calls use named arguments in declaration order:

```text
let y: own u64 = helper(x: x, b: move b);
```

Use `move` when passing an affine owned value.  Borrow arguments are formed
inside a matching region:

```text
region 'q {
  let y: own u64 = helper(out: &uniq 'q data);
}
```

For a kernel called by a foreign driver, `main` is not required.

## Entry requirements

Put a `requires` block between the signature and body.  It can contain typed
`let` bindings and `check` statements.  Its checks run at function entry.

```text
fn bounded (x: own u64) -> own u64 traps requires {
  check ile<u64>(x, 100_u64) else trap "limit";
} {
  return x;
}
```

The effect row covers both the requirement and the body.

## Bindings, mutation, and returns

Every local has an explicit mode and type:

```text
let count: own u64 = 0_u64;
set count = iadd.wrap<u64>(count, 1_u64);
return count;
```

Do not redeclare or shadow a name.  A binding introduced with `let` is changed
with `set`.  Operation arguments must be atoms: literals, places, `move place`,
or borrows.  Nested operations are rejected, so bind each intermediate result
to a local.

## Buffers and places

Buffer operations are generic in their element type. For byte input and u32
output, the relevant forms are:

```text
let source_length: own u64 = len<u8>(src);
let byte: own u8 = index<u8>(src, i);
let output_length: own u64 = len<u32>(deref(out));
let old_event: own u32 = index<u32>(deref(out), j);
set index<u32>(deref(out), j) = codepoint;
```

`len<T>` returns `u64`. `index<T>` requires a `u64` index and is a checked
place: it can be read or assigned. Its type argument must match the buffer
element type. Use `deref` to reach the referent of a borrow. Indexing outside
the buffer traps.

## Control flow

There is one loop form.  Leave it with a matching labeled break:

```text
loop @again {
  match ieq<u64>(i, limit) {
    True() => {
      break @again;
    }
    False() => {
    }
  }
  set i = iadd.wrap<u64>(i, 1_u64);
}
```

Conditionals are exhaustive `match` statements on `Bool`; both arms are
required.  Bind a conditional value with a `match` initializer whose arms end
in `give`:

```text
let selected: own u8 = match ile<u8>(a, b) {
  True() => {
    give a;
  }
  False() => {
    give b;
  }
}
```

`break` exits only its named loop.  `return value;` exits the function.

## Integer and Boolean operations

All operations are calls with explicit types.  The forms relevant to integer
code are:

```text
ieq<T>(a, b)  ine<T>(a, b)
ilt<T>(a, b)  ile<T>(a, b)  igt<T>(a, b)  ige<T>(a, b)

iadd.wrap<T>(a, b)  isub.wrap<T>(a, b)  imul.wrap<T>(a, b)
iadd.trap<T>(a, b)  isub.trap<T>(a, b)  imul.trap<T>(a, b)
idiv.trap<T>(a, b)  irem.trap<T>(a, b)

iand<T>(a, b)  ior<T>(a, b)  ixor<T>(a, b)
ishl.wrap<T>(a, amount_u32)  ishr.wrap<T>(a, amount_u32)

band<Bool>(a, b)  bor<Bool>(a, b)  bxor<Bool>(a, b)  bnot<Bool>(a)
cvt<From, To>(value)
```

`wrap` arithmetic is modular for its result width.  `trap` arithmetic checks
its exceptional condition and exhibits the `traps` effect.  Comparisons return
`Bool`.  `cvt` requires distinct numeric source and destination types; for
example:

```text
let wide: own u64 = cvt<u8, u64>(small);
let code_unit: own u32 = cvt<u8, u32>(small);
let narrow: own u8 = cvt<u64, u8>(wide);
```

Combine comparisons by first binding each `Bool`, then applying a Boolean
operation.  As everywhere else, operation arguments cannot themselves contain
operations.

## Checks

A nonrecoverable condition uses a Boolean atom:

```text
let acceptable: own Bool = ile<u64>(used, available);
check acceptable else trap "contract";
```

The message is diagnostic text.  A failed check traps; there is no unchecked
assertion or unsafe escape.
