# Serves compute-bench's paired modules: isolate definitions without changing
# their linkage or optimization attributes. Retire with the paired-module
# build if the harness no longer links two emissions of one source together.
# Input is one emitted module followed by its host adapter. LLVM declarations
# and weak runtime definitions keep their names so the ordinary library wins
# the same bindings as in a whitefootc executable.
BEGIN {
    if (prefix !~ /^[A-Za-z_][A-Za-z_0-9]*$/ ||
        api !~ /^[A-Za-z_][A-Za-z_0-9]*$/ || mode !~ /^(par|seq)$/) {
        print "compute-bench: invalid module symbol parameters" > "/dev/stderr"
        exit 1
    }
}
{
    lines[++count] = $0
    if ($0 !~ /^define / ||
        $0 ~ /^define (.* )?(weak|weak_odr|linkonce|linkonce_odr|available_externally) /)
        next
    # Function definitions emitted by whitefootc put their name on this line.
    if (!match($0, /@("[^"\\]*"|[-A-Za-z$._0-9]+)\(/)) {
        print "compute-bench: unrecognized LLVM function definition" > "/dev/stderr"
        failed = 1
        next
    }
    name = substr($0, RSTART + 1, RLENGTH - 2)
    if (substr(name, 1, 1) == "\"") name = substr(name, 2, length(name) - 2)
    names[name] = prefix "_" name
    if (name == api) names[name] = api "_" mode
    if (name == api "_release") names[name] = api "_" mode "_release"
}
END {
    if (failed || !(api in names)) {
        print "compute-bench: missing adapter or unsupported module definition" > "/dev/stderr"
        exit 1
    }
    for (i = 1; i <= count; i++) {
        rest = lines[i]
        out = ""
        # Consume string literals and comments too: a global-looking substring
        # inside either is data, not a reference to rename. Both bare and quoted
        # references to a definition use the same mapping.
        while (match(rest, /@("([^"\\]|\\[0-9A-Fa-f][0-9A-Fa-f])*"|[-A-Za-z$._0-9]+)|"([^"\\]|\\[0-9A-Fa-f][0-9A-Fa-f])*"|;.*/)) {
            token = substr(rest, RSTART, RLENGTH)
            out = out substr(rest, 1, RSTART - 1)
            rest = substr(rest, RSTART + RLENGTH)
            if (substr(token, 1, 1) == "@") {
                name = substr(token, 2)
                if (substr(name, 1, 1) == "\"") name = substr(name, 2, length(name) - 2)
                if (name in names) token = "@\"" names[name] "\""
            }
            out = out token
        }
        print out rest
    }
}
