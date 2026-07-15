#!/usr/bin/env python3
"""Fail-closed verification and mutation tests for performance protocol v5."""

from __future__ import annotations

import copy
import csv
import hashlib
import json
import subprocess
from collections import Counter
from fractions import Fraction
from functools import lru_cache
from math import comb
from pathlib import Path
from typing import Any, Callable

import dense_performance_registry as registry


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[3]


class VerificationError(RuntimeError):
    """A protocol invariant failed."""


def fail(message: str) -> None:
    raise VerificationError(message)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def read_tsv(key: str) -> list[dict[str, str]]:
    path = HERE / registry.OUTPUTS[key]
    with path.open(encoding="utf-8", newline="") as handle:
        rows = list(csv.DictReader(handle, delimiter="\t"))
    if not rows or list(rows[0]) != registry.SCHEMAS[key]:
        fail(f"missing or stale schema: {path.name}")
    return rows


def read_jsonl(key: str) -> list[dict[str, Any]]:
    path = HERE / registry.OUTPUTS[key]
    with path.open(encoding="ascii") as handle:
        rows = [json.loads(line) for line in handle if line.strip()]
    if not rows:
        fail(f"empty JSONL artifact: {path.name}")
    return rows


def read_json(key: str) -> dict[str, Any]:
    return json.loads(
        (HERE / registry.OUTPUTS[key]).read_text(encoding="ascii")
    )


def git_bytes(commit: str, path: str) -> bytes:
    result = subprocess.run(
        ["git", "show", f"{commit}:{path}"],
        cwd=REPO,
        check=False,
        capture_output=True,
    )
    if result.returncode != 0:
        fail(f"cannot read protected source {commit}:{path}")
    return result.stdout


def require_unique(
    rows: list[dict[str, str]], field: str, label: str
) -> None:
    values = [row[field] for row in rows]
    if len(values) != len(set(values)):
        fail(f"duplicate {label}")


@lru_cache(maxsize=None)
def scheduled_mixture_tail_table(n: int) -> tuple[Fraction, ...]:
    if n < 0:
        fail("scheduled-mixture sample size is negative")
    denominator = 2 ** n
    suffix = 0
    tails = [Fraction(1, 1)] * (n + 1)
    for s in range(n, -1, -1):
        suffix += comb(n, s)
        if 2 * s > n:
            tails[s] = Fraction(suffix, denominator)
    return tuple(tails)


def scheduled_mixture_tail(n: int, s: int) -> Fraction:
    if not (0 <= s <= n):
        fail("scheduled-mixture success count is out of range")
    return scheduled_mixture_tail_table(n)[s]


def critical_success_count(n: int, alpha: Fraction) -> int:
    if alpha >= 1:
        return 0
    if alpha < 0:
        fail("registered alpha is negative")
    tails = scheduled_mixture_tail_table(n)
    low = n // 2 + 1
    high = n + 1
    while low < high:
        middle = (low + high) // 2
        if tails[middle] <= alpha:
            high = middle
        else:
            low = middle + 1
    if low <= n:
        return low
    fail("registered alpha is unreachable")
    raise AssertionError("unreachable")


@lru_cache(maxsize=None)
def verify_hoeffding_extrema() -> None:
    for n in (60, 90, 120):
        expected = scheduled_mixture_tail_table(n)
        attained = [False] * (n + 1)
        half = n // 2
        for ones in range(half + 1):
            remaining_mean = half - ones
            first_equal_count = 0 if remaining_mean == 0 else remaining_mean
            for equal_count in range(first_equal_count, n - ones + 1):
                if equal_count == 0:
                    denominator = 1
                    coefficients = [1]
                else:
                    denominator = equal_count ** equal_count
                    coefficients = [
                        comb(equal_count, k)
                        * remaining_mean ** k
                        * (equal_count - remaining_mean) ** (equal_count - k)
                        for k in range(equal_count + 1)
                    ]
                suffix = 0
                tails = [0] * (len(coefficients) + 1)
                for k in range(len(coefficients) - 1, -1, -1):
                    suffix += coefficients[k]
                    tails[k] = suffix
                for s in range(n + 1):
                    offset = s - ones
                    if offset <= 0:
                        numerator = denominator
                    elif offset >= len(tails):
                        numerator = 0
                    else:
                        numerator = tails[offset]
                    registered = expected[s]
                    left = numerator * registered.denominator
                    right = registered.numerator * denominator
                    if left > right:
                        fail(
                            "Hoeffding extremal exceeds the registered tail "
                            f"for n={n}, a={ones}, b={equal_count}, s={s}"
                        )
                    if left == right:
                        attained[s] = True
        if not all(attained):
            fail(f"Hoeffding extremal identity changed for n={n}")


@lru_cache(maxsize=None)
def verify_holm_cutoff_domain(family_sizes: tuple[int, ...]) -> None:
    if family_sizes != (8120, 8140, 16240, 16280):
        fail("actual benefit family-size domain changed")
    if {
        n: critical_success_count(n, Fraction(1, 200))
        for n in (60, 90, 120)
    } != {60: 41, 90: 58, 120: 75}:
        fail("maximum-alpha benefit criticals changed")
    for family_size in family_sizes:
        expected_first = (
            {60: 49, 90: 69, 120: 87}
            if family_size < 16000
            else {60: 50, 90: 69, 120: 88}
        )
        for n in (60, 90, 120):
            if critical_success_count(
                n, Fraction(1, 200 * family_size)
            ) != expected_first[n]:
                fail("actual-family first Holm critical changed")
            if critical_success_count(
                n, Fraction(1, 200 * (family_size - 4))
            ) != expected_first[n]:
                fail("complement-rank critical changed")
            for remaining in range(1, family_size + 1):
                critical = critical_success_count(
                    n, Fraction(1, 200 * remaining)
                )
                if critical - 1 < n // 2:
                    fail("Holm cutoff left the Hoeffding upper-tail region")


def polynomial_multiply(left: list[int], right: list[int]) -> list[int]:
    result = [0] * (len(left) + len(right) - 1)
    for left_index, left_value in enumerate(left):
        for right_index, right_value in enumerate(right):
            result[left_index + right_index] += left_value * right_value
    return result


@lru_cache(maxsize=None)
def verify_qpm_polynomial_identity() -> None:
    q_split = [15] + [0] * 5 + [15]
    p_split = [1]
    for _ in range(5):
        p_split = polynomial_multiply(p_split, q_split)
    if (
        Fraction(sum(p_split[6:]), sum(p_split)) != Fraction(31, 32)
        or Fraction(15, 30) != Fraction(1, 2)
    ):
        fail("per-salt mapping Q/P test vector changed")
    shared_cycle = [3 * 30 ** 5] + [0] * 29 + [30 ** 5]
    if (
        Fraction(shared_cycle[30], sum(shared_cycle)) != Fraction(1, 4)
        or Fraction(1, 4 ** 5) == Fraction(1, 4)
    ):
        fail("shared-pilot-cycle M test vector changed")
    cycle_polynomials: list[list[int]] = []
    rejected_global_pair: list[int] = []
    for cycle in range(4):
        per_salt: list[list[int]] = []
        global_counts = [0] * 31
        for salt in range(5):
            q = [0] * 7
            for ordered_pair in range(30):
                successes = (cycle + 2 * salt + 3 * ordered_pair) % 7
                q[successes] += 1
            if sum(q) != 30 or len(q) - 1 > 6:
                fail("synthetic Q polynomial invariant failed")
            per_salt.append(q)
        for ordered_pair in range(30):
            total = sum(
                (cycle + 2 * salt + 3 * ordered_pair) % 7
                for salt in range(5)
            )
            global_counts[total] += 1
        p = [1]
        for q in per_salt:
            p = polynomial_multiply(p, q)
        if sum(p) != 30 ** 5 or len(p) - 1 > 30:
            fail("synthetic P polynomial invariant failed")
        cycle_polynomials.append(p)
        if not rejected_global_pair:
            rejected_global_pair = global_counts
        else:
            if len(rejected_global_pair) < len(global_counts):
                rejected_global_pair.extend(
                    [0] * (len(global_counts) - len(rejected_global_pair))
                )
            for index, value in enumerate(global_counts):
                rejected_global_pair[index] += value
    m = [sum(p[index] for p in cycle_polynomials) for index in range(31)]
    if sum(m) != 4 * 30 ** 5 or m == rejected_global_pair:
        fail("synthetic M polynomial or per-salt mapping distinction failed")
    for repetitions in (2, 3, 4):
        law = [1]
        for _ in range(repetitions):
            law = polynomial_multiply(law, m)
        if (
            sum(law) != (4 * 30 ** 5) ** repetitions
            or len(law) - 1 > 30 * repetitions
        ):
            fail(f"synthetic Q/P/M law failed for r={repetitions}")


@lru_cache(maxsize=None)
def verify_benefit_category_algebra() -> None:
    candidates = ("A", "B", "C", "D", "E")
    winner = "A"
    timing_cells = ("T1", "T2")
    positive_memory_cells = ("M1", "M2")
    zero_memory_cells = ("Z1",)
    q = "M2"
    p_bt = {"T1": Fraction(1, 101), "T2": Fraction(2, 103)}
    p_bm = {"M1": Fraction(3, 107), "M2": Fraction(5, 109)}
    p_rm_q = Fraction(7, 113)
    explicit = Fraction(0, 1)
    for cell in timing_cells:
        for left in candidates:
            for right in candidates:
                if left != right:
                    explicit += p_bt[cell]
    for cell in (*positive_memory_cells, *zero_memory_cells):
        for left in candidates:
            for right in candidates:
                if left == right or (
                    cell == q and left == winner and right != winner
                ):
                    continue
                if cell in zero_memory_cells:
                    marginal = Fraction(0, 1)
                elif cell == q and right == winner:
                    marginal = p_rm_q
                else:
                    marginal = p_bm[cell]
                explicit += marginal
    formula = (
        20 * sum(p_bt.values(), Fraction(0, 1))
        + 20 * sum(p_bm.values(), Fraction(0, 1))
        - 8 * p_bm[q]
        + 4 * p_rm_q
    )
    if explicit != formula:
        fail("memory complement category probability algebra changed")


def verify_benefit_partition_domains(
    branches: list[dict[str, str]], matrix: list[dict[str, str]]
) -> None:
    for branch in branches:
        if branch["branch_class"] != "ACTIVE_POWER_BRANCH":
            continue
        branch_id = branch["branch_id"]
        cells = sorted(
            row["cell_id"] for row in matrix
            if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
            and branch_id in row["owner_branch_ids"].split(",")
        )
        non_zst = sorted(
            row["cell_id"] for row in matrix
            if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
            and row["payload_id"] != "P-ZST-AFFINE"
            and branch_id in row["owner_branch_ids"].split(",")
        )
        family = set(registry.benefit_hypothesis_ids(branch_id, matrix))
        expected_family = {
            f"{left}>{right}|{cell}|END-RAW-TRACE-NS"
            for left in registry.CANDIDATES
            for right in registry.CANDIDATES if left != right
            for cell in cells
        } | {
            f"{left}>{right}|{cell}|END-PEAK-ACQUIRED-BYTES"
            for left in registry.CANDIDATES
            for right in registry.CANDIDATES if left != right
            for cell in non_zst
        }
        if family != expected_family:
            fail(f"benefit family identity changed: {branch_id}")
        timing_category = {
            hypothesis_id for hypothesis_id in family
            if hypothesis_id.endswith("|END-RAW-TRACE-NS")
        }
        memory_by_cell = {
            cell: {
                f"{left}>{right}|{cell}|END-PEAK-ACQUIRED-BYTES"
                for left in registry.CANDIDATES
                for right in registry.CANDIDATES if left != right
            }
            for cell in non_zst
        }
        if (
            len(timing_category) != 20 * len(cells)
            or set().union(*memory_by_cell.values()) != family - timing_category
        ):
            fail("benefit timing/memory category identity changed")
        for q_index, q in enumerate(non_zst):
            for winner in registry.CANDIDATES:
                losers = [
                    candidate for candidate in registry.CANDIDATES
                    if candidate != winner
                ]
                injected = {
                    f"{winner}>{loser}|{q}|END-PEAK-ACQUIRED-BYTES"
                    for loser in losers
                }
                if (
                    len(injected) != 4
                    or not injected <= family
                ):
                    fail("F/N benefit partition identity changed")
                q_complement = memory_by_cell[q] - injected
                q_baseline = {
                    f"{left}>{right}|{q}|END-PEAK-ACQUIRED-BYTES"
                    for left in losers for right in losers if left != right
                }
                q_reverse = {
                    f"{loser}>{winner}|{q}|END-PEAK-ACQUIRED-BYTES"
                    for loser in losers
                }
                if (
                    len(q_baseline) != 12
                    or len(q_reverse) != 4
                    or q_baseline & q_reverse
                    or q_baseline | q_reverse != q_complement
                ):
                    fail("benefit q-cell complement identity changed")
                for positive in (
                    set(non_zst),
                    {
                        cell for index, cell in enumerate(non_zst)
                        if index % 2 == q_index % 2
                    } | {q},
                ):
                    zero = set(non_zst) - positive
                    other_positive = positive - {q}
                    if (
                        q not in positive
                        or positive & zero
                        or positive | zero != set(non_zst)
                        or len(timing_category)
                        + 20 * len(other_positive)
                        + len(q_baseline)
                        + len(q_reverse)
                        + 20 * len(zero)
                        != len(family) - 4
                    ):
                        fail("benefit N category identity changed")
                if len(registry.digest_value(sorted(injected))) != 64:
                    fail("benefit partition hash construction failed")


POWER_LAW_CLASS_SPECS = (
    (
        "TIMING_NI_BASE", "END-RAW-TRACE-NS", "NONINFERIORITY", "BASE",
        "1/1", "1/1", "1/1", "51/50",
        "50*scaled_left_elapsed_ns < 51*scaled_right_elapsed_ns", "T",
    ),
    (
        "TIMING_NI_FORWARD", "END-RAW-TRACE-NS", "NONINFERIORITY",
        "INJECTED_WINNER_OVER_BASELINE", "17/20", "1/1", "17/20",
        "51/50", "50*scaled_left_elapsed_ns < 51*scaled_right_elapsed_ns", "T",
    ),
    (
        "TIMING_NI_REVERSE", "END-RAW-TRACE-NS", "NONINFERIORITY",
        "BASELINE_OVER_INJECTED_WINNER", "1/1", "17/20", "20/17",
        "51/50", "50*scaled_left_elapsed_ns < 51*scaled_right_elapsed_ns", "T",
    ),
    (
        "TIMING_BENEFIT_FORWARD", "END-RAW-TRACE-NS", "BENEFIT",
        "INJECTED_WINNER_OVER_BASELINE", "17/20", "1/1", "17/20",
        "9/10", "10*scaled_left_elapsed_ns < 9*scaled_right_elapsed_ns", "T",
    ),
    (
        "TIMING_BENEFIT_BASE", "END-RAW-TRACE-NS", "BENEFIT", "BASE",
        "1/1", "1/1", "1/1", "9/10",
        "10*scaled_left_elapsed_ns < 9*scaled_right_elapsed_ns", "T",
    ),
    (
        "MEMORY_BENEFIT_BASE", "END-PEAK-ACQUIRED-BYTES", "BENEFIT", "BASE",
        "1/1", "1/1", "1/1", "17/20",
        "20*scaled_left_peak_acquired_bytes < 17*scaled_right_peak_acquired_bytes",
        "M",
    ),
    (
        "MEMORY_BENEFIT_FORWARD", "END-PEAK-ACQUIRED-BYTES", "BENEFIT",
        "INJECTED_WINNER_OVER_BASELINE", "4/5", "1/1", "4/5", "17/20",
        "20*scaled_left_peak_acquired_bytes < 17*scaled_right_peak_acquired_bytes",
        "M",
    ),
    (
        "MEMORY_BENEFIT_REVERSE", "END-PEAK-ACQUIRED-BYTES", "BENEFIT",
        "BASELINE_OVER_INJECTED_WINNER", "1/1", "4/5", "5/4", "17/20",
        "20*scaled_left_peak_acquired_bytes < 17*scaled_right_peak_acquired_bytes",
        "M",
    ),
)


def expected_power_law_classes() -> list[dict[str, str]]:
    fields = (
        "law_key_class_id", "endpoint_id", "decision_family", "orientation",
        "left_pilot_multiplier", "right_pilot_multiplier",
        "relative_injected_ratio", "acceptance_ratio",
        "strict_success_cross_product", "count_formula",
    )
    return [dict(zip(fields, row)) for row in POWER_LAW_CLASS_SPECS]


def expected_benefit_partition_category_protocol() -> dict[str, Any]:
    return {
        "schema": "xlang-dense-benefit-partition-categories-v1",
        "symbols": {
            "T": "all selected-branch timing cells",
            "L": "all selected-branch non-ZST memory cells",
            "M": "positive-reference memory subset",
            "Z": "L-M structural-zero memory subset",
            "q": "the injected positive-reference memory cell",
        },
        "complement_categories": [
            {
                "category_id": "BASELINE_TIMING",
                "identity_count": "20*T",
                "marginal": "p_bt(cell)",
            },
            {
                "category_id": "OTHER_POSITIVE_MEMORY_BASELINE",
                "identity_count": "20*(M-1)",
                "marginal": "p_bm(cell)",
            },
            {
                "category_id": "INJECTED_CELL_BASELINE",
                "identity_count": "12",
                "marginal": "p_bm(q)",
            },
            {
                "category_id": "INJECTED_CELL_REVERSE",
                "identity_count": "4",
                "marginal": "p_rm(q)",
            },
            {
                "category_id": "STRUCTURAL_ZERO_MEMORY",
                "identity_count": "20*Z",
                "marginal": "0",
            },
        ],
        "complement_identity_count_formula": "20*(T+L)-4=m-4",
        "probability_sum_terms": [
            {"domain": "T", "marginal": "p_bt(cell)", "coefficient": 20},
            {"domain": "M", "marginal": "p_bm(cell)", "coefficient": 20},
            {"domain": "q", "marginal": "p_bm(q)", "coefficient": -8},
            {"domain": "q", "marginal": "p_rm(q)", "coefficient": 4},
        ],
        "probability_sum_formula": (
            "20*sum_T(p_bt)+20*sum_M(p_bm)-8*p_bm(q)+4*p_rm(q)"
        ),
        "cache_scope": (
            "Compute once per branch, winner, memory endpoint, injected q, and "
            "target; cache the exact partition across n=60,90,120, then compute "
            "and cache a separate exact marginal sum for each n after joining "
            "that n's laws."
        ),
    }


def expected_maximum_power_law_key_identity_rows(
    branch_id: str,
    matrix: list[dict[str, str]],
) -> list[dict[str, str]]:
    classes = expected_power_law_classes()
    rows: list[dict[str, str]] = []
    for cell in matrix:
        if (
            cell["primary_endpoint_id"] != "END-RAW-TRACE-NS"
            or branch_id not in cell["owner_branch_ids"].split(",")
        ):
            continue
        for law_class in classes:
            if (
                law_class["endpoint_id"] == "END-PEAK-ACQUIRED-BYTES"
                and cell["payload_id"] == "P-ZST-AFFINE"
            ):
                continue
            rows.append({
                "branch_id": branch_id,
                "cell_id": cell["cell_id"],
                "target_id": cell["target_id"],
                "endpoint_id": law_class["endpoint_id"],
                "law_key_class_id": law_class["law_key_class_id"],
                "orientation": law_class["orientation"],
                "left_pilot_multiplier": law_class["left_pilot_multiplier"],
                "right_pilot_multiplier": law_class["right_pilot_multiplier"],
                "relative_injected_ratio": law_class[
                    "relative_injected_ratio"
                ],
                "acceptance_ratio": law_class["acceptance_ratio"],
                "strict_success_cross_product": law_class[
                    "strict_success_cross_product"
                ],
            })
    identity_fields = (
        "branch_id", "cell_id", "target_id", "endpoint_id",
        "law_key_class_id",
    )
    return sorted(
        rows,
        key=lambda row: tuple(row[field] for field in identity_fields),
    )


def expected_maximum_power_task_identity_rows(
    branch_id: str,
    matrix: list[dict[str, str]],
    power_engine_protocol_sha256: str,
) -> list[dict[str, Any]]:
    family_sha256 = registry.digest_value(
        registry.benefit_hypothesis_ids(branch_id, matrix)
    )
    cells = [
        row for row in matrix
        if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
        and branch_id in row["owner_branch_ids"].split(",")
    ]
    rows: list[dict[str, Any]] = []
    for winner in registry.CANDIDATES:
        for endpoint_id in (
            "END-RAW-TRACE-NS", "END-PEAK-ACQUIRED-BYTES",
        ):
            alternative = registry.exact_alternative_matrix(winner, endpoint_id)
            for cell in cells:
                if (
                    endpoint_id == "END-PEAK-ACQUIRED-BYTES"
                    and cell["payload_id"] == "P-ZST-AFFINE"
                ):
                    continue
                alternative_sha256 = registry.digest_value({
                    "alternative_matrix": alternative,
                    "benefit_cell_id": cell["cell_id"],
                    "target_id": cell["target_id"],
                })
                for block_count in (60, 90, 120):
                    rows.append({
                        "owner_branch_id": branch_id,
                        "true_winner_id": winner,
                        "benefit_endpoint_id": endpoint_id,
                        "benefit_cell_id": cell["cell_id"],
                        "target_id": cell["target_id"],
                        "selected_block_count": block_count,
                        "injected_alternative_sha256": alternative_sha256,
                        "selected_family_sha256": family_sha256,
                        "power_engine_protocol_sha256":
                            power_engine_protocol_sha256,
                    })
    identity_fields = (
        "owner_branch_id", "true_winner_id", "benefit_endpoint_id",
        "benefit_cell_id", "target_id", "selected_block_count",
        "injected_alternative_sha256", "selected_family_sha256",
        "power_engine_protocol_sha256",
    )
    return sorted(
        rows,
        key=lambda row: tuple(str(row[field]) for field in identity_fields),
    )


def verify_maximum_power_task_event_domains(
    branches: list[dict[str, str]],
    matrix: list[dict[str, str]],
    stats: dict[str, Any],
) -> None:
    engine_sha256 = stats["power_calculation"][
        "power_engine_protocol_sha256"
    ]
    domains: list[tuple[str, list[dict[str, Any]]]] = []
    all_identity_ids: set[str] = set()
    for branch in branches:
        if branch["branch_class"] != "ACTIVE_POWER_BRANCH":
            continue
        rows = expected_maximum_power_task_identity_rows(
            branch["branch_id"], matrix, engine_sha256
        )
        identity_ids = {registry.digest_value(row) for row in rows}
        if (
            len(identity_ids) != len(rows)
            or all_identity_ids & identity_ids
        ):
            fail("maximum power-task identities are duplicated")
        all_identity_ids |= identity_ids
        domains.append((branch["branch_id"], rows))
    branch_id, tasks = min(
        domains,
        key=lambda item: (-len(item[1]), item[0]),
    )
    if len(tasks) != 12210:
        fail("maximum power-task identity domain is not 12,210 rows")
    timing_cells = sorted(
        row["cell_id"] for row in matrix
        if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
        and branch_id in row["owner_branch_ids"].split(",")
    )
    class_counts: Counter[str] = Counter()
    total = 0
    for task in tasks:
        task_identity_id = registry.digest_value(task)
        winner = task["true_winner_id"]
        losers = [
            candidate for candidate in registry.CANDIDATES
            if candidate != winner
        ]
        local_ids: set[tuple[str, ...]] = set()
        for right in registry.TREATMENTS:
            if right == winner:
                continue
            for timing_cell in timing_cells:
                local_ids.add((
                    task_identity_id,
                    "WINNER_OUTGOING_NI_CELL_FAILURE",
                    winner,
                    right,
                    timing_cell,
                ))
                class_counts["WINNER_OUTGOING_NI_CELL_FAILURE"] += 1
        for loser in losers:
            local_ids.add((
                task_identity_id,
                "WINNER_REQUIRED_BENEFIT_FAILURE",
                winner,
                loser,
                task["benefit_cell_id"],
                task["benefit_endpoint_id"],
            ))
            class_counts["WINNER_REQUIRED_BENEFIT_FAILURE"] += 1
        if task["benefit_endpoint_id"] == "END-RAW-TRACE-NS":
            for loser in losers:
                local_ids.add((
                    task_identity_id,
                    "TIMING_REVERSE_NI_ERRONEOUS_PASS",
                    loser,
                    winner,
                ))
                class_counts["TIMING_REVERSE_NI_ERRONEOUS_PASS"] += 1
            expected_local_count = 5 * len(timing_cells) + 9
        else:
            local_ids.add((
                task_identity_id,
                "MEMORY_COMPLEMENT_BENEFIT_REJECTION_BOUND",
                "BENEFIT_PARTITION_SHA256",
                "CATEGORY_SUM_SHA256",
            ))
            class_counts[
                "MEMORY_COMPLEMENT_BENEFIT_REJECTION_BOUND"
            ] += 1
            expected_local_count = 5 * len(timing_cells) + 6
        local_ids.add((
            task_identity_id,
            "PROTOCOL_INVALIDITY",
            "PREFLIGHT_LEDGER_SHA256",
        ))
        class_counts["PROTOCOL_INVALIDITY"] += 1
        if len(local_ids) != expected_local_count:
            fail("task-scoped failure-event domain has duplicate or missing IDs")
        total += len(local_ids)
    timing_task_count = sum(
        task["benefit_endpoint_id"] == "END-RAW-TRACE-NS"
        for task in tasks
    )
    memory_task_count = len(tasks) - timing_task_count
    expected_counts = Counter({
        "WINNER_OUTGOING_NI_CELL_FAILURE": len(tasks) * 5 * len(timing_cells),
        "WINNER_REQUIRED_BENEFIT_FAILURE": len(tasks) * 4,
        "TIMING_REVERSE_NI_ERRONEOUS_PASS": timing_task_count * 4,
        "MEMORY_COMPLEMENT_BENEFIT_REJECTION_BOUND": memory_task_count,
        "PROTOCOL_INVALIDITY": len(tasks),
    })
    if (
        class_counts != expected_counts
        or total != 25000020
        or sum(class_counts.values()) != total
    ):
        fail("maximum streamed failure-event domain changed")


def expected_bundle() -> dict[str, Any]:
    contracts = registry.assert_frozen_contract_inputs()
    authority, dispositions = registry.derive_dispositions(contracts)
    branches = registry.owner_branch_base_rows()
    gates, algorithms = registry.build_operation_gates_and_algorithms(
        contracts, dispositions
    )
    matrix, descriptors = registry.build_matrix(
        contracts, dispositions, gates, branches
    )
    registry.finalize_owner_branches(branches, matrix)
    inputs = registry.generated_input_rows(descriptors)
    statistics = registry.statistics_payload(matrix, branches)
    tables = registry.table_rows(
        dispositions, gates, branches, algorithms, matrix
    )
    return {
        "contracts": contracts,
        "authority": authority,
        "dispositions": dispositions,
        "branches": branches,
        "gates": gates,
        "matrix": matrix,
        "descriptors": descriptors,
        "inputs": inputs,
        "statistics": statistics,
        "tables": tables,
    }


def verify_freshness(bundle: dict[str, Any]) -> None:
    for key, expected in bundle["tables"].items():
        actual = read_tsv(key)
        if actual != expected:
            fail(f"generated table is stale: {registry.OUTPUTS[key]}")
    if read_jsonl("descriptors") != bundle["descriptors"]:
        fail("descriptor artifact is stale")
    if read_jsonl("inputs") != bundle["inputs"]:
        fail("generated input artifact is stale")
    if read_json("statistics") != bundle["statistics"]:
        fail("statistics artifact is stale")
    records = [
        registry.artifact_record(HERE / registry.OUTPUTS[key])
        for key in sorted(bundle["tables"])
    ] + [
        registry.artifact_record(HERE / registry.OUTPUTS["descriptors"]),
        registry.artifact_record(HERE / registry.OUTPUTS["inputs"]),
        registry.artifact_record(HERE / registry.OUTPUTS["statistics"]),
    ]
    expected_report = registry.render_protocol_report(
        bundle["dispositions"], bundle["gates"], bundle["branches"],
        bundle["matrix"], records,
    )
    actual_report = (HERE / registry.OUTPUTS["protocol"]).read_text(
        encoding="ascii"
    )
    if actual_report != expected_report:
        fail("protocol report is stale")


def validate_dispositions(
    contracts: list[dict[str, str]],
    authority: list[dict[str, str]],
    dispositions: list[dict[str, str]],
) -> None:
    if len(contracts) != 303 or len(authority) != 303 or len(dispositions) != 303:
        fail("exact contract derivation is not total")
    require_unique(contracts, "contract_id", "contract ID")
    require_unique(authority, "contract_id", "authority contract ID")
    require_unique(dispositions, "contract_id", "disposition contract ID")
    contract_ids = {row["contract_id"] for row in contracts}
    if contract_ids != {row["contract_id"] for row in authority}:
        fail("authority is not an exact contract-ID partition")
    if contract_ids != {row["contract_id"] for row in dispositions}:
        fail("dispositions are not an exact contract-ID partition")
    allowed = {
        "TIMED_PRIMARY", "STRUCTURAL_ONLY", "FUNCTIONAL_ONLY", "EXCLUDED",
    }
    if any(row["disposition"] not in allowed for row in dispositions):
        fail("forbidden or unknown performance disposition")
    expected_counts = {
        "TIMED_PRIMARY": 137,
        "FUNCTIONAL_ONLY": 144,
        "STRUCTURAL_ONLY": 13,
        "EXCLUDED": 9,
    }
    if Counter(row["disposition"] for row in dispositions) != expected_counts:
        fail("exact disposition counts changed")
    by_contract = {row["contract_id"]: row for row in contracts}
    source_fields = list(contracts[0])
    for row in dispositions:
        source = by_contract[row["contract_id"]]
        digest = registry.source_row_digest(source, source_fields)
        if row["source_contract_sha256"] != digest:
            fail(f"source contract digest mismatch: {row['contract_id']}")
        if (
            row["contract_id"] not in row["exact_reason"]
            or digest not in row["exact_reason"]
            or row["derivation_kind"] not in {
                "REPRESENTATIVE", "STRUCTURAL", "FUNCTIONAL", "EXCLUDED",
            }
        ):
            fail(f"exact derivation reason is incomplete: {row['contract_id']}")


def validate_gates_and_matrix(
    dispositions: list[dict[str, str]],
    gates: list[dict[str, str]],
    matrix: list[dict[str, str]],
) -> None:
    if len(gates) != 97 or len(matrix) != 520:
        fail("operation-gate or matrix count changed")
    require_unique(gates, "operation_gate_id", "operation gate")
    require_unique(matrix, "cell_id", "matrix cell")
    require_unique(matrix, "trace_sha256", "trace digest")
    require_unique(matrix, "oracle_sha256", "oracle digest")
    if any(row["candidate_execution_authorized"] != "NO" for row in matrix):
        fail("matrix authorizes candidate execution")
    primary = [
        row for row in matrix
        if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
    ]
    if len(primary) != 502:
        fail("timed primary cell count changed")
    if any(row["rust_floor_upper_ratio"] != "1.02" for row in primary):
        fail("a primary cell lost its independent 1.02 Rust floor")
    blocker_rows = registry.blocker_rows()
    blocker_branches = {
        row["blocker_id"]: set(
            row["applicable_owner_branch_ids"].split(",")
        )
        for row in blocker_rows
    }
    for row in matrix:
        row_blockers = set(row["blocker_ids"].split(","))
        row_branches = set(row["owner_branch_ids"].split(","))
        if "PENDING_EXTERNAL_REPOSITORY_BASELINE" not in row_blockers:
            fail(f"matrix cell lacks the common repository baseline: {row['cell_id']}")
        if not row_blockers <= set(blocker_branches):
            fail(f"matrix cell names an unknown blocker: {row['cell_id']}")
        for blocker_id in row_blockers:
            if not row_branches <= blocker_branches[blocker_id]:
                fail(
                    "matrix cell exceeds blocker branch applicability: "
                    f"{row['cell_id']}/{blocker_id}"
                )
    exact_primary_members = {
        row["member_contract_id"] for row in dispositions
        if row["disposition"] == "TIMED_PRIMARY"
    }
    matrix_members = {
        row["member_contract_id"] for row in matrix
        if row["cell_role"] == "PRIMARY_OPERATION"
    }
    if exact_primary_members != matrix_members or len(matrix_members) != 84:
        fail("standalone operation primary coverage is incomplete")
    by_gate = {row["operation_gate_id"]: row for row in gates}
    matrix_by_gate: dict[str, list[dict[str, str]]] = {}
    for row in primary:
        matrix_by_gate.setdefault(row["operation_gate_id"], []).append(row)
    for gate_id, gate in by_gate.items():
        cells = matrix_by_gate.get(gate_id, [])
        if not cells:
            fail(f"operation gate has no primary cells: {gate_id}")
        ids = sorted(row["cell_id"] for row in cells)
        if (
            gate["primary_cell_ids"] != ",".join(ids)
            or gate["primary_cell_set_sha256"] != registry.digest_value(ids)
        ):
            fail(f"operation gate cell freeze mismatch: {gate_id}")
        required_shapes = set(gate["required_shape_ids"].split(","))
        for contract_id in gate["representative_contract_ids"].split(","):
            if contract_id == "OD4-POLICY-SCOPED-CONSUME":
                continue
            contract_cells = [
                row for row in cells if row["contract_id"] == contract_id
                and row["cell_role"] == "PRIMARY_OPERATION"
            ]
            for shape in required_shapes:
                for target in registry.NATIVE_TARGETS:
                    if not any(
                        row["shape_id"] == shape and row["target_id"] == target
                        for row in contract_cells
                    ):
                        fail(
                            f"missing independent operation cell: "
                            f"{contract_id} {shape} {target}"
                        )
    required_shape_sets = {
        "DENSE-SORT-STABLE": {
            "SORT-RANDOM", "SORT-SORTED", "SORT-REVERSE",
            "SORT-ORGAN-PIPE", "SORT-DUPLICATE-90",
        },
        "DENSE-RETAIN": {"RETAIN-10", "RETAIN-50", "RETAIN-90"},
        "DENSE-SWAP": {
            "SWAP-EQUAL", "SWAP-FRONT-BACK", "SWAP-ADJACENT-MIDDLE",
        },
        "DENSE-CLONE-FROM": {
            "CLONE-FROM-DST-SHORTER", "CLONE-FROM-EQUAL",
            "CLONE-FROM-DST-LONGER",
        },
    }
    for member, shapes in required_shape_sets.items():
        actual = {
            row["shape_id"] for row in primary
            if row["member_contract_id"] == member
        }
        if not shapes <= actual:
            fail(f"independent workload shapes missing for {member}")
    splice_shapes = {
        row["shape_id"] for row in primary
        if row["member_contract_id"] == "DENSE-EAGER-SPLICE"
    }
    if len({shape for shape in splice_shapes if shape.startswith("SPLICE-")}) != 9:
        fail("splice position/replacement cells are incomplete")
    traversal = {
        row["member_contract_id"] for row in primary
        if row["member_contract_id"] in {
            "DENSE-ITER-SHARED", "DENSE-ITER-UNIQ", "DENSE-ITER-OWN",
        }
    }
    if traversal != {
        "DENSE-ITER-SHARED", "DENSE-ITER-UNIQ", "DENSE-ITER-OWN",
    }:
        fail("traversal modes are not independent primary operations")
    for row in primary:
        blocker_ids = set(row["blocker_ids"].split(","))
        if row["cell_role"] == "OD4_SCOPED_PRIMARY" and not {
            "PENDING_EXTERNAL_OD4_SCOPED_CONTRACT",
            "PENDING_EXTERNAL_OD4_CANDIDATE_ARTIFACTS",
        } <= blocker_ids:
            fail(f"OD4 cell lacks split pilot/construction gates: {row['cell_id']}")


def validate_payloads_and_zst(matrix: list[dict[str, str]]) -> None:
    separators = [
        row for row in matrix
        if row["cell_role"] == "PAYLOAD_SEPARATOR_PRIMARY"
    ]
    counts = Counter(row["payload_code"] for row in separators)
    if counts != {code: 2 for code in registry.PAYLOAD_CODES}:
        fail("nine-payload native separator coverage is incomplete")
    zst = [row for row in matrix if row["payload_id"] == "P-ZST-AFFINE"]
    if not zst:
        fail("ZST has no timed latency witness")
    for row in zst:
        bits = registry.TARGET_BITS[row["target_id"]]
        if (
            row["logical_bytes"] != "0"
            or row["allocator_id"] != "ALLOC-NONE-ZST"
            or row["growth_policy_id"] != "GROW-ZST-USIZE-MAX"
            or int(row["initial_capacity"]) != (1 << bits) - 1
            or "SG-ZST" not in row["structural_gate_ids"].split(",")
        ):
            fail(f"ZST policy contradiction: {row['cell_id']}")


def validate_branches(
    branches: list[dict[str, str]],
    matrix: list[dict[str, str]],
) -> None:
    active = [
        row for row in branches
        if row["branch_class"] == "ACTIVE_POWER_BRANCH"
    ]
    blocked = [
        row for row in branches
        if row["branch_class"] == "BLOCKED_OR_REOPEN_REQUIRED"
    ]
    if len(active) != 8 or len(blocked) != 4:
        fail("owner branch partition changed")
    combinations = {
        (
            row["od1_option_id"], row["od2_option_id"], row["od3_option_id"]
        )
        for row in active
    }
    if len(combinations) != 8:
        fail("OD1 x OD2 x OD3 branch product is incomplete")
    for row in active:
        if (
            row["od0_option_id"]
            != "OD-0-COMMON-EXPERIMENTAL-SUBSTRATE"
            or row["od4_option_id"]
            != "OD-4-EAGER-AND-SCOPED-CONSUME"
            or row["od5_option_id"] != registry.NO_CROSSOVER
        ):
            fail("active branch changed OD0, OD4, or OD5 common condition")
        ids = sorted(
            cell["cell_id"] for cell in matrix
            if cell["primary_endpoint_id"] == "END-RAW-TRACE-NS"
            and row["branch_id"] in cell["owner_branch_ids"].split(",")
        )
        if (
            int(row["primary_cell_count"]) != len(ids)
            or row["primary_cell_ids_sha256"] != registry.digest_value(ids)
        ):
            fail(f"owner branch cell freeze mismatch: {row['branch_id']}")
    required_blocked = {
        "BLOCKED-OD0-SEPARATE",
        "REOPEN-OD4-EAGER-ONLY",
        "REOPEN-OD4-PROMOTE-LAZY",
        "REOPEN-OD5-CROSSOVER",
    }
    if {row["branch_id"] for row in blocked} != required_blocked:
        fail("blocked/reopening owner alternatives changed")


def validate_substrate(rows: list[dict[str, str]]) -> None:
    if {row["candidate_id"] for row in rows} != set(registry.CANDIDATES):
        fail("common substrate does not cover all five arms")
    contract_hashes = {row["substrate_contract_sha256"] for row in rows}
    cost_hashes = {row["cost_model_sha256"] for row in rows}
    if len(contract_hashes) != 1 or len(cost_hashes) != 1:
        fail("common substrate or cost differs across candidate arms")
    for row in rows:
        joined = json.dumps(row, sort_keys=True)
        for fragment in (
            "allocator", "seal", "generic", "reborrow",
            "provenance", "interval", "cursor", "private",
        ):
            if fragment not in joined.lower():
                fail(f"common substrate lost {fragment}")


def validate_protected_controls(matrix: list[dict[str, str]]) -> None:
    controls = {"B-FIX", "B-P2", "H-FLATSET", "W-SMALL", "W-GAP"}
    cells = [
        row for row in matrix if row["cell_role"] == "PROTECTED_STRUCTURAL"
    ]
    expected = {
        (control, target)
        for control in controls for target in registry.ALL_TARGETS
    }
    actual = {(row["member_contract_id"], row["target_id"]) for row in cells}
    if actual != expected or len(cells) != 15:
        fail("protected controls do not cover all three target layouts")


def validate_stage_prerequisites(
    blockers: list[dict[str, str]],
    stats: dict[str, Any],
    branches: list[dict[str, str]],
) -> None:
    expected_by_stage = {
        "REFERENCE_PILOT": {
            "PENDING_EXTERNAL_OWNER_AUTHORIZATION",
            "PENDING_EXTERNAL_OWNER_BRANCH_SELECTION",
            "PENDING_EXTERNAL_OD4_SCOPED_CONTRACT",
            "PENDING_EXTERNAL_AARCH64_ENVIRONMENT",
            "PENDING_EXTERNAL_X86_RUNNER",
            "PENDING_EXTERNAL_ALLOCATOR_ADAPTER",
            "PENDING_EXTERNAL_HARNESS",
            "PENDING_EXTERNAL_RANDOMIZATION_CUSTODY",
            "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        },
        "CANDIDATE_CONSTRUCTION": {
            "PENDING_EXTERNAL_REPOSITORY_BASELINE",
            "PENDING_EXTERNAL_CANDIDATE_AUTHOR_IDENTITIES",
            "PENDING_EXTERNAL_SERVICE_SNAPSHOTS",
            "PENDING_EXTERNAL_DISCLOSURE_AUTHORITY",
            "PENDING_EXTERNAL_CONSTRUCTION_BUDGET",
            "PENDING_EXTERNAL_FEEDBACK_PROTOCOL",
            "PENDING_EXTERNAL_COMMON_SUBSTRATE_ARTIFACTS",
            "PENDING_EXTERNAL_OD4_CANDIDATE_ARTIFACTS",
            "PENDING_EXTERNAL_H_FLATSET_CUSTODY",
            "PENDING_EXTERNAL_I686_TOOLCHAIN",
            "PENDING_EXTERNAL_REFERENCE_PILOT",
            "PENDING_EXTERNAL_W_SMALL_FIXTURE",
            "PENDING_EXTERNAL_W_GAP_FIXTURE",
        },
        "CANDIDATE_FREEZE_B": {
            "PENDING_EXTERNAL_FACT_REPORTS",
            "PENDING_EXTERNAL_CANDIDATE_BUILDS",
            "PENDING_EXTERNAL_AARCH64_CANDIDATE_MODULES",
        },
        "DESCRIPTIVE_COUNTER_REPORT": {
            "PENDING_EXTERNAL_AARCH64_COUNTER_PROTOCOL",
            "PENDING_EXTERNAL_X86_COUNTER_PROTOCOL",
        },
    }
    expected_ids = set().union(*expected_by_stage.values())
    if len(blockers) != 27 or len(expected_ids) != 27:
        fail("operational blocker domain is not the exact 27-row set")
    by_id = {row["blocker_id"]: row for row in blockers}
    if len(by_id) != len(blockers) or set(by_id) != expected_ids:
        fail("operational blocker IDs are missing, duplicated, or unknown")
    for stage, blocker_ids in expected_by_stage.items():
        if any(
            by_id[blocker_id]["earliest_blocked_stage"] != stage
            for blocker_id in blocker_ids
        ):
            fail(f"earliest blocker stage changed for {stage}")
    if any(row["status"] != "BLOCKING" for row in blockers):
        fail("operational blocker no longer fails closed")

    active = {
        row["branch_id"]: row for row in branches
        if row["branch_class"] == "ACTIVE_POWER_BRANCH"
    }
    dual = {
        branch_id for branch_id, row in active.items()
        if row["od2_option_id"] == "OD-2-DUAL-NATIVE"
    }
    for blocker_id, row in by_id.items():
        applicable = set(row["applicable_owner_branch_ids"].split(","))
        expected_applicable = (
            dual
            if blocker_id in {
                "PENDING_EXTERNAL_X86_RUNNER",
                "PENDING_EXTERNAL_X86_COUNTER_PROTOCOL",
            }
            else set(active)
        )
        if applicable != expected_applicable:
            fail(f"blocker branch applicability changed: {blocker_id}")

    pilot = by_id["PENDING_EXTERNAL_REFERENCE_PILOT"]
    if "every applicable per-branch REFERENCE_PILOT prerequisite" not in pilot[
        "required_resolution"
    ]:
        fail("reference pilot retains an incomplete prerequisite shortcut")
    aarch = by_id["PENDING_EXTERNAL_AARCH64_ENVIRONMENT"]
    for fragment in ("Rust compiler", "linker", "target tools", "flags"):
        if fragment not in aarch["missing_fact"]:
            fail("AArch64 pilot toolchain baseline is incomplete")
    baseline = by_id["PENDING_EXTERNAL_REPOSITORY_BASELINE"]
    for fragment in ("starting commit", "worktree-status digest", "equality rule"):
        if fragment not in baseline["missing_fact"]:
            fail("candidate repository baseline is incomplete")

    protocol = stats.get("stage_prerequisite_protocol")
    if (
        not isinstance(protocol, dict)
        or protocol.get("schema") != "xlang-dense-stage-prerequisites-v1"
        or protocol.get("pipeline_stage_order")
        != ["REFERENCE_PILOT", "CANDIDATE_CONSTRUCTION", "CANDIDATE_FREEZE_B"]
        or protocol.get("side_stages") != ["DESCRIPTIVE_COUNTER_REPORT"]
        or "transitively blocks every later" not in protocol.get(
            "earliest_stage_rule", ""
        )
    ):
        fail("stage prerequisite schema or cumulative rule changed")
    stage_order = protocol["pipeline_stage_order"]
    per_branch = protocol.get("per_owner_branch", {})
    if set(per_branch) != set(active):
        fail("stage prerequisite branch domain changed")
    for branch_id in sorted(active):
        applicable = {
            blocker_id for blocker_id, row in by_id.items()
            if branch_id in row["applicable_owner_branch_ids"].split(",")
        }
        expected_cumulative: set[str] = set()
        branch_protocol = per_branch[branch_id]
        for stage in stage_order:
            direct = expected_by_stage[stage] & applicable
            expected_cumulative |= direct
            recorded = branch_protocol["pipeline"][stage]
            if (
                recorded["direct_blocker_ids"] != sorted(direct)
                or recorded["cumulative_blocker_ids"]
                != sorted(expected_cumulative)
            ):
                fail(f"stage prerequisite closure changed: {branch_id}/{stage}")
        side = branch_protocol["side_stages"]["DESCRIPTIVE_COUNTER_REPORT"]
        if side != sorted(
            expected_by_stage["DESCRIPTIVE_COUNTER_REPORT"] & applicable
        ):
            fail(f"counter side-stage applicability changed: {branch_id}")

    construction = stats["construction_protocol"]
    expected_construction = {
        branch_id: per_branch[branch_id]["pipeline"][
            "CANDIDATE_CONSTRUCTION"
        ]["cumulative_blocker_ids"]
        for branch_id in sorted(active)
    }
    expected_union = sorted({
        blocker_id
        for blocker_ids in expected_construction.values()
        for blocker_id in blocker_ids
    })
    if (
        construction.get("required_blocker_ids_by_owner_branch")
        != expected_construction
        or construction.get("required_blocker_ids") != expected_union
        or "before the first candidate prompt" not in construction.get(
            "first_candidate_prompt_gate", ""
        )
    ):
        fail("candidate construction prerequisite closure is incomplete")


def validate_statistics(
    stats: dict[str, Any],
    branches: list[dict[str, str]],
    matrix: list[dict[str, str]],
) -> None:
    if stats.get("schema") != "xlang-dense-performance-statistics-v5":
        fail("statistics schema is not v5")
    if stats["candidate_construction_authorized"] is not False:
        fail("statistics authorizes construction")
    validate_stage_prerequisites(registry.blocker_rows(), stats, branches)
    design = stats["primary_design"]
    if (
        design["candidate_count_k"] != 5
        or design["treatment_count"] != 6
        or design["numeric_sequences"] != registry.WILLIAMS_NUMERIC
        or design["allowed_block_counts"] != [60, 90, 120]
    ):
        fail("primary Williams design changed")
    sequences = design["numeric_sequences"]
    adjacent = Counter(
        (row[index], row[index + 1])
        for row in sequences for index in range(5)
    )
    expected_pairs = {
        (left, right)
        for left in range(1, 7) for right in range(1, 7)
        if left != right
    }
    if set(adjacent) != expected_pairs or set(adjacent.values()) != {1}:
        fail("Williams carryover is not exactly balanced")
    for position in range(6):
        if sorted(row[position] for row in sequences) != list(range(1, 7)):
            fail("Williams period positions are not balanced")
    multiplicity = stats["multiplicity"]
    power_alpha = stats["power_calculation"]["alpha_control"]
    if multiplicity != power_alpha:
        fail("power and verifier alpha controls differ")
    if (
        multiplicity["global_family_total_alpha"] != "0.01"
        or multiplicity["noninferiority_family_alpha"] != "0.005"
        or multiplicity["global_benefit_family_alpha"] != "0.005"
        or multiplicity["noninferiority_per_directed_claim_alpha"] != "0.0002"
        or multiplicity["noninferiority_directed_claim_count"] != 25
        or "Memory does not enter NI" not in
        multiplicity["noninferiority_method"]
        or "one global holm" not in multiplicity["benefit_method"].lower()
        or "no candidate pair" not in multiplicity["forbidden_reset"].lower()
    ):
        fail("single global alpha family changed")
    pseudo = stats["reference_only_pilot"]["pseudo_treatments"]
    if len(pseudo) != 6 or {row["source_id"] for row in pseudo} != {"RUST-1.97"}:
        fail("reference pilot is not six identical Rust pseudo-treatments")
    identity_fields = (
        "source_sha256", "compiler_sha256", "executable_sha256",
        "allocator_adapter_sha256", "environment_sha256",
    )
    for field in identity_fields:
        if len({row[field] for row in pseudo}) != 1:
            fail(f"Rust pseudo-treatment identity differs: {field}")
    if stats["reference_only_pilot"]["canonical_pseudo_ids"] != [
        f"RUST-PSEUDO-{index + 1}" for index in range(6)
    ] or "manifest assigns" not in stats["reference_only_pilot"]["identity_rule"]:
        fail("pilot manifest-derived pseudo mapping changed")
    pilot = stats["reference_only_pilot"]
    if (
        "pseudo_to_numeric_symbol_mapping" in pilot
        or
        pilot["fixed_complete_blocks_per_cell_target"] != 120
        or pilot["fixed_crossed_cycles_per_cell_target"] != 4
        or pilot["fixed_observations_per_cell_target"] != 720
        or "No candidate source" not in pilot["candidate_data_rule"]
        or "PROTOCOL_INFEASIBLE" not in pilot["stopping_rule"]
    ):
        fail("fixed nonadaptive reference pilot changed")
    estimand = stats["inferential_estimand"]
    response_models = estimand["endpoint_responses"]
    if response_models != registry.endpoint_response_protocol():
        fail("endpoint-indexed reference response models changed")
    response_by_endpoint = {
        row["endpoint_id"]: row for row in response_models
    }
    if set(response_by_endpoint) != {
        "END-RAW-TRACE-NS", "END-PEAK-ACQUIRED-BYTES",
    }:
        fail("timing or memory reference response is missing")
    memory_response = response_by_endpoint["END-PEAK-ACQUIRED-BYTES"]
    if (
        memory_response["raw_field"]
        != "allocator_counters.peak_acquired_bytes"
        or "all zero" not in memory_response["zero_or_mixed_rule"].lower()
        or "zero/positive mixture invalidates" not in
        memory_response["zero_or_mixed_rule"]
    ):
        fail("memory response or zero/mixed classification changed")
    memory_freeze = stats["reference_only_pilot"][
        "memory_eligibility_freeze"
    ]
    for fragment in (
        "POSITIVE_REFERENCE_MEMORY", "STRUCTURAL_ZERO_MEMORY",
        "zero/positive mixture invalidates", "SHA-256",
    ):
        if fragment not in memory_freeze:
            fail("reference-only memory eligibility freeze changed")
    if (
        estimand["scheduled_mixture_tail_protocol"]
        != registry.scheduled_mixture_tail_protocol()
        or "raw elapsed-nanosecond" not in estimand["raw_integer_rule"]
        or "No log" not in estimand["raw_integer_rule"]
        or stats["descriptive_nuisance_diagnostics"]["selection_use"].split(":", 1)[0]
        != "DESCRIPTIVE_ONLY"
    ):
        fail("raw scheduled-mixture estimand changed")
    benefit_testing = stats["benefit_testing"]
    if benefit_testing != registry.benefit_testing_protocol():
        fail("exact benefit testing protocol changed")
    tail_protocol = registry.scheduled_mixture_tail_protocol()
    if (
        benefit_testing["family_alpha"] != "1/200"
        or benefit_testing["family_size_range"] != [8120, 16280]
        or benefit_testing["maximum_family_size"] != 16280
        or benefit_testing["minimum_block_count"] != 60
        or Fraction(benefit_testing["minimum_attainable_p_value"])
        >= Fraction(benefit_testing["most_stringent_holm_threshold"])
        or benefit_testing["most_stringent_holm_threshold"]
        != "1/3256000"
        or benefit_testing["scheduled_mixture_tail_protocol_sha256"]
        != registry.digest_value(tail_protocol)
        or "If 2*s<=n, p=1/1" not in benefit_testing["one_sided_p_value"]
        or "bytewise ASCII hypothesis_id" not in
        benefit_testing["holm_order"]
        or "Stop at first nonrejection" not in
        benefit_testing["holm_step_down"]
        or "no fitted" not in benefit_testing["inference_engine_rule"].lower()
        or "logarithm" not in benefit_testing["inference_engine_rule"].lower()
        or "equal repetitions of all thirty" not in
        benefit_testing["block_validity_rule"]
        or "UNAVAILABLE" not in benefit_testing["descriptive_ratio_interval"]
        or "Rust is deliberately absent" not in benefit_testing["family_membership"]
    ):
        fail("benefit scheduled-mixture or Holm protocol is incomplete")
    endpoint_nulls = {
        row["endpoint_id"]: row
        for row in benefit_testing["endpoint_nulls"]
    }
    if (
        endpoint_nulls["END-RAW-TRACE-NS"]["strict_success_cross_product"]
        != "10*C.elapsed_ns < 9*D.elapsed_ns"
        or "20*C.peak_acquired_bytes < 17*D.peak_acquired_bytes" not in
        endpoint_nulls["END-PEAK-ACQUIRED-BYTES"][
            "strict_success_cross_product"
        ]
        or "p=1/1" not in
        benefit_testing["zero_reference_memory_rule"]
        or "counts as failure" not in
        benefit_testing["tie_and_unusable_rule"]
    ):
        fail("endpoint benefit null, tie, or zero rule changed")
    verify_hoeffding_extrema()
    if {
        n: critical_success_count(n, Fraction(1, 5000))
        for n in (60, 90, 120)
    } != {60: 44, 90: 63, 120: 80}:
        fail("NI critical success counts changed")
    first_holm = {
        m: {
            n: critical_success_count(n, Fraction(1, 200 * m))
            for n in (60, 90, 120)
        }
        for m in (8120, 16280)
    }
    if first_holm != {
        8120: {60: 49, 90: 69, 120: 87},
        16280: {60: 50, 90: 69, 120: 88},
    }:
        fail("benefit first-Holm critical counts changed")
    actual_family_sizes = {8120, 8140, 16240, 16280}
    if {
        len(registry.benefit_hypothesis_ids(branch["branch_id"], matrix))
        for branch in branches if branch["branch_class"] == "ACTIVE_POWER_BRANCH"
    } != actual_family_sizes:
        fail("actual benefit family sizes changed")
    verify_holm_cutoff_domain(tuple(sorted(actual_family_sizes)))
    noninferiority = stats["noninferiority_testing"]
    if noninferiority != registry.noninferiority_testing_protocol():
        fail("exact noninferiority protocol changed")
    if (
        noninferiority["strict_cell_success"]
        != "50*C.elapsed_ns < 51*D.elapsed_ns"
        or noninferiority["critical_success_counts"]
        != {"60": 44, "90": 63, "120": 80}
        or "semantic cells" not in noninferiority["null"]
        or "UNAVAILABLE" not in noninferiority["point_estimator"]
        or "p-value to 1/1" not in noninferiority["missing_nonfinite_rule"]
    ):
        fail("noninferiority cell/IUT rule changed")
    power = stats["power_calculation"]
    alternatives = power["injected_alternative_matrices"]
    expected_endpoint_specs = [
        {
            "endpoint_id": "END-RAW-TRACE-NS",
            "acceptance_upper_ratio": "90/100",
            "injected_true_ratio": "85/100",
            "eligibility_rule": "Every exact primary timed cell.",
        },
        {
            "endpoint_id": "END-PEAK-ACQUIRED-BYTES",
            "acceptance_upper_ratio": "85/100",
            "injected_true_ratio": "80/100",
            "eligibility_rule": (
                "Every non-ZST primary cell whose frozen reference-only Rust "
                "peak-acquired-byte value is positive. A zero Rust value remains "
                "a structural equality case and is never an injected benefit."
            ),
        },
    ]
    if power["benefit_endpoint_alternatives"] != expected_endpoint_specs:
        fail("endpoint-specific power alternatives changed")
    if alternatives != [
        registry.exact_alternative_matrix(
            candidate, spec["endpoint_id"]
        )
        for spec in expected_endpoint_specs
        for candidate in registry.CANDIDATES
    ]:
        fail("exact true-winner alternative matrices changed")
    if len(alternatives) != 10:
        fail("power does not cover five winners by two benefit endpoints")
    for alternative in alternatives:
        if not (
            Fraction(alternative["injected_true_ratio"])
            < Fraction(alternative["acceptance_upper_ratio"])
        ):
            fail("power alternative is not strictly inside acceptance bound")
    benefit_testing_sha256 = registry.digest_value(benefit_testing)
    if (
        power["benefit_testing_protocol_sha256"]
        != benefit_testing_sha256
        or power["noninferiority_testing_protocol_sha256"]
        != registry.digest_value(noninferiority)
        or "Q[j,s](z)" not in power["exact_pass_probability"]
        or "M(z)^r/(4*30^5)^r" not in power["exact_pass_probability"]
        or "17/20" not in power["injected_integer_ratios"]
        or "4/5" not in power["injected_integer_ratios"]
        or "single pilot cycle j" not in power["cluster_dependence_rule"]
        or "common to all" not in power["cluster_dependence_rule"]
        or "30 ordered numeric-symbol" not in power["mapping_integration"]
    ):
        fail("whole-cycle exact power law changed")
    event_schema = power["failure_event_ledger_schema"]
    if event_schema != registry.power_failure_event_ledger_schema():
        fail("failure-event ledger schema changed")
    event_classes = [row["event_class"] for row in event_schema]
    if event_classes != [
        "WINNER_OUTGOING_NI_CELL_FAILURE",
        "WINNER_REQUIRED_BENEFIT_FAILURE",
        "TIMING_REVERSE_NI_ERRONEOUS_PASS",
        "MEMORY_COMPLEMENT_BENEFIT_REJECTION_BOUND",
        "PROTOCOL_INVALIDITY",
    ] or len(event_classes) != len(set(event_classes)):
        fail("failure-event classes are incomplete or duplicated")
    if any(
        not row["key_fields"]
        or row["key_fields"][0] != "power_task_id"
        for row in event_schema
    ) or "power_task_id" not in power["failure_event_uniqueness"]:
        fail("failure events are not uniquely scoped by power task")
    law_protocol = power["law_key_manifest_protocol"]
    expected_law_identity_fields = [
        "branch_id", "cell_id", "target_id", "endpoint_id",
        "law_key_class_id",
    ]
    expected_law_binding_fields = [
        "orientation", "left_pilot_multiplier", "right_pilot_multiplier",
        "relative_injected_ratio", "acceptance_ratio",
        "strict_success_cross_product", "pilot_cycle_support_sha256",
        "q_polynomial_ledger_sha256", "p_polynomial_ledger_sha256",
        "m_polynomial_sha256", "law_n60_sha256", "law_n90_sha256",
        "law_n120_sha256",
    ]
    if (
        law_protocol["schema"] != "xlang-dense-power-law-key-manifest-v1"
        or law_protocol["key_count_formula"] != "5*T+3*M"
        or law_protocol["key_identity_fields"] != expected_law_identity_fields
        or len(law_protocol["key_identity_fields"])
        != len(set(law_protocol["key_identity_fields"]))
        or law_protocol["required_binding_fields"]
        != expected_law_binding_fields
        or law_protocol["key_classes"] != expected_power_law_classes()
        or "5*T+3*L" not in law_protocol["prepilot_identity_domain_rule"]
        or "non-ZST" not in law_protocol["prepilot_identity_domain_rule"]
        or "5*T+3*M" not in law_protocol["active_domain_rule"]
        or "structural-zero" not in law_protocol["active_domain_rule"]
    ):
        fail("power law-key class/domain protocol changed")
    task_protocol = power["power_task_manifest_protocol"]
    expected_task_identity_fields = [
        "owner_branch_id", "true_winner_id", "benefit_endpoint_id",
        "benefit_cell_id", "target_id", "selected_block_count",
        "injected_alternative_sha256", "selected_family_sha256",
        "power_engine_protocol_sha256",
    ]
    expected_task_common_bindings = [
        "reference_pilot_raw_sha256", "reference_assignment_manifest_sha256",
        "timing_whole_cycle_support_sha256",
        "memory_eligibility_ledger_sha256", "power_sign_table_key_manifest_sha256",
        "randomization_protocol_sha256",
    ]
    expected_task_endpoint_bindings = {
        "END-RAW-TRACE-NS": [
            "required_benefit_f_ids_sha256",
            "timing_reverse_ni_bound_ledger_sha256",
        ],
        "END-PEAK-ACQUIRED-BYTES": [
            "memory_whole_cycle_support_sha256",
            "benefit_partition_sha256",
            "benefit_category_sum_sha256",
        ],
    }
    if (
        task_protocol["schema"] != "xlang-dense-power-task-manifest-v1"
        or task_protocol["identity_fields"] != expected_task_identity_fields
        or len(task_protocol["identity_fields"])
        != len(set(task_protocol["identity_fields"]))
        or task_protocol["common_artifact_binding_fields"]
        != expected_task_common_bindings
        or task_protocol["endpoint_artifact_binding_fields"]
        != expected_task_endpoint_bindings
        or "No applicable binding may be PENDING" not in
        task_protocol["power_task_id_rule"]
        or "endpoint-inapplicable binding is forbidden" not in
        task_protocol["power_task_id_rule"]
        or "incremental SHA-256" not in task_protocol["event_stream_hash_rule"]
    ):
        fail("power-task identity or artifact bindings changed")
    category_protocol = power["benefit_partition_category_protocol"]
    if category_protocol != expected_benefit_partition_category_protocol():
        fail("benefit partition category algebra changed")
    verify_benefit_category_algebra()
    expected_engine_authority = {
        "schema": "xlang-dense-power-engine-protocol-authority-v1",
        "method": "EXACT_REFERENCE_EMPIRICAL_DP_NO_MONTE_CARLO",
        "scheduled_mixture_tail_protocol_sha256": registry.digest_value(
            estimand["scheduled_mixture_tail_protocol"]
        ),
        "benefit_testing_protocol_sha256": benefit_testing_sha256,
        "noninferiority_testing_protocol_sha256": registry.digest_value(
            noninferiority
        ),
        "randomization_protocol_sha256": registry.digest_value(
            stats["randomization_and_custody"]
        ),
        "law_key_manifest_protocol_sha256": registry.digest_value(law_protocol),
        "power_task_manifest_protocol_sha256": registry.digest_value(
            task_protocol
        ),
        "failure_event_ledger_schema_sha256": registry.digest_value(event_schema),
        "benefit_partition_category_protocol_sha256": registry.digest_value(
            category_protocol
        ),
        "resource_limits_sha256": registry.digest_value(power["resource_limits"]),
    }
    if (
        power["power_engine_protocol_authority"]
        != expected_engine_authority
        or power["power_engine_protocol_sha256"]
        != registry.digest_value(expected_engine_authority)
    ):
        fail("power-engine protocol authority changed")
    if (
        "|F|=4" not in power["benefit_partition_ledger"]
        or "|N|=m-4" not in power["benefit_partition_ledger"]
        or "ZERO" not in power["benefit_partition_ledger"]
        or "rank <=5" not in power["holm_power_rule"]
        or "1/(200*(m-4))" not in power["holm_power_rule"]
        or "not an exact joint probability" not in power["bound_interpretation"]
    ):
        fail("benefit partition or conservative bound changed")
    limits = power["resource_limits"]
    maximum_timing = 408
    maximum_memory = 406
    law_keys = 5 * maximum_timing + 3 * maximum_memory
    sign_rows = law_keys * 30 * 4 * 5
    comparisons = sign_rows * 6
    polynomial_operations = 15983 * law_keys
    coefficient_hash_updates = 568 * law_keys
    partition_inspections = 5 * maximum_memory * 16280
    event_terms = 25000020
    derived_total = (
        comparisons + polynomial_operations + coefficient_hash_updates
        + partition_inspections
        + sign_rows + 12210 + 3 * event_terms
    )
    if limits != registry.power_resource_limits() or (
        limits["maximum_sign_table_keys"] != law_keys
        or limits["maximum_sign_table_rows"] != sign_rows
        or limits["maximum_raw_integer_sign_comparisons"] != comparisons
        or limits["maximum_bigint_polynomial_coefficient_operations"]
        != polynomial_operations
        or limits["maximum_qpm_and_n_law_coefficient_ledger_hash_updates"]
        != coefficient_hash_updates
        or limits["maximum_memory_task_partition_identity_inspections"]
        != partition_inspections
        or limits["maximum_streamed_failure_event_terms"] != event_terms
        or limits["maximum_failure_event_identity_serializations_and_hashes"]
        != event_terms
        or limits["maximum_cached_probability_lookups"] != event_terms
        or limits["maximum_rational_union_bound_additions"] != event_terms
        or limits["derived_minimum_counted_operations"] != derived_total
        or limits["counted_operation_headroom"] != 200000000 - derived_total
        or limits["maximum_counted_primitive_operations"] != 200000000
    ):
        fail("power task or resource ceilings changed")
    verify_qpm_polynomial_identity()
    randomization = stats["randomization_and_custody"]
    if randomization != registry.randomization_protocol():
        fail("randomization/custody protocol changed")
    if (
        "two independent uniform 32-byte" not in randomization["seed_custody"]
        or "x>=65520" not in randomization["uniform_rank_sampling_protocol"]
        or "complete-cycle" not in randomization["mapping_scope"]
        or "layout salt" not in randomization["mapping_scope"]
        or "computationally pseudorandom" not in randomization["randomness_interpretation"]
        or "block-identity" not in randomization["global_block_order"]
        or "owner_branch_id" not in randomization["global_block_order"]
        or "williams_row_id" not in randomization["global_block_order"]
        or len(randomization["test_vectors"]["block_order_rows"]) != 3
    ):
        fail("randomization rank law or test vectors changed")
    vectors = randomization["test_vectors"]
    if (
        vectors["reference_seed_commitment_sha256"]
        != "630dcd2966c4336691125448bbb25b4ff412a49c732db2c8abc1b8581bd710dd"
        or vectors["candidate_seed_commitment_sha256"]
        != "72dbb7336c76780023f83da4c355f2eeea85733b13d3477697917790c1229084"
        or vectors["reference_root_message_sha256"]
        != "5e472b1d5bee6248d3cd549a8ceea74fa858626a5e520aa8e1b650b06264bdf1"
        or vectors["candidate_root_message_sha256"]
        != "3d39bf5936bcdf4d416c690754a3012d7c601ef668990ea1619ce81fa18c158f"
        or vectors["permutation_manifest_row_message_sha256"]
        != "791cac5e8198378f1cf05e94c318f961675981ba55936ba7d0f729628c9437e0"
        or [
            (row["block_id"], row["global_execution_rank"], row["key_sha256"])
            for row in vectors["block_order_rows"]
        ] != [
            (
                "BLOCK-86A565E45FB69BFEAD11C88A5C5DE8CF7A9535BD37F3C5E4DE203725DA94F4C3",
                1,
                "0a68421f2f051c550ab8544ac93dbd86ca51df8e3bcfc347dbbe881ac40b98ea",
            ),
            (
                "BLOCK-32F30D73705BF3721EDB83C9CAA1DCD20B9E9BE2C8FB9C57D627CB0321EACC82",
                2,
                "70330e0da0bd49700bf9b4e0abbd22c9af8aa7616a1d27f7a13aa8afd4091ec6",
            ),
            (
                "BLOCK-3825E458709F65627D5077993AF105A0E93607ADB8F2397B6DB76DF7068FAD2C",
                3,
                "f551d86946f2a1aa8b5fb2e42fb4c71b59f6ce7933f78a55bd3acd8b0c2073c3",
            ),
        ]
    ):
        fail("randomization byte grammar test vector changed")
    stale_keys = {
        "residual_covariance_and_carryover", "estimator_and_resampling",
        "power_simulation",
    }
    if stale_keys & set(stats):
        fail("stale fitted-adjustment statistics authority recurred")
    stats_text = json.dumps(stats, sort_keys=True)
    for forbidden in (
        "adjusted_log", "noninferiority_bootstrap", "1,000,000",
        "k_min", "Binomial(n,k_min/4)",
    ):
        if forbidden in stats_text:
            fail(f"stale statistics semantics recurred: {forbidden}")
    active = [
        row for row in branches
        if row["branch_class"] == "ACTIVE_POWER_BRANCH"
    ]
    plans = stats["branch_power_plans"]
    if len(plans) != len(active) or len(plans) != 8:
        fail("power is not specified for every owner branch")
    plan_by_branch = {row["branch_id"]: row for row in plans}
    for branch in active:
        plan = plan_by_branch.get(branch["branch_id"])
        if plan is None:
            fail(f"missing branch power plan: {branch['branch_id']}")
        benefits = registry.benefit_hypothesis_ids(
            branch["branch_id"], matrix
        )
        if (
            plan["primary_cell_count"] != int(branch["primary_cell_count"])
            or plan["primary_cell_ids_sha256"]
            != branch["primary_cell_ids_sha256"]
            or plan["global_benefit_hypothesis_count"] != len(benefits)
            or plan["global_benefit_hypothesis_ids_sha256"]
            != registry.digest_value(benefits)
            or plan["benefit_testing_protocol_sha256"]
            != benefit_testing_sha256
        ):
            fail(f"branch power family mismatch: {branch['branch_id']}")
        pending_endpoint_fields = (
            "memory_eligibility_ledger_sha256",
            "timing_whole_cycle_support_sha256",
            "memory_whole_cycle_support_sha256",
            "descriptive_nuisance_summary_sha256",
            "power_sign_table_key_manifest_sha256",
            "power_failure_event_ledger_sha256",
            "power_benefit_partition_ledger_sha256",
            "power_task_manifest_sha256",
            "power_engine_source_sha256",
            "power_engine_binary_sha256",
            "power_engine_compiler_and_flags_sha256",
            "power_engine_resource_result_sha256",
        )
        if any(
            plan[field] != registry.PENDING
            for field in pending_endpoint_fields
        ):
            fail(f"branch endpoint power artifacts changed: {branch['branch_id']}")
        cells = [
            row for row in matrix
            if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
            and branch["branch_id"] in row["owner_branch_ids"].split(",")
        ]
        memory_cells = [
            row for row in cells if row["payload_id"] != "P-ZST-AFFINE"
        ]
        expected_law_rows = expected_maximum_power_law_key_identity_rows(
            branch["branch_id"], matrix
        )
        expected_task_rows = expected_maximum_power_task_identity_rows(
            branch["branch_id"], matrix,
            power["power_engine_protocol_sha256"],
        )
        if (
            plan["power_law_key_manifest_protocol_sha256"]
            != registry.digest_value(law_protocol)
            or plan["maximum_power_law_key_identity_count"]
            != len(expected_law_rows)
            or plan["maximum_power_law_key_identity_domain_sha256"]
            != registry.digest_value(expected_law_rows)
            or plan["power_task_manifest_protocol_sha256"]
            != registry.digest_value(task_protocol)
            or plan["benefit_partition_category_protocol_sha256"]
            != registry.digest_value(category_protocol)
            or plan["power_engine_protocol_sha256"]
            != power["power_engine_protocol_sha256"]
            or plan["maximum_power_task_identity_count"]
            != len(expected_task_rows)
            or plan["maximum_power_task_identity_domain_sha256"]
            != registry.digest_value(expected_task_rows)
            or len({
                tuple(str(row[field]) for field in expected_task_identity_fields)
                for row in expected_task_rows
            }) != len(expected_task_rows)
        ):
            fail(f"branch power law/task identity domain changed: {branch['branch_id']}")
        timing_events = 5 * len(cells) + 9
        memory_events = 5 * len(cells) + 6
        per_n = (
            5 * len(cells) * timing_events
            + 5 * len(memory_cells) * memory_events
        )
        if (
            plan["positive_reference_memory_cell_max_count"] != len(memory_cells)
            or plan["maximum_alternatives_per_block_count"]
            != 5 * (len(cells) + len(memory_cells))
            or plan["maximum_alternative_block_tasks"]
            != len(expected_task_rows)
            or plan["failure_event_terms_per_block_count"] != per_n
            or plan["maximum_streamed_failure_event_terms"] != 3 * per_n
        ):
            fail(f"branch task/event domain changed: {branch['branch_id']}")
    selection = stats["selection"]
    if (
        "1/5000" not in selection["rust_floor"]
        or "No semantic cell is pooled" not in selection["rust_floor"]
        or "never pooled" not in selection["no_pooling"]
        or "exactly five" not in
        selection["qualifying_candidate_rule"]["outgoing_ni_claims"]
        or "never require all 25" not in
        selection["qualifying_candidate_rule"]["all_25_role"]
    ):
        fail("atomic Rust floor or no-pooling rule weakened")
    no_selection = selection["no_selection"]
    for fragment in (
        "qualifying set S", "Select only when |S|=1", "If |S| != 1",
        "|S|=0", "2 through 5", "NO-SELECTION",
    ):
        if fragment not in no_selection:
            fail("no-selection cardinality rule is incomplete")
    if "ZST allocator bytes" not in stats["endpoint_ratio_rules"]["memory_zero_rule"]:
        fail("ZST zero-byte structural rule changed")
    schemas = stats["raw_sample_schemas"]
    random_fields = {
        "randomization_commitment_sha256", "assignment_manifest_sha256",
        "assignment_row_id", "global_execution_rank",
    }
    if (
        not random_fields <= set(schemas["reference_pilot_required_fields"])
        or not random_fields <= set(schemas["candidate_primary_required_fields"])
        or "pseudo_treatment_id" not in schemas["reference_pilot_required_fields"]
        or "candidate_freeze_b_sha256" in
        schemas["reference_pilot_required_fields"]
        or "candidate_freeze_b_sha256" not in
        schemas["candidate_primary_required_fields"]
        or "MISSING_RAW_ROW" not in schemas["unusable_row_sentinel_rule"]
        or "pseudo ID" not in schemas["manifest_join_rule"]
    ):
        fail("pilot/candidate raw schema or manifest joins changed")
    timeouts = stats["execution_timeouts"]
    if (
        timeouts != registry.execution_timeout_protocol()
        or "No retry" not in timeouts["retry_rule"]
        or timeouts["per_child_timeout_ns"]
        != "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL"
    ):
        fail("timeout disposition or external budget boundary changed")
    construction = stats["construction_protocol"]
    if (
        construction["arbitrary_resource_default_forbidden"] is not True
        or "mechanism result" not in construction["mechanism_failure_rule"]
        or not {
            "PENDING_EXTERNAL_OWNER_AUTHORIZATION",
            "PENDING_EXTERNAL_REPOSITORY_BASELINE",
            "PENDING_EXTERNAL_REFERENCE_PILOT",
            "PENDING_EXTERNAL_RANDOMIZATION_CUSTODY",
            "PENDING_EXTERNAL_POWER_ENGINE_RESOURCE_PROTOCOL",
        } <= set(construction["required_blocker_ids"])
    ):
        fail("candidate construction resource/failure boundary weakened")


def validate_endpoints_and_counters() -> None:
    endpoints = read_tsv("endpoints")
    required_latency = {
        f"END-{scope}-LATENCY-{quantile}"
        for scope in ("TRACE", "OP") for quantile in ("P50", "P95", "P99")
    }
    by_id = {row["endpoint_id"]: row for row in endpoints}
    expected_primary_estimators = {
        "END-RAW-TRACE-NS": (
            "Exact scheduled-mixture count of strict raw-integer successes "
            "under the registered timing cross-product"
        ),
        "END-PEAK-ACQUIRED-BYTES": (
            "Exact scheduled-mixture count of strict raw-integer successes "
            "under the registered memory cross-product and zero-byte rule"
        ),
    }
    if any(
        by_id.get(endpoint_id, {}).get("estimator") != estimator
        for endpoint_id, estimator in expected_primary_estimators.items()
    ):
        fail("primary endpoint estimator contradicts raw-integer inference")
    if not required_latency <= set(by_id):
        fail("p50/p95/p99 trace/op endpoints are incomplete")
    for endpoint_id in required_latency:
        row = by_id[endpoint_id]
        if (
            row["selection_role"] != "DESCRIPTIVE_ONLY"
            or "never enters selection" not in row["pooling_rule"]
        ):
            fail(f"descriptive latency endpoint changed: {endpoint_id}")
    counters = read_tsv("counter_policies")
    if {row["target_id"] for row in counters} != set(registry.ALL_TARGETS):
        fail("counter policy does not cover every target")
    for row in counters:
        unavailable_rule = row["unavailable_counter_rule"].lower()
        if (
            "zero" not in unavailable_rule
            or not any(
                fragment in unavailable_rule
                for fragment in ("never write zero", "no zero")
            )
        ):
            fail(f"counter missing-data rule permits zero: {row['target_id']}")


def validate_descriptors(
    matrix: list[dict[str, str]],
    descriptors: list[dict[str, Any]],
    inputs: list[dict[str, Any]],
) -> None:
    if len(matrix) != len(descriptors) or len(matrix) != len(inputs):
        fail("matrix, descriptors, and generated inputs differ in count")
    if (
        [row["cell_id"] for row in matrix]
        != [row["cell_id"] for row in descriptors]
        or [row["cell_id"] for row in matrix]
        != [row["cell_id"] for row in inputs]
    ):
        fail("ordered cell IDs differ across matrix and inputs")
    for cell, descriptor, generated in zip(matrix, descriptors, inputs):
        if (
            descriptor["candidate_execution_authorized"] is not False
            or generated["candidate_execution_authorized"] is not False
        ):
            fail(f"input authorizes candidate execution: {cell['cell_id']}")
        if (
            registry.digest_value(descriptor["trace_plan"])
            != cell["trace_sha256"]
            or registry.digest_value(descriptor["oracle_plan"])
            != cell["oracle_sha256"]
            or registry.digest_value(descriptor)
            != generated["descriptor_sha256"]
        ):
            fail(f"input digest mismatch: {cell['cell_id']}")


def verify_source_pins() -> None:
    references = read_tsv("references")
    for row in references:
        if row["rawvec_sha256"] != registry.RAWVEC_SHA256:
            fail(f"RawVec pin changed: {row['reference_route_id']}")
        if row["rustc_sha256"] != registry.RUSTC_SHA256:
            fail(f"rustc pin changed: {row['reference_route_id']}")
        if row["baseline_commit"] != registry.BASELINE_COMMIT:
            fail(f"baseline commit changed: {row['reference_route_id']}")
        if row["rust_version"] != "NOT_APPLICABLE" and (
            row["rust_version"] != registry.RUST_VERSION
            or row["rust_commit"] != registry.RUST_COMMIT
        ):
            fail(f"Rust source pin changed: {row['reference_route_id']}")
    controls = {row["control_id"]: row for row in read_tsv("controls")}
    if sha256(git_bytes(
        registry.BASELINE_COMMIT, "prototype/democ/examples/soa_kernel.xl"
    )) != controls["B-FIX"]["source_sha256"]:
        fail("B-FIX source authority changed")
    source_list = git_bytes(
        registry.BASELINE_COMMIT, "compiler/sources.txt"
    ).decode("utf-8").splitlines()
    pieces = []
    for spelling in source_list:
        spelling = spelling.strip()
        if spelling:
            pieces.append(
                git_bytes(
                    registry.BASELINE_COMMIT, f"compiler/{spelling}"
                ).decode("utf-8").rstrip("\n")
            )
    canonical = ("\n\n".join(pieces) + "\n").encode("ascii")
    if sha256(canonical) != controls["B-P2"]["source_sha256"]:
        fail("B-P2 canonical source authority changed")


def verify_summary() -> None:
    summary = read_json("summary")
    if (
        summary["schema"] != "xlang-dense-performance-registry-summary-v5"
        or summary["candidate_construction_authorized"] is not False
        or summary["exact_contract_count"] != 303
        or summary["operation_gate_count"] != 97
        or summary["matrix_cell_count"] != 520
        or summary["active_owner_branch_count"] != 8
        or summary["explicit_blocker_count"] != 27
    ):
        fail("registry summary counts or boundary changed")
    for entry in summary["artifacts"]:
        path = HERE / entry["path"]
        data = path.read_bytes()
        if len(data) != entry["bytes"] or sha256(data) != entry["sha256"]:
            fail(f"summary artifact hash mismatch: {entry['path']}")
    for entry in summary["source_authorities"]:
        path = HERE / entry["path"]
        data = path.read_bytes()
        if len(data) != entry["bytes"] or sha256(data) != entry["sha256"]:
            fail(f"summary source hash mismatch: {entry['path']}")
    required = {
        *registry.OUTPUTS.values(),
        "dense_performance_registry.py",
        "verify_dense_performance.py",
        "generate_dense_performance_inputs.py",
    }
    for spelling in required:
        data = (HERE / spelling).read_bytes()
        try:
            data.decode("ascii")
        except UnicodeDecodeError:
            fail(f"performance artifact is not English/ASCII: {spelling}")
    generated_text = "\n".join(
        (HERE / spelling).read_text(encoding="ascii")
        for spelling in registry.OUTPUTS.values()
    )
    if "TIMED_SECONDARY" in generated_text:
        fail("generated artifacts retain forbidden secondary timing")


def expect_mutation_failure(
    name: str,
    action: Callable[[], None],
) -> None:
    try:
        action()
    except VerificationError:
        return
    fail(f"mutation was not rejected: {name}")


def run_mutation_tests(bundle: dict[str, Any]) -> int:
    tests: list[tuple[str, Callable[[], None]]] = []
    contracts = bundle["contracts"]

    def dispositions_case(
        mutate: Callable[
            [list[dict[str, str]], list[dict[str, str]]], None
        ]
    ) -> None:
        authority = copy.deepcopy(bundle["authority"])
        dispositions = copy.deepcopy(bundle["dispositions"])
        mutate(authority, dispositions)
        validate_dispositions(contracts, authority, dispositions)

    tests.append((
        "missing exact disposition",
        lambda: dispositions_case(lambda a, d: d.pop()),
    ))
    tests.append((
        "forbidden secondary disposition",
        lambda: dispositions_case(
            lambda a, d: d[0].__setitem__("disposition", "TIMED_SECONDARY")
        ),
    ))
    tests.append((
        "blank exact derivation",
        lambda: dispositions_case(
            lambda a, d: d[0].__setitem__("exact_reason", "")
        ),
    ))

    def matrix_case(
        mutate: Callable[
            [list[dict[str, str]], list[dict[str, str]]], None
        ]
    ) -> None:
        gates = copy.deepcopy(bundle["gates"])
        matrix = copy.deepcopy(bundle["matrix"])
        mutate(gates, matrix)
        validate_gates_and_matrix(bundle["dispositions"], gates, matrix)
        validate_payloads_and_zst(matrix)
        validate_protected_controls(matrix)

    tests.append((
        "missing primary cell",
        lambda: matrix_case(lambda g, m: m.pop()),
    ))
    tests.append((
        "aggregate Rust floor",
        lambda: matrix_case(
            lambda g, m: next(
                row for row in m
                if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
            ).__setitem__("rust_floor_upper_ratio", "AGGREGATE")
        ),
    ))
    tests.append((
        "missing sort shape",
        lambda: matrix_case(
            lambda g, m: m.__setitem__(
                slice(None),
                [
                    row for row in m
                    if not (
                        row["member_contract_id"] == "DENSE-SORT-STABLE"
                        and row["shape_id"] == "SORT-REVERSE"
                    )
                ],
            )
        ),
    ))
    tests.append((
        "missing payload separator",
        lambda: matrix_case(
            lambda g, m: m.__setitem__(
                slice(None),
                [
                    row for row in m
                    if not (
                        row["cell_role"] == "PAYLOAD_SEPARATOR_PRIMARY"
                        and row["payload_code"] == "ROW56"
                    )
                ],
            )
        ),
    ))
    tests.append((
        "ZST allocator bytes route",
        lambda: matrix_case(
            lambda g, m: next(
                row for row in m if row["payload_id"] == "P-ZST-AFFINE"
            ).__setitem__(
                "allocator_id", "ALLOC-COMMON-COUNTED-SYSTEM-V1"
            )
        ),
    ))
    tests.append((
        "missing protected layout",
        lambda: matrix_case(
            lambda g, m: m.__setitem__(
                slice(None),
                [
                    row for row in m
                    if not (
                        row["member_contract_id"] == "B-FIX"
                        and row["target_id"] == "TARGET-I686-STRUCTURAL"
                    )
                ],
            )
        ),
    ))
    tests.append((
        "execution authorization",
        lambda: matrix_case(
            lambda g, m: m[0].__setitem__(
                "candidate_execution_authorized", "YES"
            )
        ),
    ))
    tests.append((
        "primary cell missing repository baseline",
        lambda: matrix_case(
            lambda g, m: next(
                row for row in m
                if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
            ).__setitem__(
                "blocker_ids",
                ",".join(
                    blocker_id
                    for blocker_id in next(
                        row for row in m
                        if row["primary_endpoint_id"] == "END-RAW-TRACE-NS"
                    )["blocker_ids"].split(",")
                    if blocker_id != "PENDING_EXTERNAL_REPOSITORY_BASELINE"
                ),
            )
        ),
    ))
    tests.append((
        "structural cell missing repository baseline",
        lambda: matrix_case(
            lambda g, m: next(
                row for row in m
                if row["primary_endpoint_id"] != "END-RAW-TRACE-NS"
            ).__setitem__(
                "blocker_ids",
                ",".join(
                    blocker_id
                    for blocker_id in next(
                        row for row in m
                        if row["primary_endpoint_id"] != "END-RAW-TRACE-NS"
                    )["blocker_ids"].split(",")
                    if blocker_id != "PENDING_EXTERNAL_REPOSITORY_BASELINE"
                ),
            )
        ),
    ))
    tests.append((
        "matrix cell unknown blocker",
        lambda: matrix_case(
            lambda g, m: m[0].__setitem__(
                "blocker_ids", m[0]["blocker_ids"] + ",PENDING_EXTERNAL_UNKNOWN"
            )
        ),
    ))
    tests.append((
        "Mac-local cell imports dual-native blocker",
        lambda: matrix_case(
            lambda g, m: next(
                row for row in m
                if row["target_id"] == "TARGET-AARCH64-DARWIN"
            ).__setitem__(
                "blocker_ids",
                next(
                    row for row in m
                    if row["target_id"] == "TARGET-AARCH64-DARWIN"
                )["blocker_ids"] + ",PENDING_EXTERNAL_X86_RUNNER",
            )
        ),
    ))

    def branch_case(
        mutate: Callable[[list[dict[str, str]]], None]
    ) -> None:
        branches = copy.deepcopy(bundle["branches"])
        mutate(branches)
        validate_branches(branches, bundle["matrix"])

    tests.append((
        "missing owner branch",
        lambda: branch_case(lambda rows: rows.pop(0)),
    ))
    tests.append((
        "active crossover",
        lambda: branch_case(
            lambda rows: next(
                row for row in rows
                if row["branch_class"] == "ACTIVE_POWER_BRANCH"
            ).__setitem__("od5_option_id", "OD-5-ENUMERATED-CROSSOVER")
        ),
    ))

    def substrate_case(
        mutate: Callable[[list[dict[str, str]]], None]
    ) -> None:
        rows = copy.deepcopy(registry.common_substrate_rows())
        mutate(rows)
        validate_substrate(rows)

    tests.append((
        "private substrate cost",
        lambda: substrate_case(
            lambda rows: rows[0].__setitem__(
                "cost_model_sha256", "0" * 64
            )
        ),
    ))

    def blocker_case(
        mutate: Callable[[list[dict[str, str]]], None]
    ) -> None:
        blockers = copy.deepcopy(registry.blocker_rows())
        mutate(blockers)
        validate_stage_prerequisites(
            blockers, bundle["statistics"], bundle["branches"]
        )

    tests.append((
        "late pilot authorization gate",
        lambda: blocker_case(
            lambda rows: next(
                row for row in rows
                if row["blocker_id"]
                == "PENDING_EXTERNAL_OWNER_AUTHORIZATION"
            ).__setitem__("earliest_blocked_stage", "CANDIDATE_CONSTRUCTION")
        ),
    ))
    tests.append((
        "x86 runner applied to Mac-local branches",
        lambda: blocker_case(
            lambda rows: next(
                row for row in rows
                if row["blocker_id"] == "PENDING_EXTERNAL_X86_RUNNER"
            ).__setitem__(
                "applicable_owner_branch_ids",
                ",".join(registry.ALL_ACTIVE_BRANCH_IDS),
            )
        ),
    ))
    tests.append((
        "missing repository baseline gate",
        lambda: blocker_case(
            lambda rows: rows.__setitem__(
                slice(None),
                [
                    row for row in rows
                    if row["blocker_id"]
                    != "PENDING_EXTERNAL_REPOSITORY_BASELINE"
                ],
            )
        ),
    ))
    tests.append((
        "recombined OD4 pilot and candidate blockers",
        lambda: blocker_case(
            lambda rows: rows.__setitem__(
                slice(None),
                [
                    row for row in rows
                    if row["blocker_id"]
                    != "PENDING_EXTERNAL_OD4_CANDIDATE_ARTIFACTS"
                ],
            )
        ),
    ))

    def stats_case(mutate: Callable[[dict[str, Any]], None]) -> None:
        stats = copy.deepcopy(bundle["statistics"])
        mutate(stats)
        validate_statistics(stats, bundle["branches"], bundle["matrix"])

    tests.append((
        "global alpha inflation",
        lambda: stats_case(
            lambda stats: stats["multiplicity"].__setitem__(
                "global_family_total_alpha", "0.02"
            )
        ),
    ))
    tests.append((
        "pilot prerequisite removed from stage manifest",
        lambda: stats_case(
            lambda stats: next(iter(
                stats["stage_prerequisite_protocol"]["per_owner_branch"].values()
            ))["pipeline"]["REFERENCE_PILOT"]["direct_blocker_ids"].remove(
                "PENDING_EXTERNAL_OWNER_AUTHORIZATION"
            )
        ),
    ))
    tests.append((
        "pairwise alpha reset",
        lambda: stats_case(
            lambda stats: stats["multiplicity"].__setitem__(
                "benefit_method", "Separate Holm family for each pair"
            )
        ),
    ))
    tests.append((
        "nonidentical Rust pseudo build",
        lambda: stats_case(
            lambda stats: stats["reference_only_pilot"][
                "pseudo_treatments"
            ][0].__setitem__("executable_sha256", "DIFFERENT")
        ),
    ))
    tests.append((
        "missing true-winner alternative",
        lambda: stats_case(
            lambda stats: stats["power_calculation"][
                "injected_alternative_matrices"
            ].pop()
        ),
    ))
    tests.append((
        "memory alternative on acceptance boundary",
        lambda: stats_case(
            lambda stats: next(
                row for row in stats["power_calculation"][
                    "injected_alternative_matrices"
                ]
                if row["benefit_endpoint_id"]
                == "END-PEAK-ACQUIRED-BYTES"
            ).__setitem__("injected_true_ratio", "85/100")
        ),
    ))
    tests.append((
        "missing memory reference response",
        lambda: stats_case(
            lambda stats: stats["inferential_estimand"][
                "endpoint_responses"
            ].pop()
        ),
    ))
    tests.append((
        "empirical benefit p-value",
        lambda: stats_case(
            lambda stats: stats["benefit_testing"].__setitem__(
                "one_sided_p_value",
                "Estimate p with 1,000,000 bootstrap resamples.",
            )
        ),
    ))
    tests.append((
        "inexact Holm reset",
        lambda: stats_case(
            lambda stats: stats["benefit_testing"].__setitem__(
                "holm_step_down",
                "Reset a floating-point Holm family for each candidate pair.",
            )
        ),
    ))
    tests.append((
        "slotwise iid power law",
        lambda: stats_case(
            lambda stats: stats["power_calculation"].__setitem__(
                "exact_pass_probability", "Use Binomial(n,k/4) per slot."
            )
        ),
    ))
    tests.append((
        "fitted adjustment recurrence",
        lambda: stats_case(
            lambda stats: stats.__setitem__(
                "residual_covariance_and_carryover", {"fit": "period adjustment"}
            )
        ),
    ))
    tests.append((
        "noninferiority floating threshold",
        lambda: stats_case(
            lambda stats: stats["noninferiority_testing"].__setitem__(
                "strict_cell_success", "log(C/D) < log(1.02)"
            )
        ),
    ))
    tests.append((
        "stratumwise null mismatch",
        lambda: stats_case(
            lambda stats: stats["inferential_estimand"][
                "scheduled_mixture_tail_protocol"
            ].__setitem__("null", "Every nuisance stratum must pass.")
        ),
    ))
    tests.append((
        "adaptive reference pilot",
        lambda: stats_case(
            lambda stats: stats["reference_only_pilot"].__setitem__(
                "fixed_crossed_cycles_per_cell_target", 3
            )
        ),
    ))
    tests.append((
        "all-25 qualification conjunction",
        lambda: stats_case(
            lambda stats: stats["selection"]["qualifying_candidate_rule"].__setitem__(
                "all_25_role", "Require all 25 claims for every candidate."
            )
        ),
    ))
    tests.append((
        "minimum marginal called joint power",
        lambda: stats_case(
            lambda stats: stats["power_calculation"].__setitem__(
                "bound_interpretation", "Minimum marginal is exact joint power."
            )
        ),
    ))
    tests.append((
        "nominal FWER power import",
        lambda: stats_case(
            lambda stats: stats["power_calculation"][
                "failure_event_ledger_schema"
            ][3].__setitem__("event", "Subtract nominal benefit FWER 1/200.")
        ),
    ))
    tests.append((
        "missing complement partition",
        lambda: stats_case(
            lambda stats: stats["power_calculation"].__setitem__(
                "benefit_partition_ledger", "Four benefits only."
            )
        ),
    ))
    tests.append((
        "duplicate failure event class",
        lambda: stats_case(
            lambda stats: stats["power_calculation"][
                "failure_event_ledger_schema"
            ].append(copy.deepcopy(stats["power_calculation"][
                "failure_event_ledger_schema"
            ][0]))
        ),
    ))
    tests.append((
        "single randomization seed",
        lambda: stats_case(
            lambda stats: stats["randomization_and_custody"].__setitem__(
                "seed_custody", "Use one public seed."
            )
        ),
    ))
    tests.append((
        "mapping redrawn by Williams row",
        lambda: stats_case(
            lambda stats: stats["randomization_and_custody"].__setitem__(
                "mapping_scope", "Draw a new mapping for every Williams row."
            )
        ),
    ))
    tests.append((
        "static pilot pseudo mapping",
        lambda: stats_case(
            lambda stats: stats["reference_only_pilot"].__setitem__(
                "pseudo_to_numeric_symbol_mapping", "STATIC"
            )
        ),
    ))
    tests.append((
        "missing raw manifest rank",
        lambda: stats_case(
            lambda stats: stats["raw_sample_schemas"][
                "reference_pilot_required_fields"
            ].remove("global_execution_rank")
        ),
    ))
    tests.append((
        "invented timeout default",
        lambda: stats_case(
            lambda stats: stats["execution_timeouts"].__setitem__(
                "per_child_timeout_ns", 300000000000
            )
        ),
    ))
    tests.append((
        "underbounded sign comparisons",
        lambda: stats_case(
            lambda stats: stats["power_calculation"]["resource_limits"].__setitem__(
                "maximum_raw_integer_sign_comparisons", 1000
            )
        ),
    ))
    tests.append((
        "incomplete no-selection cardinality",
        lambda: stats_case(
            lambda stats: stats["selection"].__setitem__(
                "no_selection",
                "If two candidates tie, return NO-SELECTION.",
            )
        ),
    ))
    tests.append((
        "arbitrary construction resource",
        lambda: stats_case(
            lambda stats: stats["construction_protocol"].__setitem__(
                "arbitrary_resource_default_forbidden", False
            )
        ),
    ))
    for name, action in tests:
        expect_mutation_failure(name, action)
    return len(tests)


def main() -> None:
    bundle = expected_bundle()
    verify_freshness(bundle)
    validate_dispositions(
        bundle["contracts"], bundle["authority"], bundle["dispositions"]
    )
    validate_gates_and_matrix(
        bundle["dispositions"], bundle["gates"], bundle["matrix"]
    )
    validate_payloads_and_zst(bundle["matrix"])
    validate_branches(bundle["branches"], bundle["matrix"])
    validate_substrate(registry.common_substrate_rows())
    validate_protected_controls(bundle["matrix"])
    validate_statistics(
        bundle["statistics"], bundle["branches"], bundle["matrix"]
    )
    verify_benefit_partition_domains(bundle["branches"], bundle["matrix"])
    verify_maximum_power_task_event_domains(
        bundle["branches"], bundle["matrix"], bundle["statistics"]
    )
    validate_endpoints_and_counters()
    validate_descriptors(
        bundle["matrix"], bundle["descriptors"], bundle["inputs"]
    )
    verify_source_pins()
    verify_summary()
    mutation_count = run_mutation_tests(bundle)
    print(
        "Dense performance protocol verification: PASS - "
        f"{len(bundle['dispositions'])} exact derivations, "
        f"{len(bundle['gates'])} operation gates, "
        f"{len(bundle['matrix'])} cells, 8 owner power branches, "
        f"{mutation_count} hostile mutations rejected"
    )


if __name__ == "__main__":
    main()
