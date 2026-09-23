#!/usr/bin/env perl
# Explicit, fixed DAG cost trial; called only by dag-fanin-cost-run.
# Keep with the investigation's probe and retire when that probe is retired.
use strict;
use warnings;
use Cwd qw(abs_path);
use Digest::SHA qw(sha256_hex);
use FindBin qw($RealBin);
use File::Path qw(make_path);

my @fixtures = ((map { "n-$_-3" } 0..15), qw(wide-cheap wide-costly spine-8-0 spine-8-1));
my @widths = (1, 4);
sub cells {
    my ($mode) = @_;
    return (['n-0-3', 1], ['n-9-3', 4]) if $mode eq 'slow';
    return map { my $f = $_; map { [$f, $_] } @widths } @fixtures;
}
sub repeats {
    my ($fixture, $width) = @_;
    return $width == 1 ? 1048576 : 16384
        if $fixture eq 'n-0-3' || $fixture eq 'wide-cheap' || $fixture eq 'spine-8-0';
    return 32;
}
sub read_file {
    my ($path) = @_;
    open my $file, '<', $path or die "read $path: $!\n";
    local $/;
    return <$file> // '';
}
sub write_file {
    my ($path, $value) = @_;
    open my $file, '>', $path or die "write $path: $!\n";
    print {$file} $value;
    close $file or die "close $path: $!\n";
}
sub median { my @sorted = sort { $a <=> $b } @_; return $sorted[2] }
sub judge {
    my ($mode, $fixture, $width, $wall, $cpu, $wall_worse, $cpu_worse, $wall_better) = @_;
    return (($wall < .97 || $wall > 1/.97 || $cpu < .90 || $cpu > 1/.90)
        ? 'inconclusive_null' : 'pass') if $mode eq 'null';
    return (($wall > 1/1.5 || $cpu > 1/1.5 || $wall_worse < 4 || $cpu_worse < 4)
        ? 'failed_slowdown_control' : 'pass') if $mode eq 'slow';
    my @adverse;
    push @adverse, $wall_worse >= 4 ? 'persistent_wall' : 'noisy_wall' if $wall < .97;
    push @adverse, $cpu_worse >= 4 ? 'persistent_cpu' : 'noisy_cpu' if $cpu < .90;
    if ($mode eq 'bridge' && $fixture eq 'n-9-3' && $width == 4) {
        push @adverse, 'inconclusive_benefit' if $wall_better < 4;
        push @adverse, 'insufficient_benefit' if $wall_better >= 4 && $wall < 1/.9;
    }
    return @adverse ? join(',', @adverse) : 'pass';
}
sub reduce_rows {
    my ($mode, $lines) = @_;
    my %allowed = map { join("\t", @$_) => 1 } cells($mode);
    my %raw;
    for my $line (@$lines) {
        chomp $line;
        next if $line =~ /^#/ || $line eq '';
        my @f = split /\t/, $line;
        @f == 11 && shift(@f) eq 'measure' or die "invalid measurement row\n";
        my ($fixture, $arm, $width, $pass, $sample, $wall, $cpu, $batch, $actual, $tasks) = @f;
        $allowed{"$fixture\t$width"} && $arm =~ /\A(?:baseline|candidate)\z/ &&
            $pass =~ /\A[0-4]\z/ && $sample =~ /\A[0-5]\z/
            or die "unexpected measurement key\n";
        for ($wall, $cpu, $batch, $actual, $tasks) {
            /\A[0-9]+\z/ && $_ > 0 or die "invalid numeric measurement\n";
        }
        my $expected = repeats($fixture, $width);
        my $multiplier = $mode eq 'slow' && $arm eq 'candidate' ? 2 : 1;
        $batch == $expected && $actual == $expected * $multiplier &&
            $tasks == ($fixture =~ /^spine/ ? 16 : 4) or die "batch/task identity mismatch\n";
        !$sample || ($wall >= 1000000 && $cpu >= 1000000) or die "interval below 1 ms\n";
        my $key = join("\t", @f[0..4]);
        !exists $raw{$key} or die "duplicate measurement $key\n";
        $raw{$key} = [$wall, $cpu];
    }
    my @paired;
    my $failed = 0;
    for my $cell (cells($mode)) {
        my ($fixture, $width) = @$cell;
        my (@ratios, @cpu_ratios, @base_wall, @cand_wall, @base_cpu, @cand_cpu, @deltas);
        my ($wall_worse, $cpu_worse, $wall_better, $cpu_better) = (0, 0, 0, 0);
        for my $pass (0..4) {
            my @process;
            for my $arm (qw(baseline candidate)) {
                my (@wall, @cpu);
                for my $sample (0..5) {
                    my $key = join("\t", $fixture, $arm, $width, $pass, $sample);
                    exists $raw{$key} or die "missing measurement $key\n";
                    next unless $sample;
                    push @wall, $raw{$key}[0]; push @cpu, $raw{$key}[1];
                }
                push @process, [median(@wall), median(@cpu)];
            }
            push @ratios, $process[0][0] / $process[1][0];
            push @cpu_ratios, $process[0][1] / $process[1][1];
            $wall_worse++ if $process[1][0] > $process[0][0];
            $cpu_worse++ if $process[1][1] > $process[0][1];
            $wall_better++ if $process[1][0] < $process[0][0];
            $cpu_better++ if $process[1][1] < $process[0][1];
            my $batch = repeats($fixture, $width);
            push @base_wall, $process[0][0] / $batch; push @cand_wall, $process[1][0] / $batch;
            push @base_cpu, $process[0][1] / $batch; push @cand_cpu, $process[1][1] / $batch;
            push @deltas, ($process[1][0] - $process[0][0]) / $batch;
        }
        my $wall = median(@ratios); my $cpu = median(@cpu_ratios);
        my $verdict = judge($mode, $fixture, $width, $wall, $cpu, $wall_worse, $cpu_worse, $wall_better);
        $failed++ if $verdict ne 'pass';
        push @paired, join("\t", $fixture, $width,
            (map { sprintf '%.17g', $_ } $wall, $cpu),
            $wall_worse, $cpu_worse, $wall_better, $cpu_better,
            (map { sprintf '%.17g', $_ } median(@base_wall), median(@cand_wall),
                median(@base_cpu), median(@cand_cpu), median(@deltas)), $verdict);
    }
    return ($failed, \@paired);
}
sub synthetic {
    my ($change) = @_;
    my @rows;
    for my $cell (cells('null')) {
        my ($fixture, $width) = @$cell;
        for my $arm (qw(baseline candidate)) { for my $pass (0..4) { for my $sample (0..5) {
            my ($wall, $cpu) = $change ? $change->($fixture, $width, $arm, $pass) : (10000000, 10000000);
            my $batch = repeats($fixture, $width);
            push @rows, join("\t", 'measure', $fixture, $arm, $width, $pass, $sample,
                $wall, $cpu, $batch, $batch, $fixture =~ /^spine/ ? 16 : 4);
        } } }
    }
    return \@rows;
}
sub self_test {
    my $rows = synthetic();
    my ($failed, $paired) = reduce_rows('null', $rows);
    !$failed && @$paired == 40 or die "identical synthetic control failed\n";
    for my $broken ([@$rows[1..$#$rows]], [@$rows, $rows->[0]]) {
        my $accepted = eval { reduce_rows('null', $broken); 1 };
        !$accepted or die "missing/duplicate sample accepted\n";
    }
    for my $edit ([6, 999999], [8, 65537], [1, 'unselected-fixture']) {
        my @broken = @$rows;
        my @fields = split /\t/, $broken[1];
        $fields[$edit->[0]] = $edit->[1]; $broken[1] = join("\t", @fields);
        my $accepted = eval { reduce_rows('null', \@broken); 1 };
        !$accepted or die "short interval/wrong batch/unknown fixture accepted\n";
    }
    for my $directions (3, 4) {
        my $cpu_rows = synthetic(sub {
            my ($fixture, $width, $arm, $pass) = @_;
            return (10000000, $fixture eq 'n-0-3' && $width == 1 &&
                $arm eq 'candidate' && $pass < $directions ? 12000000 : 10000000);
        });
        my (undef, $cpu_pairs) = reduce_rows('runtime', $cpu_rows);
        my $expected = $directions == 4 ? 'persistent_cpu' : 'noisy_cpu';
        $cpu_pairs->[0] =~ /\t0\t$directions\t0\t0\t.*\t$expected\z/
            or die "CPU directions were not independently counted\n";
    }
    judge('bridge', 'n-9-3', 4, 1/.9, 1, 0, 0, 4) eq 'pass' or die "10% benefit boundary\n";
    judge('bridge', 'n-9-3', 4, 1.1, 1, 0, 0, 5) eq 'insufficient_benefit' or die "1.10 is not 10% reduction\n";
    judge('runtime', 'n-0-3', 1, .97, .90, 5, 5, 0) eq 'pass' or die "protection boundary\n";
    judge('slow', 'n-0-3', 1, .5, .5, 5, 5, 0) eq 'pass' or die "slowdown control\n";
    print "PASS\tDAG cost reducer identity/cardinality/CPU-direction/exact-threshold controls\n";
}
sub capture {
    my ($log, @command) = @_;
    my $pid = fork; defined $pid or die "fork: $!\n";
    if (!$pid) {
        open STDOUT, '>', "$log.stdout" or die "stdout: $!\n";
        open STDERR, '>', "$log.stderr" or die "stderr: $!\n";
        exec @command; die "exec $command[0]: $!\n";
    }
    waitpid($pid, 0) == $pid or die "wait: $!\n";
    return ($? & 127) ? 128 + ($? & 127) : $? >> 8;
}
sub one_measurement {
    my ($mode, $base, $candidate, $results, $fixture, $width, $pass, $arm) = @_;
    local $ENV{WF_WORKERS} = $width;
    local $ENV{WF_SPLIT_WORK}; delete $ENV{WF_SPLIT_WORK};
    my $image = $arm eq 'baseline' ? $base : $candidate;
    my @slow = $mode eq 'slow' && $arm eq 'candidate' ? ('slow') : ();
    my $log = "$results/logs/$fixture-W$width-p$pass-$arm";
    my $status = capture($log, $image, 'measure', $arm, $fixture, $width, $pass, @slow);
    open my $raw, '>>', "$results/raw.tsv" or die "append raw: $!\n";
    print {$raw} read_file("$log.stdout"); close $raw;
    $status == 0 or die "measurement stopped (exit $status): $log.stderr\n";
}
sub phase {
    my ($mode, $base, $candidate, $results, $pass, $position) = @_;
    if ($mode =~ /^verify/) {
        my @arms = $mode eq 'verify-runtime' ? ('candidate') : qw(baseline candidate);
        for my $arm (@arms) { for my $width (@widths) {
            local $ENV{WF_WORKERS} = $width;
            local $ENV{WF_SPLIT_WORK}; delete $ENV{WF_SPLIT_WORK};
            my $log = "$results/logs/verify-$arm-W$width";
            my $status = capture($log, $arm eq 'baseline' ? $base : $candidate, 'wf');
            $status == 0 or die "correctness stopped (exit $status): $log.stderr\n";
        } }
    } elsif ($mode eq 'slow') {
        my @cells = cells($mode);
        for my $p (0..4) { for my $j (0..1) {
            my ($fixture, $width) = @{$cells[($j + $p) % 2]};
            my @arms = (qw(baseline candidate)); @arms = reverse @arms if ($p + $j) % 2;
            one_measurement($mode, $base, $candidate, $results, $fixture, $width, $p, $_) for @arms;
        } }
    } else {
        my $width = $widths[($position + $pass) % 2];
        for my $j (0..$#fixtures) {
            my $fixture = $fixtures[($j + $pass) % @fixtures];
            my @arms = (qw(baseline candidate)); @arms = reverse @arms if ($pass + $j + $position) % 2;
            one_measurement($mode, $base, $candidate, $results, $fixture, $width, $pass, $_) for @arms;
        }
    }
}
sub campaign {
    my ($mode, $base, $candidate, $results) = @_;
    $mode =~ /\A(?:verify|verify-runtime|null|slow|bridge|runtime)\z/ or die "unknown cost stage\n";
    $base = abs_path($base) // die "missing baseline\n";
    $candidate = abs_path($candidate) // die "missing candidate\n";
    -x $base && -x $candidate or die "images must already be executable\n";
    ($mode ne 'null' && $mode ne 'slow') || $base eq $candidate or die "control requires identical image path\n";
    !-e $results or die "results path already exists; retries are forbidden\n";
    make_path("$results/logs"); $results = abs_path($results);
    my @cpu_command = $^O eq 'darwin' ? ('sysctl', '-n', 'hw.logicalcpu') : ('nproc');
    open my $cpu_pipe, '-|', @cpu_command or die "CPU inventory: $!\n";
    my $cpus = <$cpu_pipe> // ''; close $cpu_pipe or die "CPU inventory failed\n";
    chomp $cpus; $cpus =~ /^\d+$/ && $cpus >= 4 or die "four available CPUs required\n";
    my $base_hash = sha256_hex(read_file($base)); my $candidate_hash = sha256_hex(read_file($candidate));
    write_file("$results/manifest.txt", "mode=$mode\ncpus=$cpus\nbaseline=$base\nsha256=$base_hash\ncandidate=$candidate\nsha256=$candidate_hash\npasses=5\nwarmups=1\nsamples=5\nphase_timeout_seconds=30\n");
    write_file("$results/raw.tsv", '');
    my @phases = ($mode =~ /^verify/ || $mode eq 'slow') ? ([0, 0]) :
        map { my $p = $_; map { [$p, $_] } (0, 1) } 0..4;
    for my $at (@phases) {
        my ($p, $v) = @$at;
        local $ENV{WHITEFOOT_CHECK_TIMEOUT} = 30;
        my $label = "dag-cost-$mode-p$p-v$v";
        my $status = capture("$results/$label", 'perl', "$RealBin/../../../.github/run-check.pl",
            $label, 'perl', "$RealBin/dag_fanin_cost.pl", '_phase', $mode, $base, $candidate, $results, $p, $v);
        print "$label\texit=$status\n";
        $status == 0 or die "phase stopped: $results/$label.stderr\n";
    }
    sha256_hex(read_file($base)) eq $base_hash && sha256_hex(read_file($candidate)) eq $candidate_hash
        or die "image changed during campaign\n";
    if ($mode =~ /^verify/) { write_file("$results/verdict.txt", "PASS correctness\n"); return 0 }
    my @lines = split /\n/, read_file("$results/raw.tsv");
    my ($failed, $paired) = reduce_rows($mode, \@lines);
    write_file("$results/paired.tsv", "# fixture\twidth\twall_B_over_C\tcpu_B_over_C\twall_C_slower_pairs\tcpu_C_slower_pairs\twall_C_faster_pairs\tcpu_C_faster_pairs\tB_wall_ns_per_call\tC_wall_ns_per_call\tB_cpu_ns_per_call\tC_cpu_ns_per_call\tpaired_wall_delta_ns_per_call\tverdict\n" . join("\n", @$paired) . "\n");
    my $verdict = ($failed ? 'NOT_QUALIFIED' : 'PASS') . "\tmode=$mode\tcells=" . scalar(@$paired) . "\tfailed_cells=$failed\n";
    write_file("$results/verdict.txt", $verdict); print $verdict;
    return $failed ? (($mode eq 'null' || $mode eq 'slow') ? 2 : 1) : 0;
}
my $status = eval {
    if (@ARGV == 1 && $ARGV[0] eq 'self-test') { self_test(); 0 }
    elsif (@ARGV == 7 && $ARGV[0] eq '_phase') { shift @ARGV; phase(@ARGV); 0 }
    elsif (@ARGV == 4) { campaign(@ARGV) }
    else { die "usage: dag_fanin_cost.pl self-test | verify|verify-runtime|null|slow|bridge|runtime BASE CANDIDATE FRESH_RESULTS\n" }
};
if ($@) { warn $@; exit 2 }
exit $status;
