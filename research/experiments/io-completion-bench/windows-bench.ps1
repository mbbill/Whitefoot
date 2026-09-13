param(
    [Parameter(Mandatory = $true)]
    [string]$Root,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [int]$Rounds = 15,

    [int]$Warmup = 2,

    [switch]$CompareWorkers
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
$LogicalProcessors = [Math]::Min(64, [Environment]::ProcessorCount)
if ($LogicalProcessors -lt 2) {
    throw "the Windows scheduler observation requires at least two logical processors"
}
# Keep one logical processor's worth of capacity available to the hosted VM.
# The affinity mask is unchanged; this does not reserve a physical core.
$Workers = [Math]::Max(2, $LogicalProcessors - 1)
if ($CompareWorkers -and $Workers -eq $LogicalProcessors) {
    throw "the worker comparison requires at least three logical processors"
}
if ($LogicalProcessors -eq 64) {
    $AffinityMask = [UInt64]::MaxValue
} else {
    $AffinityMask = ([UInt64]1 -shl $LogicalProcessors) - 1
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
        "-Wpedantic", "-I", $Backend, "-I", $Completion,
        "-c", $Source, "-o", $Output
    )
    Invoke-Tool -File $Clang -Arguments $arguments -Description "compile $Source"
}

# Inspect a definition, independent of whether a call or declaration occurs
# earlier in the module. Emitted definition braces end at the start of a line.
function Get-EmittedFunction {
    param(
        [Parameter(Mandatory = $true)][string]$Module,
        [Parameter(Mandatory = $true)][string]$Symbol
    )
    $definition = [regex]::Match(
        $Module, '(?m)^define[^\r\n]*@' + [regex]::Escape($Symbol) + '\('
    )
    if (-not $definition.Success) {
        throw "the emitted mixed module defines no $Symbol"
    }
    $at = $definition.Index
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
$NativeTarget = Join-Path $Target "native-control"
$NativeControl = Join-Path $NativeTarget "release/paired-layout.exe"

Invoke-Tool -File $Cargo -Arguments @(
    "build", "--manifest-path", (Join-Path $Root "compiler/Cargo.toml"),
    "--profile", "gate", "--locked", "--offline", "--bin", "whitefootc"
) -Description "build whitefootc"

Invoke-Tool -File $Cargo -Arguments @(
    "rustc", "--manifest-path", (Join-Path $Root "research/investigations/proof-derived-parallelism/bench/rust/Cargo.toml"),
    "--target-dir", $NativeTarget, "--release", "--locked", "--offline",
    "--bin", "paired-layout", "--", "-C", "no-vectorize-loops", "-C", "no-vectorize-slp"
) -Description "build the scalar native scheduling control"

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
$NativeFullExpected = Join-Path $Out "native-full.expected"
$NativeShortExpected = Join-Path $Out "native-short.expected"
# These are par_layout's two existing fold oracles. Reset-seed batches keep
# the first result unchanged; a 4096-word twin matches its half-width fold.
Write-AsciiFile -Path $NativeFullExpected -Text "420a993efa7437a1`n"
Write-AsciiFile -Path $NativeShortExpected -Text "41fa962893d45299`n"

$ComputeSeq = Join-Path $Bin "compute-seq.exe"
$ComputePar = Join-Path $Bin "compute-par.exe"
$IoNoOverlap = Join-Path $Bin "io-no-overlap.exe"
$IoDefault = Join-Path $Bin "io-default.exe"
$MixedNoOverlap = Join-Path $Bin "mixed-no-overlap.exe"
$MixedDefault = Join-Path $Bin "mixed-default.exe"
$MixedPar = Join-Path $Bin "mixed-par.exe"
$MixedIr = Join-Path $Out "mixed-par.ll"
$MixedSource = Join-Path $Programs "windows_runtime_mixed.wf"

Invoke-Tool -File $Wfc -Arguments @(
    (Join-Path $Root "tests/programs/par_layout.wf"), "-o", $ComputeSeq
) -Description "compile the compute reference"
Invoke-Tool -File $Wfc -Arguments @(
    "--par", (Join-Path $Root "tests/programs/par_layout.wf"), "-o", $ComputePar
) -Description "compile the native compute-pool contender"
Invoke-Tool -File $Wfc -Arguments @(
    "--no-overlap", (Join-Path $Programs "read_heavy_wide8_4k.wf"),
    "-o", $IoNoOverlap
) -Description "compile the no-overlap positioned-read reference"
Invoke-Tool -File $Wfc -Arguments @(
    (Join-Path $Programs "read_heavy_wide8_4k.wf"), "-o", $IoDefault
) -Description "compile the default positioned-read contender"
Invoke-Tool -File $Wfc -Arguments @(
    "--no-overlap", $MixedSource, "-o", $MixedNoOverlap
) -Description "compile the no-overlap mixed reference"
Invoke-Tool -File $Wfc -Arguments @($MixedSource, "-o", $MixedDefault) `
    -Description "compile the default mixed control"
Invoke-Tool -File $Wfc -Arguments @("--par", $MixedSource, "-o", $MixedPar) `
    -Description "compile the ordinary parallel-compute mixed contender"
Invoke-Tool -File $Wfc -Arguments @(
    "--par", "--emit-llvm", $MixedSource, "-o", $MixedIr
) -Description "emit the observed mixed module"

# C2 deletes PAR-3 completion overlap. The source window remains two ordinary
# read calls with compute between them; each read returns before the next
# statement executes. The native body may use IOCP internally, which is not
# a compiler acceptance or overlap classification. Inspect the actual worker
# function, not the build launcher that receives Inputs.
$Ir = [IO.File]::ReadAllText($MixedIr)
$Exercise = Get-EmittedFunction -Module $Ir -Symbol "wf_exercise"
$Pair = Get-EmittedFunction -Module $Ir -Symbol "wf_compute_pair"
$FirstReadAt = $Exercise.IndexOf("call void @wf_read_at(", [StringComparison]::Ordinal)
$ComputeAt = if ($FirstReadAt -ge 0) {
    $Exercise.IndexOf("call i64 @wf_compute_pair(", $FirstReadAt, [StringComparison]::Ordinal)
} else { -1 }
$LastReadAt = if ($ComputeAt -ge 0) {
    $Exercise.IndexOf("call void @wf_read_at(", $ComputeAt, [StringComparison]::Ordinal)
} else { -1 }
if ($FirstReadAt -lt 0 -or $ComputeAt -le $FirstReadAt -or $LastReadAt -le $ComputeAt) {
    throw "mixed IR does not preserve the ordinary read, compute, read call order"
}
# Ordinary computation still uses the existing lane protocol. This check is
# independent of how the linked read implementation completes its operation.
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

# The observed link contains the same ordinary implementations, callable ABI
# wrappers and private runtime dependencies as whitefootc. The only observer
# addition is grant_observer.c, which reports ordinary scheduler hand-outs to
# another worker. No library function is replaced by a benchmark stub.
#
# WF_REQUIRE_WINDOWS_IOCP=1 independently requires the linked read body to
# submit at least one operation to the completion port and reap every native
# submission. This observes an implementation path; it does not grant early
# result or loan release, or assert overlap between source calls.
$ObservedUnits = @(
    "ordinary_values.c",
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
$OrdinaryIr = Join-Path $Objects "ordinary_values_ir.o"
Invoke-Tool -File $Clang -Arguments @(
    "-O2", "-Wno-override-module", "-x", "ir", "-c",
    (Join-Path $Backend "ordinary_values.ll"), "-o", $OrdinaryIr
) -Description "compile the ordinary callable ABI wrappers"
$ObservedObjects += $OrdinaryIr
$MixedObserved = Join-Path $Bin "mixed-observed.exe"
# The emitted launcher defines ordinary C main. Only windows_runner.c, whose
# entry is wmain, uses -municode. Winsock and shell32 are dependencies of the
# ordinary linked library, matching whitefootc's native Windows link.
$LinkArguments = @(
    "-std=c11", "-O2", "-g", "-x", "ir", $MixedIr,
    "-x", "none"
) + $ObservedObjects + @("-Wno-override-module", "-o", $MixedObserved, "-lws2_32", "-lshell32")
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
    "guest processor cores: $($Cpu.NumberOfCores)",
    "guest processor logical processors: $($Cpu.NumberOfLogicalProcessors)",
    "workers: $Workers",
    "worker policy: one fewer than the affinity mask allows, with a minimum of two; no exclusive core reservation",
    "worker comparison: $CompareWorkers (full mask count versus primary worker count)",
    "affinity mask: 0x$AffinityHex",
    "compute batches per sampled child: $($ComputeArguments.Count + 1)",
    "memory bytes: $($Os.TotalVisibleMemorySize * 1024)",
    "power: $Power",
    "clang: $ClangVersion",
    "rust: $RustVersion",
    "native control: existing Rust/Rayon scalar layout twin; W=$Workers; loop and SLP vectorization disabled",
    "native kernels: full-width 8192 words x 800 reps x 3 reset-seed batches; short 4096 x 800 x 1",
    "stability: Whitefoot/reference MAD and spread may exceed native/reference by at most 0.05 and 0.10 respectively",
    "cache state: warm, verified by a full sequential pre-read of all eight files",
    $(if ($CompareWorkers) {
        "protocol: $Warmup warm-up rounds, $Rounds recorded four-child balanced rounds, QPC wall and child process CPU"
    } else {
        "protocol: $Warmup warm-up triplets, $Rounds recorded position-balanced triplets, QPC wall and child process CPU"
    })
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
    "io.no-overlap" = @{
        Exe = $IoNoOverlap; Args = $IoArguments; Expected = $IoExpected
        Workers = $false; RequireIocp = $false
    }
    "io.default" = @{
        Exe = $IoDefault; Args = $IoArguments; Expected = $IoExpected
        Workers = $false; RequireIocp = $true
    }
    "mixed.no-overlap" = @{
        Exe = $MixedNoOverlap; Args = @("f00000.dat", "f00001.dat")
        Expected = $MixedExpected; Workers = $false; RequireIocp = $false
    }
    "mixed.default" = @{
        Exe = $MixedDefault; Args = @("f00000.dat", "f00001.dat")
        Expected = $MixedExpected; Workers = $false; RequireIocp = $true
    }
    "mixed.par" = @{
        Exe = $MixedPar; Args = @("f00000.dat", "f00001.dat")
        Expected = $MixedExpected; Workers = $true; RequireIocp = $true
    }
    "control.compute" = @{
        Exe = $NativeControl; Args = @("rayoncut", "bal", "6", "8192", "800", [string]$Workers, "3")
        Expected = $NativeFullExpected; Workers = $false; RequireIocp = $false
    }
    "control.short" = @{
        Exe = $NativeControl; Args = @("rayoncut", "bal", "6", "4096", "800", [string]$Workers, "1")
        Expected = $NativeShortExpected; Workers = $false; RequireIocp = $false
    }
}

$RawPath = Join-Path $Out "raw.tsv"
$Raw = [IO.StreamWriter]::new($RawPath, $false, [Text.UTF8Encoding]::new($false))
$Raw.WriteLine("cohort`tattempt`tpair`torder`tvariant`twall_ms`tuser_ms`tkernel_ms`tworker_limit")

function Invoke-Sample {
    param(
        [Parameter(Mandatory = $true)][string]$Variant,
        [Parameter(Mandatory = $true)][string]$Label,
        [int]$WorkerLimit = $Workers
    )
    $configuration = $Variants[$Variant]
    [Environment]::SetEnvironmentVariable("WF_WORKERS", $null, "Process")
    [Environment]::SetEnvironmentVariable("WF_REQUIRE_WINDOWS_IOCP", $null, "Process")
    if ($configuration.Workers) {
        [Environment]::SetEnvironmentVariable("WF_WORKERS", [string]$WorkerLimit, "Process")
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
    $sample = [pscustomobject]@{
        Variant = $Variant
        Wall = [double]::Parse($fields[1], [Globalization.CultureInfo]::InvariantCulture)
        User = [double]::Parse($fields[2], [Globalization.CultureInfo]::InvariantCulture)
        Kernel = [double]::Parse($fields[3], [Globalization.CultureInfo]::InvariantCulture)
    }
    foreach ($value in @($sample.Wall, $sample.User, $sample.Kernel)) {
        if (-not [double]::IsFinite($value) -or $value -lt 0) {
            throw "invalid native timing for $Variant"
        }
    }
    if ($sample.Wall -eq 0) { throw "invalid native timing for $Variant" }
    return $sample
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

function Get-SampleOrder {
    param([object[]]$Settings, [int]$Round)
    if ($Settings.Count -eq 4) {
        # A four-treatment Williams design balances positions and immediate
        # precedence every four rounds. Fifteen rounds differ by one row.
        $base = @(0, 1, 3, 2)
        $indices = @($base | ForEach-Object { ($_ + $Round) % 4 })
        return $Settings[$indices]
    }
    if ($Settings.Count -eq 3) {
        # Every three rounds put each policy at each position once. Every
        # six also balance pairwise precedence and reference distance.
        $indices = switch ($Round % 6) {
            0 { 0, 1, 2 }
            1 { 2, 0, 1 }
            2 { 1, 2, 0 }
            3 { 2, 1, 0 }
            4 { 0, 2, 1 }
            5 { 1, 0, 2 }
        }
        return $Settings[$indices]
    }
    if (($Round % 2) -eq 0) { return $Settings }
    return $Settings[1, 0]
}

function Measure-Distribution {
    param([double[]]$Values)
    foreach ($value in $Values) {
        if (-not [double]::IsFinite($value) -or $value -le 0) {
            throw "invalid wall or ratio distribution"
        }
    }
    $median = Median -Values $Values
    $deviations = [double[]]@($Values | ForEach-Object { [Math]::Abs($_ - $median) })
    $p10 = Percentile -Values $Values -Fraction 0.10
    $p90 = Percentile -Values $Values -Fraction 0.90
    return [pscustomobject]@{
        Median = $median
        MadFraction = (Median -Values $deviations) / $median
        SpreadFraction = ($p90 - $p10) / $median
        P10 = $p10; P90 = $p90
    }
}

function Test-CohortStability {
    param($Result)
    foreach ($value in @(
        $Result.MadFraction, $Result.SpreadFraction,
        $Result.ControlMadFraction, $Result.ControlSpreadFraction
    )) {
        if (-not [double]::IsFinite($value)) { return $false }
    }
    return (($Result.MadFraction - $Result.ControlMadFraction) -le 0.05 -and
        ($Result.SpreadFraction - $Result.ControlSpreadFraction) -le 0.10)
}

function Run-CohortAttempt {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Reference,
        [Parameter(Mandatory = $true)][string]$Candidate,
        [Parameter(Mandatory = $true)][string]$Control,
        [Parameter(Mandatory = $true)][int]$Attempt,
        [int[]]$WorkerLimits = @($Workers)
    )
    # The optional same-host comparison uses the existing two-cohort sample
    # allowance: fifteen candidates at each worker count, sharing each serial
    # reference and one native control. No Whitefoot pair, round or retry is
    # added. Balance positions and precedence across all measured children.
    if ($Variants[$Reference].Workers) {
        throw "a shared comparison reference must not depend on the worker limit"
    }
    $settings = @(@{ Key = "reference"; Variant = $Reference; Workers = $Workers })
    $ratios = @{}
    $candidateWalls = @{}
    $controlWalls = [Collections.Generic.List[double]]::new()
    $controlRatios = [Collections.Generic.List[double]]::new()
    foreach ($limit in $WorkerLimits) {
        $settings += @{ Key = "candidate.$limit"; Variant = $Candidate; Workers = $limit }
        $ratios[$limit] = [Collections.Generic.List[double]]::new()
        $candidateWalls[$limit] = [Collections.Generic.List[double]]::new()
    }
    $settings += @{ Key = "control"; Variant = $Control; Workers = $Workers }
    for ($warm = 0; $warm -lt $Warmup; $warm += 1) {
        $warmOrder = @(Get-SampleOrder -Settings $settings -Round $warm)
        foreach ($setting in $warmOrder) {
            [void](Invoke-Sample -Variant $setting.Variant -WorkerLimit $setting.Workers `
                -Label "warm.$Name.$warm.$($setting.Key)")
        }
    }
    $referenceWalls = [Collections.Generic.List[double]]::new()
    for ($pair = 0; $pair -lt $Rounds; $pair += 1) {
        $order = @(Get-SampleOrder -Settings $settings -Round $pair)
        $pairSamples = @{}
        for ($position = 0; $position -lt $order.Count; $position += 1) {
            $setting = $order[$position]
            $sample = Invoke-Sample -Variant $setting.Variant -WorkerLimit $setting.Workers `
                -Label "$Name.$Attempt.$pair.$position.$($setting.Key)"
            $pairSamples[$setting.Key] = $sample
            $Raw.WriteLine((
                "{0}`t{1}`t{2}`t{3}`t{4}`t{5:F3}`t{6:F3}`t{7:F3}`t{8}" -f
                $Name, $Attempt, $pair, $position, $setting.Variant,
                $sample.Wall, $sample.User, $sample.Kernel, $setting.Workers
            ))
            $Raw.Flush()
        }
        $referenceWall = [double]$pairSamples["reference"].Wall
        $referenceWalls.Add($referenceWall)
        $controlWall = [double]$pairSamples["control"].Wall
        $controlWalls.Add($controlWall)
        $controlRatios.Add($controlWall / $referenceWall)
        foreach ($limit in $WorkerLimits) {
            $candidateWall = [double]$pairSamples["candidate.$limit"].Wall
            $candidateWalls[$limit].Add($candidateWall)
            $ratios[$limit].Add($candidateWall / $referenceWall)
        }
    }
    $controlStats = Measure-Distribution -Values ([double[]]$controlRatios.ToArray())
    $controlWallStats = Measure-Distribution -Values ([double[]]$controlWalls.ToArray())
    foreach ($limit in $WorkerLimits) {
        $ratioValues = [double[]]$ratios[$limit].ToArray()
        $ratioStats = Measure-Distribution -Values $ratioValues
        $referenceMedian = Median -Values ([double[]]$referenceWalls.ToArray())
        $candidateWallStats = Measure-Distribution -Values ([double[]]$candidateWalls[$limit].ToArray())
        [pscustomobject]@{
            Name = $Name
            Reference = $Reference
            Candidate = $Candidate
            Attempt = $Attempt
            WorkerLimit = $limit
            ReferenceMedian = $referenceMedian
            CandidateMedian = $candidateWallStats.Median
            CandidateWallMad = $candidateWallStats.MadFraction
            CandidateWallSpread = $candidateWallStats.SpreadFraction
            Ratio = $ratioStats.Median
            MadFraction = $ratioStats.MadFraction
            SpreadFraction = $ratioStats.SpreadFraction
            P10 = $ratioStats.P10; P90 = $ratioStats.P90
            Control = $Control; ControlMedian = $controlWallStats.Median
            ControlWallMad = $controlWallStats.MadFraction
            ControlWallSpread = $controlWallStats.SpreadFraction
            ControlMadFraction = $controlStats.MadFraction
            ControlSpreadFraction = $controlStats.SpreadFraction
        }
    }
}

# Keep one fixed cohort even when its timing spread is wide. Stability is
# descriptive data, never a reason to select another sample or fail CI.
function Run-Cohort {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Reference,
        [Parameter(Mandatory = $true)][string]$Candidate,
        [Parameter(Mandatory = $true)][string]$Control
    )
    return Run-CohortAttempt -Name $Name -Reference $Reference `
        -Candidate $Candidate -Control $Control -Attempt 1
}

try {
    [void](Invoke-Sample -Variant "io.no-overlap" -Label "preflight.io.no-overlap")
    [void](Invoke-Sample -Variant "io.default" -Label "preflight.io.default")
    $Cohorts = @(
        @{ Name = "compute"; Reference = "compute.seq"; Candidate = "compute.par"; Control = "control.compute" }
        @{ Name = "io-warm"; Reference = "io.no-overlap"; Candidate = "io.default"; Control = "control.short" }
        @{ Name = "mixed-default"; Reference = "mixed.no-overlap"; Candidate = "mixed.default"; Control = "control.short" }
        @{ Name = "mixed-par"; Reference = "mixed.default"; Candidate = "mixed.par"; Control = "control.short" }
        @{ Name = "mixed-total"; Reference = "mixed.no-overlap"; Candidate = "mixed.par"; Control = "control.short" }
    )
    $Results = @(foreach ($cohort in $Cohorts) {
        if ($CompareWorkers) {
            Run-CohortAttempt @cohort -Attempt 1 -WorkerLimits @($LogicalProcessors, $Workers)
        } else {
            Run-Cohort @cohort
        }
    })
} finally {
    $Raw.Dispose()
    [Environment]::SetEnvironmentVariable("WF_WORKERS", $null, "Process")
    [Environment]::SetEnvironmentVariable("WF_REQUIRE_WINDOWS_IOCP", $null, "Process")
}

$SummaryPath = Join-Path $Out "summary.md"
$Summary = [IO.StreamWriter]::new($SummaryPath, $false, [Text.UTF8Encoding]::new($false))
$Summary.WriteLine("## Windows native runtime measurements")
$Summary.WriteLine()
$Summary.WriteLine('```text')
foreach ($line in $HostLines) {
    $Summary.WriteLine($line)
}
$Summary.WriteLine((Get-Content -Raw -LiteralPath (Join-Path $Out "mixed-observer.txt")).TrimEnd())
$Summary.WriteLine('```')
$Summary.WriteLine()
if ($CompareWorkers) {
    $Summary.WriteLine("The full-worker rows are diagnostic; W=$Workers is the primary measured candidate. Each round shares its serial reference and W=$Workers native control. IO-only candidates repeat the unchanged configuration. No extra Whitefoot pairs, rounds or retries are added.")
    $Summary.WriteLine()
}
$Summary.WriteLine("Correctness, ordinary worker publication and the native IOCP path are checked. Ratios and relative stability are measurements; no speed or spread threshold selects success. Each ordinary read returns before the following statement.")
$Summary.WriteLine()
$Summary.WriteLine("| cohort | workers | reference median ms | candidate median ms | paired candidate/reference | MAD | p90-p10 / median | attempt |")
$Summary.WriteLine("|---|---:|---:|---:|---:|---:|---:|---:|")
foreach ($result in $Results) {
    $Summary.WriteLine((
        "| {0} | {1} | {2:F3} | {3:F3} | {4:F4} | {5:P2} | {6:P2} | {7} |" -f
        $result.Name, $result.WorkerLimit, $result.ReferenceMedian, $result.CandidateMedian,
        $result.Ratio, $result.MadFraction, $result.SpreadFraction,
        $result.Attempt
    ))
}
$Summary.WriteLine()
$Summary.WriteLine("Raw candidate wall-time distributions, before division by the reference:")
$Summary.WriteLine()
$Summary.WriteLine("| cohort | workers | Whitefoot MAD | Whitefoot spread | native MAD | native spread |")
$Summary.WriteLine("|---|---:|---:|---:|---:|---:|")
foreach ($result in $Results) {
    $Summary.WriteLine((
        "| {0} | {1} | {2:P2} | {3:P2} | {4:P2} | {5:P2} |" -f
        $result.Name, $result.WorkerLimit, $result.CandidateWallMad, $result.CandidateWallSpread,
        $result.ControlWallMad, $result.ControlWallSpread
    ))
}
$Summary.WriteLine()
$Summary.WriteLine("Stability uses percentage-point excess over the native/reference distribution, reported against reference margins MAD 5% and spread 10%. The table above remains unadjusted; the native control is a CPU-availability witness, not an I/O throughput baseline.")
$Summary.WriteLine()
$Summary.WriteLine("| cohort | native median ms | native/reference MAD | native/reference spread | W=$Workers excess MAD | W=$Workers excess spread | stability |")
$Summary.WriteLine("|---|---:|---:|---:|---:|---:|---|")
foreach ($result in $Results | Where-Object { $_.WorkerLimit -eq $Workers }) {
    $Summary.WriteLine((
        "| {0} | {1:F3} | {2:P2} | {3:P2} | {4:P2} | {5:P2} | {6} |" -f
        $result.Name, $result.ControlMedian, $result.ControlMadFraction, $result.ControlSpreadFraction,
        [Math]::Max(0.0, $result.MadFraction - $result.ControlMadFraction),
        [Math]::Max(0.0, $result.SpreadFraction - $result.ControlSpreadFraction),
        $(if (Test-CohortStability -Result $result) { "within reference bands" } else { "wide spread" })
    ))
}
$Summary.Dispose()

Get-Content -LiteralPath $HostPath
Get-Content -LiteralPath $SummaryPath
