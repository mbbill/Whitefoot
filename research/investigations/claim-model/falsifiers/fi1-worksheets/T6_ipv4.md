# T6 - the counted ipv4 restructure (named by F-I1 as "of 4.4")

## First result: the trace does not exist
F-I1 says "Implement `[IND-7]` against the traces of 3.9.3, 3.9.4 and 3.9.5,
plus the counted ipv4 restructure of 4.4", and 2.4 lists among "the drafted
traces ... the four bucket-B statements, the counted ipv4 restructure".
Neither is drafted anywhere in the file. 2.8 dispositions `ipv4_checksum.wf:22`
to **the pair guard** (`t8`, compiled) and 4.4's V4 row records the congruence
route as "not bought - no customer"; the four bucket-B claims are likewise all
routed to guard rewrites or restructure. So two of 2.4's six named traces have
no `[IND-7]` derivation to preserve.

## Second result: constructed, it does not verify
The counted restructure is `L26_ipv4_counted.wf` (compiled here:
REJECT `[OP-4] residual: offset < len(deref(header))`):
```
let length = len(deref(header));
let half = length / 2_u64;
for @words k in 0_u64..half { let offset = k *wrap 2_u64; ... }
```
The needed fact is `2k + 2 <= length`, and the probe's own doc names the route:
"2k+2 <= 2*half <= length follows from k < half and the division witness".

(1) Labelled `bound @words w: ile(2_u64 * k + 2_u64, len(deref(header)));`
    P = `2k + 2 - length <= 0`.
    BASE [IND-5], post-capture, `k = 0`: p0 = `2 - length`.
    RELAX = 2 + (-1)*cl(length) = 2 - 0 = 2 > 0.
    H group 3 slots: `k - length <= 0` is derivable (k = 0, length >= 0);
      sigma(k) = it -> p := 1*p0 - 2*(k - length) = `length + 2`, worse;
      sigma(length) = it (a = -1, b = -1) -> p := p0 - (k - length) = `k + 2`,
      RELAX = 0 + 2 = 2 > 0.
    REFUSED. The base needs `length >= 2`, i.e. `2*half - length <= 0` together
    with `half >= 1`; `2*half - length <= 0` is not a difference bound, is not
    published by any row, and is not named by any group of H.

(2) `bound @words w: ile(2_u64 * k, len(deref(header)));`
    Base: p0 = `-length`, RELAX = 0 <= 0. VERIFIED.
    STEP: P[k := k+1] = `2k + 2 - length`; no body commit touches k or length,
    so p0 = `2k + 2 - length`. H1 = `2k - length`.
      sigma(k) = H1 : a = +2, b = +2 -> p := 2*p0 - 2*H1 = `4` ; s = 2
      floor(4/2) = 2 > 0. REFUSED.
      sigma(k) = the pair slot `k - length <= -1` (if derivable):
        p := 1*p0 - 2*(k - length + 1) = `length` ; RELAX = cu(length) > 0. REFUSED.
      sigma(length) = `k - length + 1 <= 0` : p := p0 - h = `k + 1`,
        RELAX = cu(k) + 1 > 0. REFUSED.
    REFUSED. The statement is TRUE (2k <= 2*half <= length) and unprovable:
    the step needs `2*half - length <= 0`, coefficient 2, again unnameable.

(3) Local `[IND-10]` inside the body, `bound w: ilt(offset, length);` after
    `let offset = k *wrap 2_u64;`  (clause (b) admits the `*wrap`):
    p0 = `2k + 1 - length`; sigma(k) = `k - length + 1 <= 0` ->
    p := 1*p0 - 2*h = `length - 1`; RELAX > 0. REFUSED.

(4) The enabling fact is itself statable and provable as a local statement
    before the loop - `bound halved: ile(2_u64 * half, len(deref(header)));`
    p = `2*half - length`, backward pass gives `2q - length` with the clause (d)
    witnesses, sigma(q) = `2q - length <= 0` (a = +2, b = +2) -> p := 0, s = 2,
    VERIFIED - but `[IND-8]` CANNOT PUBLISH IT: `half` carries coefficient 2 and
    the projection publishes only terms "with coefficient `a` in `{+1, -1}`",
    and the difference-bound clause requires `b = -a`. The projection language
    is unit-coefficient difference bounds, so the proved fact dies at the
    statement boundary and never reaches the loop head.

## Verdict on T6
F-I1's own refutation criterion - "*Refuted if* any needs a hypothesis the rule
does not name" - fires. The counted ipv4 restructure needs `2*half - length <= 0`
at an `[IND-7]` check point; no clause of `[IND-4]`, `[IND-6]`, `[IND-7]` or
`[IND-8]` can put it there.
