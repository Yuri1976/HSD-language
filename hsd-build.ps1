<#
.SYNOPSIS
    Build/run helper for HSD programs.

.DESCRIPTION
    Searches the repo for a .hsd file by name, then either runs it with the
    interpreter or compiles it to a native .exe via the C backend + MSVC
    (cl), without leaving stray .obj files in the repo.

.PARAMETER File
    Name of the .hsd file, without extension (e.g. "Euler05").

.PARAMETER Modo
    "run" for interpreter, "build" for compiled, "both" for both.
    If omitted, you'll be asked interactively.

.PARAMETER Benchmark
    Times execution with Measure-Command and prints the elapsed time
    after the run. Works with run, build, or both.

.PARAMETER Runs
    Number of times to repeat each measured execution (default 1).
    Only relevant together with -Benchmark. Prints each run plus the
    average.

.EXAMPLE
    .\hsd-build.ps1 -File Euler05
    .\hsd-build.ps1 -File Euler05 -Modo build
    .\hsd-build.ps1 -File Euler05 -Modo both -Benchmark
    .\hsd-build.ps1 -File Euler05 -Modo build -Benchmark -Runs 3
#>

param(
    [Parameter(Mandatory=$true)]
    [string]$File,

    [ValidateSet("run", "build", "both", "")]
    [string]$Modo = "",

    [switch]$Force,    # ignore cached .exe and recompile anyway

    [switch]$Benchmark, # time execution with Measure-Command

    [int]$Runs = 1      # how many timed runs per mode, when -Benchmark is set
)

$ErrorActionPreference = "Stop"

# strip a trailing .hsd if the user passed it by mistake (e.g. -File Euler05.hsd)
if ($File -match '\.hsd$') {
    $File = $File -replace '\.hsd$', ''
}

# ---------- 1. find the .hsd file anywhere in the repo ----------
$candidati = Get-ChildItem -Path . -Recurse -Filter "$File.hsd" -ErrorAction SilentlyContinue
if (-not $candidati) {
    Write-Host "File '$File.hsd' not found anywhere under the current folder." -ForegroundColor Red
    exit 1
}
if ($candidati.Count -gt 1) {
    Write-Host "Multiple files named '$File.hsd' found:" -ForegroundColor Yellow
    $candidati | ForEach-Object { Write-Host "  $($_.FullName)" -ForegroundColor Yellow }
    Write-Host "Using the first one. Rename or remove duplicates if this is wrong." -ForegroundColor Yellow
}
$trovato = $candidati | Select-Object -First 1
$hsdPath = $trovato.FullName
$hsdDir  = $trovato.DirectoryName
$cPath   = Join-Path $hsdDir "$File.c"
$exePath = Join-Path $hsdDir "$File.exe"

Write-Host "Found: $hsdPath" -ForegroundColor DarkGray

# ---------- helper: run a scriptblock N times, report each time + average ----------
# A warm-up run is always done first and shown, but excluded from the average,
# since the first execution is consistently slower due to cold filesystem
# cache / antivirus scan / page mapping - not representative of real speed.
function Invoke-Timed {
    param(
        [scriptblock]$Block,
        [string]$Label,
        [int]$Count
    )

    $warmup = Measure-Command { & $Block }
    Write-Host ("  warm-up (excluded): {0:N3} s" -f $warmup.TotalSeconds) -ForegroundColor DarkGray

    $times = @()
    for ($i = 1; $i -le $Count; $i++) {
        $elapsed = Measure-Command { & $Block }
        $times += $elapsed.TotalSeconds
        Write-Host ("  run {0}/{1}: {2:N3} s" -f $i, $Count, $elapsed.TotalSeconds) -ForegroundColor DarkGray
    }
    $avg = ($times | Measure-Object -Average).Average
    Write-Host ("== {0}: average {1:N3} s over {2} run(s), warm-up excluded ==" -f $Label, $avg, $Count) -ForegroundColor Green
}

# ---------- 2. ask interpreter vs compiled, if not given ----------
if ($Modo -eq "") {
    $scelta = Read-Host "Run as (1) interpreter or (2) compiled? [1/2]"
    $Modo = if ($scelta -eq "2") { "build" } else { "run" }
}

# ---------- 3. interpreter path ----------
if ($Modo -eq "run" -or $Modo -eq "both") {
    Write-Host "`n== Running with interpreter ==" -ForegroundColor Cyan
    if ($Benchmark) {
        Invoke-Timed -Block { cargo run "$hsdPath" | Out-Null } -Label "Interpreter" -Count $Runs
    } else {
        cargo run "$hsdPath"
    }
}

# ---------- 4. compiled path ----------
if ($Modo -eq "build" -or $Modo -eq "both") {

    # make sure cl.exe is available in this shell session
    if (-not (Get-Command cl -ErrorAction SilentlyContinue)) {
        Write-Host "`n== Loading MSVC developer environment ==" -ForegroundColor Cyan
        $vsDevShell = "C:\Program Files\Microsoft Visual Studio\18\Community\Common7\Tools\Microsoft.VisualStudio.DevShell.dll"
        if (-not (Test-Path $vsDevShell)) {
            Write-Host "DevShell module not found at expected path. Adjust `$vsDevShell in this script." -ForegroundColor Red
            exit 1
        }
        Import-Module $vsDevShell
        Enter-VsDevShell -VsInstallPath "C:\Program Files\Microsoft Visual Studio\18\Community" -SkipAutomaticLocation
    }

    Write-Host "`n== Generating C from HSD ==" -ForegroundColor Cyan

    $needsRebuild = $true
    if ((Test-Path $exePath) -and -not $Force) {
        $exeTime = (Get-Item $exePath).LastWriteTime
        $hsdTime = (Get-Item $hsdPath).LastWriteTime
        if ($exeTime -gt $hsdTime) {
            $needsRebuild = $false
            Write-Host "Existing $exePath is newer than the .hsd source - skipping rebuild." -ForegroundColor DarkGray
            Write-Host "(use -Force to recompile anyway)" -ForegroundColor DarkGray
        }
    }

    if ($needsRebuild) {
        cargo run build "$hsdPath"

        if (-not (Test-Path $cPath)) {
            Write-Host "Expected generated file not found: $cPath" -ForegroundColor Red
            exit 1
        }

        # keep intermediate .obj files out of the repo
        $objDir = Join-Path $env:TEMP "hsd_obj"
        New-Item -ItemType Directory -Force -Path $objDir | Out-Null

        # find runtime.c relative to repo root (assumes script sits at repo root)
        $runtimeC = Join-Path $PSScriptRoot "runtime\runtime.c"
        $runtimeI = Join-Path $PSScriptRoot "runtime"

        Write-Host "== Compiling with cl ==" -ForegroundColor Cyan
        cl "$cPath" "$runtimeC" /Fe:"$exePath" /Fo:"$objDir\" /I "$runtimeI" /nologo
    }

    if (Test-Path $exePath) {
        Write-Host "`n== Running compiled binary ==" -ForegroundColor Cyan
        if ($Benchmark) {
            Invoke-Timed -Block { & $exePath | Out-Null } -Label "Compiled binary" -Count $Runs
            # also show the actual program output once, uncounted, so you still see it
            Write-Host "`n-- output --" -ForegroundColor DarkGray
            & $exePath
        } else {
            & $exePath
        }
    } else {
        Write-Host "Compilation failed: $exePath was not produced." -ForegroundColor Red
        exit 1
    }
}
