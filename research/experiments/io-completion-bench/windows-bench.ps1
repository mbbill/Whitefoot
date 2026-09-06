param(
    [Parameter(Mandatory = $true)]
    [string]$Root,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [int]$Rounds = 15,

    [int]$Warmup = 2,

    [switch]$Enforce
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if ($Rounds -lt 5 -or ($Rounds % 2) -eq 0) {
    throw "Rounds must be an odd integer of at least five"
}
if ($Warmup -lt 1) {
    throw "Warmup must be positive"
}

$Root = [IO.Path]::GetFullPath($Root)
$Out = [IO.Path]::GetFullPath($Out)
if (Test-Path -LiteralPath $Out) {
    throw "benchmark output directory must not already exist: $Out"
}
[void](New-Item -ItemType Directory -Path $Out)

$Bundle = Join-Path $Root "research/experiments/io-completion-bench"
$Programs = Join-Path $Bundle "programs"
$Backend = Join-Path $Root "compiler/src/backend"
$Completion = Join-Path $Backend "completion"
$Tree = Join-Path $Out "tree"
$Bin = Join-Path $Out "bin"
$Objects = Join-Path $Out "objects"
[void](New-Item -ItemType Directory -Path $Bin)
[void](New-Item -ItemType Directory -Path $Objects)

$Clang = (Get-Command clang.exe -ErrorAction Stop).Source
$Cargo = (Get-Command cargo.exe -ErrorAction Stop).Source
$Git = (Get-Command git.exe -ErrorAction Stop).Source
$Workers = [Math]::Min(64, [Environment]::ProcessorCount)
if ($Workers -lt 2) {
    throw "the Windows compute qualification requires at least two logical processors"
}
if ($Workers -eq 64) {
    $AffinityMask = [UInt64]::MaxValue
} else {
    $AffinityMask = ([UInt64]1 -shl $Workers) - 1
}
$AffinityHex = $AffinityMask.ToString("x", [Globalization.CultureInfo]::InvariantCulture)

function Invoke-Tool {
    param(
        [Parameter(Mandatory = $true)][string]$File,
        [Parameter(Mandatory = $true)][string[]]$Arguments,
        [Parameter(Mandatory = $true)][string]$Description
    )
    & $File @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Description failed with exit $LASTEXITCODE"
    }
}

function Write-AsciiFile {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Text
    )
    [IO.File]::WriteAllBytes($Path, [Text.Encoding]::ASCII.GetBytes($Text))
}

# One compilation rule for every observed unit, and it is the repository
# gate's own warning set: the runtime this script links is the shipped runtime,
# so a unit that would not build under `make check` must not build here either.
# No unit takes a `-D` of its own any more -- the runtime has one shipped form
# and no probe variants to select.
function Compile-Object {
    param(
        [Parameter(Mandatory = $true)][string]$Source,
        [Parameter(Mandatory = $true)][string]$Output
    )
    $arguments = @(
        "-std=c11", "-O2", "-g", "-Wall", "-Wextra", "-Werror",
        "-Wpedantic", "-municode", "-I", $Backend, "-I", $Completion,
        "-c", $Source, "-o", $Output
    )
    Invoke-Tool -File $Clang -Arguments $arguments -Description "compile $Source"
}

# The body of one emitted definition, so an order is pinned inside the
# function that carries it rather than anywhere in the module. `wf_main` and
# `wf_compute_pair` are both defined before anything calls them, so the first
# occurrence of the symbol is its definition, and an emitted body is the only
# text whose closing brace starts a line.
function Get-EmittedFunction {
    param(
        [Parameter(Mandatory = $true)][string]$Module,
        [Parameter(Mandatory = $true)][string]$Symbol
    )
    $at = $Module.IndexOf("@$Symbol(", [StringComparison]::Ordinal)
    if ($at -lt 0) {
        throw "the emitted mixed module defines no $Symbol"
    }
    $end = $Module.IndexOf("`n}`n", $at, [StringComparison]::Ordinal)
    if ($end -lt 0) {
        throw "the emitted definition of $Symbol is unterminated"
    }
    return $Module.Substring($at, $end - $at)
}

function Warm-Tree {
    $buffer = [byte[]]::new(1024 * 1024)
    for ($index = 0; $index -lt 8; $index += 1) {
        $path = Join-Path $Tree ("f{0:D5}.dat" -f $index)
        $stream = [IO.File]::OpenRead($path)
        try {
            while ($stream.Read($buffer, 0, $buffer.Length) -ne 0) {
            }
        } finally {
            $stream.Dispose()
        }
    }
}

$Target = if (Test-Path Env:CARGO_TARGET_DIR) {
    [IO.Path]::GetFullPath($env:CARGO_TARGET_DIR)
} else {
    Join-Path $Root "compiler/target"
}
$Wfc = Join-Path $Target "gate/whitefootc.exe"
$Runner = Join-Path $Bin "windows-runner.exe"
$Generator = Join-Path $Bin "gen.exe"

Invoke-Tool -File $Cargo -Arguments @(
    "build", "--manifest-path", (Join-Path $Root "compiler/Cargo.toml"),
    "--profile", "gate", "--locked", "--offline", "--bin", "whitefootc"
) -Description "build whitefootc"

Invoke-Tool -File $Clang -Arguments @(
    "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-Wpedantic",
    "-municode", (Join-Path $Bundle "windows_runner.c"), "-o", $Runner
) -Description "build the native Windows sample runner"

Invoke-Tool -File $Clang -Arguments @(
    "-std=c11", "-O2", "-Wall", "-Wextra", "-Werror", "-Wpedantic",
    (Join-Path $Bundle "gen.c"), "-o", $Generator
) -Description "build the deterministic data generator"

Invoke-Tool -File $Generator -Arguments @($Tree, "8", "65536", "fixed") `
    -Description "generate the fixed 8 x 64 MiB tree"
Warm-Tree

$ComputeExpected = Join-Path $Out "compute.expected"
$IoExpected = Join-Path $Out "io.expected"
$MixedExpected = Join-Path $Out "mixed.expected"
$ComputeArguments = @("batch", "batch", "batch")
$IoArguments = @(
    "f00000.dat", "f00001.dat", "f00002.dat", "f00003.dat",
    "f00004.dat", "f00005.dat", "f00006.dat", "f00007.dat"
)
Write-AsciiFile -Path $ComputeExpected -Text "420a993efa7437a1 41fa962893d45299`n"
Write-AsciiFile -Path $IoExpected -Text "18028327385673861873 00000000000134217728`n"
Write-AsciiFile -Path $MixedExpected -Text "17574306422404092952`n"

$ComputeSeq = Join-Path $Bin "compute-seq.exe"
$ComputePar = Join-Path $Bin "compute-par.exe"
$IoDirect = Join-Path $Bin "io-direct.exe"
$IoIocp = Join-Path $Bin "io-iocp.exe"
$MixedSeq = Join-Path $Bin "mixed-seq.exe"
$MixedIocp = Join-Path $Bin "mixed-iocp.exe"
$MixedFull = Join-Path $Bin "mixed-full.exe"
$MixedIr = Join-Path $Out "mixed-full.ll"
$MixedSource = Join-Path $Programs "windows_runtime_mixed.wf"

Invoke-Tool -File $Wfc -Arguments @(
    (Join-Path $Root "tests/programs/par_layout.wf"), "-o", $ComputeSeq
) -Description "compile the compute reference"
Invoke-Tool -File $Wfc -Arguments @(
    "--par", (Join-Path $Root "tests/programs/par_layout.wf"), "-o", $ComputePar
) -Description "compile the native compute-pool contender"
Invoke-Tool -File $Wfc -Arguments @(
    "--no-overlap", (Join-Path $Programs "read_heavy_wide8_4k.wf"),
    "-o", $IoDirect
) -Description "compile the sequential positioned-read reference"
Invoke-Tool -File $Wfc -Arguments @(
    (Join-Path $Programs "read_heavy_wide8_4k.wf"), "-o", $IoIocp
) -Description "compile the IOCP positioned-read contender"
Invoke-Tool -File $Wfc -Arguments @(
    "--no-overlap", $MixedSource, "-o", $MixedSeq
) -Description "compile the fully sequential mixed reference"
Invoke-Tool -File $Wfc -Arguments @($MixedSource, "-o", $MixedIocp) `
    -Description "compile the IOCP-only mixed control"
Invoke-Tool -File $Wfc -Arguments @("--par", $MixedSource, "-o", $MixedFull) `
    -Description "compile the unified compute and IOCP contender"
Invoke-Tool -File $Wfc -Arguments @(
    "--par", "--emit-llvm", $MixedSource, "-o", $MixedIr
) -Description "emit the observed mixed module"

# What this pins, and why it is that shape.
#
# The mixed window is source-level `read_at, compute_pair, read_at`, and the
# permission judgment reads it as one three-member chain -- `--par-ledger`
# prints `run(read_at, compute_pair, read_at)  3 members` for it. Every I/O
# operation now has exactly one lowering, submit and then join
# (`research/investigations/io-model/PARK-ON-MISS.md` section 8, "One lowering
# for every I/O operation"), so there is no direct-read family left to look
# for: `@wf.sys.read_at.v1` is the always-inlined wrapper that submits and
# joins, not a direct target read.
#
# So `@wf_main` carries, in this order:
#
#   1. `wf__completion_file_pread_submit` for the first read, which the group
#      leaves in flight,
#   2. the call to `@wf_compute_pair` on this thread,
#   3. the source-last read through `@wf.sys.read_at.v1`, which submits and
#      joins in place, and
#   4. `wf__completion_file_join` for the first read.
#
# That order is exactly what this cohort claims to measure: the first read is
# outstanding across both the compute member and the second read.
#
# The group hands none of its own members to a compute lane, and that is the
# emitter's stated rule rather than an accident: a group whose join site is
# itself a submitting member keeps the pure completion lowering
# (`ordinary_overlap_lane_frames`, `compiler/src/backend/emitter.rs`), and here
# the source-last member is a `read_at`. Section 4 leaves a completion member
# where it was published in either case. The compute overlap this cohort
# measures is one level down, inside `@wf_compute_pair`, whose own
# `pair(churn, churn)` group takes the lane protocol -- acquire a lane, publish
# into it, join it, release it -- 1024 times per run. That is the hand-out the
# observed link below reads back as `grants=`.
#
# The order is a property of the emitter and not of the target, so reading it
# here on the Windows host reads the same shape a Linux `--emit-llvm` of this
# program reads. The completion schedule and the overlap group are both
# produced by lowering, which takes no target; `ordinary_overlap_lane_frames`
# declines this group before it reaches anything target-dependent; and the
# Windows and native POSIX columns name the same submit and join symbols
# (`HostFacilities` in `compiler/src/backend/qualification.rs`).
$Ir = [IO.File]::ReadAllText($MixedIr)
$Main = Get-EmittedFunction -Module $Ir -Symbol "wf_main"
$Pair = Get-EmittedFunction -Module $Ir -Symbol "wf_compute_pair"
$SubmitAt = $Main.IndexOf(
    "call void @wf__completion_file_pread_submit(", [StringComparison]::Ordinal
)
$ComputeAt = if ($SubmitAt -ge 0) {
    $Main.IndexOf("call i64 @wf_compute_pair(", $SubmitAt, [StringComparison]::Ordinal)
} else { -1 }
$LastReadAt = if ($ComputeAt -ge 0) {
    $Main.IndexOf("@wf.sys.read_at.v1(", $ComputeAt, [StringComparison]::Ordinal)
} else { -1 }
$CompletionJoinAt = if ($LastReadAt -ge 0) {
    $Main.IndexOf(
        "call void @wf__completion_file_join(", $LastReadAt, [StringComparison]::Ordinal
    )
} else { -1 }
if ($SubmitAt -lt 0 -or $ComputeAt -le $SubmitAt -or $LastReadAt -le $ComputeAt `
    -or $CompletionJoinAt -le $LastReadAt) {
    throw "mixed IR does not keep the first read in flight across the compute member and the source-last read"
}
$AcquireAt = $Pair.IndexOf("call ptr @wf__par_acquire_lane(", [StringComparison]::Ordinal)
$PublishAt = if ($AcquireAt -ge 0) {
    $Pair.IndexOf("call void @wf__par_publish(", $AcquireAt, [StringComparison]::Ordinal)
} else { -1 }
$ParJoinAt = if ($PublishAt -ge 0) {
    $Pair.IndexOf("call void @wf__par_join(", $PublishAt, [StringComparison]::Ordinal)
} else { -1 }
$ReleaseAt = if ($ParJoinAt -ge 0) {
    $Pair.IndexOf("call void @wf__par_release(", $ParJoinAt, [StringComparison]::Ordinal)
} else { -1 }
if ($AcquireAt -lt 0 -or $PublishAt -le $AcquireAt -or $ParJoinAt -le $PublishAt `
    -or $ReleaseAt -le $ParJoinAt) {
    throw "the mixed program's compute member does not offer a lane, publish into it, join it and release it"
}

# The observed link: the shipped runtime, plus one unit no shipped program
# carries.
#
# The production executables above are `whitefootc.exe`'s own links and need
# nothing added to them. This one link exists to read back a fact that correct
# bytes alone would also be produced without: that the mixed contender really
# stole compute work and really carried its reads on the completion port. It is
# `io-hosts.yml`'s `completion-windows` step "Require native workers for a real
# --par program" applied to this program, and the unit list below is exactly
# what `whitefootc` stages for a module that both hands work out and submits
# operations -- `runtime_units(core, completion)` in
# `compiler/src/bin/whitefootc.rs`: the floor, the scheduler core with its
# Windows leaf, and the completion runtime with its Windows wait, host leaf and
# ring. The one addition is `sched/grant_observer.c`, which registers an
# `atexit` report of the core's own steal count and is linked into no shipped
# program.
#
# What the retired probes measured, and what carries it now. The Windows
# parallel runtime, its identity probe and its mixed probe are gone with the
# second copy of the runtime they belonged to, so their counters are gone with
# them:
#
#   `wf__par_started_worker_count` and `wf__par_worker_execution_count`
#     -> `wf__par_grants`, reported as the `grants=` line. The core counts
#        hand-outs that ran on a thread other than the one that offered them,
#        which is the property those two were together standing in for: a pool
#        that started and never granted a lane cannot produce a positive count.
#   `publishes`, `outstanding_publishes` and `kernel_overlap_publishes`
#     (`WF_PAR_MIXED_PROBE`)
#     -> the IR assertion above. That the compute member runs while the first
#        read is outstanding is a property of the one lowering, fixed for every
#        iteration by the emitted order, so it is pinned where it is decided
#        instead of counted once per run.
#   the IOCP inline and dequeued completion counts
#     -> nothing on the protocol's side, deliberately. The ring still keeps
#        both (`wf_windows_iocp_statistics` in `completion/windows_iocp.h`) and
#        the bridge still exports the shared halves of that split
#        (`wf__completion_native_ring_submissions` and
#        `wf__completion_inline_executions`), but the split was a throughput
#        fact and never a verdict, and what this protocol needs from the ring
#        -- that it carried the reads and reaped them -- is the exit assertion
#        below.
#   `submissions`, `publications`, `consumes`, `helpers` and `fallback`
#     -> the shared bridge's own statistics entries
#        (`completion/bridge.h`: `wf__completion_file_submissions`,
#        `wf__completion_publications`,
#        `wf__completion_file_helper_executions`,
#        `wf__completion_file_fallback_submissions`). There is no consume step
#        left to count: the record is the submitting frame's, so a completion
#        is published into it and joined there.
#
# The second half of the protocol is the runtime's own: with
# `WF_REQUIRE_WINDOWS_IOCP=1` the bridge asserts at exit that the port carried
# at least one submission and that it reaped every submission it made
# (`wf_bridge_verify_required_ring` in `completion/bridge.c`), and fails the
# process otherwise. So a run that silently reached no ring cannot answer zero
# here; it cannot exit zero at all.
$ObservedUnits = @(
    "sched/core.c",
    "sched/prim_windows.c",
    "sched/entry.c",
    "completion/runtime.c",
    "completion/wait_windows.c",
    "completion/file_adapter.c",
    "completion/file_windows.c",
    "completion/bridge.c",
    "completion/windows_iocp.c",
    "windows_runtime.c",
    "wf_floor_windows.c",
    "sched/grant_observer.c"
)
$ObservedObjects = @()
foreach ($unit in $ObservedUnits) {
    $object = Join-Path $Objects (($unit -replace '/', '-') -replace '\.c$', '.o')
    Compile-Object -Source (Join-Path $Backend $unit) -Output $object
    $ObservedObjects += $object
}
$MixedObserved = Join-Path $Bin "mixed-observed.exe"
# Winsock is on the line because the completion port's unit and the Windows
# host runtime name it since the TCP routes landed, exactly as `whitefootc`'s
# own Windows link and `io-hosts.yml`'s hand-written ones do.
$LinkArguments = @(
    "-std=c11", "-O2", "-g", "-municode", "-x", "ir", $MixedIr,
    "-x", "none"
) + $ObservedObjects + @("-Wno-override-module", "-o", $MixedObserved, "-lws2_32")
Invoke-Tool -File $Clang -Arguments $LinkArguments `
    -Description "link the observed mixed executable"

$ObservedStart = [Diagnostics.ProcessStartInfo]::new()
$ObservedStart.FileName = $MixedObserved
$ObservedStart.WorkingDirectory = $Tree
$ObservedStart.UseShellExecute = $false
$ObservedStart.RedirectStandardOutput = $true
$ObservedStart.RedirectStandardError = $true
$ObservedStart.ArgumentList.Add("f00000.dat")
$ObservedStart.ArgumentList.Add("f00001.dat")
$ObservedStart.Environment["WF_WORKERS"] = [string]$Workers
$ObservedStart.Environment["WF_REQUIRE_WINDOWS_IOCP"] = "1"
$ObservedProcess = [Diagnostics.Process]::Start($ObservedStart)
$ObservedStdoutTask = $ObservedProcess.StandardOutput.ReadToEndAsync()
$ObservedStderrTask = $ObservedProcess.StandardError.ReadToEndAsync()
if (-not $ObservedProcess.WaitForExit(120000)) {
    $ObservedProcess.Kill($true)
    $ObservedProcess.WaitForExit()
    $ObservedProcess.Dispose()
    throw "the observed mixed run exceeded the 120000 ms timeout"
}
$ObservedStdout = $ObservedStdoutTask.GetAwaiter().GetResult()
$ObservedStderr = $ObservedStderrTask.GetAwaiter().GetResult()
$ObservedExitCode = $ObservedProcess.ExitCode
$ObservedProcess.Dispose()
$ExpectedMixedText = [Text.Encoding]::ASCII.GetString([IO.File]::ReadAllBytes($MixedExpected))
# The whole verdict, in three parts. The exit status carries the required-ring
# assertion: with `WF_REQUIRE_WINDOWS_IOCP=1` a run that submitted nothing to
# the port, or left anything it submitted unreaped, ends nonzero. The bytes
# carry the program. And the one diagnostic line carries the steal: the
# observer is the only thing in this link that writes to the diagnostic
# channel, so anything else on it is a runtime complaint and fails the run
# whatever it says.
$ObservedLines = @(
    ($ObservedStderr -replace "`r", "") -split "`n" | Where-Object { $_ -ne "" }
)
$ObservedPattern = '^grants=[1-9][0-9]*$'
if ($ObservedExitCode -ne 0 -or $ObservedStdout -cne $ExpectedMixedText `
    -or $ObservedLines.Count -ne 1 -or $ObservedLines[0] -cnotmatch $ObservedPattern) {
    throw "the observed mixed run did not show a steal and a reaped port submission: exit=$ObservedExitCode stdout=[$ObservedStdout] stderr=[$ObservedStderr]"
}
$ObservedLine = $ObservedLines[0]
[IO.File]::WriteAllText(
    (Join-Path $Out "mixed-observer.txt"),
    $ObservedLine + "`n",
    [Text.UTF8Encoding]::new($false)
)

$HostPath = Join-Path $Out "host.txt"
$Os = Get-CimInstance Win32_OperatingSystem
$Cpu = Get-CimInstance Win32_Processor | Select-Object -First 1
$Power = (& powercfg.exe /getactivescheme) -join " "
$ClangVersion = (& $Clang --version | Select-Object -First 1)
$RustVersion = (& rustc.exe --version)
$Revision = (& $Git -C $Root rev-parse HEAD)
$HostLines = @(
    "revision: $Revision",
    "runner image: $env:ImageOS $env:ImageVersion",
    "os: $($Os.Caption) $($Os.Version) build $($Os.BuildNumber)",
    "cpu: $($Cpu.Name)",
    "logical processors visible: $([Environment]::ProcessorCount)",
    "workers: $Workers",
    "affinity mask: 0x$AffinityHex",
    "compute batches per sampled child: $($ComputeArguments.Count + 1)",
    "memory bytes: $($Os.TotalVisibleMemorySize * 1024)",
    "power: $Power",
    "clang: $ClangVersion",
    "rust: $RustVersion",
    "cache state: warm, verified by a full sequential pre-read of all eight files",
    "protocol: $Warmup warm-up pairs, $Rounds recorded alternating pairs, QPC wall and child process CPU"
)
[IO.File]::WriteAllLines($HostPath, $HostLines, [Text.UTF8Encoding]::new($false))

$Variants = @{
    "compute.seq" = @{
        Exe = $ComputeSeq; Args = $ComputeArguments; Expected = $ComputeExpected
        Workers = $false; RequireIocp = $false
    }
    "compute.par" = @{
        Exe = $ComputePar; Args = $ComputeArguments; Expected = $ComputeExpected
        Workers = $true; RequireIocp = $false
    }
    "io.direct" = @{
        Exe = $IoDirect; Args = $IoArguments; Expected = $IoExpected
        Workers = $false; RequireIocp = $false
    }
    "io.iocp" = @{
        Exe = $IoIocp; Args = $IoArguments; Expected = $IoExpected
        Workers = $false; RequireIocp = $true
    }
    "mixed.seq" = @{
        Exe = $MixedSeq; Args = @("f00000.dat", "f00001.dat")
        Expected = $MixedExpected; Workers = $false; RequireIocp = $false
    }
    "mixed.iocp" = @{
        Exe = $MixedIocp; Args = @("f00000.dat", "f00001.dat")
        Expected = $MixedExpected; Workers = $false; RequireIocp = $true
    }
    "mixed.full" = @{
        Exe = $MixedFull; Args = @("f00000.dat", "f00001.dat")
        Expected = $MixedExpected; Workers = $true; RequireIocp = $true
    }
}

$RawPath = Join-Path $Out "raw.tsv"
$Raw = [IO.StreamWriter]::new($RawPath, $false, [Text.UTF8Encoding]::new($false))
$Raw.WriteLine("cohort`tattempt`tpair`torder`tvariant`twall_ms`tuser_ms`tkernel_ms")

function Invoke-Sample {
    param(
        [Parameter(Mandatory = $true)][string]$Variant,
        [Parameter(Mandatory = $true)][string]$Label
    )
    $configuration = $Variants[$Variant]
    [Environment]::SetEnvironmentVariable("WF_WORKERS", $null, "Process")
    [Environment]::SetEnvironmentVariable("WF_REQUIRE_WINDOWS_IOCP", $null, "Process")
    if ($configuration.Workers) {
        [Environment]::SetEnvironmentVariable("WF_WORKERS", [string]$Workers, "Process")
    }
    if ($configuration.RequireIocp) {
        [Environment]::SetEnvironmentVariable("WF_REQUIRE_WINDOWS_IOCP", "1", "Process")
    }
    $arguments = @(
        $Label, $Tree, $configuration.Expected, $AffinityHex,
        $configuration.Exe
    ) + $configuration.Args
    $lines = @(& $Runner @arguments)
    if ($LASTEXITCODE -ne 0 -or $lines.Count -ne 1) {
        throw "invalid native sample for $Variant"
    }
    $fields = $lines[0] -split "`t"
    if ($fields.Count -ne 5 -or $fields[0] -cne $Label -or $fields[4] -cne "0") {
        throw "malformed native sample for ${Variant}: $($lines[0])"
    }
    return [pscustomobject]@{
        Variant = $Variant
        Wall = [double]::Parse($fields[1], [Globalization.CultureInfo]::InvariantCulture)
        User = [double]::Parse($fields[2], [Globalization.CultureInfo]::InvariantCulture)
        Kernel = [double]::Parse($fields[3], [Globalization.CultureInfo]::InvariantCulture)
    }
}

function Median {
    param([Parameter(Mandatory = $true)][double[]]$Values)
    $ordered = @($Values | Sort-Object)
    $middle = [int][Math]::Floor($ordered.Count / 2)
    if (($ordered.Count % 2) -eq 1) {
        return [double]$ordered[$middle]
    }
    return ([double]$ordered[$middle - 1] + [double]$ordered[$middle]) / 2.0
}

function Percentile {
    param(
        [Parameter(Mandatory = $true)][double[]]$Values,
        [Parameter(Mandatory = $true)][double]$Fraction
    )
    $ordered = @($Values | Sort-Object)
    $index = [int][Math]::Round(
        ($ordered.Count - 1) * $Fraction,
        [MidpointRounding]::AwayFromZero
    )
    return [double]$ordered[$index]
}

function Run-CohortAttempt {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Reference,
        [Parameter(Mandatory = $true)][string]$Candidate,
        [Parameter(Mandatory = $true)][int]$Attempt
    )
    for ($warm = 0; $warm -lt $Warmup; $warm += 1) {
        $warmOrder = if (($warm % 2) -eq 0) {
            @($Reference, $Candidate)
        } else {
            @($Candidate, $Reference)
        }
        foreach ($variant in $warmOrder) {
            [void](Invoke-Sample -Variant $variant -Label "warm.$Name.$warm.$variant")
        }
    }
    $ratios = [Collections.Generic.List[double]]::new()
    $referenceWalls = [Collections.Generic.List[double]]::new()
    $candidateWalls = [Collections.Generic.List[double]]::new()
    for ($pair = 0; $pair -lt $Rounds; $pair += 1) {
        $order = if (($pair % 2) -eq 0) {
            @($Reference, $Candidate)
        } else {
            @($Candidate, $Reference)
        }
        $pairSamples = @{}
        for ($position = 0; $position -lt 2; $position += 1) {
            $variant = $order[$position]
            $sample = Invoke-Sample -Variant $variant `
                -Label "$Name.$Attempt.$pair.$position.$variant"
            $pairSamples[$variant] = $sample
            $Raw.WriteLine((
                "{0}`t{1}`t{2}`t{3}`t{4}`t{5:F3}`t{6:F3}`t{7:F3}" -f
                $Name, $Attempt, $pair, $position, $variant,
                $sample.Wall, $sample.User, $sample.Kernel
            ))
            $Raw.Flush()
        }
        $referenceWall = [double]$pairSamples[$Reference].Wall
        $candidateWall = [double]$pairSamples[$Candidate].Wall
        $referenceWalls.Add($referenceWall)
        $candidateWalls.Add($candidateWall)
        $ratios.Add($candidateWall / $referenceWall)
    }
    $ratioValues = [double[]]$ratios.ToArray()
    $ratioMedian = Median -Values $ratioValues
    $deviations = [double[]]@($ratioValues | ForEach-Object { [Math]::Abs($_ - $ratioMedian) })
    $mad = Median -Values $deviations
    $p10 = Percentile -Values $ratioValues -Fraction 0.10
    $p90 = Percentile -Values $ratioValues -Fraction 0.90
    $referenceMedian = Median -Values ([double[]]$referenceWalls.ToArray())
    $candidateMedian = Median -Values ([double[]]$candidateWalls.ToArray())
    return [pscustomobject]@{
        Name = $Name
        Reference = $Reference
        Candidate = $Candidate
        Attempt = $Attempt
        ReferenceMedian = $referenceMedian
        CandidateMedian = $candidateMedian
        Ratio = $ratioMedian
        MadFraction = $mad / $ratioMedian
        SpreadFraction = ($p90 - $p10) / $ratioMedian
        P10 = $p10
        P90 = $p90
    }
}

function Run-QualifiedCohort {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Reference,
        [Parameter(Mandatory = $true)][string]$Candidate
    )
    for ($attempt = 1; $attempt -le 2; $attempt += 1) {
        $result = Run-CohortAttempt -Name $Name -Reference $Reference `
            -Candidate $Candidate -Attempt $attempt
        if ($result.MadFraction -le 0.05 -and $result.SpreadFraction -le 0.10) {
            return $result
        }
    }
    throw "$Name remained unstable after two complete cohorts"
}

try {
    [void](Invoke-Sample -Variant "io.direct" -Label "preflight.io.direct")
    [void](Invoke-Sample -Variant "io.iocp" -Label "preflight.io.iocp")
    $Results = @(
        Run-QualifiedCohort -Name "compute" -Reference "compute.seq" -Candidate "compute.par"
        Run-QualifiedCohort -Name "io-warm" -Reference "io.direct" -Candidate "io.iocp"
        Run-QualifiedCohort -Name "mixed-iocp" -Reference "mixed.seq" -Candidate "mixed.iocp"
        Run-QualifiedCohort -Name "mixed-full" -Reference "mixed.iocp" -Candidate "mixed.full"
        Run-QualifiedCohort -Name "mixed-total" -Reference "mixed.seq" -Candidate "mixed.full"
    )
} finally {
    $Raw.Dispose()
    [Environment]::SetEnvironmentVariable("WF_WORKERS", $null, "Process")
    [Environment]::SetEnvironmentVariable("WF_REQUIRE_WINDOWS_IOCP", $null, "Process")
}

$SummaryPath = Join-Path $Out "summary.md"
$Summary = [IO.StreamWriter]::new($SummaryPath, $false, [Text.UTF8Encoding]::new($false))
$Summary.WriteLine("## Windows native runtime qualification")
$Summary.WriteLine()
$Summary.WriteLine('```text')
foreach ($line in $HostLines) {
    $Summary.WriteLine($line)
}
$Summary.WriteLine((Get-Content -Raw -LiteralPath (Join-Path $Out "mixed-observer.txt")).TrimEnd())
$Summary.WriteLine('```')
$Summary.WriteLine()
$Summary.WriteLine("| cohort | reference median ms | candidate median ms | paired candidate/reference | MAD | p10..p90 | attempt |")
$Summary.WriteLine("|---|---:|---:|---:|---:|---:|---:|")
foreach ($result in $Results) {
    $Summary.WriteLine((
        "| {0} | {1:F3} | {2:F3} | {3:F4} | {4:P2} | {5:F4}..{6:F4} | {7} |" -f
        $result.Name, $result.ReferenceMedian, $result.CandidateMedian,
        $result.Ratio, $result.MadFraction, $result.P10, $result.P90,
        $result.Attempt
    ))
}
$Summary.Dispose()

if ($Enforce) {
    $ByName = @{}
    foreach ($result in $Results) {
        $ByName[$result.Name] = $result
    }
    if ($ByName["compute"].Ratio -gt 0.90) {
        throw "native compute pool failed its 0.90 paired-ratio qualification"
    }
    if ($ByName["io-warm"].Ratio -gt 1.10) {
        throw "warm IOCP path exceeded the 1.10 framework-overhead ceiling"
    }
    if ($ByName["mixed-full"].Ratio -gt 0.95) {
        throw "unified compute/IOCP path failed its 0.95 paired-ratio qualification"
    }
    if ($ByName["mixed-total"].Ratio -gt 0.95) {
        throw "unified compute/IOCP path failed its 0.95 total mixed qualification"
    }
}

Get-Content -LiteralPath $HostPath
Get-Content -LiteralPath $SummaryPath
