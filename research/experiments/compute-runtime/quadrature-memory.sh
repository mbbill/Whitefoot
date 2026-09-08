#!/bin/sh
# Preserve and validate diagnostic stderr for quadrature full/batch checks.
# Reuse the pinned caller-worker lifecycle grammar; retire with quadrature.
set -eu
test "$#" = 5
profile=$1; log=$2; status=$3; form=$4; stats=$5
test -f "$log"
case "$profile" in address|clean) ;; *) exit 1;; esac
case "$stats" in 0|1) ;; *) exit 1;; esac
leak=0
case "$(uname -s)/$(uname -m)/$profile/$stats/$form" in
    Linux/x86_64/address/1/rayon|Linux/x86_64/address/1/rayon-left) leak=1;;
esac
if test "$leak" = 0; then
    test "$status" = 0 && test ! -s "$log" || {
        echo "quadrature clean diagnostic/status rejected: $status" >&2; exit 1;
    }
    exit 0
fi
test "$status" = 23 || { echo "quadrature Rayon lifecycle status rejected: $status" >&2; exit 1; }
directory=$(CDPATH= cd "$(dirname "$0")" && pwd)
scratch=$(mktemp -d "${TMPDIR:-/tmp}/whitefoot-quadrature-memory.XXXXXX")
trap 'rm -rf "$scratch"' EXIT HUP INT TERM
awk -v leak=1 -v pool_owner=quadrature -v object_basename=sanitized \
    -v prefix="$scratch/prefix" -f "$directory/records-scheduler-memory.awk" "$log"
test ! -s "$scratch/prefix"
