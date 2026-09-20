#!/usr/bin/env perl
use strict;
use warnings;

@ARGV == 1 or die "usage: perl summarize.pl raw.csv\n";
open my $in, '<', $ARGV[0] or die "$ARGV[0]: $!\n";
chomp(my $header = <$in>);
my @columns = split /,/, $header;
my (%samples, %rows, %resources);
my ($measured, $warmups) = (0, 0);
while (<$in>) {
    chomp;
    my @values = split /,/;
    @values == @columns or die "wrong CSV width at line $.\n";
    my %r; @r{@columns} = @values;
    if ($r{phase} eq 'warmup') { ++$warmups; next; }
    $r{phase} eq 'measure' or die "unknown phase\n";
    ++$measured;
    my $cell = join '/', @r{qw(tier payload load workload)};
    exists $samples{$cell}{$r{variant}}{$r{rep}} and die "duplicate measurement\n";
    $samples{$cell}{$r{variant}}{$r{rep}} = 0 + $r{ns_per_op};
    $rows{$cell}{$r{variant}} = \%r;
    for my $field (qw(minor_faults major_faults invol_cs)) {
        $resources{$r{workload}}{$field} += $r{$field};
    }
}
close $in;
$measured == 5400 && $warmups == 900 or die "incomplete run: $measured measurements, $warmups warmups\n";
my @variants = qw(a_raw b_tag c_box d_folded e_proved);
my @workloads = qw(hit miss mix dependent insert);
my @pairs = ([qw(b_tag a_raw)], [qw(b_tag d_folded)], [qw(c_box a_raw)],
             [qw(c_box d_folded)], [qw(b_tag e_proved)], [qw(a_raw d_folded)]);

sub quantile {
    my ($q, @x) = @_;
    @x = sort {$a <=> $b} @x;
    my $position = $q * $#x;
    my $lo = int $position;
    return $x[$lo] if $lo == $#x;
    return $x[$lo] + ($position - $lo) * ($x[$lo + 1] - $x[$lo]);
}
sub observations {
    my ($cell, $v) = @_;
    my $s = $samples{$cell}{$v} // die "missing cell $cell/$v\n";
    keys(%$s) == 12 or die "wrong repetition count $cell/$v\n";
    return map { defined($s->{$_}) ? $s->{$_} : die "missing repetition $_\n" } 0..11;
}
sub ratios {
    my ($cell, $a, $b) = @_;
    my @a = observations($cell, $a);
    my @b = observations($cell, $b);
    return map { 100 * ($a[$_] / $b[$_] - 1) } 0..11;
}
sub display {
    my @x = @_;
    return sprintf '%.2f [%.2f, %.2f]', quantile(.5, @x), quantile(.25, @x), quantile(.75, @x);
}
print "# Full measurements\n\n";
print "Generated from raw.csv by summarize.pl. $measured measured samples, $warmups warmup samples.\n";
print "Each cell is median [p25, p75], with 12 repetitions and linear-interpolated quantiles.\n";
print "Percentages are medians of paired repetition ratios, not ratios of medians.\n\n";
for my $tier (qw(L1 L2 large)) {
    for my $p (8, 32, 128) {
        print "## $tier, $p-byte payload\n\n";
        print "Time in ns/operation.\n\n| Load | Workload | A | B | C | D | E |\n";
        print "| --- | --- | --- | --- | --- | --- | --- |\n";
        for my $load (qw(0.500 0.875)) {
            for my $work (@workloads) {
                my $cell = "$tier/$p/$load/$work";
                print "| $load | $work | ", join(' | ', map { display(observations($cell, $_)) } @variants), " |\n";
            }
        }
        print "\nPaired overhead in percent; positive means the numerator is slower.\n\n";
        print "| Load | Workload | B/A | B/D | C/A | C/D | B/E | A/D control |\n";
        print "| --- | --- | --- | --- | --- | --- | --- | --- |\n";
        for my $load (qw(0.500 0.875)) {
            for my $work (@workloads) {
                my $cell = "$tier/$p/$load/$work";
                print "| $load | $work | ", join(' | ', map { display(ratios($cell, @$_)) } @pairs), " |\n";
            }
        }
        print "\n";
    }
}
print "## Registered criterion\n\n";
print "All four conditions must pass: hit and dependent at each load.\n\n";
print "| Tier | Payload | B/D passing cells | B/E passing cells | Noisy A/D cells | Verdict |\n";
print "| --- | --- | --- | --- | --- | --- |\n";
for my $tier (qw(L1 L2 large)) { for my $p (8, 32, 128) {
    my ($bd, $be, $noise) = (0, 0, 0);
    for my $load (qw(0.500 0.875)) { for my $work (qw(hit dependent)) {
        my $cell = "$tier/$p/$load/$work";
        $bd += quantile(.25, ratios($cell, 'b_tag', 'd_folded')) > 5;
        $be += quantile(.25, ratios($cell, 'b_tag', 'e_proved')) > 5;
        $noise += abs(quantile(.5, ratios($cell, 'a_raw', 'd_folded'))) > 5;
    }}
    my $verdict = $noise ? 'inconclusive: control noise' : $bd < 4 ? 'threshold not met' :
        $be == 4 ? 'duplicate-check criterion met' : 'representation only';
    print "| $tier | $p | $bd/4 | $be/4 | $noise/4 | $verdict |\n";
}}
print "\n## Active table footprints\n\n";
print "Table + occupied entry allocation bytes, excluding table descriptors and allocator metadata.\n";
print "Entry allocation uses malloc_size on this Apple run. Query bytes are additional.\n\n";
print "| Tier | P | Load | Capacity | A = D bytes | B = E bytes | C bytes | Query bytes |\n";
print "| --- | --- | --- | --- | --- | --- | --- | --- |\n";
for my $tier (qw(L1 L2 large)) { for my $p (8, 32, 128) { for my $load (qw(0.500 0.875)) {
    my $c = "$tier/$p/$load/hit";
    my @r = map { $rows{$c}{$_} } @variants;
    print "| $tier | $p | $load | $r[0]{capacity} | $r[0]{table_bytes} | $r[1]{table_bytes} | ",
        $r[2]{table_bytes} + $r[2]{entry_bytes}, " | $r[0]{query_bytes} |\n";
}}}
print "\n## Resource observations\n\n| Workload | Minor faults | Major faults | Involuntary switches |\n";
print "| --- | --- | --- | --- |\n";
for my $work (@workloads) {
    print "| $work | ", join(' | ', map { $resources{$work}{$_} // 0 } qw(minor_faults major_faults invol_cs)), " |\n";
}
print "\n## Raw ranges\n\nMinimum and maximum ns/operation; these are not confidence intervals.\n\n";
print "| Tier | P | Load | Workload | A min/max | B min/max | C min/max | D min/max | E min/max |\n";
print "| --- | --- | --- | --- | --- | --- | --- | --- | --- |\n";
for my $tier (qw(L1 L2 large)) { for my $p (8, 32, 128) { for my $load (qw(0.500 0.875)) {
    for my $work (@workloads) {
        my $c = "$tier/$p/$load/$work";
        print "| $tier | $p | $load | $work | ", join(' | ', map {
            my @x = observations($c, $_); sprintf '%.2f/%.2f', quantile(0, @x), quantile(1, @x)
        } @variants), " |\n";
    }
}}}
