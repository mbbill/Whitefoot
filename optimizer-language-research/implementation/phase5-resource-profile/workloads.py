"""Deterministic canonical source generators for resource-profile evidence."""

import argparse
from hashlib import sha256
from pathlib import Path

from evidence_manifest import build_manifest, encode_manifest, generator_revision


class WorkloadError(ValueError):
    pass


MAX_WORKLOAD_UNITS = 32_768


def _validate_units(units: int, family: str) -> None:
    if units < 1 or units > MAX_WORKLOAD_UNITS:
        raise WorkloadError(
            f"{family} workload units must be in 1..{MAX_WORKLOAD_UNITS}"
        )


def _compiler_unit(index: int) -> str:
    suffix = f"{index:06d}"
    next_suffix = f"{index + 1:06d}"
    return f"""const seed_{suffix}: u64 = 1_u64;

struct CompilerRecord{suffix} {{
  value: u64;
}}

fn worker_{suffix}(input: own u64) -> own u64 pure {{
  let local_{suffix}: own u64 = iadd.wrap<u64>(input, seed_{suffix});
  return local_{suffix};
}}

fn dispatch_{suffix}(input: own u64) -> own u64 pure {{
  let output_{suffix}: own u64 = worker_{next_suffix}(input: input);
  return output_{suffix};
}}

fn wrap_{suffix}(input: own u64) -> own CompilerRecord{suffix} pure {{
  return CompilerRecord{suffix}(value: input);
}}
"""


def compiler_workload(units: int) -> bytes:
    _validate_units(units, "compiler")
    parts = [_compiler_unit(index) for index in range(units)]
    final_suffix = f"{units:06d}"
    parts.append(
        f"""fn worker_{final_suffix}(input: own u64) -> own u64 pure {{
  return input;
}}

fn main() -> own unit pure {{
  return unit;
}}
"""
    )
    return "\n".join(parts).encode("ascii")


def _codec_unit(index: int) -> str:
    suffix = f"{index:06d}"
    return f"""enum CodecResult{suffix} {{
  CodecOk{suffix}(value: u64);
  CodecErr{suffix}(code: u64);
}}

contract CodecContract{suffix} {{
  fn apply(input: own u64) -> own u64 pure;
}}

fn codec_apply_{suffix}(input: own u64) -> own u64 pure {{
  return iadd.wrap<u64>(input, 1_u64);
}}

conform u64: CodecContract{suffix} {{
  apply = codec_apply_{suffix};
}}

fn codec_run_{suffix}(input: own u64) -> own CodecResult{suffix} traps requires {{
  let stable_{suffix}: own Bool = ieq<u64>(input, input);
  check stable_{suffix} else trap "codec precondition";
}} {{
  let transformed_{suffix}: own u64 = codec_apply_{suffix}(input: input);
  loop @scan_{suffix} {{
    break @scan_{suffix};
  }}
  return CodecOk{suffix}(value: transformed_{suffix});
}}
"""


def codec_workload(units: int) -> bytes:
    _validate_units(units, "codec")
    parts = [_codec_unit(index) for index in range(units)]
    parts.append(
        """fn main() -> own unit pure {
  return unit;
}
"""
    )
    return "\n".join(parts).encode("ascii")


def build(family: str, units: int) -> bytes:
    if family == "compiler":
        return compiler_workload(units)
    if family == "codec":
        return codec_workload(units)
    raise WorkloadError(f"unknown workload family: {family}")


def manifest(family: str, units: int, source: bytes) -> bytes:
    """Describe generator inputs and source identity without expected results."""

    _validate_units(units, family)
    data = build_manifest(
        family=family,
        units=units,
        revision=generator_revision(Path(__file__)),
        parameters={
            "name_decimal_width": 6,
            "source_records": 1,
            "unit_count": units,
        },
        sources=((f"demand/{family}-{units:06d}.wf", source),),
    )
    return encode_manifest(data)


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--family", choices=("compiler", "codec"), required=True)
    parser.add_argument("--units", type=int, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--manifest-output", type=Path)
    arguments = parser.parse_args()
    source = build(arguments.family, arguments.units)
    arguments.output.write_bytes(source)
    if arguments.manifest_output is not None:
        arguments.manifest_output.write_bytes(
            manifest(arguments.family, arguments.units, source)
        )
    print(
        f"family={arguments.family} units={arguments.units} "
        f"bytes={len(source)} sha256={sha256(source).hexdigest()}"
    )


if __name__ == "__main__":
    main()
