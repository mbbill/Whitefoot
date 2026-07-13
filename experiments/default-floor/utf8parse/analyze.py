#!/usr/bin/env python3
"""Analyze one complete preregistered utf8parse scoring campaign."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
import sys
from typing import Any, Iterable


MASK64 = (1 << 64) - 1
BOOTSTRAP_SEED = 0x5044424F4F5432
BOOTSTRAP_RESAMPLES = 10_000
ORDER_SEED = 0x50444F5244455233
LOWER_INDEX = 249
UPPER_INDEX = 9_749
SCORE_BYTES = 134_217_728
VARIANTS = ("facts-on", "facts-off", "rust")
ORDER_STRATA = (
    "facts-on,facts-off,rust",
    "facts-on,rust,facts-off",
    "facts-off,facts-on,rust",
    "facts-off,rust,facts-on",
    "rust,facts-on,facts-off",
    "rust,facts-off,facts-on",
)


class XorShift64Star:
    def __init__(self, seed: int) -> None:
        self.state = seed

    def next(self) -> int:
        self.state ^= self.state >> 12
        self.state ^= (self.state << 25) & MASK64
        self.state ^= self.state >> 27
        self.state &= MASK64
        return (self.state * 2_685_821_657_736_338_717) & MASK64


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def median(values: Iterable[float]) -> float:
    ordered = sorted(values)
    if not ordered:
        raise ValueError("median of empty data")
    middle = len(ordered) // 2
    if len(ordered) % 2:
        return ordered[middle]
    return (ordered[middle - 1] + ordered[middle]) / 2.0


def mad_over_median(values: list[float]) -> float:
    center = median(values)
    if center == 0:
        raise ValueError("zero throughput median")
    return median(abs(value - center) for value in values) / center


def expected_schedule() -> list[str]:
    orders = list(ORDER_STRATA) * 5
    rng = XorShift64Star(ORDER_SEED)
    for index in range(29, 0, -1):
        swap = rng.next() % (index + 1)
        orders[index], orders[swap] = orders[swap], orders[index]
    return orders


def verdict(interval: list[float], winner: str, loser: str) -> str:
    lower, upper = interval
    if lower > 1.02:
        return f"meaningful {winner} win"
    if lower >= 0.98 and upper <= 1.02:
        return "practical parity"
    if upper < 0.98:
        return f"meaningful {loser} win"
    return "inconclusive against the 2% band"


def read_json(path: Path) -> Any:
    return json.loads(path.read_text(encoding="utf-8"))


def read_raw(path: Path) -> list[dict[str, Any]]:
    records: list[dict[str, Any]] = []
    with path.open(encoding="utf-8") as source:
        for line_number, line in enumerate(source, 1):
            if not line.strip():
                raise ValueError(f"blank raw row at line {line_number}")
            value = json.loads(line)
            if not isinstance(value, dict):
                raise ValueError(f"raw line {line_number} is not an object")
            records.append(value)
    return records


def validate_campaign(
    campaign: Path,
) -> tuple[dict[str, Any], dict[str, Any], list[dict[str, Any]]]:
    metadata = read_json(campaign / "metadata.json")
    schedule = read_json(campaign / "schedule.json")
    raw_path = campaign / "raw.jsonl"
    records = read_raw(raw_path)
    if metadata.get("mode") != "score" or metadata.get("not_a_score") is not False:
        raise ValueError("analysis refuses non-scoring campaign metadata")
    if metadata.get("status") != "complete":
        raise ValueError("analysis requires a complete campaign")
    if metadata.get("raw_sha256") != sha256_file(raw_path):
        raise ValueError("raw sample SHA-256 does not match metadata")
    if metadata.get("schedule_sha256") != sha256_file(campaign / "schedule.json"):
        raise ValueError("schedule SHA-256 does not match metadata")
    if schedule.get("strata_order") != list(ORDER_STRATA):
        raise ValueError("schedule uses the wrong order-stratum identity")
    if schedule.get("repetitions_per_stratum") != 5:
        raise ValueError("score schedule must repeat each order exactly five times")
    scheduled = schedule.get("orders")
    if not isinstance(scheduled, list) or len(scheduled) != 30 or len(records) != 30:
        raise ValueError("score requires exactly 30 scheduled and recorded blocks")
    scheduled_orders = [row.get("order") for row in scheduled if isinstance(row, dict)]
    if scheduled_orders != expected_schedule():
        raise ValueError("schedule does not match the frozen seeded Fisher-Yates vector")

    counts = {stratum: 0 for stratum in ORDER_STRATA}
    positions = {
        variant: {ordinal: 0 for ordinal in range(3)} for variant in VARIANTS
    }
    for index, (expected, record) in enumerate(zip(scheduled, records)):
        order = expected.get("order")
        if expected.get("block_index") != index or order not in counts:
            raise ValueError(f"invalid schedule row {index}")
        if (
            record.get("block_index") != index
            or record.get("order") != order
            or record.get("order_stratum") != order
            or record.get("not_a_score") is not False
        ):
            raise ValueError(f"raw block {index} does not match schedule")
        if record.get("power_or_thermal_transition") is not None:
            raise ValueError(f"raw block {index} recorded a power or thermal transition")
        samples = record.get("samples")
        if not isinstance(samples, list) or len(samples) != 3:
            raise ValueError(f"raw block {index} lacks exactly three samples")
        sample_map: dict[str, dict[str, Any]] = {}
        expected_order = order.split(",")
        for ordinal, sample in enumerate(samples):
            if (
                not isinstance(sample, dict)
                or sample.get("variant") != expected_order[ordinal]
                or sample.get("ordinal") != ordinal
                or sample.get("input_bytes") != SCORE_BYTES
                or type(sample.get("elapsed_ns")) is not int
                or sample["elapsed_ns"] <= 0
                or type(sample.get("output_events")) is not int
                or not 0 <= sample["output_events"] <= SCORE_BYTES
                or not isinstance(sample.get("output_sha256"), str)
                or len(sample["output_sha256"]) != 64
                or any(
                    byte not in "0123456789abcdef"
                    for byte in sample["output_sha256"]
                )
            ):
                raise ValueError(f"malformed sample in raw block {index}")
            sample_map[sample["variant"]] = sample
            positions[sample["variant"]][ordinal] += 1
        if set(sample_map) != set(VARIANTS):
            raise ValueError(f"raw block {index} does not contain each variant once")
        if len({sample["output_events"] for sample in samples}) != 1:
            raise ValueError(f"raw block {index} returned different output lengths")
        if len({sample["output_sha256"] for sample in samples}) != 1:
            raise ValueError(f"raw block {index} returned different output digests")
        counts[order] += 1
        record["_sample_map"] = sample_map

    if any(count != 5 for count in counts.values()):
        raise ValueError(f"order strata are not balanced: {counts}")
    for variant in VARIANTS:
        if any(positions[variant][ordinal] != 10 for ordinal in range(3)):
            raise ValueError(f"ordinal positions are not balanced for {variant}")
    return metadata, schedule, records


def throughput(sample: dict[str, Any]) -> float:
    return sample["input_bytes"] * 1_000_000_000.0 / sample["elapsed_ns"]


def bootstrap(records: list[dict[str, Any]]) -> tuple[list[float], list[float]]:
    groups = {
        stratum: [record for record in records if record["order_stratum"] == stratum]
        for stratum in ORDER_STRATA
    }
    if any(len(group) != 5 for group in groups.values()):
        raise ValueError("bootstrap strata do not each contain five blocks")
    rng = XorShift64Star(BOOTSTRAP_SEED)
    primary: list[float] = []
    attribution: list[float] = []
    for _ in range(BOOTSTRAP_RESAMPLES):
        primary_draw: list[float] = []
        attribution_draw: list[float] = []
        # Frozen stratum visitation order. The exact same selected records feed
        # both ratios, so attribution shares the primary resample indices.
        for stratum in ORDER_STRATA:
            group = groups[stratum]
            for _draw in range(5):
                record = group[rng.next() % 5]
                samples = record["_sample_map"]
                facts_on_ns = samples["facts-on"]["elapsed_ns"]
                facts_off_ns = samples["facts-off"]["elapsed_ns"]
                rust_ns = samples["rust"]["elapsed_ns"]
                # Input sizes are validated equal, so these are exactly the
                # corresponding paired throughput ratios without an avoidable
                # intermediate floating-point division.
                primary_draw.append(rust_ns / facts_on_ns)
                attribution_draw.append(facts_off_ns / facts_on_ns)
        primary.append(median(primary_draw))
        attribution.append(median(attribution_draw))
    return primary, attribution


def summarize(campaign: Path) -> dict[str, Any]:
    metadata, schedule, records = validate_campaign(campaign)
    per_variant: dict[str, list[float]] = {variant: [] for variant in VARIANTS}
    raw_samples: list[dict[str, Any]] = []
    primary_ratios: list[float] = []
    attribution_ratios: list[float] = []
    for record in records:
        samples = record["_sample_map"]
        measured = {variant: throughput(samples[variant]) for variant in VARIANTS}
        primary = samples["rust"]["elapsed_ns"] / samples["facts-on"]["elapsed_ns"]
        attribution = (
            samples["facts-off"]["elapsed_ns"]
            / samples["facts-on"]["elapsed_ns"]
        )
        primary_ratios.append(primary)
        attribution_ratios.append(attribution)
        for variant in VARIANTS:
            per_variant[variant].append(measured[variant])
            sample = dict(samples[variant])
            sample.update(
                {
                    "block_index": record["block_index"],
                    "order_stratum": record["order_stratum"],
                    "throughput_bytes_per_second": measured[variant],
                }
            )
            raw_samples.append(sample)

    bootstrap_primary, bootstrap_attribution = bootstrap(records)
    bootstrap_primary.sort()
    bootstrap_attribution.sort()
    primary_interval = [
        bootstrap_primary[LOWER_INDEX],
        bootstrap_primary[UPPER_INDEX],
    ]
    attribution_interval = [
        bootstrap_attribution[LOWER_INDEX],
        bootstrap_attribution[UPPER_INDEX],
    ]

    variant_summary: dict[str, Any] = {}
    order_position: dict[str, Any] = {}
    order_stratum: dict[str, Any] = {}
    for variant in VARIANTS:
        values = per_variant[variant]
        variant_summary[variant] = {
            "sample_count": len(values),
            "median_bytes_per_second": median(values),
            "median_mib_per_second": median(values) / (1024.0 * 1024.0),
            "mad_over_median": mad_over_median(values),
        }
        order_position[variant] = {}
        for ordinal in range(3):
            position_values = [
                throughput(record["_sample_map"][variant])
                for record in records
                if record["_sample_map"][variant]["ordinal"] == ordinal
            ]
            order_position[variant][str(ordinal)] = {
                "sample_count": len(position_values),
                "median_bytes_per_second": median(position_values),
            }
        order_stratum[variant] = {}
        for stratum in ORDER_STRATA:
            stratum_values = [
                throughput(record["_sample_map"][variant])
                for record in records
                if record["order_stratum"] == stratum
            ]
            order_stratum[variant][stratum] = {
                "sample_count": len(stratum_values),
                "median_bytes_per_second": median(stratum_values),
            }

    result = {
        "schema_version": 1,
        "kind": "utf8parse-score-analysis",
        "campaign": str(campaign.resolve()),
        "protocol_sha256": metadata["protocol_sha256"],
        "frozen_source": metadata["frozen_source"],
        "corpus": metadata["corpus"],
        "raw_sha256": metadata["raw_sha256"],
        "schedule_sha256": metadata["schedule_sha256"],
        "schedule": {
            "strata_order": schedule["strata_order"],
            "repetitions_per_stratum": 5,
        },
        "statistics": {
            "median_definition": "odd: middle; even: arithmetic mean of two middle values",
            "bootstrap": {
                "resamples": BOOTSTRAP_RESAMPLES,
                "seed_hex": f"0x{BOOTSTRAP_SEED:016x}",
                "strata_visit_order": list(ORDER_STRATA),
                "within_stratum_source_order": "ascending campaign block_index",
                "draws_per_stratum": 5,
                "draw_rule": "next()%5 with replacement",
                "shared_primary_attribution_indices": True,
                "interval": "empirical nearest-rank 95% percentile",
                "sorted_zero_based_indices": [LOWER_INDEX, UPPER_INDEX],
            },
        },
        "variants": variant_summary,
        "order_position_medians": order_position,
        "order_stratum_medians": order_stratum,
        "primary": {
            "ratio": "facts-on/rust",
            "point_estimate_median": median(primary_ratios),
            "bootstrap_95_percent_interval": primary_interval,
            "practical_band": [0.98, 1.02],
            "verdict": verdict(primary_interval, "xlang", "Rust"),
            "paired_block_ratios": primary_ratios,
        },
        "facts_attribution": {
            "ratio": "facts-on/facts-off",
            "point_estimate_median": median(attribution_ratios),
            "bootstrap_95_percent_interval": attribution_interval,
            "practical_band": [0.98, 1.02],
            "verdict": verdict(
                attribution_interval, "facts-on", "facts-off"
            ),
            "paired_block_ratios": attribution_ratios,
            "cannot_change_primary_verdict": True,
        },
        "raw_samples": raw_samples,
        "scope": (
            "This result applies only to this implementation, frozen corpus, "
            "machine, and campaign."
        ),
    }
    for number in (
        primary_ratios
        + attribution_ratios
        + primary_interval
        + attribution_interval
    ):
        if not math.isfinite(number):
            raise ValueError("analysis produced a non-finite statistic")
    return result


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("campaign", type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    try:
        campaign = args.campaign.resolve()
        result = summarize(campaign)
        output = campaign / "analysis.json"
        if output.exists():
            raise ValueError(f"refusing to overwrite {output}")
        output.write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        print(
            json.dumps(
                {
                    "status": "complete",
                    "analysis": str(output),
                    "primary_verdict": result["primary"]["verdict"],
                },
                sort_keys=True,
            )
        )
        return 0
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        print(f"analysis error: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
