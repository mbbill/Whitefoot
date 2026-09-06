#!/bin/sh
WFC=/tmp/claude-0/-home-user-Whitefoot/1dea2c46-aae2-5e06-954f-b60e9c5b442c/scratchpad/target/release/whitefootc
for f in "$@"; do
  echo "### $f"
  out=$($WFC --emit-llvm -o /dev/null "$f" 2>&1)
  code=$?
  echo "exit=$code"
  if [ -n "$out" ]; then echo "$out"; fi
  echo
done
