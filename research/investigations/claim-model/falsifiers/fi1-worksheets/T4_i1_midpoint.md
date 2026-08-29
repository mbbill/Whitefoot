# T4 - I1, the midpoint, the local statement (DESIGN.md 3.9.4)

```
let span = hi -checked lo;      // Ok arm
let half = span / 2_u64;
let mid  = lo +checked half;    // Ok arm
bound probe_inside: ilt(mid, hi);
```
P = `mid - hi + 1 <= 0`.  [IND-10]: region = the run ending at the statement;
`mid` is committed in it, `hi` is live and uncommitted. Admitted.

## Reading A - the file's reading (witness hypotheses are substituted too)
Backward pass:
  `let mid = lo +checked half` -> clause (a) Ok-arm: p = `lo + half - hi + 1`
  `let half = span / 2_u64`    -> clause (d): `half := q`, hypotheses
                                  `2q - span <= 0`, `span - 2q <= 1`
  `let span = hi -checked lo`  -> `span` no longer occurs IN THE POLYNOMIAL;
                                  the file rewrites it INSIDE THE HYPOTHESES.
p0 = `lo + q - hi + 1`
H  = { H1 = `2q - hi + lo <= 0`, H2 = `hi - lo - 2q - 1 <= 0`,
       H3 = `lo - hi + 1 <= 0` (the (lo,hi) pair slot of H group 3) }
Elimination terms: `q` (witness, first), then `hi`, `lo`. Three, <= 4. OK.

Certificate sigma(q) = H1, sigma(hi) = H3:
  t = q  : a = +1, b = +2, a*b > 0
           p := 2*p0 - 1*H1 = 2lo + 2q - 2hi + 2 - 2q + hi - lo = `lo - hi + 2`
           s := 2
  t = hi : a = -1, b = -1, a*b > 0
           p := 1*p - 1*H3 = (lo - hi + 2) - (lo - hi + 1) = `1` ;  s := 2
  t = lo : outside sigma -> skipped
  RELAX(1) = 1 ; floor(1/2) = 0 <= 0  ->  VERIFIED.
Reproduces the file step for step, including the integer tightening. ARITHMETIC OK.

Also confirms 2.4's third repair option is no longer needed: with the (q,hi)
pair slot PRESENT the certificate above still exists, so the "syntactically
total hypothesis list breaks I1" objection is dissolved by the search. GOOD.

## Reading B - the drafted sentence, read literally (FINDING F2)
"replacing, at each `let` or `set` commit whose destination occurs IN THE
POLYNOMIAL at the moment the pass reaches that commit, that destination by the
polynomial of the commit's right-hand side" - the two division witnesses are
hypotheses, not "the polynomial", so `span` survives in them:
H = { `2q - span <= 0`, `span - 2q - 1 <= 0`, `lo - hi + 1 <= 0` }
  sigma(q) = `2q - span <= 0` : p := 2*p0 - 1*h = `2lo - 2hi + span + 2` ; s = 2
  sigma(hi) = `lo - hi + 1 <= 0`: a = -2, b = -1 -> p := 1*p - 2*h = `span` ; s = 2
  `span` is NOT an elimination term (the set is fixed from p0), so it cannot be
  eliminated. RELAX(span) = cu(span) = max(u64) at the region entry, where
  `span` is not even in scope. floor(max(u64)/2) > 0  ->  REFUSED.
  No other sigma reaches 0: skipping q leaves `q` in the residual with no
  constant bound at all (FINDING F9).
Two readings, different accepted sets, on the file's one surviving customer for
[IND-10]. DETERMINISM FINDING.

## Second ambiguity on the same trace (FINDING F3)
Clause (a) covers "`a + b`, `a - b`, `a * b`, AND THE `Ok`-ARM BINDING of
`+checked`, `-checked` or `*checked`: the exact polynomial, provided the state
at that commit on that path discharges the operation's `[ENT-6]` domain
obligation". A `+checked` has no `[ENT-6]` domain obligation. Read as "vacuously
satisfied", I1 works; read as "there is no such obligation, so nothing
discharges it", BOTH `span` and `mid` fall to clause (e) and I1 dies. This is
exactly the vacuity the file upholds as A1 / judge-2's FATAL-1c against the
drafted wrap proviso, repaired in clause (b) and re-created in clause (a).

## Certificate space count for this trace (adversarial (c))
Elimination terms 3; |H| = 3 (2 division witnesses + 1 derivable pair slot).
Partial injections = sum_k C(3,k)*(3)_k = 1 + 9 + 18 + 6 = 34. Finite. OK.
The SIZE is capped by the rule; the CONTENT is not - H group 3 and the RELAX
intervals are whatever is "derivable at the check point".
