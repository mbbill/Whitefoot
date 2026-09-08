# Bind each raw report to its invocation; timings are observations, not limits.
function integer(value) { return value ~ /^[0-9]+$/ }
function reject() { failed = 1; exit 1 }
BEGIN { FS = "\t" }
NR == 1 {
    count = split($0, header, " ")
    if (count != 8 || header[1] != "#" || header[2] != "workers=" width ||
        header[3] != "blocked=" (width - 1) || header[4] != "depth=" depth ||
        header[5] != "tasks=262144" || header[6] != "pass=" pass ||
        header[7] !~ /^warmup_pairs=[1-9][0-9]*$/ ||
        header[8] !~ /^warmup_ns=[0-9]+$/ || substr(header[8], 11) + 0 < 100000000) reject()
    next
}
NR == 2 {
    if ($0 != "sample\tphase\tmode\twall_ns\towner_cpu_ns\tsteals") reject()
    next
}
NR >= 3 && NR <= 20 {
    row = NR - 3
    sample = int(row / 2)
    mode = ((row % 2 + sample + pass) % 2) ? "task" : "direct"
    if (NF != 6 || $1 != sample || $2 != (sample ? "warm" : "first") ||
        $3 != mode || !integer($4) || $4 <= 0 || !integer($5) || $5 <= 0 ||
        $6 != "0") reject()
    next
}
NR == 21 {
    if ($0 != "# protocol PASS: samples=18 outputs=4718592") reject()
    next
}
{ reject() }
END { if (failed || NR != 21) exit 1 }
