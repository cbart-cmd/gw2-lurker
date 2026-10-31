# Updates data/wvw-teams.json: the team id -> name table the addon uses to label
# WvW teams. The official GW2 API exposes team ids (11xxx/12xxx in `all_worlds`
# and /v2/wvw/guilds/:region) but not their names, so names are curated here.
#
# Only id/name/nation are taken from the curated source. Aggregate stats
# (players/guilds/commanders) are NOT copied: they are third-party compilation
# data, change constantly and the addon does not use them.
#
# Usage: powershell -File scripts/update_wvw_teams.ps1 [-DryRun]

param(
    [string]$OutFile = (Join-Path $PSScriptRoot '..\data\wvw-teams.json'),
    [string]$Source = 'https://api.gw2mists.com/worlds/alliance',
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$headers = @{
    'User-Agent' = 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/141.0 Safari/537.36'
    'Accept'     = 'application/json'
    'Origin'     = 'https://gw2mists.com'
    'Referer'    = 'https://gw2mists.com/'
}

function Get-Json([string]$uri, [hashtable]$headers) {
    $response = Invoke-WebRequest -Uri $uri -Headers $headers -UseBasicParsing
    return $response.Content | ConvertFrom-Json
}

# Names: id/name/nation only.
$curated = Get-Json $Source $headers
$names = @{}
foreach ($team in $curated) {
    if ($team.id -and $team.name) {
        $names[[int]$team.id] = [ordered]@{
            id     = [int]$team.id
            name   = [string]$team.name
            nation = [string]$team.nation
        }
    }
}

# Ids actually in use right now, from the official API (no names there).
$matches = Get-Json 'https://api.guildwars2.com/v2/wvw/matches?ids=all' @{}
$inUse = [System.Collections.Generic.HashSet[int]]::new()
foreach ($match in $matches) {
    foreach ($color in 'red', 'blue', 'green') {
        foreach ($world in $match.all_worlds.$color) {
            if ($world -ge 11000) { [void]$inUse.Add([int]$world) }
        }
    }
}

# Keep previously known teams so a source failure never wipes the table.
$previous = @{}
if (Test-Path $OutFile) {
    $old = Get-Content $OutFile -Raw | ConvertFrom-Json
    foreach ($team in $old.teams) { $previous[[int]$team.id] = $team }
    foreach ($id in $previous.Keys) {
        if (-not $names.ContainsKey($id)) { $names[$id] = $previous[$id] }
    }
    # Translations are typed by a human (the API does not translate team names): never lose
    # them when regenerating, and never invent them either.
    foreach ($id in @($names.Keys)) {
        $kept = $previous[$id].names
        if ($kept) { $names[$id].names = $kept }
    }
}

$missing = @($inUse | Where-Object { -not $names.ContainsKey($_) } | Sort-Object)
$teams = @($names.Values | Sort-Object id)

$payload = [ordered]@{
    note       = 'WvW team id -> name. Ids come from the official API (all_worlds, /v2/wvw/guilds/:region); names are curated because the API does not expose them. Regenerate with scripts/update_wvw_teams.ps1.'
    updated_at = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
    teams      = $teams
}
$json = ($payload | ConvertTo-Json -Depth 5) -replace '\\u003e', '>' -replace '\\u003c', '<' -replace '\\u0026', '&'

# Avoid a new timestamp when nothing changed, so the diff stays clean.
if (Test-Path $OutFile) {
    $old = Get-Content $OutFile -Raw | ConvertFrom-Json
    $oldPairs = ($old.teams | ForEach-Object { "$($_.id)=$($_.name)" }) -join ','
    $newPairs = ($teams | ForEach-Object { "$($_.id)=$($_.name)" }) -join ','
    if ($oldPairs -eq $newPairs) {
        Write-Host "Sin cambios: $($teams.Count) equipos ya al dia."
        if ($missing.Count -gt 0) { Write-Warning "Equipos en uso sin nombre: $($missing -join ', ')" }
        exit 0
    }
}

if ($DryRun) {
    Write-Host '--- DryRun, no se escribe nada ---'
    Write-Host $json
} else {
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutFile) | Out-Null
    [System.IO.File]::WriteAllText($OutFile, $json, (New-Object System.Text.UTF8Encoding($false)))
    Write-Host "Escrito $OutFile con $($teams.Count) equipos."
}

if ($missing.Count -gt 0) {
    Write-Warning "Equipos en uso sin nombre (anadir a mano o revisar la fuente): $($missing -join ', ')"
}
