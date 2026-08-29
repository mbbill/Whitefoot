# CENSUS — what the corpus actually asks the fact language for

Batch 0106, measurement B0. Tree read: `batch/0106-claim-model-design` at
`b1f57cc8` (base `integration/2026-08-28c`, tip `b1367c82`), spec v0.39 ACTIVE.
Companion to `DESIGN.md` and `TERRAIN.md` in this directory.

The design in `DESIGN.md` bets that the fact language of [ENT-2] — difference
bounds `t1 - t2 <= c` over two terms, plus disequalities `t1 != t2`, plus
opaque signed Booleans (2870, 2901–2905) — is wide enough that the obligations
a real program cannot answer inside it are rare, and that almost every claim in
the tree names a *missing publisher* rather than a missing vocabulary. This file
counts that, mechanically, and says where the count is a judgment rather than a
number.

Per the owner's standing ruling, a corpus count is never evidence that a
language need is absent. §4 is the counterweight: every wall a blind writer
actually hit in this area, and whether bounds-plus-publishers dissolves it.

---

## 0. How to reproduce every number

Every command below is run from the repository root of this worktree with this
preamble. The scratch directory is outside the tree; nothing here writes into
the tree.

```sh
WT=$(pwd)                              # repository root
SC=$(mktemp -d)                        # scratch, outside $WT
cd "$WT/compiler" && CARGO_TARGET_DIR="$SC/target" \
  cargo build --profile gate --bin whitefootc --locked --offline
W="$SC/target/gate/whitefootc"; cd "$WT"

# the string-stripper every text census uses: claim justifications are one long
# string literal per claim and would otherwise be counted as source.
cat > "$SC/strip.awk" <<'AWK'
{ line=$0; out=""; ins=0; n=length(line)
  for (i=1;i<=n;i++){ ch=substr(line,i,1)
    if (ins) { if (ch=="\"") ins=0 } else { if (ch=="\"") ins=1; else out=out ch } }
  print out }
AWK
```

Two methods are used and every number below is labelled with one of them.

- **[compiler]** — a verdict the worktree compiler produced. Either a baseline
  compile, or the *claim-deletion differential* of §0.2.
- **[text]** — a regular-expression census of the source, after string
  stripping. Its known error modes are stated in §5.

### 0.1 The claim table

```sh
grep -rn '^[[:space:]]*claim ' --include='*.wf' tests/ research/ \
  | awk -F: '{file=$1; line=$2; rest=substr($0, index($0,$3))
      name=""; pred=""; gap=""
      if (match(rest, /claim [A-Za-z0-9_]+:/)) name=substr(rest, RSTART+6, RLENGTH-7)
      p=index(rest, " because "); if (p>0) pred=substr(rest, index(rest,":")+2, p-index(rest,":")-2)
      g=index(rest, "checker gap: ")
      if (g>0) { tail=substr(rest, g+13); q=index(tail, "\\n")
                 gap = (q>0) ? substr(tail,1,q-1) : tail }
      gsub(/";$/,"",gap)
      printf "%s\t%s\t%s\t%s\t%s\n", file, line, name, pred, gap }' > "$SC/claims.tsv"
wc -l < "$SC/claims.tsv"                       # 177 over tests/ and research/
awk -F'\t' '$1 ~ /^tests\//' "$SC/claims.tsv" | wc -l    # 135
```

### 0.2 The claim-deletion differential [compiler]

For each claim: delete its line and recompile. If the claim was the function's
only trap, the declared effect row is now over-declared and [EFF-2] fires first
and masks the real answer; the loop therefore applies the compiler's own named
EFF-2 fix (remove the extra category, restoring `pure` if the row empties) and
recompiles, up to ten times, before recording the first non-EFF-2 diagnostic.
The recorded rule is the obligation the claim was discharging.

```sh
mkdir -p "$SC/work"; : > "$SC/diff.tsv"
while IFS=$'\t' read -r file line name pred gap; do
  tmp="$SC/work/mut.wf"; awk -v n="$line" 'NR!=n' "$file" > "$tmp"
  for round in $(seq 1 10); do
    out=$("$W" --emit-llvm -o "$SC/work/o.ll" "$tmp" 2>&1 \
          | grep '^whitefootc: ' | grep -v '^whitefootc: note' | head -1)
    if [ -z "$out" ]; then rule="STILL-ACCEPTS"; kind=""; break; fi
    rule=$(printf '%s' "$out" | grep -oP '\[[A-Z]+-[0-9]+\]' | head -1)
    kind=$(printf '%s' "$out" | grep -oP 'kind: [A-Za-z]+' | head -1 | sed 's/kind: //')
    [ "$rule" = "[EFF-2]" ] || break
    extra=$(printf '%s' "$out" | grep -oP 'extra: \["[a-z]+"\]' | grep -oP '"[a-z]+"' | tr -d '"' | head -1)
    ln=$(printf '%s' "$out" | grep -oP 'mut\.wf:[0-9]+:' | grep -oP '[0-9]+' | head -1)
    [ -n "$extra" ] && [ -n "$ln" ] || break
    awk -v n="$ln" -v e="$extra" '
      NR==n{ p1=", " e; p2=" " e ","; p3=" " e
             i=index($0,p1); if(i>0){ $0=substr($0,1,i-1) substr($0,i+length(p1)) }
             else { i=index($0,p2); if(i>0){ $0=substr($0,1,i-1) substr($0,i+length(p2)-1) }
                    else { i=index($0,p3); if(i>0){ $0=substr($0,1,i-1) substr($0,i+length(p3)) } } }
             if ($0 !~ /(pure|reads|writes|allocates|traps)/) { j=index($0,"{"); if(j>0) $0=substr($0,1,j-1) "pure " substr($0,j) } }
      {print}' "$tmp" > "$tmp.n" && mv "$tmp.n" "$tmp"
  done
  printf '%s\t%s\t%s\t%s\t%s\n' "$file" "$line" "$name" "$rule" "$kind" >> "$SC/diff.tsv"
done < "$SC/claims.tsv"
```

Baseline verdict per claim-bearing file, needed to read the differential:

```sh
cut -f1 "$SC/claims.tsv" | sort -u | while read f; do
  r=$("$W" --emit-llvm -o "$SC/b.ll" "$f" 2>&1 | grep '^whitefootc: ' \
      | grep -v note | grep -oP '\[[A-Z]+-[0-9]+\]' | head -1)
  printf '%s\t%s\n' "$f" "${r:-accept}"
done > "$SC/base.tsv"
awk -F'\t' '{print $2}' "$SC/base.tsv" | sort | uniq -c | sort -rn
```

---

## 1. Obligations: how each partial operation is admitted today

### 1.1 The families

`TERRAIN.md` §1.9 reads [ENT-6] 3130–3131 as attaching **exactly four**
obligation families; §1.9's own frame and 3169–3173 add the two cross-function
proof roots that [CLM-2] admits as consumers. The complete list of proof-required
sites, and the source shape each is attached to:

| family | rule | one obligation per | goal |
| --- | --- | --- | --- |
| SubscriptBounds | [ENT-6] 3132, [OP-4] | every `psuffix` in a subscript chain, read/write/`set` alike | `i < len(P)` |
| IntegerDomain | [ENT-6] 3133–3153, [OP-2] | every occurrence of the bare infix `+ - * / %` and the dotless `ineg iabs ishl ishr` | that operation's total `.defined` goal |
| AllocationFit | [ENT-6] 3156–3161, [OP-9] | every `buffer_new` / `buffer_vacant` | `buffer_fits<T>(n)` |
| SystemRange | [ENT-6] 3163–3167, [SYS-8] 2537+ | **two** per call to `read_at`, `write_once`, `directory_next`, `host_copy_bytes`, `host_copy_utf8`, `open_directory`, `open_file` | `start <= end`, then `end <= len(deref(buffer))` |
| call requirement | [FN-8] 1228+ | every `requires` clause of the callee, at each call site | the caller-instantiated requirement |
| selected-return proof | [FN-9] 1273+ | every `ensures` clause, in the callee | the declared normal-return relation |

### 1.2 Site counts and how each site is discharged

`sites` is **[text]**; `by claim` is **[compiler]** (§0.2); `by static fact` is
the residual, and it is exact because every source counted here compiles, so an
obligation not discharged by a claim was discharged by the entailment state.

**A. `tests/programs/` — 25 files, 6,656 lines.** All 25 compile; the four
`raw_deflate_*` files are one four-file module and were compiled as a unit.

| family | sites | by claim | by static fact |
| --- | --- | --- | --- |
| SubscriptBounds | 426 | **18** | 408 (95.8%) |
| IntegerDomain | 15 | 0 | 15 |
| AllocationFit | 86 | 0 | 86 |
| SystemRange | 15 calls → 30 goals | 0 | 30 |
| call requirement [FN-8] | 43 call sites → 48 goals | 0 | 48 |
| selected-return [FN-9] | 3 `ensures` | 0 | 3 |

**B. `tests/conformance/cases/` — the 252 cases whose manifest `expect.kind` is
`accept` or `run`; 7,100 lines.** (The other 265 are `reject` cases and are not
a discharge population.)

| family | sites | by claim | by static fact |
| --- | --- | --- | --- |
| SubscriptBounds | 211 | **37** | 174 |
| IntegerDomain | 82 | **25** | 57 |
| AllocationFit | 75 | 0 | 75 |
| SystemRange | 35 calls → 70 goals | 0 | 70 |
| call requirement [FN-8] | 19 goals | **2** | 17 |
| selected-return [FN-9] | 10 `ensures` | **1** | 9 |

**C. `research/experiments/blind-writer/2026-08-28/programs/p1..p5` — the five
programs a writer with no prior Whitefoot exposure wrote; 1,694 lines.** All
five compile.

| family | sites | by claim | by static fact |
| --- | --- | --- | --- |
| SubscriptBounds | 66 | **0** | 66 |
| IntegerDomain | 8 | 0 | 8 |
| AllocationFit | 23 | 0 | 23 |
| SystemRange | 17 calls → 34 goals | 0 | 34 |
| call requirement [FN-8] | 13 call sites → 17 goals | 0 | 17 |
| selected-return [FN-9] | 0 | 0 | 0 |

**D. `tests/codegen/` — 95 files, 2,027 lines.** Its README states "preserved
future-facing corpus; not an active gate", and the measurement bears that out:
**every one of the 13 claim-bearing fixtures is rejected by the current
compiler**, all thirteen citing [CLM-1] (§6.1). They are not a discharge
population and contribute nothing to this table.

**E. `research/experiments/` — 45 `.wf` files, 8,861 lines, of which
`blind-writer` is 12 files / 2,234 lines.** Whole-directory site counts: 251
subscripts, 59 exact infix, 145 allocations, 146 SYS-8 calls, 22 `requires`, 6
`ensures`. **Zero `claim` statements anywhere under `research/experiments`.**

Reproduce (A shown; substitute the path set for B/C/D/E):

```sh
find tests/programs -name '*.wf' -print0 | xargs -0 cat | awk -f "$SC/strip.awk" > "$SC/s.tmp"
grep -oP '[A-Za-z0-9_)\]]\['                              "$SC/s.tmp" | wc -l  # subscripts 426
grep -oP '(?<=[A-Za-z0-9_)\] ]) [-+*/%] (?=[A-Za-z0-9_(])' "$SC/s.tmp" | wc -l  # exact infix 15
grep -oP '\b(ineg|iabs|ishl|ishr)\('                      "$SC/s.tmp" | wc -l  # dotless 0
grep -oP '\bbuffer_(new|vacant)\b'                        "$SC/s.tmp" | wc -l  # 86
grep -oP '\b(read_at|write_once|directory_next|host_copy_bytes|host_copy_utf8|open_directory|open_file)[<(]' \
                                                          "$SC/s.tmp" | wc -l  # 15
grep -cP '^\s*ensures\b'                                  "$SC/s.tmp"          # 3
```

The accepting-conformance file set:

```sh
python3 - <<'PY' > "$SC/acc.txt"
import json
for l in open('tests/conformance/manifest.jsonl'):
    l=l.strip()
    if not l or l.startswith('#'): continue
    d=json.loads(l)
    if 'id' in d and d['expect']['kind']!='reject':
        print('tests/conformance/cases/'+d['id']+'.wf')
PY
wc -l < "$SC/acc.txt"    # 252
```

### 1.3 The third column the question asks for: shaped to avoid

No mechanical measure separates "the writer shaped the program to avoid an
obligation" from "the obligation never arose". What *is* measurable is the one
avoidance move the language actually offers: writing a total operation where an
exact one would have attached an IntegerDomain obligation. **[text]**

| corpus | exact infix (obligation) | `+wrap`-class (no obligation) | `.wrap`/`.strict` calls | `+checked`-class |
| --- | --- | --- | --- | --- |
| `tests/programs` | 15 | 306 | 26 | 7 |
| conformance accept/run | 82 | 124 | (25 over all 517) | (29 over all 517) |
| blind-writer p1..p5 | 8 | 58 | 0 | 0 |
| `tests/codegen` | 0 | 421 | 36 | 0 |

**In real programs, 95.8% of integer arithmetic is written in a spelling that
attaches no proof obligation at all** (306+26+7 = 339 total-form occurrences
against 15 exact ones). The IntegerDomain family is not where writers spend proof
effort; it is where they decline to.

For SubscriptBounds there is no avoidance spelling — every subscript carries its
obligation — so the A/B/C tables above are a complete account of that family.
The discharging *shape* is what 0098 reported and what these counts corroborate,
measured identically on both corpora: **[text]**

| corpus | subscripts | `if` lines | comparison bindings | `len(` uses |
| --- | --- | --- | --- | --- |
| `tests/programs` | 426 | 544 | 366 | 104 |
| blind-writer p1..p5 | 66 | 85 | 86 | 51 |

The blind writer wrote more comparison bindings than subscripts and 51 `len(`
uses for 66 subscripts. That is 0098's sentence — "every subscript … was
discharged by ordinary `if` branches and `len()` rebinding" — as a ratio.

### 1.4 What the claims actually consume

The differential gives the exact obligation each claim discharges. Over the
**83 claims that live in sources the current compiler accepts**:

| obligation the claim discharges | claims |
| --- | --- |
| SubscriptBounds ([OP-4] `UndischargedBoundsObligation`) | **55** |
| IntegerDomain ([OP-2] `UndischargedIntegerDomainObligation`) | **25** |
| call requirement ([FN-8] `UndischargedCallRequirement`) | 2 |
| selected-return ([FN-9] `UndischargedPostcondition`) | 1 |
| AllocationFit | 0 |
| SystemRange | 0 |
| none — the claim was redundant | **0** |

Two rows needed hand resolution because the mutation confounded them, and both
were finished by hand with the same compiler:
`tests/programs/raw_deflate_dynamic_decode.wf:32` (compiled as its four-file
module; residual `bounded < len(code_length_order)`, so [OP-4]) and
`tests/conformance/cases/x-base64-rfc-vectors-run.wf:16` ([FN-9]
`UndischargedPostcondition` — the **only** selected-return claim in the tree).

**All 18 claims in real programs discharge a SubscriptBounds obligation. Not one
discharges an allocation, a system range, a call requirement, or a return
proof.** The claim construct, in the corpus of programs someone wanted, is a
subscript-bounds construct.

The other 52 claims live in 46 files the compiler rejects: 20 files [CLM-1] (13
of them the stale codegen fixtures of §6.1, the rest deliberate negative
conformance cases), 6 [PRV-2], 6 [CLM-3], 5 [EFF-2], 4 [CLM-2], 3 [PRV-3], one
[FN-8] and one [FN-3]. (A single-file compile of the 101 claim-bearing files
reports 47 rejections; the forty-seventh is `raw_deflate_dynamic_decode.wf`
citing [TYPE-5], which compiles as part of its module and is counted among the
83 above.)

---

## 2. Claims: the gap tokens, and the decisive bucket

### 2.1 Recount

**135** `claim` occurrences under `tests/`, in 101 files — the same total
`TERRAIN.md` §2 reports, independently recounted. Split by home: 102 in
`tests/conformance/cases/` (81 files), 18 in `tests/programs/` (7 of 25 files),
15 in `tests/codegen/cases/bounds/` (13 files). **Zero** anywhere under
`research/experiments/`, including the entire blind-writer corpus. 120 of the
135 carry a `checker gap:` field; the 15 that do not are all codegen fixtures
(12 one-line `ieq(value, N)` drift oracles and 3 `False()` preemption markers).
57 distinct gap texts.

```sh
awk -F'\t' '$1 ~ /^tests\//' "$SC/claims.tsv" | wc -l                        # 135
awk -F'\t' '$1 ~ /^tests\// && $5 != ""' "$SC/claims.tsv" | wc -l            # 120
awk -F'\t' '$1 ~ /^tests\// && $5 != ""{print $5}' "$SC/claims.tsv" | sort -u | wc -l   # 57
grep -rc '^[[:space:]]*claim ' --include='*.wf' research/experiments | awk -F: '{s+=$2} END{print s}'  # 0
```

### 2.2 Token classification

`DESIGN.md` §5.2 publishes four gap tokens plus the one that is not a gap. The
rule used here, applied to the claim's own `checker gap:` text, in this priority
order (first match wins):

1. `boundary` — the text says the subject is a value a callable boundary
   produced: "CLM-1 must refuse", "callable boundary returned", "uncontracted",
   "unverified callee body", "system-call result", "closed caller arguments
   through …", "borrowed buffer length through the child call".
2. `content` — the fact is about an element of an array, slice or buffer.
3. `vocabulary` — the fact or an indispensable premise is not a two-term
   difference bound or disequality: a congruence, a three-term relation.
4. `flow` — the text names a loop, an induction, a recurrence, a back edge, or a
   join across two merge points.
5. `image` — otherwise: the operation row publishes no image, or none unique.
6. `none-fixture` — the text says there is no gap, or the claim has no gap field.

```sh
awk -F'\t' '$1 ~ /^tests\//{ g=$5
  if (g=="" || g ~ /there is no checker gap|^none;/) t="none-fixture"
  else if (g ~ /CLM-1 must refuse|callable boundary returned|uncontracted|unverified callee body|unstated property of a system-call result|closed caller arguments through read_pair|borrowed buffer length through the child call/) t="boundary"
  else if (g ~ /buffer initializer value through a selected subscript result/) t="content"
  else if (g ~ /stride parity|normalize the remaining-length guard/) t="vocabulary"
  else if (g ~ /loop|induction|recurrence|backedge|join payload-derived result bounds/) t="flow"
  else t="image"
  print t }' "$SC/claims.tsv" | sort | uniq -c | sort -rn
```

| token | claims | what the writers wrote under it |
| --- | --- | --- |
| `image` | **55** | 46 remainder-result-range (43 "ENT proves the remainder operation domain but does not publish its result range" and variants + 3 "no residue for a literal remainder"), 7 nominal payload / constructor-field / projection, 1 `imin` (the other 4 name a loop as well and fall to `flow`), 1 "invert the wrapping subtraction …" |
| `flow` | **38** | the loop family. 39 claims mention a loop; 38 land here, 1 goes to `vocabulary` (parity) and 1 to `boundary` (a loop endpoint that is a call result). Includes the 8 two-length-correlation claims and 4 of the 5 `imin` claims |
| `none-fixture` | **21** | 15 codegen fixtures with no gap field, 2 `False()`, 1 repeated occurrence, 3 "none; …" |
| `boundary` | **17** | the deliberate cross-function refusals and the FN-8 shapes |
| `vocabulary` | **3** | `ipv4_checksum.wf:22`, `percent_decode.wf:28`, `percent_decode.wf:31` |
| `content` | **1** | `prv3-neg-read-offset-taint.wf:44` |

Sub-family recounts, against `DESIGN.md` §7.4's table:

| family | this census | §7.4 | agrees? |
| --- | --- | --- | --- |
| remainder result range, all variants | 46 | 43 + 3 | yes |
| `imin` result range | 5 | 5 | yes |
| claims mentioning a loop | 39 | 39 | yes |
| two-length correlation | 8 | 8 | yes, count only — see §6.3 for the token |
| nominal payload / constructor / projection | 7 | 3 | **no** — §7.4 lists only two of the five wordings |
| borrowed buffer length through the child call | 2 | 2 | yes, count only — tokened `boundary` here, `flow` there |

### 2.3 The decisive bucket: publisher gap, or residue

The question is not which token a claim carries but whether *bounds could hold
the fact at all*. Asked literally — "is the fact the obligation needs a
conjunction of two-term difference bounds and disequalities?" — the answer is
**yes for all 114 gap-stating claims**, because every obligation in the language
is itself such a fact ([ENT-6] 3132 normalizes a subscript bound to
`i - len(P) <= -1`, and the [ENT-2] terms it needs always exist). That reading
of the question is therefore not decisive, and reporting `114 / 0` would be
true and useless.

The decisive question is one level down: **can a publisher over the two-term
state establish it?** That splits three ways, and the split is the load-bearing
judgment in this file:

- **Bucket P — publisher gap.** A publisher whose inputs are two-term facts
  already in the state can establish the fact by a *forward* transfer: a row
  image from operand facts to result facts, a merge or loop-head transfer, a
  `requires` or `ensures` on the boundary. This is exactly the shape
  `DESIGN.md` §5.1 gives publisher 1 — "a closure indexed by the operation table
  and the control graph" — and §5.3's examples (`r <= d - 1`, `imin(a,b) <= a`)
  are all forward.
- **Bucket B — backward-transfer gap.** The fact is two-term and every input it
  needs is in the state, but no *forward* transfer reaches it: the derivation
  runs from a fact about a row's **result** back to a fact about its
  **operands**, across an equality (`r = a ± b`) the DBM cannot hold as a fact.
  A DBM cannot hold that equality and neither can an octagon; but a row rule can
  discharge it without ever holding it, by publishing `a - b >= k` when the
  state carries `r >= k`, or `a != b` when it carries `r != 0`.
- **Bucket R — residue.** The information the fact needs is not in the two-term
  state and cannot be put there by any publisher: a congruence, or a value the
  language has no term for.

| bucket | claims | share of the 114 gap-stating claims |
| --- | --- | --- |
| **P — forward publisher gap** | **108** | 94.7% |
| **B — backward transfer over `±wrap`** | **4** | 3.5% |
| **R — true residue** | **2** | 1.8% |
| (no gap stated: fixtures) | 21 | — |

B and R together are the six claims §2.4 lists, and they are the six the design
should be judged against. The three-way split matters because the two remedies
are different sizes: **B is one row rule in the existing vocabulary** and closes
four claims in three real programs; **R is a vocabulary change** (congruences,
or element-content terms) and closes two.

Within P, one sub-bucket deserves its own name because it is not a *fact* gap
but a *normalization* gap: **P′, 8 claims** whose predicate is an exact-domain
goal over two non-constant operands (`a +defined b`, `first +defined second`,
`left +defined right`, `sum +defined i`, `total +defined i`, `p.x +defined p.y`,
`o.inner.v +defined o.k`, and `x +defined x`). Every premise those claims need
is a two-term bound on each operand; what is missing is the step from two bounds
to the sum, which 3146 currently forbids outright ("Two nonconstant add,
subtract, or multiply operands have no L0 normalization route"). A sufficient
route — `a <= ka`, `b <= kb`, `ka + kb <= max(T)` — needs no new vocabulary.

### 2.4 The residue and the backward-transfer gap, in full

Six claims: four in bucket B (rows T1–T4), two in bucket R (rows R1–R2). Five are in three real programs;
the sixth is a conformance case. For each: the fact needed, and what stops a
forward publisher from reaching it.

| # | bucket | site | token | fact the obligation needs | what stops a forward publisher | m |
| --- | --- | --- | --- | --- | --- | --- |
| R1 | **R** | `tests/programs/ipv4_checksum.wf:22` (`low_in_header`) | `vocabulary` | `offset + 1 < length` | **congruence.** `offset` is even by the loop's stride, `length` is even by the contract's `iand(length,1) == 0`. `offset < length` alone gives `offset + 1 <= length`, not `< length`; closing the last unit needs `offset ≡ 0 (mod 2)` and `length ≡ 0 (mod 2)`, and [ENT-2] has no congruence | 0 |
| T1 | **B** | `tests/programs/percent_decode.wf:28` (`high_in_source`) | `vocabulary` | `input_index + 1 < source_length` | **three-term.** The guard is `remaining >= 3` where `remaining = source_length -wrap input_index`. The conclusion is two-term; the only bridge is the equality `remaining = source_length - input_index`, three distinct terms, which a DBM cannot hold in either direction | 0 |
| T2 | **B** | `tests/programs/percent_decode.wf:31` (`low_in_source`) | `vocabulary` | `input_index + 2 < source_length` | same bridge as T1 | 0 |
| T3 | **B** | `tests/programs/wfgrep.wf:434` (`carry_in_input`) | `image` | `carry < input_room` | **three-term, through a disequality.** `carry <= input_room` is available from the `admitted` branch; the strictness comes from `room != 0` where `room = input_room -wrap carry`. [ENT-4] rule 2 would close `carry != input_room` with `carry <= input_room` into `carry - input_room <= -1` — but deriving `carry != input_room` from `room != 0` needs the same three-term equality | 0 |
| T4 | **B** | `tests/programs/wfgrep.wf:553` (`shift_read_in_input`) | `flow` | `source_index < input_room` | **three-term, through a sum.** `source_index = bounded_scan +wrap moved` and `tail = bounded_available -wrap bounded_scan`; the chain `bounded_scan + moved < bounded_scan + tail = bounded_available <= input_room` needs both sums. (Its sibling at `wfgrep.wf:556` needs only `tail <= bounded_available`, a two-term image of unsigned subtraction, and is therefore in bucket P) | 0 |
| R2 | **R** | `tests/conformance/cases/prv3-neg-read-offset-taint.wf:44` (`derived_index_in_bytes`) | `content` | `derived_offset < room` | **quantified over elements.** The conclusion is two-term over the binding `derived_offset`, but its only premise is "every element of `offsets` is `0_u64`". [ENT-2] 2870(a) excludes subscript suffixes from terms, so no term names an element, and 3218's single conservative all-elements component cannot carry a value | 0 |

Three observations about that list, each decision-relevant:

1. **All four of bucket B are one shape, and one rule closes them.** T1–T4 want
   the same thing: a fact about a `±wrap` row's **result** turned into a fact
   about its **operands**, across `r = a ± b` with `r`, `a`, `b` distinct.
   Neither a DBM nor an octagon can hold that equality, and holding it would
   need three-variable linear relations. But nothing has to hold it. Given the
   no-wrap side condition the state already carries (`b <= a` for a subtraction),
   one row rule discharges all four in the existing vocabulary: publish
   `a - b >= k` when the state carries `r >= k`, and `a != b` when it carries
   `r != 0`. **This is the only fact-language work the whole corpus asks for,
   and it is asked for by three real programs.**
2. **The direction of `DESIGN.md`'s image column is the thing to check.** §5.1
   builds publisher 1 as a closure indexed by the operation table, and every
   example in §5.3 maps operand facts forward to result facts. That column, as
   specified, closes 108 claims and **none of T1–T4**. Whether the image column
   is allowed to run backwards is not a detail of B1's enumeration; it decides
   four of the six hardest claims in the tree, and §8's F3 (the image-uniqueness
   audit) does not currently ask the question.
3. **The true residue is two claims.** One congruence, one element content, and
   both are named ceiling clauses `DESIGN.md` §5.2 already publishes
   (`vocabulary` and `content`). Neither is in a program anyone shipped for its
   own sake: R1 is one of two claims in `ipv4_checksum.wf` and R2 is a
   provenance negative fixture.
4. **The residue is not where the volume is.** Six retained runtime checks, five
   of them in ~6,600 lines of real program.

Bucket assignment was made by reading the source of every claim flagged by the
gap-text families that could plausibly be residue (three-term, congruence,
product, element content, cross-value correlation), and of all 18 real-program
claims. A safety scan over every claim's justification prose for the words
`parity|even|odd|divisib|multiple of|product of|difference|minus|congru`
returned 22 hits and no seventh member of B or R — every other "minus" is
subtraction of a *constant*, which is two-term:

```sh
grep -rn '^[[:space:]]*claim ' --include='*.wf' tests/ \
  | grep -iP 'parity|even |odd |divisib|multiple of|product of|difference|minus|congru' | wc -l   # 22
```

---

## 3. Selector width m

### 3.1 The tracing rule

**m(claim) = the number of distinct boundary-derived selecting conditions that
choose between two or more different reaching definitions of a support place of
the claim's predicate.**

Read from [ENT-6] 3233–3239 as `TERRAIN.md` §1.10 quotes it:

- a *boundary-derived* selector is a condition, match scrutinee or tag, or
  counted endpoint whose value is a `BoundaryResult` — seeded unconditionally by
  every ordinary user call and every system call (3222–3225). Parameters,
  literals, named consts and command-entry parameters are `Local` and are not
  selectors for this purpose.
- *choosing* means what 3235–3237 says it means: the selector reaches a
  component "whose reaching definition on one incoming edge is a different
  definition occurrence from their reaching definition on another", plus each
  binder an arm introduces and each value `value_if`/`value_match` delivers.
  Standing on a boundary-selected edge is not itself a selection, and a
  definition formed after the join from Local operands is Local.
- distinctness is by the identity of the selecting condition value, not by
  syntactic position; two claims under the same `if` share one selector.

### 3.2 The mechanical upper bound, then the hand trace

A scanner counted, per claim, the `if`/`match`/`for` selectors dominating the
claim inside its own function whose condition binding is defined in that
function from a call to a function declared in the file or to a [SYS-8]
operation. That over-counts (it does not check that a support definition is
actually selected), so it is an upper bound.

**Result: 125 of 135 claims have upper bound 0; 10 have upper bound 1; no claim
anywhere in the tree has upper bound 2 or more.** The bound is 0 for a blunt
reason worth stating plainly: for **all 18 real-program claims the enclosing
function contains no user call and no system call at all**, so no
`BoundaryResult` exists in scope to select anything.

Hand-tracing the 10 with a nonzero bound against the rule in §3.1:

| site | m | why |
| --- | --- | --- |
| `reject-clm1-claim-on-delivered-selection.wf:12` | **1** | `picked` is delivered by `if condition` where `condition = hidden_true()` |
| `reject-clm1-claim-on-storage-written-under-selection.wf:13` | **1** | `cursor` has two reaching definitions joined under the same call-derived `condition` |
| `reject-clm1-claim-on-loop-carried-update.wf:11` | **1** | `cursor` is updated under a `for` whose endpoint `upper = endpoint(...)` is a call result |
| `reject-clm1-claim-on-selected-payload.wf:8` | 0 | `measured` *is* the payload; one definition, nothing selected. Refused on subject matter, not on width |
| `accept-clm1-local-claim-after-boundary-exit.wf:16` | 0 | the `match` on `write_once` reaches the claim, but `seed`/`offset` have one literal-built definition each |
| `accept-clm1-local-claim-after-boundary-join.wf:16` | 0 | same |
| `accept-clm1-local-claim-inside-selected-arm.wf:9` | 0 | `position = cursor % 4` inside the arm; `cursor` is a parameter, one definition |
| `prv3-neg-external-claim.wf:38` | 0 today | the `match arg_get` selects whether `copy_host` wrote `bytes`' elements — and 3241–3243 exempts call-written storage. **1 under `DESIGN.md` §3.4's write seed** |
| `prv3-neg-external-claim-conjunction.wf:43` | 0 today | same, same caveat |
| `prv3-neg-read-offset-taint.wf:44` (residue R2) | 0 today | same, same caveat |

### 3.3 The distribution

| m | claims (v0.39 as it stands) | claims (with `DESIGN.md` §3.4's write seed) |
| --- | --- | --- |
| 0 | **132** | 129 |
| 1 | **3** | 6 |
| ≥ 2 | **0** | **0** |

Restricted to the six B and R claims of §2.4: **m = 0 for all six.**
Restricted to real programs and to the blind-writer corpus: **m = 0 for every
claim, without exception.**

For the guarded-refuter feasibility study this is the number that matters. A
refuter that enumerates arm combinations pays `2^m`; the entire corpus at
`b1f57cc8`, across 517 conformance cases, 25 real programs, 95 codegen fixtures
and 45 experiment sources, contains **no claim with m > 1** and only three with
m = 1. Whatever a guarded refuter costs, it is not being asked to be exponential
by anything anyone has written. The honest complement: it also means the corpus
provides **no evidence whatever** about behaviour at m ≥ 2, and a feasibility
study that wants such evidence must construct the programs, not find them.

```sh
cat > "$SC/mbound.awk" <<'AWK'
{ lines[NR]=$0 }
END{
  n=NR
  for(i=1;i<=n;i++){ if (lines[i] ~ /^[[:space:]]*(command[[:space:]]+)?fn[[:space:]]+[A-Za-z_]/){
      fnline[i]=1; s=lines[i]; match(s,/fn[[:space:]]+[A-Za-z_][A-Za-z0-9_]*/)
      nm=substr(s,RSTART,RLENGTH); sub(/^fn[[:space:]]+/,"",nm); declared[nm]=1 } }
  split("read_at write_once directory_next host_copy_bytes host_copy_utf8 open_directory open_file arg_get", so, " ")
  for(k in so) sysop[so[k]]=1
  for(i=1;i<=n;i++){
    if (lines[i] !~ /^[[:space:]]*claim /) continue
    f=0; for(j=i;j>=1;j--) if(fnline[j]){f=j;break}
    e=n; for(j=i+1;j<=n;j++) if(fnline[j]){e=j-1;break}
    delete callb
    for(j=f;j<=e;j++){ L=lines[j]; gsub(/"[^"]*"/,"",L)
      if (match(L,/^[[:space:]]*let[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*=/)) {
        b=substr(L,RSTART,RLENGTH); sub(/^[[:space:]]*let[[:space:]]+/,"",b); sub(/[[:space:]]*=$/,"",b)
        R=substr(L,RSTART+RLENGTH)
        while (match(R,/[A-Za-z_][A-Za-z0-9_]*[<(]/)) { t=substr(R,RSTART,RLENGTH-1)
          R=substr(R,RSTART+RLENGTH); if(declared[t]||sysop[t]) callb[b]=1 } } }
    cnt=0
    for(j=f;j<i;j++){ L=lines[j]; gsub(/"[^"]*"/,"",L); cond=""
      if (match(L,/^[[:space:]]*if[[:space:]]+[A-Za-z_][A-Za-z0-9_]*/)) {
        cond=substr(L,RSTART,RLENGTH); sub(/^[[:space:]]*if[[:space:]]+/,"",cond) }
      else if (match(L,/=[[:space:]]*if[[:space:]]+[A-Za-z_][A-Za-z0-9_]*/)) {
        cond=substr(L,RSTART,RLENGTH); sub(/^=[[:space:]]*if[[:space:]]+/,"",cond) }
      if (cond!="" && callb[cond]) cnt++
      if (L ~ /^[[:space:]]*(let[[:space:]]+[A-Za-z_][A-Za-z0-9_]*[[:space:]]*=[[:space:]]*)?match[[:space:]]/) {
        R=L; while (match(R,/[A-Za-z_][A-Za-z0-9_]*[<(]/)) { t=substr(R,RSTART,RLENGTH-1)
          R=substr(R,RSTART+RLENGTH); if(declared[t]||sysop[t]) { cnt++; break } } }
      if (L ~ /^[[:space:]]*for[[:space:]]/) {
        R=L; while (match(R,/[A-Za-z_][A-Za-z0-9_]*/)) { t=substr(R,RSTART,RLENGTH)
          R=substr(R,RSTART+RLENGTH); if(callb[t]) { cnt++; break } } }
    }
    printf "%s\t%d\t%d\n", FILENAME, i, cnt
  }
}
AWK
find tests -name '*.wf' | sort | while read f; do awk -f "$SC/mbound.awk" "$f"; done > "$SC/mbound.tsv"
awk -F'\t' '{print $3}' "$SC/mbound.tsv" | sort -n | uniq -c
#   125 0
#    10 1
awk -F'\t' '$3>0{print $1":"$2}' "$SC/mbound.tsv"      # the ten sites hand-traced above
```

---

## 4. The owner's doctrine check: the walls a blind writer actually hit

A corpus count never proves a need absent. So: every wall recorded in
`docs/done/0098-blind-writer.md` and `docs/done/0100-writer-defaults-2.md`, and
whether bounds-plus-publishers would have dissolved it.

### 4.1 0098 — the fourteen defaults (D1–D14)

The record's own defect table, read against this area:

| # | the wall | area | would bounds+publishers have dissolved it? |
| --- | --- | --- | --- |
| D1 | one `write_once` per iteration denies staged permission | [PAR-3] permission | no — not a fact question |
| D2 | `reserve_file` + `open_*` behind one helper flips the staged verdict | [OWN-6]/[PAR-3] | no |
| D3 | `&uniq deref(factory)` in a two-statement region → `InvalidChildReborrow`; **"the one wall of the session"** | [OWN-6] borrows | no |
| D4 | `TerminalSet(384244…)` printed for a syntax error | diagnostics | no |
| D5 | `CanonicalIssue` with no expected bytes | [FORM-2] | no |
| D6 | every diagnostic names `input0.wf` | diagnostics | no |
| D7 | no standard input; the Unix filter genre unwritable | [SYS] surface | no |
| D8 | the staged denial is silent without `--par-ledger` | diagnostics | no |
| D9 | [STOR-1]'s mechanical fix leads to a form [OWN-1] rejects | ownership | no |
| D10 | an all-scalar struct is affine | ownership | no |
| D11 | hand-encoded byte lists cannot be documented or checked | literals | no |
| D12 | **`let room = len(b);` re-bound after every callee write — 34 of 41 `len` bindings written defensively; not required** | **this area** | **partly — see below** |
| D13 | 120 `region` blocks in 1,694 lines | ceremony | no |
| D14 | the granted verdict lowers to nothing | [PAR-3] | no |

**Thirteen of fourteen are not proof-obligation defects at all.** The record
says so itself: "Zero `claim` statements in 1,694 lines. Every subscript, every
`%` and `/`, every system range call was discharged by ordinary `if` branches
and `len()` rebinding. The proof obligations — the part of this language
everyone expects to be the wall — were not the wall."

**D12 is the one that touches this area, and it is instructive.** The writer
re-bound `len(b)` after every callee write because they did not know whether the
length fact survived. It does: [ENT-5] support (3036–3049) puts the root binding
of `P` in a `len(P)` fact's support but not `P`'s element storage, so "an element
write never kills a length fact", and `DESIGN.md` §7.3 proposes
`accept-clm1-length-survives-a-callee-element-write.wf` to pin exactly that.
Bounds already hold `len(P) = n`; a publisher already establishes it (S6); what
was missing was **that the guarantee was published to the writer**. That is
[DIAG-1]/C-IV, `DESIGN.md`'s fourth consequence, and it is the one place where
this design's teaching channel would have changed what a blind writer wrote.
Note what it is *not*: not a claim, not a refusal, not a missing fact. It cost
34 redundant `let` bindings, not one runtime check.

### 4.2 0100 — the second and third writers (W1–W11, B1–B4, G1–G2)

Seventeen items. Two touch this area, and both are diagnostic rendering:

- **W8 — [FN-8] renders its goal in source terms.** The call-requirement goal
  was printed as a structural dump (`Boolean(And)<types=[], consts=[]>(Integer {
  operation: Greater, … }`) where [OP-4] and [SYS-8] already printed source
  terms; it now prints `band(igt(value, 0_u64), ilt(value, 10_u64))`. A writer
  could not read the obligation they had failed to discharge.
- **B4 — [SYS-8]'s residual names the caller's buffer.** The second range
  conjunct was rendered against the system operation's *declared parameter*, so
  `wide <= len(buffer)` did not say which of the caller's buffers was meant; it
  now prints `wide <= len(header)`.

Neither is dissolved by bounds or by a publisher — the facts were expressible
and derivable, and both are already fixed. Both are evidence for the same thing
D12 is evidence for: **in this area the measured writer cost has been legibility
of the obligation, not reach of the prover.** 0100 §12 also records the
acceptance-invariance of that batch — `files=630 exit-status-differences=0
ir-differences=0` — so nothing in it moved a verdict.

### 4.3 What the doctrine check does and does not license

It does not license "the residue is empty" — §2.4 lists six members and four of
them are in `wfgrep.wf`, `percent_decode.wf` and `ipv4_checksum.wf`, programs
someone wanted. What it licenses is narrower and more useful: **no writer in either recorded blind
trial was stopped by the fact language, by claim locality, or by an
undischargeable obligation.** They were stopped by permissions, borrows,
diagnostics, and a missing standard input. `DESIGN.md` §8's F8 predicts the
repeat trial will find loop induction and two-term vocabulary as the walls and
**not** locality; this census cannot confirm that prediction, but it can say
that the last two trials found neither — they found nothing in this area at all
except two unreadable diagnostics and one unpublished guarantee.

---

## 5. Limitations, stated plainly

1. **The site counts in §1.2 are a text census, not the compiler's ledger.** The
   compiler computes `ObligationOutcome` per site, but `semantic` is not a
   public module of the `whitefoot` crate and `whitefootc` has no obligation
   ledger flag (it has `--par-ledger` and `--stack-ledger` only), so there is no
   way to enumerate obligations from outside without modifying the tree, which
   this measurement was not allowed to do. Known error modes of the regexes:
   subscripts are counted as `[A-Za-z0-9_)\]]\[`, which correctly excludes the
   `=[` array-literal form and rule citations in prose (verified: zero matches
   of `[A-Za-z0-9_)\]]\[[A-Z]{2,}[-0-9]` anywhere), but a subscript written
   with whitespace before the bracket would be missed; exact infix requires
   spaces on both sides, which is the corpus's uniform style but is not
   guaranteed by the grammar; a `for` statement's compiler-owned capture terms
   are not obligations and are not counted.
2. **The differential's effect-row repair is a mutation the compiler guided but
   did not bless.** Deleting a claim from a `traps` function over-declares its
   row; the loop applies the [EFF-2] payload's own `extra:` list, cascading to
   callers. Two claims defeated the loop and were finished by hand
   (`raw_deflate_dynamic_decode.wf:32`, `x-base64-rfc-vectors-run.wf:16`) and
   both are reported at their hand-verified values.
3. **`raw_deflate_*` is one four-file module.** Compiled singly, four of the 25
   programs report [TYPE-5]/[OP-1]; compiled as a unit
   (`raw_deflate.wf raw_deflate_dynamic.wf raw_deflate_dynamic_decode.wf
   raw_deflate_boundary.wf`) the module compiles. `raw_deflate_vectors.wf`
   declares a second `main` and is a separate program.
4. **Bucket P/B/R assignment is a reading, not a measurement.** No machine
   decided it. It was made by reading the source of every candidate and of all
   18 real-program claims, cross-checked by the prose scan in §2.4. The call I
   would re-check first is T4 against its sibling at `wfgrep.wf:556`: the two
   claims sit three lines apart in the same loop and I put them in different
   buckets, on the ground that :556 needs only `tail <= bounded_available` and
   T4 needs `bounded_scan + moved < bounded_available`.
5. **m was traced by hand for 10 claims and bounded mechanically for 125.** The
   mechanical bound is sound (it over-counts), so the "no claim has m ≥ 2"
   result does not depend on the hand trace; the hand trace only moves claims
   from 1 to 0.
6. **`tests/codegen/` contributes nothing.** Its 15 claims are in 13 files the
   current compiler rejects (§6.1), so they are neither writer evidence nor a
   discharge population. They are counted in the 135 because they are `claim`
   statements in the tree, and excluded from every discharge table.
7. **Nothing here was run under `make check`.** This is a measurement document;
   it adds no test, no gate wiring, and no derived material.

---

## 6. What surprised me

### 6.1 The codegen bounds corpus does not compile

All 13 claim-bearing fixtures under `tests/codegen/cases/bounds/` are rejected
by the current compiler, every one citing [CLM-1]. The twelve `masked-index`
drift oracles claim `ieq(value, N)` where `value` is a *direct user-call result*
— publisher 2's subject matter, refused by [CLM-1] since long before v0.39 — and
`output-capacity-lockstep/p08` claims `False()`, which fails CLM-1 fact-free
formation.

```sh
"$W" --emit-llvm -o /dev/null tests/codegen/cases/bounds/masked-index/p01-mask3-table4.wf
# whitefootc: Semantics/Source [CLM-1]: …
```

`TERRAIN.md` §2 describes these as "fixture instrumentation, not writer
evidence", which is right, but understates it: they are *stale* fixture
instrumentation, in a corpus whose own README says "preserved future-facing
corpus; not an active gate". `DESIGN.md` §7.5 plans to "re-cut" them under the
image closure. The measurement says the re-cut is not an adjustment to a working
fixture set — it is a rewrite of thirteen files that do not currently compile,
and the estimate should say so.

### 6.2 Every claim in every compiling file is load-bearing

Zero of the 83 claims in accepted sources is redundant; deleting any one of them
produces a named undischarged obligation. Given that [CLM-2] rejects a redundant
claim outright, this is the expected result — but it is the first time it has
been measured end to end, and it means `DESIGN.md` §7.1's predicted verdict
moves (three `accept-clm1-*` cases becoming CLM-2 duplicate-publication
rejections once `%` publishes an image) are moves of *currently load-bearing*
claims, not cleanup of dead ones. Each will need its accept sibling written, as
§7.1 already says.

### 6.3 `DESIGN.md` §7.4 tokens the two-length claims wrongly

§7.4 assigns the eight "ENT does not correlate the two borrowed column lengths /
the two nominal field lengths" claims the token `vocabulary`, with the reason "a
relation between two independent lengths is not two-term". It is two-term.
[ENT-2] 2870(b) makes `len(P)` a term and 2901 makes `t1 - t2 <= c` an atomic
fact, so `len(left) - len(right) <= 0` together with `len(right) - len(left) <=
0` is exactly the equality those claims want, in the vocabulary the language
already has. What is missing is a *publisher*: in
`x-struct-of-buffers-checksum-run.wf` both lengths come from `buffer_new(6_u64,
…)` and S6 could publish both, and in `x-buffer-borrowed-columns-run.wf` the
equality is a caller fact that a `requires` could publish. All eight are bucket
P. The token should be `flow` (the loop range is what actually blocks them, and
each gap text says so first) with the length correlation supplied by S6 or by a
contract.

### 6.4 The `boundary` token covers two different situations, and only one is a refusal

Seventeen claims carry the `boundary` token, and the baseline verdicts split
them cleanly in two:

- **9 in files the compiler rejects** — the deliberate refusal fixtures. Six
  cite [CLM-1] (`clm1-neg-user-result-claim-locality`,
  `clm1-neg-system-result-claim-locality`, and the four `reject-clm1-claim-on-*`
  selection cases) and three cite [PRV-2] (`prv2-neg-{recursive,mutual}-demand`,
  `prv2-neg-direct-system-result`). These are the gate working.
- **8 in files the compiler accepts** — and these are not refusals at all.
  `fn5-pos-match-dispatch:9,11`, `prog1-pos-closed-unit:4`,
  `op2-pos-division-claim-backstop:3`, `ent3-pos-band-check-decomposition:6`,
  `x-fn-cross-fn-call-chain:3` and `x-child-reborrow-run:8,19` all claim a fact
  about a *parameter* or about a borrowed buffer's length — `Local` under
  3222 — whose truth is a fact of the **caller**. [CLM-1] admits them; they are
  load-bearing; and every one of them is a two-term fact that a `requires`
  clause on the same function would publish. They are the corpus's measure of
  what an absent contract costs: eight retained runtime checks standing in for
  eight `requires` lines.

The distinction matters for `DESIGN.md` §3.8's diagnostic. A writer whose claim
carries `boundary` today is in one of these two situations and [DIAG-1] 1859's
two fixed restructurings address only the first. The eight accepted ones would
never see a diagnostic at all, because nothing is wrong — which is exactly why
nobody has written the `requires`.

On F9: the corpus contains **zero** programs where a writer wanted a claim over
a value a callee or the world produced and had no repair available. F9 is
unrefuted here, and the census can add that it is unrefuted by 682 source files
rather than by inattention.

### 6.5 The claim construct is a subscript construct

All 18 real-program claims discharge SubscriptBounds. Adding the accepting
conformance cases, 55 of 83 do. Zero claims anywhere discharge AllocationFit or
SystemRange, and one discharges an [FN-9] return proof. If `DESIGN.md` §5.3's
image column has to be prioritised inside batch B1, the rows that feed a
subscript index — `%`, `/`, `imin`/`imax`, `iand`, and the loop head — are the
whole of the measured demand.

### 6.6 The one number I expected to be larger

Real programs write 15 exact integer operations and 339 total-form ones. The
IntegerDomain family — the one with the most elaborate normalization machinery
in [ENT-6] 3133–3153 — is attached to 15 sites in 6,656 lines of real program,
and discharges all 15 without a claim. Every one of the 25 IntegerDomain claims
in the tree is in a conformance case written to exercise the rule.
