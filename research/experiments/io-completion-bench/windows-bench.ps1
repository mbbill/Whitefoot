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

function Compile-Object {
    param(
        [Parameter(Mandatory = $true)][string]$Source,
        [Parameter(Mandatory = $true)][string]$Output,
        [string[]]$Definitions = @()
    )
    $arguments = @(
        "-std=c11", "-O2", "-g", "-Wall", "-Wextra", "-Werror",
        "-Wpedantic", "-municode", "-I", $Backend, "-I", $Completion
    )
    foreach ($definition in $Definitions) {
        $arguments += "-D$definition"
    }
    $arguments += @("-c", $Source, "-o", $Output)
    Invoke-Tool -File $Clang -Arguments $arguments -Description "compile $Source"
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
) -Description "compile the direct positioned-read reference"
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

$Ir = [IO.File]::ReadAllText($MixedIr)
$SubmitAt = $Ir.IndexOf("call i32 @wf__completion_file_pread_submit", [StringComparison]::Ordinal)
$ComputeAt = if ($SubmitAt -ge 0) {
    $Ir.IndexOf("call i64 @wf_compute_pair", $SubmitAt, [StringComparison]::Ordinal)
} else { -1 }
$DirectAt = if ($ComputeAt -ge 0) {
    $Ir.IndexOf("@wf.sys.read_at.v1", $ComputeAt, [StringComparison]::Ordinal)
} else { -1 }
$CompletionJoinAt = if ($DirectAt -ge 0) {
    $Ir.IndexOf("call void @wf__completion_file_join", $DirectAt, [StringComparison]::Ordinal)
} else { -1 }
$PairAt = $Ir.IndexOf("define internal i64 @wf_compute_pair", [StringComparison]::Ordinal)
$PublishAt = if ($PairAt -ge 0) {
    $Ir.IndexOf("call void @wf__par_publish", $PairAt, [StringComparison]::Ordinal)
} else { -1 }
$ParJoinAt = if ($PublishAt -ge 0) {
    $Ir.IndexOf("call void @wf__par_join", $PublishAt, [StringComparison]::Ordinal)
} else { -1 }
if ($SubmitAt -lt 0 -or $ComputeAt -le $SubmitAt -or $DirectAt -le $ComputeAt `
    -or $CompletionJoinAt -le $DirectAt -or $PublishAt -le $PairAt `
    -or $ParJoinAt -le $PublishAt) {
    throw "mixed IR does not preserve submit -> compute publish/join -> source-last read -> completion join"
}

$ObjectSources = @(
    [pscustomobject]@{ Name = "windows-runtime"; Source = (Join-Path $Backend "windows_runtime.c") }
    [pscustomobject]@{ Name = "wf-floor-windows"; Source = (Join-Path $Backend "wf_floor_windows.c") }
    [pscustomobject]@{ Name = "windows-completion"; Source = (Join-Path $Completion "windows_completion.c") }
    [pscustomobject]@{ Name = "windows-iocp"; Source = (Join-Path $Completion "windows_iocp.c") }
    [pscustomobject]@{ Name = "windows-blocking"; Source = (Join-Path $Completion "windows_blocking.c") }
    [pscustomobject]@{ Name = "writer-scheduler"; Source = (Join-Path $Completion "writer_scheduler_windows.c") }
)
$ObservedObjects = @()
foreach ($unit in $ObjectSources) {
    $object = Join-Path $Objects ($unit.Name + ".o")
    Compile-Object -Source $unit.Source -Output $object
    $ObservedObjects += $object
}
$ParObject = Join-Path $Objects "par-runtime-observed.o"
Compile-Object -Source (Join-Path $Backend "par_runtime_windows.c") `
    -Output $ParObject -Definitions @(
        "WF_PAR_WITH_WRITER_SCHEDULER=1",
        "WF_PAR_IDENTITY_PROBE=1",
        "wf__par_publish=wf__mixed_observed_par_publish"
    )
$BridgeObject = Join-Path $Objects "windows-bridge-observed.o"
Compile-Object -Source (Join-Path $Completion "windows_bridge.c") `
    -Output $BridgeObject -Definitions @(
        "wf__completion_file_join=wf__mixed_observed_file_join",
        "wf__completion_file_open_join=wf__mixed_observed_file_open_join"
    )
$ObserverObject = Join-Path $Objects "mixed-observer.o"
Compile-Object -Source (Join-Path $Backend "par_runtime_windows_probe.c") `
    -Output $ObserverObject -Definitions @(
        "WF_PAR_IDENTITY_PROBE=1", "WF_PAR_MIXED_PROBE=1"
    )
$ObservedObjects += @($ParObject, $BridgeObject, $ObserverObject)
$MixedObserved = Join-Path $Bin "mixed-observed.exe"
$LinkArguments = @(
    "-std=c11", "-O2", "-g", "-municode", "-x", "ir", $MixedIr,
    "-x", "none"
) + $ObservedObjects + @("-Wno-override-module", "-o", $MixedObserved)
Invoke-Tool -File $Clang -Arguments $LinkArguments `
    -Description "link the native mixed identity observer"

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
    throw "mixed identity observer exceeded the 120000 ms timeout"
}
$ObservedStdout = $ObservedStdoutTask.GetAwaiter().GetResult()
$ObservedStderr = $ObservedStderrTask.GetAwaiter().GetResult()
$ObservedExitCode = $ObservedProcess.ExitCode
$ObservedProcess.Dispose()
$ExpectedMixedText = [Text.Encoding]::ASCII.GetString([IO.File]::ReadAllBytes($MixedExpected))
$ObservedLine = $ObservedStderr.TrimEnd([char[]]"`r`n")
if ($ObservedExitCode -ne 0 -or $ObservedStdout -cne $ExpectedMixedText `
    -or $ObservedLine -notmatch '^windows-native-mixed-probe status=pass started=[1-9][0-9]* executed=[1-9][0-9]* grants=[1-9][0-9]* overlap_publishes=[1-9][0-9]* submissions=[1-9][0-9]* publications=[1-9][0-9]* consumes=[1-9][0-9]* fallback=0$') {
    throw "mixed identity observer failed: exit=$ObservedExitCode stdout=[$ObservedStdout] stderr=[$ObservedStderr]"
}
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
        Exe = $ComputeSeq; Args = @(); Expected = $ComputeExpected
        Workers = $false; RequireIocp = $false
    }
    "compute.par" = @{
        Exe = $ComputePar; Args = @(); Expected = $ComputeExpected
        Workers = $true; RequireIocp = $false
    }
    "io.direct" = @{
        Exe = $IoDirect; Args = @(); Expected = $IoExpected
        Workers = $false; RequireIocp = $false
    }
    "io.iocp" = @{
        Exe = $IoIocp; Args = @(); Expected = $IoExpected
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
