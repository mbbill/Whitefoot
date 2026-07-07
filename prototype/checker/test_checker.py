"""Positive/negative tests for the checker-core prototype (D1a gate evidence).

Each negative test asserts BOTH rejection and the exact kernel-spec-v0 rule ID,
matching the DIAG-1 contract (diagnostics cite one rule ID).
"""

import unittest
from checker import check_fn, CheckError


def own():
    return {"kind": "own"}


def ref(region, uniq=False):
    return {"kind": "ref", "region": region, "uniq": uniq}


def place(base, *path):
    return {"base": base, "path": list(path)}


def fn(body, params=None, regions=None):
    return {"kind": "fn", "name": "t", "params": params or [],
            "regions": regions or [], "body": body}


class Negative(unittest.TestCase):
    def expect(self, rule, f):
        with self.assertRaises(CheckError) as cm:
            check_fn(f)
        self.assertEqual(cm.exception.rule, rule, str(cm.exception))

    def test_use_after_move(self):
        self.expect("OWN-1", fn([
            {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
            {"kind": "let", "name": "y", "mode": own(),
             "init": {"kind": "move", "place": place("x")}},
            {"kind": "expr", "expr": {"kind": "use", "place": place("x")}},
        ]))

    def test_double_mutable_borrow(self):
        self.expect("OWN-5", fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "let", "name": "p", "mode": ref("a", True),
                 "init": {"kind": "borrow", "region": "a", "uniq": True,
                          "place": place("x")}},
                {"kind": "let", "name": "q", "mode": ref("a", True),
                 "init": {"kind": "borrow", "region": "a", "uniq": True,
                          "place": place("x")}},
            ]},
        ]))

    def test_shared_then_mutable_overlap(self):
        self.expect("OWN-5", fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "let", "name": "p", "mode": ref("a"),
                 "init": {"kind": "borrow", "region": "a", "uniq": False,
                          "place": place("x", "f")}},
                {"kind": "let", "name": "q", "mode": ref("a", True),
                 "init": {"kind": "borrow", "region": "a", "uniq": True,
                          "place": place("x")}},          # overlaps x.f [OWN-7]
            ]},
        ]))

    def test_move_while_borrowed(self):
        self.expect("OWN-5", fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "let", "name": "p", "mode": ref("a"),
                 "init": {"kind": "borrow", "region": "a", "uniq": False,
                          "place": place("x")}},
                {"kind": "let", "name": "y", "mode": own(),
                 "init": {"kind": "move", "place": place("x")}},
            ]},
        ]))

    def test_escape_inner_region_to_outer(self):
        self.expect("OWN-4", fn([
            {"kind": "region", "name": "outer", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "region", "name": "inner", "body": [
                    {"kind": "let", "name": "p", "mode": ref("outer"),
                     "init": {"kind": "borrow", "region": "inner", "uniq": False,
                              "place": place("x")}},     # inner does not outlive outer
                ]},
            ]},
        ]))

    def test_return_local_region_borrow(self):
        self.expect("OWN-4", fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "return",
                 "expr": {"kind": "borrow", "region": "a", "uniq": False,
                          "place": place("x")}},
            ]},
        ]))

    def test_write_through_shared_borrow(self):
        self.expect("OWN-5", fn([
            {"kind": "set", "place": place("p"), "expr": {"kind": "lit"}},
        ], params=[{"name": "p", "mode": ref("r0", False)}], regions=["r0"]))

    def test_read_while_mutably_borrowed(self):
        self.expect("OWN-5", fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "let", "name": "p", "mode": ref("a", True),
                 "init": {"kind": "borrow", "region": "a", "uniq": True,
                          "place": place("x")}},
                {"kind": "expr", "expr": {"kind": "use", "place": place("x")}},
            ]},
        ]))

    def test_move_through_borrow_binding(self):
        self.expect("OWN-1", fn([
            {"kind": "let", "name": "y", "mode": own(),
             "init": {"kind": "move", "place": place("p")}},
        ], params=[{"name": "p", "mode": ref("r0", True)}], regions=["r0"]))

    def test_dangling_return_of_own_param(self):
        # the critique's counterexample: fn f(x: own T) ['r0] { return &r0 x; }
        self.expect("OWN-10", fn([
            {"kind": "return",
             "expr": {"kind": "borrow", "region": "r0", "uniq": False,
                      "place": place("x")}},
        ], params=[{"name": "x", "mode": own()}], regions=["r0"]))

    def test_dangling_local_into_caller_region(self):
        self.expect("OWN-10", fn([
            {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
            {"kind": "expr",
             "expr": {"kind": "borrow", "region": "r0", "uniq": False,
                      "place": place("x")}},
        ], regions=["r0"]))

    def test_holder_rooted_alias_of_mut_borrow(self):
        # let p = &mut 'a x; &'a p.f resolves to x.f overlapping the &mut of x
        self.expect("OWN-5", fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "let", "name": "p", "mode": ref("a", True),
                 "init": {"kind": "borrow", "region": "a", "uniq": True,
                          "place": place("x")}},
                {"kind": "let", "name": "q", "mode": ref("a"),
                 "init": {"kind": "borrow", "region": "a", "uniq": False,
                          "place": place("p", "f")}},
            ]},
        ]))

    def test_loop_borrow_of_outer_region(self):
        self.expect("OWN-11", fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "loop", "label": "l", "body": [
                    {"kind": "expr",
                     "expr": {"kind": "borrow", "region": "a", "uniq": False,
                              "place": place("x")}},
                ]},
            ]},
        ]))

    def test_loop_move_of_outer_binding(self):
        self.expect("OWN-11", fn([
            {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
            {"kind": "loop", "label": "l", "body": [
                {"kind": "let", "name": "y", "mode": own(),
                 "init": {"kind": "move", "place": place("x")}},
            ]},
        ]))

    def test_call_two_uniq_args_alias(self):
        self.expect("OWN-12", fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "expr", "expr": {"kind": "call", "args": [
                    {"kind": "borrow", "region": "a", "uniq": True, "place": place("x")},
                    {"kind": "borrow", "region": "a", "uniq": True, "place": place("x")},
                ]}},
            ]},
        ]))

    def test_call_shared_plus_uniq_overlap(self):
        self.expect("OWN-12", fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "expr", "expr": {"kind": "call", "args": [
                    {"kind": "borrow", "region": "a", "uniq": False, "place": place("x", "f")},
                    {"kind": "borrow", "region": "a", "uniq": True, "place": place("x")},
                ]}},
            ]},
        ]))

    def test_unknown_region(self):
        self.expect("OWN-3", fn([
            {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
            {"kind": "expr",
             "expr": {"kind": "borrow", "region": "nowhere", "uniq": False,
                      "place": place("x")}},
        ]))


class Positive(unittest.TestCase):
    def ok(self, f):
        self.assertIsNone(check_fn(f))

    def test_sequential_disjoint_borrows_after_region_end(self):
        self.ok(fn([
            {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "p", "mode": ref("a", True),
                 "init": {"kind": "borrow", "region": "a", "uniq": True,
                          "place": place("x")}},
                {"kind": "set", "place": place("p"), "expr": {"kind": "lit"}},
            ]},
            {"kind": "region", "name": "b", "body": [
                {"kind": "let", "name": "q", "mode": ref("b", True),
                 "init": {"kind": "borrow", "region": "b", "uniq": True,
                          "place": place("x")}},
            ]},
        ]))

    def test_disjoint_field_borrows(self):
        self.ok(fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "let", "name": "p", "mode": ref("a"),
                 "init": {"kind": "borrow", "region": "a", "uniq": False,
                          "place": place("x", "f")}},
                {"kind": "let", "name": "q", "mode": ref("a"),
                 "init": {"kind": "borrow", "region": "a", "uniq": False,
                          "place": place("x", "g")}},   # f and g disjoint [OWN-7]
            ]},
        ]))

    def test_two_shared_borrows_same_place(self):
        self.ok(fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "let", "name": "p", "mode": ref("a"),
                 "init": {"kind": "borrow", "region": "a", "uniq": False,
                          "place": place("x")}},
                {"kind": "let", "name": "q", "mode": ref("a"),
                 "init": {"kind": "borrow", "region": "a", "uniq": False,
                          "place": place("x")}},
                {"kind": "expr", "expr": {"kind": "use", "place": place("x")}},
            ]},
        ]))

    def test_return_reborrow_of_caller_borrow(self):
        # sound: re-borrowing through a caller-region borrow into the same region
        self.ok(fn([
            {"kind": "return",
             "expr": {"kind": "borrow", "region": "r0", "uniq": False,
                      "place": place("p")}},
        ], params=[{"name": "p", "mode": ref("r0", False)}], regions=["r0"]))

    def test_write_through_mut_borrow_holder(self):
        self.ok(fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "let", "name": "p", "mode": ref("a", True),
                 "init": {"kind": "borrow", "region": "a", "uniq": True,
                          "place": place("x")}},
                {"kind": "set", "place": place("p"), "expr": {"kind": "lit"}},
            ]},
        ]))

    def test_loop_region_per_iteration_ok(self):
        self.ok(fn([
            {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
            {"kind": "loop", "label": "l", "body": [
                {"kind": "region", "name": "it", "body": [
                    {"kind": "let", "name": "p", "mode": ref("it", True),
                     "init": {"kind": "borrow", "region": "it", "uniq": True,
                              "place": place("x")}},
                    {"kind": "set", "place": place("p"), "expr": {"kind": "lit"}},
                ]},
            ]},
        ]))

    def test_call_two_shared_args_ok(self):
        self.ok(fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "expr", "expr": {"kind": "call", "args": [
                    {"kind": "borrow", "region": "a", "uniq": False, "place": place("x")},
                    {"kind": "borrow", "region": "a", "uniq": False, "place": place("x")},
                ]}},
            ]},
        ]))

    def test_call_disjoint_uniq_fields_ok(self):
        self.ok(fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "expr", "expr": {"kind": "call", "args": [
                    {"kind": "borrow", "region": "a", "uniq": True, "place": place("x", "f")},
                    {"kind": "borrow", "region": "a", "uniq": True, "place": place("x", "g")},
                ]}},
            ]},
        ]))

    def test_match_own_scrutinee_moves(self):
        with self.assertRaises(CheckError) as cm:
            check_fn(fn([
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "match", "scrut": {"kind": "use", "place": place("x")},
                 "arms": [{"binders": [], "body": []}]},
                {"kind": "expr", "expr": {"kind": "use", "place": place("x")}},
            ]))
        self.assertEqual(cm.exception.rule, "OWN-1")

    def test_match_uniq_binder_conflicts_with_root_borrow(self):
        with self.assertRaises(CheckError) as cm:
            check_fn(fn([
                {"kind": "match", "scrut": {"kind": "use", "place": place("p")},
                 "arms": [{"binders": ["v"], "body": [
                     {"kind": "let", "name": "q", "mode": ref("r0", True),
                      "init": {"kind": "borrow", "region": "r0", "uniq": True,
                               "place": place("p")}}]}]},
            ], params=[{"name": "p", "mode": ref("r0", True)}], regions=["r0"]))
        self.assertEqual(cm.exception.rule, "OWN-5")

    def test_match_ref_scrutinee_stays_live(self):
        self.assertIsNone(check_fn(fn([
            {"kind": "match", "scrut": {"kind": "use", "place": place("p")},
             "arms": [{"binders": ["v"], "body": [
                 {"kind": "expr", "expr": {"kind": "use", "place": place("v")}}]}]},
            {"kind": "expr", "expr": {"kind": "use", "place": place("p")}},
        ], params=[{"name": "p", "mode": ref("r0", False)}], regions=["r0"])))

    def test_match_arm_isolation_uniq_in_both_arms(self):
        # sequential approximation used to false-reject this
        self.ok(fn([
            {"kind": "region", "name": "a", "body": [
                {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
                {"kind": "match", "scrut": {"kind": "lit"}, "arms": [
                    {"binders": [], "body": [
                        {"kind": "let", "name": "p", "mode": ref("a", True),
                         "init": {"kind": "borrow", "region": "a", "uniq": True,
                                  "place": place("x")}}]},
                    {"binders": [], "body": [
                        {"kind": "let", "name": "q", "mode": ref("a", True),
                         "init": {"kind": "borrow", "region": "a", "uniq": True,
                                  "place": place("x")}}]},
                ]},
            ]},
        ]))

    def test_match_join_move_in_one_arm(self):
        with self.assertRaises(CheckError) as cm:
            check_fn(fn([
            {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
            {"kind": "match", "scrut": {"kind": "lit"}, "arms": [
                {"binders": [], "body": [
                    {"kind": "let", "name": "y", "mode": own(),
                     "init": {"kind": "move", "place": place("x")}}]},
                {"binders": [], "body": []},
            ]},
            {"kind": "expr", "expr": {"kind": "use", "place": place("x")}},
        ]))
        self.assertEqual(cm.exception.rule, "OWN-1")

    def test_move_then_rebind_fresh(self):
        self.ok(fn([
            {"kind": "let", "name": "x", "mode": own(), "init": {"kind": "lit"}},
            {"kind": "let", "name": "y", "mode": own(),
             "init": {"kind": "move", "place": place("x")}},
            {"kind": "expr", "expr": {"kind": "use", "place": place("y")}},
        ]))


if __name__ == "__main__":
    unittest.main(verbosity=2)
