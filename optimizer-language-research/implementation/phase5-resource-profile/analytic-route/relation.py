"""Private analytic relations derived only from neutral construction knobs.

The relation is intentionally not a parser. It expands the two generator
contracts into abstract token, production, scope, role, lookup, and action
records without accepting source bytes.
"""

from __future__ import annotations

from dataclasses import dataclass
from manifest import FAMILY_CODEC, FAMILY_COMPILER, ManifestError, WorkloadManifest


PRELUDE_SPELLINGS = (
    "Bool", "True", "False", "Option", "T", "None", "Some", "value",
    "Result", "T", "E", "Ok", "value", "Err", "error", "Overflow",
    "Overflow", "DivError", "DivideByZero", "DivOverflow", "NarrowError",
    "NarrowError", "Int", "Float",
)
PRELUDE_LOOKUP_SPELLINGS = (
    "Bool", "True", "False", "Option", "None", "Some", "Result", "Ok",
    "Err", "Overflow", "Overflow", "DivError", "DivideByZero", "DivOverflow",
    "NarrowError", "NarrowError", "Int", "Float",
)
DOMAIN_LEXICAL = 1
DOMAIN_NOMINAL = 2
DOMAIN_STRUCT_CONSTRUCTOR = 3
DOMAIN_ENUM_CONSTRUCTOR = 4
DOMAIN_CONTRACT = 5
DOMAIN_LABEL = 6
DOMAIN_OPERATION = 7
OPERATION_SPELLINGS = (
    "iadd.wrap", "isub.wrap", "imul.wrap", "iadd.trap", "isub.trap",
    "imul.trap", "iadd.checked", "isub.checked", "imul.checked",
    "idiv.trap", "irem.trap", "idiv.checked", "irem.checked", "ineg.wrap",
    "ineg.trap", "ineg.checked", "ieq", "ine", "ilt", "ile", "igt",
    "ige", "eeq", "ene", "fadd.strict", "fsub.strict", "fmul.strict",
    "fdiv.strict", "feq", "flt", "fle", "fgt", "fge", "fne", "band",
    "bor", "bxor", "bnot", "cvt", "len", "slice_of", "box_new",
    "arena_new", "array_new", "buffer_new", "iand", "ior", "ixor",
    "inot", "ishl.wrap", "ishr.wrap", "ishl.trap", "ishr.trap", "irotl",
    "irotr", "ipopcount", "iclz", "ictz", "ibswap", "imulhi", "iadd.sat",
    "isub.sat", "imul.sat", "imin", "imax", "iabs.wrap", "iabs.trap",
    "iabs.checked", "reinterpret", "fneg", "fabs", "fcopysign", "fmin",
    "fmax", "ffloor", "fceil", "ftrunc", "froundeven", "frem",
    "fsqrt.strict", "ffma.strict", "finf", "fnan",
)
MODE_WORDS = ("wrap", "trap", "checked", "sat", "strict")
RESERVATION_SPELLINGS = tuple(
    spelling for spelling in OPERATION_SPELLINGS if "." not in spelling
) + MODE_WORDS


@dataclass(frozen=True)
class Role:
    category: str
    spelling: str
    domain: int
    partition: int
    scope: int
    event: int
    lookup_multiplicity: int
    node_depth: int
    constructor_owner: str | None = None


@dataclass(frozen=True)
class LookupEntry:
    partition: int
    domain: int
    spelling: str
    origin_kind: int
    source_event: int
    class_ordinal: int
    declaration_class: str
    constructor_owner: str | None


@dataclass(frozen=True)
class PrivateRelation:
    family: str
    units: int
    source_bytes: int
    token_bytes: int
    # Aggregate-preserving witnesses realize independently known cardinality,
    # byte-sum, peak, and depth values. They are not token/grammar traces and
    # cannot be used for work fields or per-production claims.
    aggregate_token_length_witness: tuple[int, ...]
    trivia_runs: int
    aggregate_production_parent_witness: tuple[int | None, ...]
    scopes: tuple[int | None, ...]
    roles: tuple[Role, ...]
    lookup_entries: tuple[LookupEntry, ...]
    requires_shapes: tuple[tuple[str, ...], ...]


@dataclass(frozen=True)
class _FamilyShape:
    source_base: int
    source_per_unit: int
    token_base: int
    token_per_unit: int
    token_byte_base: int
    token_byte_per_unit: int
    trivia_base: int
    trivia_per_unit: int
    node_base: int
    node_per_unit: int
    tree_depth: int


SHAPES = {
    FAMILY_COMPILER: _FamilyShape(116, 477, 30, 100, 89, 398, 25, 73, 27, 90, 12),
    FAMILY_CODEC: _FamilyShape(46, 696, 13, 137, 34, 583, 11, 99, 11, 112, 13),
}


def _aggregate_token_length_witness(
    count: int, total: int, maximum: int
) -> tuple[int, ...]:
    """Realize aggregate token facts without claiming token-by-token shape."""
    if count < 1 or maximum < 1 or total < count or total < maximum:
        raise ManifestError("private token dimensions are impossible")
    values = [maximum]
    remaining_count = count - 1
    remaining = total - maximum
    if remaining < remaining_count:
        raise ManifestError("private token byte total cannot cover every token")
    for index in range(remaining_count):
        slots = remaining_count - index
        value = min(maximum, remaining - (slots - 1))
        values.append(value)
        remaining -= value
    if remaining:
        raise ManifestError("private token length relation did not close")
    return tuple(values)


def _aggregate_parent_witness(
    count: int, maximum_depth: int
) -> tuple[int | None, ...]:
    """Realize aggregate node/depth facts without claiming grammar topology."""
    if count < maximum_depth + 1:
        raise ManifestError("private production topology cannot reach its depth")
    parents: list[int | None] = [None]
    for index in range(1, count):
        if index <= maximum_depth:
            parents.append(index - 1)
        else:
            parents.append(
                1 + ((index - maximum_depth - 1) % max(1, maximum_depth - 1))
            )
    return tuple(parents)


def _scope_tree(family: str, units: int) -> tuple[int | None, ...]:
    parents: list[int | None] = [None]
    if family == FAMILY_COMPILER:
        for _ in range(units):
            for _ in range(3):
                signature = len(parents)
                parents.append(0)
                parents.append(signature)
        for _ in range(2):
            signature = len(parents)
            parents.append(0)
            parents.append(signature)
    else:
        for _ in range(units):
            parents.append(0)  # contract member signature
            signature = len(parents)
            parents.extend((0, signature))  # codec_apply signature and body
            run_signature = len(parents)
            parents.append(0)
            parents.extend((run_signature, run_signature))  # requires and body
            body = len(parents) - 1
            label = len(parents)
            parents.append(body)
            parents.append(label)
        signature = len(parents)
        parents.extend((0, signature))
    return tuple(parents)


class _RoleBuilder:
    def __init__(self) -> None:
        self.roles: list[Role] = []
        self.next_event = 0

    def add(
        self,
        category: str,
        spelling: str,
        domain: int,
        partition: int,
        scope: int,
        lookup: int,
        depth: int,
        constructor_owner: str | None = None,
    ) -> None:
        event = self.next_event
        self.next_event += 1
        self.roles.append(
            Role(
                category,
                spelling,
                domain,
                partition,
                scope,
                event,
                lookup,
                depth,
                constructor_owner,
            )
        )


def _compiler_roles(units: int) -> tuple[Role, ...]:
    builder = _RoleBuilder()
    for index in range(units):
        suffix = f"{index:06d}"
        next_suffix = f"{index + 1:06d}"
        worker_partition = index * 3 + 1
        dispatch_partition = worker_partition + 1
        wrap_partition = worker_partition + 2
        names = {
            "seed": f"seed_{suffix}", "record": f"CompilerRecord{suffix}",
            "worker": f"worker_{suffix}", "next_worker": f"worker_{next_suffix}",
            "dispatch": f"dispatch_{suffix}", "wrap": f"wrap_{suffix}",
            "local": f"local_{suffix}", "output": f"output_{suffix}",
        }
        builder.add("declaration", names["seed"], 1, 0, 0, 1, 2)
        builder.add("declaration", names["record"], DOMAIN_NOMINAL, 0, 0, 2, 2)
        builder.add("dependent", "value", 0, 0, 0, 0, 3)
        for owner, local, partition in (
            (names["worker"], names["local"], worker_partition),
            (names["dispatch"], names["output"], dispatch_partition),
        ):
            builder.add("declaration", owner, 1, 0, 0, 1, 2)
            builder.add("declaration", "input", 1, partition, 1, 1, 5)
            builder.add("declaration", local, 1, partition, 2, 1, 6)
        builder.add("declaration", names["wrap"], 1, 0, 0, 1, 2)
        builder.add("declaration", "input", 1, wrap_partition, 1, 1, 5)
        for spelling, partition, domain in (
            ("iadd.wrap", worker_partition, DOMAIN_OPERATION),
            ("input", worker_partition, DOMAIN_LEXICAL),
            (names["seed"], worker_partition, DOMAIN_LEXICAL),
            (names["local"], worker_partition, DOMAIN_LEXICAL),
            (names["next_worker"], dispatch_partition, DOMAIN_LEXICAL),
            ("input", dispatch_partition, DOMAIN_LEXICAL),
            (names["output"], dispatch_partition, DOMAIN_LEXICAL),
            (names["record"], wrap_partition, DOMAIN_NOMINAL),
            (names["record"], wrap_partition, DOMAIN_STRUCT_CONSTRUCTOR),
            ("input", wrap_partition, DOMAIN_LEXICAL),
        ):
            builder.add("lexical", spelling, domain, partition, 2, 0, 7)
        builder.add("deferred", "input", 0, dispatch_partition, 2, 0, 7)
        builder.add("deferred", "value", 0, wrap_partition, 2, 0, 7)
    final = f"worker_{units:06d}"
    builder.add("declaration", final, 1, 0, 0, 1, 2)
    final_partition = units * 3 + 1
    builder.add("declaration", "input", 1, final_partition, 1, 1, 5)
    builder.add("lexical", "input", 1, final_partition, 2, 0, 6)
    builder.add("declaration", "main", 1, 0, 0, 1, 2)
    return tuple(builder.roles)


def _codec_roles(units: int) -> tuple[Role, ...]:
    builder = _RoleBuilder()
    for index in range(units):
        suffix = f"{index:06d}"
        contract_partition = index * 3 + 1
        apply_partition = contract_partition + 1
        run_partition = contract_partition + 2
        result = f"CodecResult{suffix}"
        ok = f"CodecOk{suffix}"
        err = f"CodecErr{suffix}"
        contract = f"CodecContract{suffix}"
        apply = f"codec_apply_{suffix}"
        run = f"codec_run_{suffix}"
        stable = f"stable_{suffix}"
        transformed = f"transformed_{suffix}"
        label = f"@scan_{suffix}"
        builder.add("declaration", result, DOMAIN_NOMINAL, 0, 0, 1, 3)
        for spelling in (ok, err):
            builder.add(
                "declaration",
                spelling,
                DOMAIN_ENUM_CONSTRUCTOR,
                0,
                0,
                1,
                3,
                result,
            )
        builder.add("dependent", "value", 0, 0, 0, 0, 4)
        builder.add("dependent", "code", 0, 0, 0, 0, 4)
        builder.add("declaration", contract, DOMAIN_CONTRACT, 0, 0, 1, 3)
        builder.add("dependent", "apply", 0, contract_partition, 0, 0, 4)
        builder.add("declaration", "input", 1, contract_partition, 1, 1, 5)
        builder.add("declaration", apply, 1, 0, 0, 1, 2)
        builder.add("declaration", "input", 1, apply_partition, 1, 1, 5)
        for spelling, partition, domain in (
            ("iadd.wrap", apply_partition, DOMAIN_OPERATION),
            ("input", apply_partition, DOMAIN_LEXICAL),
            (contract, 0, DOMAIN_CONTRACT),
            (apply, 0, DOMAIN_LEXICAL),
        ):
            builder.add("lexical", spelling, domain, partition, 2, 0, 7)
        builder.add("deferred", "apply", 0, contract_partition, 2, 0, 6)
        for spelling, scope, depth, domain in (
            (run, 0, 2, DOMAIN_LEXICAL),
            ("input", 1, 5, DOMAIN_LEXICAL),
            (stable, 2, 7, DOMAIN_LEXICAL),
            (transformed, 2, 7, DOMAIN_LEXICAL),
            (label, 3, 9, DOMAIN_LABEL),
        ):
            builder.add(
                "declaration",
                spelling,
                domain,
                0 if scope == 0 else run_partition,
                scope,
                1,
                depth,
            )
        for spelling, domain in (
            (result, DOMAIN_NOMINAL), ("Bool", DOMAIN_NOMINAL),
            ("ieq", DOMAIN_OPERATION), ("input", DOMAIN_LEXICAL),
            ("input", DOMAIN_LEXICAL), (stable, DOMAIN_LEXICAL),
            (apply, DOMAIN_LEXICAL), ("input", DOMAIN_LEXICAL),
            (label, DOMAIN_LABEL), (ok, DOMAIN_ENUM_CONSTRUCTOR),
            (transformed, DOMAIN_LEXICAL),
        ):
            builder.add("lexical", spelling, domain, run_partition, 2, 0, 10)
        for spelling in ("input", "value"):
            builder.add("deferred", spelling, 0, run_partition, 2, 0, 10)
    builder.add("declaration", "main", 1, 0, 0, 1, 2)
    return tuple(builder.roles)


def _lookup_entries(roles: tuple[Role, ...]) -> tuple[LookupEntry, ...]:
    entries: list[LookupEntry] = []
    prelude = (
        ("Bool", DOMAIN_NOMINAL, None),
        ("True", DOMAIN_ENUM_CONSTRUCTOR, "Bool"),
        ("False", DOMAIN_ENUM_CONSTRUCTOR, "Bool"),
        ("Option", DOMAIN_NOMINAL, None),
        ("None", DOMAIN_ENUM_CONSTRUCTOR, "Option"),
        ("Some", DOMAIN_ENUM_CONSTRUCTOR, "Option"),
        ("Result", DOMAIN_NOMINAL, None),
        ("Ok", DOMAIN_ENUM_CONSTRUCTOR, "Result"),
        ("Err", DOMAIN_ENUM_CONSTRUCTOR, "Result"),
        ("Overflow", DOMAIN_NOMINAL, None),
        ("Overflow", DOMAIN_ENUM_CONSTRUCTOR, "Overflow"),
        ("DivError", DOMAIN_NOMINAL, None),
        ("DivideByZero", DOMAIN_ENUM_CONSTRUCTOR, "DivError"),
        ("DivOverflow", DOMAIN_ENUM_CONSTRUCTOR, "DivError"),
        ("NarrowError", DOMAIN_NOMINAL, None),
        ("NarrowError", DOMAIN_ENUM_CONSTRUCTOR, "NarrowError"),
        ("Int", DOMAIN_CONTRACT, None),
        ("Float", DOMAIN_CONTRACT, None),
    )
    if tuple(spelling for spelling, _, _ in prelude) != PRELUDE_LOOKUP_SPELLINGS:
        raise ManifestError("private PRE-1 lookup relation is not closed")
    for ordinal, (spelling, domain, owner) in enumerate(prelude):
        entries.append(
            LookupEntry(0, domain, spelling, 0, ordinal, ordinal, "prelude", owner)
        )
    for ordinal, spelling in enumerate(OPERATION_SPELLINGS):
        entries.append(
            LookupEntry(
                0,
                DOMAIN_OPERATION,
                spelling,
                1,
                ordinal,
                11,
                "operation-family",
                None,
            )
        )
    for role in roles:
        for domain_offset in range(role.lookup_multiplicity):
            domain = role.domain + domain_offset
            declaration_class = (
                "function" if role.partition == 0 and domain == DOMAIN_LEXICAL
                and (role.spelling.startswith(("worker_", "dispatch_", "wrap_", "codec_")) or role.spelling == "main")
                else "source"
            )
            entries.append(
                LookupEntry(
                    role.partition,
                    domain,
                    role.spelling,
                    2,
                    role.event,
                    1,
                    declaration_class,
                    role.constructor_owner,
                )
            )
    return tuple(entries)


def _source_shape(manifest: WorkloadManifest) -> _FamilyShape:
    expected_path = f"demand/{manifest.family}-{manifest.units:06d}.wf"
    if manifest.sources[0].logical_path != expected_path:
        raise ManifestError("logical path does not match the generator contract")
    shape = SHAPES[manifest.family]
    expected_bytes = shape.source_base + shape.source_per_unit * manifest.units
    if manifest.sources[0].byte_length != expected_bytes:
        raise ManifestError("source byte length is impossible for the generator contract")
    return shape


def build_relation(manifest: WorkloadManifest) -> PrivateRelation:
    shape = _source_shape(manifest)
    units = manifest.units
    tokens = shape.token_base + shape.token_per_unit * units
    token_bytes = shape.token_byte_base + shape.token_byte_per_unit * units
    trivia = shape.trivia_base + shape.trivia_per_unit * units
    source_bytes = shape.source_base + shape.source_per_unit * units
    nodes = shape.node_base + shape.node_per_unit * units
    aggregate_token_witness = _aggregate_token_length_witness(
        tokens, token_bytes, 20
    )
    aggregate_parent_witness = _aggregate_parent_witness(nodes, shape.tree_depth)
    scopes = _scope_tree(manifest.family, units)
    roles = _compiler_roles(units) if manifest.family == FAMILY_COMPILER else _codec_roles(units)
    entries = _lookup_entries(roles)
    return PrivateRelation(
        family=manifest.family,
        units=units,
        source_bytes=source_bytes,
        token_bytes=token_bytes,
        aggregate_token_length_witness=aggregate_token_witness,
        trivia_runs=trivia,
        aggregate_production_parent_witness=aggregate_parent_witness,
        scopes=scopes,
        roles=roles,
        lookup_entries=entries,
        requires_shapes=(
            tuple(
                tuple("ordinary-let" for _ in range(1)) + ("check",)
                for _ in range(units)
            )
            if manifest.family == FAMILY_CODEC
            else ()
        ),
    )
