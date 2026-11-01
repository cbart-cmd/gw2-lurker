# Runs the local quality gate and prints one line per step (AGENTS.md §3).
#
# Full output goes to target\validate\<step>.log; only failures print an excerpt. This keeps
# agent sessions from pasting hundreds of lines of cargo output into the context.
#
# Uso:
#   pwsh -File scripts/validate.ps1                      # fmt, clippy, loc, traza, tests
#   pwsh -File scripts/validate.ps1 -Release             # además build release y tamaño
#   pwsh -File scripts/validate.ps1 -Only fmt,clippy     # solo esos pasos
#   pwsh -File scripts/validate.ps1 -Only test -Package gw2lurker-core -Filter rf22
#
# Sale con código 1 si falla algún paso.

[CmdletBinding()]
param(
    [ValidateSet('fmt', 'clippy', 'loc', 'trace', 'test', 'release')]
    [string[]]$Only = @(),
    [switch]$Release,
    [string]$Package = '',
    [string]$Filter = ''
)

$ErrorActionPreference = 'Continue'
$RepoRoot = Split-Path $PSScriptRoot -Parent
Set-Location $RepoRoot
$LogDir = Join-Path $RepoRoot 'target\validate'
New-Item -ItemType Directory -Force -Path $LogDir | Out-Null

$Bash = (Get-Command bash -ErrorAction SilentlyContinue).Source
if (-not $Bash) { $Bash = 'C:\Program Files\Git\bin\bash.exe' }

$testArgs = @('test')
if ($Package) { $testArgs += @('-p', $Package) } else { $testArgs += '--workspace' }
if ($Filter) { $testArgs += $Filter }

$steps = [ordered]@{
    fmt     = { cargo fmt --all --check }
    clippy  = { cargo clippy --workspace --all-targets -- -D warnings }
    loc     = { & $Bash ./scripts/check_loc.sh }
    trace   = { & $Bash ./scripts/check_traceability.sh }
    test    = { cargo @testArgs }
    release = {
        cargo build --release
        if ($LASTEXITCODE -eq 0) { & $Bash ./scripts/check_size.sh target/release/gw2_lurker.dll }
    }
}

$selected = if ($Only.Count) { $Only } else {
    $default = @('fmt', 'clippy', 'loc', 'trace', 'test')
    if ($Release) { $default += 'release' }
    $default
}

$failed = $false
foreach ($name in $selected) {
    $log = Join-Path $LogDir "$name.log"
    $watch = [Diagnostics.Stopwatch]::StartNew()
    & $steps[$name] *> $log
    $code = $LASTEXITCODE
    $secs = [math]::Round($watch.Elapsed.TotalSeconds, 1)

    $detail = ''
    switch ($name) {
        'test' {
            $passed = 0; $fails = 0; $ignored = 0
            Select-String -Path $log -Pattern 'test result: \w+\. (\d+) passed; (\d+) failed; (\d+) ignored' |
                ForEach-Object {
                    $passed += [int]$_.Matches[0].Groups[1].Value
                    $fails += [int]$_.Matches[0].Groups[2].Value
                    $ignored += [int]$_.Matches[0].Groups[3].Value
                }
            $detail = "$passed pasados, $fails fallidos, $ignored ignorados"
        }
        'loc' { $detail = "$((Select-String -Path $log -Pattern '^AVISO').Count) avisos >500" }
        'trace' { $detail = (Select-String -Path $log -Pattern '^Trazabilidad:' | Select-Object -First 1).Line }
        'release' { $detail = (Select-String -Path $log -Pattern '^Tama' | Select-Object -First 1).Line }
    }

    if ($code -eq 0) {
        Write-Host ("OK    {0,-8} {1,6}s  {2}" -f $name, $secs, $detail)
    } else {
        $failed = $true
        Write-Host ("FALLO {0,-8} {1,6}s  {2}  (log: {3})" -f $name, $secs, $detail, $log) -ForegroundColor Red
        $hits = Select-String -Path $log -Pattern '^(error|warning)|FAILED|panicked|^ERROR|^Diff in|^---- ' |
            Select-Object -First 30
        if ($hits) { $hits | ForEach-Object { "      $($_.Line)" } }
        else { Get-Content $log -Tail 30 | ForEach-Object { "      $_" } }
    }
}

if ($failed) { exit 1 }
