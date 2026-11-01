# Cheap check of the shared bug inbox (AGENTS.md §7).
#
# Prints one line when bugs.md has not changed since the last call, and only the headings of
# the entries when it has. The agent opens the file only when this says it changed, instead of
# re-reading it whole after every step.
#
# Uso: pwsh -File scripts/check_bugs.ps1   (primera llamada de la sesión: lista todo)

$RepoRoot = Split-Path $PSScriptRoot -Parent
$Bugs = Join-Path $RepoRoot 'bugs.md'
$State = Join-Path $RepoRoot 'target\bugs.state'

if (-not (Test-Path $Bugs)) { Write-Host 'bugs.md no existe'; exit 0 }

$hash = (Get-FileHash -Algorithm SHA256 $Bugs).Hash
$inFence = $false
$entries = @(Get-Content $Bugs | Where-Object {
        if ($_ -match '^```') { $inFence = -not $inFence; return $false }
        (-not $inFence) -and $_ -match '^## \['
    })
$open = @($entries | Where-Object { $_ -like '## `[ `]*' })

$previous = if (Test-Path $State) { Get-Content $State -Raw } else { '' }
if ($previous -and $previous.Trim() -eq $hash) {
    Write-Host "bugs.md sin cambios ($($entries.Count) entradas, $($open.Count) abiertas)"
    exit 0
}

New-Item -ItemType Directory -Force -Path (Split-Path $State) | Out-Null
Set-Content -Path $State -Value $hash -NoNewline
Write-Host "bugs.md CAMBIADO: $($entries.Count) entradas, $($open.Count) abiertas. Ábrelo:" -ForegroundColor Yellow
$entries | ForEach-Object { "  $_" }
