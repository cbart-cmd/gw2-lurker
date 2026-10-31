# Buscador de clanes fuera del juego (T-44, RF-73).
#
# Sirve para dos cosas, sin abrir Guild Wars 2:
#   1. buscar un clan como lo hace el addon (subcadena, sin distinguir mayúsculas),
#   2. construir la caché `guild-names.json` que el addon usa para buscar (una vez; luego se
#      reutiliza, porque los nombres de clan no cambian).
#
# Uso:
#   pwsh -File scripts/guild-search.ps1 -Query "die"
#   pwsh -File scripts/guild-search.ps1 -Query "a new" -Region eu
#   pwsh -File scripts/guild-search.ps1 -BuildIndex -Region eu     # rellena la caché
#   pwsh -File scripts/guild-search.ps1 -Query die -ApiOnly        # solo el API, para ver su límite
#
# Por qué hace falta la caché: `/v2/guild/search?name=` devuelve **un solo** clan y solo cuando el
# texto empieza su nombre (medido el 2026-11-01: `die` → *Die*, no *Die Garde*; `a new` → *A New*).
# La búsqueda de verdad la hace el addon sobre esta caché.

[CmdletBinding()]
param(
    [string]$Query = '',
    [ValidateSet('eu', 'na', 'both')][string]$Region = 'both',
    [string]$Cache = '',
    [int]$Limit = 40,
    [switch]$BuildIndex,
    [switch]$ApiOnly,
    [int]$DelayMs = 120,
    [int]$MaxIndex = 0
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'
$Base = 'https://api.guildwars2.com/v2'

if (-not $Cache) {
    # La misma carpeta que usa el addon: <Documentos>\Guild Wars 2\addons\gw2_lurker\guild-names.json
    $documents = [Environment]::GetFolderPath('MyDocuments')
    $Cache = Join-Path $documents 'Guild Wars 2\addons\gw2_lurker\guild-names.json'
}

function Get-Json([string]$Url) {
    $raw = curl.exe -s $Url
    if (-not $raw) { return $null }
    try { return $raw | ConvertFrom-Json } catch { return $null }
}

function Read-Cache {
    if (Test-Path $Cache) {
        return @{ } | ForEach-Object { $h = @{}; (Get-Content $Cache -Raw | ConvertFrom-Json).PSObject.Properties | ForEach-Object { $h[$_.Name] = $_.Value }; $h }
    }
    return @{}
}

function Save-Cache($map) {
    $dir = Split-Path $Cache -Parent
    if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Force -Path $dir | Out-Null }
    $ordered = [ordered]@{}
    foreach ($key in ($map.Keys | Sort-Object)) { $ordered[$key] = $map[$key] }
    ($ordered | ConvertTo-Json -Depth 3) | Set-Content -Path $Cache -Encoding UTF8 -NoNewline
}

function Get-Guild([string]$Id) {
    return Get-Json "$Base/guild/$Id"
}

function Resolve-Name([string]$Id) {
    $guild = Get-Guild $Id
    if ($null -eq $guild) { return $null }
    $tag = if ($guild.tag) { "[$($guild.tag)]" } else { '' }
    return ("$tag $($guild.name)").Trim()
}

function Format-Guild([string]$Id) {
    $name = Resolve-Name $Id
    if (-not $name) { return "$Id (sin nombre: el API no lo devuelve)" }
    return "$name  ($Id)"
}

# --- 1. Caché -------------------------------------------------------------------------------
$cacheMap = Read-Cache
Write-Host "Caché: $($cacheMap.Count) clanes en $Cache" -ForegroundColor DarkGray

if ($BuildIndex) {
    $regions = if ($Region -eq 'both') { @('eu', 'na') } else { @($Region) }
    foreach ($r in $regions) {
        Write-Host "== Índice de $($r.ToUpper())" -ForegroundColor Cyan
        $teams = Get-Json "$Base/wvw/guilds/$r"
        if ($null -eq $teams) { throw "no pude leer /wvw/guilds/$r" }
        $ids = @($teams.PSObject.Properties.Name)
        Write-Host "   $($ids.Count) clanes con algo reclamado"
        $done = 0; $added = 0
        if ($MaxIndex -gt 0) { $ids = $ids | Select-Object -First $MaxIndex; Write-Host "   (prueba: solo los primeros $MaxIndex)" }
        foreach ($id in $ids) {
            $done++
            if ($cacheMap.ContainsKey($id)) { continue }
            $name = Resolve-Name $id
            if ($name) { $cacheMap[$id] = $name; $added++ }
            if ($done % 50 -eq 0) {
                Write-Host ("   {0}/{1} (nuevos: {2})" -f $done, $ids.Count, $added)
                Save-Cache $cacheMap
            }
            if ($DelayMs -gt 0) { Start-Sleep -Milliseconds $DelayMs }
        }
        Save-Cache $cacheMap
        Write-Host "   hecho: $added nombres nuevos" -ForegroundColor Green
    }
}

# --- 2. Búsqueda ----------------------------------------------------------------------------
if ($Query) {
    $needle = $Query.Trim().ToLowerInvariant()
    Write-Host "`n== Búsqueda local (subcadena, como el addon) para «$Query»" -ForegroundColor Cyan
    $hits = @()
    foreach ($key in $cacheMap.Keys) {
        if ($cacheMap[$key].ToLowerInvariant().Contains($needle) -or $key.ToLowerInvariant().StartsWith($needle)) {
            $hits += [pscustomobject]@{ Id = $key; Name = $cacheMap[$key] }
        }
    }
    $hits = $hits | Sort-Object { $_.Name.Length } | Select-Object -First $Limit
    if ($hits) { $hits | Format-Table -AutoSize | Out-String -Width 200 | Write-Host } else { Write-Host '   (nada)' -ForegroundColor Yellow }

    if (-not $ApiOnly) {
        Write-Host "== Lo mismo buscaría el API (una sóla coincidencia, solo por el principio)" -ForegroundColor DarkGray
    }
    $encoded = [uri]::EscapeDataString($Query.Trim())
    $apiId = Get-Json "$Base/guild/search?name=$encoded"
    if ($apiId) {
        Write-Host "   API: $(Format-Guild $apiId)" -ForegroundColor Yellow
    } else {
        Write-Host '   API: nada' -ForegroundColor Yellow
    }
}

if (-not $Query -and -not $BuildIndex) {
    Write-Host 'Nada que hacer: usa -Query o -BuildIndex (ver la cabecera del script).'
}
