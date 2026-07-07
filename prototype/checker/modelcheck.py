"""M2 model checker (FR-inherited method, budget-scaled).

Generates random small programs over the checker's covered fragment, runs the
static checker, and executes ACCEPTED programs under the independent oracle.
  - oracle violation on an accepted program  => CHECKER SOUNDNESS BUG (the prize)
  - also reports the over-rejection sample: rejected programs that run
    oracle-clean (OWN-8's reject-when-unsure cost, measured)
Deterministic seed: reproducible per META discipline.
"""
import random, sys
from checker import check_fn, CheckError
from oracle import run as oracle_run, OracleViolation

random.seed(20260707)

def gen_program(rng, depth=0):
    stmts, names = [], []
    caller = [f"cr{i}" for i in range(rng.randint(0, 2))]
    regions = list(caller)
    params = []
    for i in range(rng.randint(0, 2)):
        pn = f"p{i}"
        if caller and rng.random() < 0.5:
            params.append({"name": pn, "mode": {"kind": "ref",
                          "region": rng.choice(caller), "uniq": rng.random() < 0.5}})
        else:
            params.append({"name": pn, "mode": {"kind": "own"}})
        names.append(pn)
    def gen_block(depth, stmts, k):
        for _ in range(k):
            c = rng.random()
            if c < 0.30 or not names:
                n = f"x{len(names)}"
                stmts.append({"kind": "let", "name": n,
                              "mode": {"kind": "own"}, "init": {"kind": "lit"}})
                names.append(n)
            elif c < 0.45 and regions:
                n = f"b{len(names)}"; tgt = rng.choice(names); r = rng.choice(regions)
                stmts.append({"kind": "let", "name": n,
                              "mode": {"kind": "ref", "region": r, "uniq": rng.random() < 0.5},
                              "init": {"kind": "borrow", "region": r,
                                       "uniq": rng.random() < 0.5,
                                       "place": {"base": tgt, "path": []}}})
                names.append(n)
            elif c < 0.60:
                stmts.append({"kind": "expr",
                              "expr": {"kind": "use",
                                       "place": {"base": rng.choice(names), "path": []}}})
            elif c < 0.72:
                stmts.append({"kind": "let", "name": f"m{len(names)}",
                              "mode": {"kind": "own"},
                              "init": {"kind": "move",
                                       "place": {"base": rng.choice(names), "path": []}}})
                names.append(f"m{len(names)-1}")
            elif c < 0.82:
                stmts.append({"kind": "set",
                              "place": {"base": rng.choice(names), "path": []},
                              "expr": {"kind": "lit"}})
            elif c < 0.90 and depth < 2 and names:
                arms = []
                stmt = {"kind": "match", "scrut": {"kind": "use",
                        "place": {"base": rng.choice(names), "path": []}}, "arms": arms}
                stmts.append(stmt)
                outer = list(names)
                for _ in range(2):
                    body = []
                    arms.append({"binders": [], "body": body})
                    gen_block(depth + 1, body, rng.randint(1, 2))
                    del names[len(outer):]      # arm-locals are arm-scoped
            elif c < 0.94 and depth < 2:
                r = f"r{len(regions)}"; regions.append(r)
                inner = []
                stmts.append({"kind": "region", "name": r, "body": inner})
                gen_block(depth + 1, inner, rng.randint(1, 3))
                regions.pop()
            elif c < 0.985 and names and regions:
                nargs = rng.randint(1, 3); args = []
                for _ in range(nargs):
                    if rng.random() < 0.6:
                        args.append({"kind": "borrow", "region": rng.choice(regions),
                                     "uniq": rng.random() < 0.5,
                                     "place": {"base": rng.choice(names), "path": []}})
                    else:
                        args.append({"kind": "use",
                                     "place": {"base": rng.choice(names), "path": []}})
                stmts.append({"kind": "expr", "expr": {"kind": "call", "args": args}})
            else:
                stmts.append({"kind": "expr", "expr": {"kind": "lit"}})
    gen_block(0, stmts, rng.randint(2, 6))
    return {"kind": "fn", "name": "t", "params": params, "regions": caller, "body": stmts}

def main(n=5000):
    rng = random.Random(20260707)
    accepted = rejected = sound_bugs = over_reject_clean = 0
    by_rule = {}
    for i in range(n):
        prog = gen_program(rng)
        try:
            check_fn(prog)
            accepted += 1
            try:
                oracle_run(prog)
            except OracleViolation as v:
                sound_bugs += 1
                print(f"SOUNDNESS BUG #{sound_bugs} (program {i}): {v}")
                print(prog)
        except CheckError as ce:
            rejected += 1
            try:
                oracle_run(prog)
                over_reject_clean += 1
                key = ce.rule + ("/through-borrow" if "non-owning" in str(ce) else "")
                by_rule[key] = by_rule.get(key, 0) + 1
            except OracleViolation:
                pass
    print(f"programs={n} accepted={accepted} rejected={rejected}")
    print(f"SOUNDNESS violations on accepted: {sound_bugs}  <-- must be 0")
    print(f"rejected-but-oracle-clean: {over_reject_clean} "
          f"({100.0*over_reject_clean/max(rejected,1):.1f}% of rejected), by rule:")
    STATIC_ONLY = {"OWN-5", "OWN-12", "OWN-1/through-borrow"}  # dynamically invisible by design
    for r, n in sorted(by_rule.items(), key=lambda kv: -kv[1]):
        tag = " [static-only rule: expected]" if r in STATIC_ONLY else ""
        print(f"    {r}: {n}{tag}")
    true_over = sum(n for r, n in by_rule.items() if r not in STATIC_ONLY)
    print(f"TRUE over-rejection (excl. static-only exclusivity rules): "
          f"{true_over} ({100.0*true_over/max(rejected,1):.1f}% of rejected)")
    return sound_bugs

if __name__ == "__main__":
    sys.exit(1 if main(int(sys.argv[1]) if len(sys.argv) > 1 else 5000) else 0)
