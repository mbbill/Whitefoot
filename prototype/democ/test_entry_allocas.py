#!/usr/bin/env python3
"""Keep stage-0 fixed-size stack slots out of repeated control flow."""

import democ


SOURCE = """struct Packet {
  value: u64;
  checksum: u64;
}

fn make_packet (value: own u64) -> own Packet traps {
  let checksum: own u64 = iadd.trap<u64>(value, 1_u64);
  return Packet(value: value, checksum: checksum);
}

fn consume_packets (limit: own u64) -> own u64 traps {
  let cursor: own u64 = 0_u64;
  let total: own u64 = 0_u64;
  loop @packets {
    match ige<u64>(cursor, limit) {
      True() => {
        break @packets;
      }
      False() => {
      }
    }
    let packet: own Packet = make_packet(value: cursor);
    set total = iadd.trap<u64>(total, packet.value);
    set cursor = iadd.trap<u64>(cursor, 1_u64);
  }
  return total;
}
"""


def function_body(ir, name):
    return ir.split(f" @{name}(", 1)[1].split("\n}", 1)[0].splitlines()


def main():
    lines = function_body(democ.compile_program(SOURCE, alias=False), "consume_packets")
    body_labels = [
        index
        for index, line in enumerate(lines)
        if line.endswith(":") and line != "entry:"
    ]
    assert body_labels, "loop fixture emitted no body label"
    first_body_label = body_labels[0]
    stack_slots = [index for index, line in enumerate(lines) if " = alloca " in line]
    assert len(stack_slots) >= 3, stack_slots
    assert all(index < first_body_label for index in stack_slots), (
        "a fixed-size stack slot executes inside repeated control flow",
        [lines[index] for index in stack_slots if index >= first_body_label],
    )
    print("stage-0 stack slots: every fixed-size alloca is in the entry block")


if __name__ == "__main__":
    main()
