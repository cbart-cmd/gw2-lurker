<#
.SYNOPSIS
Sube la versión a la vez en Cargo.toml, las dependencias internas de los crates, Cargo.lock y CHANGELOG.md.
.DESCRIPTION
Toca los cuatro sitios que llevan la versión:
  - `[workspace.package] version` de Cargo.toml (raíz);
  - la `version` de las dependencias internas en crates/*/Cargo.toml;
  - las entradas gw2lurker-* de Cargo.lock (por texto, sin llamar a cargo);
  - el bloque `## [Unreleased]` de CHANGELOG.md, que pasa a `## [x.y.z] - fecha`.
Es idempotente: si el raíz ya está en esa versión, solo corrige lo que falte. Al final corre
scripts/check_version.sh; sale con código 1 si la versión no tiene forma x.y.z o si no cuadra.
.EXAMPLE
pwsh -File scripts/bump-version.ps1 -Version 0.26.4
.EXAMPLE
pwsh -File scripts/bump-version.ps1 -Version 0.26.4 -DryRun
.EXAMPLE
pwsh -File scripts/bump-version.ps1 -Version 0.26.4 -Date 2026-11-02 -SkipChangelog
#>

[CmdletBinding()]
param(
    [Parameter(Mandatory)][string]$Version,
    [string]$Date = (Get-Date -Format 'yyyy-MM-dd'),
    [switch]$DryRun,
    [switch]$SkipChangelog
)

$ErrorActionPreference = 'Stop'
$RepoRoot = Split-Path $PSScriptRoot -Parent
Set-Location $RepoRoot

if ($Version -notmatch '^\d+\.\d+\.\d+$') { Write-Host "ERROR '$Version' no es una version x.y.z"; exit 1 }
if ($Date -notmatch '^\d{4}-\d{2}-\d{2}$') { Write-Host "ERROR '$Date' no es una fecha yyyy-MM-dd"; exit 1 }

function Write-File($path, $text, $what) {
    if ($DryRun) { Write-Host "DRY   $what"; return }
    Set-Content -Path $path -Value $text -Encoding utf8NoBOM -NoNewline
    Write-Host "OK    $what"
}

function Get-Newline($text) { if ($text -match "`r`n") { "`r`n" } else { "`n" } }

# 1. Versión del workspace.
$rootPath = 'Cargo.toml'
$root = Get-Content -Raw $rootPath
$head = [regex]::new('(?m)^version = "[0-9][^"]*"')
if (-not $head.IsMatch($root)) { Write-Host "ERROR no encuentro [workspace.package] version en $rootPath"; exit 1 }
$current = $head.Match($root).Value.Substring(11).Trim('"')
if ($current -eq $Version) {
    Write-Host "OK    Cargo.toml ya estaba en $Version"
} else {
    Write-File $rootPath ($head.Replace($root, "version = `"$Version`"", 1)) "Cargo.toml $current -> $Version"
}

# 2. Dependencias internas de los crates.
$dep = [regex]::new('(?m)(?<head>gw2lurker-(?:core|api|addon)\s*=\s*\{[^}]*version\s*=\s*")[^"]+(")')
foreach ($file in Get-ChildItem 'crates/*/Cargo.toml') {
    $text = Get-Content -Raw $file.FullName
    $updated = $dep.Replace($text, ('${head}' + $Version + '"'))
    if ($updated -ceq $text) { continue }
    Write-File $file.FullName $updated "$($file.FullName.Replace("$RepoRoot\", '')) depende de $Version"
}

# 3. Cargo.lock: las entradas de los crates del workspace, sin llamar a cargo (no hace falta red).
$lockPath = 'Cargo.lock'
$lock = Get-Content -Raw $lockPath
$entry = [regex]::new('(?ms)(?<head>^\[\[package\]\]\r?\nname = "gw2lurker-[a-z]+"\r?\nversion = ")[^"]+(")')
$locked = $entry.Replace($lock, ('${head}' + $Version + '"'))
if ($locked -ceq $lock) {
    Write-Host "OK    Cargo.lock ya estaba en $Version"
} else {
    Write-File $lockPath $locked "Cargo.lock -> $Version"
}

# 4. CHANGELOG: lo que había en [Unreleased] pasa a la versión nueva y se deja uno vacío.
if ($SkipChangelog) {
    Write-Host "OK    CHANGELOG.md sin tocar (-SkipChangelog)"
} else {
    $clPath = 'CHANGELOG.md'
    $cl = Get-Content -Raw $clPath
    $nl = Get-Newline $cl
    $block = [regex]::new('(?ms)^## \[Unreleased\][^\r\n]*\r?\n(?<body>.*?)(?=^## |\z)')
    $m = $block.Match($cl)
    if (-not $m.Success) {
        Write-Host "AVISO CHANGELOG.md no tiene ## [Unreleased]: no lo toco"
    } elseif ($m.Groups['body'].Value.Trim() -eq '') {
        Write-Host "AVISO CHANGELOG.md: [Unreleased] esta vacio: no lo toco"
    } elseif ($cl -match "(?m)^## \[$([regex]::Escape($Version))\]") {
        Write-Host "AVISO CHANGELOG.md ya tiene ## [$Version]: no lo toco"
    } else {
        $replacement = "## [Unreleased]$nl$nl## [$Version] - $Date$nl" + $m.Groups['body'].Value
        Write-File $clPath ($cl.Remove($m.Index, $m.Length).Insert($m.Index, $replacement)) "CHANGELOG.md [Unreleased] -> [$Version] - $Date"
    }
}

if ($DryRun) { Write-Host "`nDRY   nada escrito. Quita -DryRun para aplicarlo."; exit 0 }

# 5. Comprobación: la misma que corre validate.ps1 como paso `version`.
$bash = (Get-Command bash -ErrorAction SilentlyContinue).Source
if (-not $bash) { $bash = 'C:\Program Files\Git\bin\bash.exe' }
& $bash ./scripts/check_version.sh
if ($LASTEXITCODE -ne 0) { Write-Host "ERROR la comprobacion de versiones fallo"; exit 1 }
Write-Host "`nSiguiente: pwsh -File scripts/validate.ps1 y pwsh -File scripts/release.ps1"
