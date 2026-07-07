"""M2 differential oracle: an INDEPENDENT tiny interpreter over the checker AST.

Executes a (straight-line + regions) program dynamically and raises
OracleViolation on true memory-safety events: use/move/write of a moved or dead
root, and any access through a dangling borrow (root moved or dead). The oracle
deliberately shares no code with checker.py — its value is disagreement.

FR-inherited method (K003): accepted-by-checker programs must run oracle-clean;
an oracle violation on an accepted program is a checker soundness bug.
"""

class OracleViolation(Exception):
    pass


class _NeedChoice(Exception):
    def __init__(self, k): self.k = k


def _interp(fn, choices):
    env = {}                      # name -> cell
    region_stack = []             # list of lists of names created in region

    def cell(mode, target=None, region=None):
        return {"mode": mode, "state": "live", "target": target, "region": region}

    def note(name):
        if region_stack:
            region_stack[-1].append(name)

    def root_of(name, seen=None):
        seen = seen or set()
        if name in seen:
            raise OracleViolation(f"borrow cycle at {name}")
        seen.add(name)
        c = env.get(name)
        if c is None or c["state"] == "dead":
            raise OracleViolation(f"access through dead binding {name}")
        if c["mode"] == "ref":
            return root_of(c["target"], seen) if c["target"] else name
        return name

    def access(name, kind):      # kind: read | write | move
        c = env.get(name)
        if c is None:
            raise OracleViolation(f"unknown binding {name}")
        if c["state"] == "dead":
            raise OracleViolation(f"{kind} of dead binding {name}")
        r = root_of(name)
        rc = env[r]
        if rc["state"] == "moved":
            raise OracleViolation(f"{kind} reaches moved root {r} via {name}")
        if rc["state"] == "dead":
            raise OracleViolation(f"dangling: {kind} reaches dead root {r} via {name}")
        if kind == "move":
            rc["state"] = "moved"

    def eval_expr(e, binding_hint=None):
        k = e["kind"]
        if k == "lit":
            return None
        if k == "use":
            access(e["place"]["base"], "read"); return None
        if k == "move":
            access(e["place"]["base"], "move"); return None
        if k == "borrow":
            access(e["place"]["base"], "read")
            return {"target": e["place"]["base"], "region": e["region"]}
        if k == "call":
            for a in e["args"]:
                eval_expr(a)
            return None
        return None

    def exec_block(stmts):
        for s in stmts:
            k = s["kind"]
            if k == "let":
                b = eval_expr(s["init"], s["name"])
                if s["mode"]["kind"] == "ref":
                    env[s["name"]] = cell("ref",
                                          target=b["target"] if b else None,
                                          region=s["mode"].get("region"))
                else:
                    env[s["name"]] = cell("own")
                note(s["name"])
            elif k == "set":
                access(s["place"]["base"], "write"); eval_expr(s["expr"])
            elif k == "expr":
                eval_expr(s["expr"])
            elif k == "return":
                eval_expr(s["expr"]); return True
            elif k == "region":
                region_stack.append([])
                done = exec_block(s["body"])
                for n in region_stack.pop():
                    env[n]["state"] = "dead"
                if done:
                    return True
            elif k == "match":
                sc = s["scrut"]; bmode = "own"; btgt = None
                if sc["kind"] == "use":
                    base = sc["place"]["base"]
                    if env.get(base, {}).get("mode") == "own":
                        access(base, "move")            # [OWN-13] own scrutinee moves
                    else:
                        access(base, "read"); bmode = "ref"; btgt = base
                else:
                    eval_expr(sc)
                if not choices:
                    raise _NeedChoice(len(s["arms"]))
                arm = s["arms"][choices.pop(0)]
                for bn in arm.get("binders", []):
                    env[bn] = cell(bmode, target=btgt); note(bn)
                if exec_block(arm["body"]):
                    return True
            elif k == "loop":
                # bounded: execute body twice to exercise the back-edge
                for _ in range(2):
                    done = exec_block(s["body"])
                    if done:
                        return True
        return False

    for p in fn.get("params", []):
        env[p["name"]] = cell("own" if p["mode"]["kind"] == "own" else "ref",
                              region=p["mode"].get("region"))
    exec_block(fn["body"])
    # caller horizon: the frame dies, but caller-region borrows survive the
    # return — touching them now surfaces post-return dangles (OWN-10's target)
    caller = set(fn.get("regions", []))
    for n, c in env.items():
        if c["mode"] == "own":
            c["state"] = "dead"
    for n, c in list(env.items()):
        if c["mode"] == "ref" and c.get("region") in caller and c["state"] == "live":
            access(n, "read")


def run(fn, _prefix=None, _budget=[0]):
    """Explore all match-arm paths (bounded); violation on ANY path raises."""
    stack = [[]]
    seen = 0
    while stack and seen < 256:
        pre = stack.pop()
        try:
            _interp(fn, list(pre)); seen += 1
        except _NeedChoice as nc:
            for i in range(nc.k):
                stack.append(pre + [i])
