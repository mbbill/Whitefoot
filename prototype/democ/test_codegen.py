#!/usr/bin/env python3
"""Focused correctness gates for stage-0 aggregate code generation."""

import subprocess
import tempfile
from pathlib import Path

import democ


CLANG = "/usr/bin/clang" if Path("/usr/bin/clang").exists() else "clang"


def compile_native(source, run=False):
    ir = democ.compile_program(source, alias=False)
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        ll = root / "program.ll"
        output = root / ("program" if run else "program.o")
        ll.write_text(ir)
        command = [CLANG, "-O2", str(ll), "-o", str(output)]
        if not run:
            command.insert(2, "-c")
        built = subprocess.run(command, capture_output=True, text=True)
        assert built.returncode == 0, built.stderr
        if run:
            executed = subprocess.run([str(output)], capture_output=True)
            assert executed.returncode == 0, executed.returncode
    return ir


local_result = """fn main () -> own unit traps {
  let result: own Result<u64,u64> = match True() {
    True() => {
      give Ok(value: 7_u64);
    }
    False() => {
      give Err(error: 9_u64);
    }
  }
  match move result {
    Ok(value: value) => {
      check ieq<u64>(value, 7_u64) else trap "wrong local Result";
    }
    Err(error: error) => {
      check ieq<u64>(0_u64, 1_u64) else trap "unexpected Err";
    }
  }
  return unit;
}
"""
local_ir = compile_native(local_result, run=True)
assert "%Result = type { i32, i64 }" in local_ir


local_option = """fn main () -> own unit pure {
  let option: own Option<u64> = match True() {
    True() => {
      give Some(value: 7_u64);
    }
    False() => {
      give None();
    }
  }
  match move option {
    None() => {
    }
    Some(value: value) => {
    }
  }
  return unit;
}
"""
option_ir = compile_native(local_option, run=True)
assert "%Option = type { i32, i64 }" in option_ir


terminating_buffer = """fn choose_buffer (flag: own Bool) -> own buffer<u8> allocates(heap), traps {
  let unreachable: own buffer<u8> = match flag {
    True() => {
      let left: own buffer<u8> = buffer_new<u8>(1_u64, 1_u8);
      return move left;
    }
    False() => {
      let right: own buffer<u8> = buffer_new<u8>(1_u64, 2_u8);
      return move right;
    }
  }
  return move unreachable;
}

fn main () -> own unit allocates(heap), traps {
  let value: own buffer<u8> = choose_buffer(flag: True());
  let size: own u64 = len<u8>(value);
  check ieq<u64>(size, 1_u64) else trap "wrong buffer";
  return unit;
}
"""
compile_native(terminating_buffer, run=True)


nested_field_borrow = """struct Inner {
  value: u64;
}

struct Outer {
  inner: Inner;
  values: buffer<u64>;
}

fn increment ['i] (inner: &uniq 'i Inner) -> own unit reads('i), writes('i), traps {
  let before: own u64 = deref(inner).value;
  set deref(inner).value = iadd.trap<u64>(before, 1_u64);
  return unit;
}

fn write_first ['v] (values: &uniq 'v buffer<u64>) -> own unit reads('v), writes('v), traps {
  set index<u64>(deref(values), 0_u64) = 9_u64;
  return unit;
}

fn route ['o] (outer: &uniq 'o Outer) -> own unit reads('o), writes('o), traps {
  region 'inner_field {
    increment(inner: &uniq 'inner_field deref(outer).inner);
  }
  region 'buffer_field {
    write_first(values: &uniq 'buffer_field deref(outer).values);
  }
  return unit;
}

fn main () -> own unit allocates(heap), traps {
  let inner: own Inner = Inner(value: 6_u64);
  let values: own buffer<u64> = buffer_new<u64>(1_u64, 0_u64);
  let outer: own Outer = Outer(inner: move inner, values: move values);
  region 'outer_borrow {
    route(outer: &uniq 'outer_borrow outer);
  }
  check ieq<u64>(outer.inner.value, 7_u64) else trap "nested field borrow used the root pointer";
  let first: own u64 = index<u64>(outer.values, 0_u64);
  check ieq<u64>(first, 9_u64) else trap "buffer field borrow lost its checked header";
  return unit;
}
"""
nested_field_ir = compile_native(nested_field_borrow, run=True)
assert "getelementptr %Outer" in nested_field_ir


aggregate_payloads = [
    """struct Pair {
  value: u64;
}

fn make () -> own Result<Pair,u64> pure {
  let pair: own Pair = Pair(value: 1_u64);
  return Ok(value: move pair);
}
""",
    """struct Pair {
  value: u64;
}

fn make () -> own Option<Pair> pure {
  let pair: own Pair = Pair(value: 1_u64);
  return Some(value: move pair);
}
""",
]
for source in aggregate_payloads:
    try:
        democ.compile_program(source, alias=False)
    except SystemExit as error:
        assert "outside the stage-0 word-erased profile" in str(error)
    else:
        raise AssertionError("aggregate prelude payload reached stage-0 LLVM codegen")

print("stage-0 codegen: local aggregate give, terminating arms, and payload profile pass")
