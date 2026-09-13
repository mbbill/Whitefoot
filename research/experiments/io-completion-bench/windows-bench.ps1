param(
    [Parameter(Mandatory = $true)]
    [string]$Root,

    [Parameter(Mandatory = $true)]
    [string]$Out,

    [int]$Rounds = 15,

    [int]$Warmup = 2
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
    throw "the Windows scheduler observation requires at least two logical processors"
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
    $stability = if (($mad / $ratioMedian) -le 0.05 -and (($p90 - $p10) / $ratioMedian) -le 0.10) {
        "within reference bands"
    } else {
        "wide spread"
    }
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
        Stability = $stability
        P10 = $p10
        P90 = $p90
    }
}

# Keep one fixed cohort even when its timing spread is wide. Stability is
# descriptive data, never a reason to select another sample or fail CI.
function Run-Cohort {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Reference,
        [Parameter(Mandatory = $true)][string]$Candidate
    )
    $result = Run-CohortAttempt -Name $Name -Reference $Reference `
        -Candidate $Candidate -Attempt 1
    return $result
}

try {
    [void](Invoke-Sample -Variant "io.no-overlap" -Label "preflight.io.no-overlap")
    [void](Invoke-Sample -Variant "io.default" -Label "preflight.io.default")
    $Results = @(
        Run-Cohort -Name "compute" -Reference "compute.seq" -Candidate "compute.par"
        Run-Cohort -Name "io-warm" -Reference "io.no-overlap" -Candidate "io.default"
        Run-Cohort -Name "mixed-default" -Reference "mixed.no-overlap" -Candidate "mixed.default"
        Run-Cohort -Name "mixed-par" -Reference "mixed.default" -Candidate "mixed.par"
        Run-Cohort -Name "mixed-total" -Reference "mixed.no-overlap" -Candidate "mixed.par"
    )
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
$Summary.WriteLine("Correctness, ordinary scheduler hand-outs and the native IOCP path were checked before timing. Ratios and spread are observations, not pass/fail thresholds. C2 runs both reads as ordinary calls that return before the following statement.")
$Summary.WriteLine()
$Summary.WriteLine("| cohort | reference median ms | candidate median ms | paired candidate/reference | MAD | p10..p90 | width | stability |")
$Summary.WriteLine("|---|---:|---:|---:|---:|---:|---:|---|")
foreach ($result in $Results) {
    $Summary.WriteLine((
        "| {0} | {1:F3} | {2:F3} | {3:F4} | {4:P2} | {5:F4}..{6:F4} | {7:P2} | {8} |" -f
        $result.Name, $result.ReferenceMedian, $result.CandidateMedian,
        $result.Ratio, $result.MadFraction, $result.P10, $result.P90,
        $result.SpreadFraction, $result.Stability
    ))
}
$Summary.Dispose()

Get-Content -LiteralPath $HostPath
Get-Content -LiteralPath $SummaryPath
