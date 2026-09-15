#!/usr/bin/env perl
# One owned process group for a heavy command and all nested Make targets.
# Perl and POSIX are already used by CI and available on the supported hosts.
use strict;
use warnings;
use Cwd qw(getcwd);
use POSIX qw(WNOHANG setpgid);
use Time::HiRes qw(clock_gettime CLOCK_MONOTONIC sleep);

@ARGV >= 2 or die "usage: run-check.pl LABEL COMMAND [ARG ...]\n";
my $label = shift;
my $lock = $ENV{WHITEFOOT_CHECK_LOCK_DIR} // "/tmp/whitefoot-check-$<.lock";
my $owner_pid = $$;
my $owns_lock = 0;
sub read_file {
    my ($path) = @_;
    open my $file, '<', $path or return '';
    local $/;
    return <$file> // '';
}
my $recorded = read_file("$lock/pid");
chomp $recorded;
if (($ENV{WHITEFOOT_CHECK_OWNER} // '') eq $recorded
    && $recorded =~ /^\d+$/ && kill(0, $recorded)) {
    # The same top-level command owns nested targets, even across worktrees.
} elsif (mkdir $lock, 0700) {
    $owns_lock = 1;
    $ENV{WHITEFOOT_CHECK_OWNER} = $$;
    open my $pid, '>', "$lock/pid" or die "write lock PID: $!\n";
    print {$pid} "$$\n";
    close $pid;
    open my $command, '>', "$lock/command" or die "write lock command: $!\n";
    print {$command} scalar(gmtime) . " UTC: " . getcwd() . ": $label @ARGV\n";
    close $command;
} else {
    warn "verification is already owned by another command:\n",
        read_file("$lock/pid"), read_file("$lock/command"),
        "lock: $lock; inspect the recorded PID before removing a stale lock\n";
    exit 75;
}
END {
    if ($owns_lock && $$ == $owner_pid) {
        unlink "$lock/pid", "$lock/command";
        rmdir $lock;
    }
}
$ENV{WHITEFOOT_CHECK_LOCK_DIR} = $lock;
for my $name (qw(CARGO_BUILD_JOBS RUST_TEST_THREADS JOBS)) {
    $ENV{$name} //= 2;
}
my $limit = $ENV{WHITEFOOT_CHECK_TIMEOUT} // 1800;
$limit =~ /^\d+$/ && $limit > 0 or die "WHITEFOOT_CHECK_TIMEOUT must be positive seconds\n";
my $started = clock_gettime(CLOCK_MONOTONIC);
my $next_report = $started + 30;
my ($cancelled, $stopping);
$SIG{INT} = sub { $cancelled //= 130 };
$SIG{TERM} = sub { $cancelled //= 143 };
$SIG{HUP} = sub { $cancelled //= 129 };
$SIG{PIPE} = sub { $cancelled //= 141 };
$| = 1;
print "== START $label (limit ${limit}s): @ARGV ==\n";
my $child = fork;
defined $child or die "fork check: $!\n";
if ($child == 0) {
    $SIG{INT} = $SIG{TERM} = $SIG{HUP} = $SIG{PIPE} = 'DEFAULT';
    if ($owns_lock) {
        defined setpgid(0, 0) or die "create check process group: $!\n";
        $ENV{WHITEFOOT_CHECK_PGID} = $$;
    }
    exec '/usr/bin/time', '-p', @ARGV;
    die "execute check: $!\n";
}
my $group = $owns_lock ? $child : $ENV{WHITEFOOT_CHECK_PGID};
setpgid($child, $child) if $owns_lock;
defined $group && $group =~ /^\d+$/ or die "missing owned check process group\n";
my ($status, $completed);
while (1) {
    my $now = clock_gettime(CLOCK_MONOTONIC);
    if (!defined $status) {
        my $waited = waitpid($child, WNOHANG);
        if ($waited == $child) {
            $status = $?;
            $completed = $now;
        }
    }
    if (defined $status && $owns_lock && kill(0, -$group)
        && $now - $completed >= 0.5 && !defined $cancelled) {
        warn "== $label left child processes after command exit ==\n";
        $cancelled = ($status & 127) ? 128 + ($status & 127) : ($status >> 8) || 1;
    }
    $cancelled //= 124 if $now - $started >= $limit;
    if (defined $cancelled && !defined $stopping) {
        $stopping = $now;
        warn "== STOP $label: ", ($cancelled == 124 ? 'deadline exceeded' : 'interrupted'), " ==\n";
        # A nested wrapper is inside the owned group. Ignore our own TERM;
        # the top-level owner stays outside, reaps, and releases the lock.
        $SIG{TERM} = 'IGNORE';
        kill 'TERM', $ENV{WHITEFOOT_CHECK_OWNER} if !$owns_lock;
        kill 'TERM', -$group;
    }
    if (defined $stopping && $now - $stopping >= 2) {
        kill 'KILL', -$group;
        waitpid($child, 0) if !defined $status;
        last;
    }
    last if defined $status && !defined $stopping
        && (!$owns_lock || !kill(0, -$group));
    if ($now >= $next_report) {
        printf "== RUNNING %s: %.0f s, child %d, group %d ==\n", $label, $now - $started, $child, $group;
        $next_report = $now + 30;
    }
    sleep 0.1;
}
my $code = $cancelled // (($status & 127) ? 128 + ($status & 127) : $status >> 8);
printf "== END %s: %.2f s, exit %d ==\n", $label, clock_gettime(CLOCK_MONOTONIC) - $started, $code;
exit $code;
