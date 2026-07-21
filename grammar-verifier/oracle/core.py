"""Small shared types for the independent grammar Oracle."""

from __future__ import annotations

from dataclasses import dataclass, field
import hashlib


CURRENT_SHA256 = "d04336f7fa8d1a6a0f03fe58a17f972b658217a73a3dff91a906b4ba295328a8"


class Failure(Exception):
    """A closed engine outcome that can be emitted as a bounded FAIL report."""

    def __init__(self, family: str, code: str) -> None:
        super().__init__(f"{family}:{code}")
        self.family = family
        self.code = code


def fail(family: str, code: str) -> None:
    raise Failure(family, code)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def lower_hex(data: bytes) -> str:
    return data.hex()


def ascii_hex(text: str) -> str:
    try:
        return text.encode("ascii").hex()
    except UnicodeEncodeError:
        fail("internal", "non_ascii_record")


@dataclass(frozen=True)
class Limits:
    """The exact parsed logical ceilings supplied in the frame."""

    values: dict[str, int]

    def get(self, name: str) -> int:
        value = self.values.get(name)
        if value is None:
            fail("internal", "missing_limit")
        return value

    def require(self, name: str, observed: int) -> None:
        if observed > self.get(name):
            fail("resource", f"limit_{name}")


@dataclass
class LogicalBudget:
    """Cumulative counters whose increments are checked before publication."""

    limits: Limits
    counters: dict[str, int] = field(default_factory=dict)

    def add(self, limit_name: str, amount: int = 1) -> int:
        if amount < 0:
            fail("internal", "negative_budget_increment")
        prior = self.counters.get(limit_name, 0)
        observed = prior + amount
        self.limits.require(limit_name, observed)
        self.counters[limit_name] = observed
        return observed

    def maximum(self, limit_name: str, observed: int) -> None:
        prior = self.counters.get(limit_name, 0)
        if observed > prior:
            self.limits.require(limit_name, observed)
            self.counters[limit_name] = observed


@dataclass(frozen=True)
class BoundInput:
    name: str
    data: bytes

    @property
    def digest(self) -> str:
        return sha256(self.data)


@dataclass(frozen=True)
class Case:
    identifier: str
    start: str
    source: bytes


@dataclass(frozen=True)
class Domain:
    identifier: str
    kind: str
    start: str
    argument: bytes


@dataclass(frozen=True)
class Inputs:
    limits_bytes: BoundInput
    current: BoundInput
    proposal: BoundInput
    cases_bytes: BoundInput
    domains_bytes: BoundInput
    limits: Limits
    cases: tuple[Case, ...]
    domains: tuple[Domain, ...]

    def bound_sections(self) -> tuple[BoundInput, ...]:
        return (
            self.limits_bytes,
            self.current,
            self.proposal,
            self.cases_bytes,
            self.domains_bytes,
        )
