#!/bin/sh
# Serves the cross-version compute twin; remove when its command-entry baseline
# is no longer compared. This changes only the nullary launcher spelling.
set -eu
header='fn main() -> status: own ExitStatus pure {'
case "$1" in
  profile)
    ordinary=$(grep -Fxc "$header" "$2" || :)
    command=$(grep -Fxc "command $header" "$2" || :)
    case "$ordinary:$command" in
      1:0) echo ordinary ;;
      0:1) echo command ;;
      *) echo 'compute-bench: unknown baseline entry interface' >&2; exit 1 ;;
    esac ;;
  adapt)
    profile=$2; input=$3; output=$4
    test "$(grep -Ec '(^| )fn main\(' "$input" || :)" = 1 \
      && test "$(grep -Fxc "$header" "$input" || :)" = 1 || {
        echo 'compute-bench: expected one exact nullary ordinary main' >&2; exit 1;
      }
    case "$profile" in
      ordinary) cp "$input" "$output" ;;
      command)
        sed '/^fn main() -> status: own ExitStatus pure {$/s/^/command /' "$input" > "$output"
        sed '/^command fn main() -> status: own ExitStatus pure {$/s/^command //' "$output" > "$output.reverse"
        cmp -s "$input" "$output.reverse" || {
          rm -f "$output.reverse"
          echo 'compute-bench: baseline adaptation changed other source bytes' >&2; exit 1;
        }
        rm -f "$output.reverse" ;;
      *) echo 'compute-bench: unknown baseline interface' >&2; exit 1 ;;
    esac ;;
  *) echo 'usage: baseline-entry.sh profile source | adapt ordinary|command input output' >&2; exit 1 ;;
esac
