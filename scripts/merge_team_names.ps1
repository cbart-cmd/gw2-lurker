# Extrae los nombres de equipo de data\equipos.xlsx a data\team-names.json.
#
# El Excel trae una hoja con cuatro columnas: Inglés, Español, Francés, Alemán. Este script la
# lee sin dependencias (un .xlsx es un ZIP con XML), normaliza las erratas conocidas y escribe
# las traducciones **indexadas por el nombre inglés**, no por id: los nombres de equipo de WvW
# rotan cada temporada, así que la traducción debe sobrevivir a que un id cambie de nombre.
#
# Se puede volver a ejecutar cuando cambie el Excel:
#   powershell -File scripts/merge_team_names.ps1 [-DryRun] [-Xlsx data\equipos.xlsx]
#
# Después, `scripts/update_wvw_teams.ps1` refresca los nombres de cada id y las traducciones
# se resuelven solas por nombre.

param(
    [string]$Xlsx = (Join-Path $PSScriptRoot '..\data\equipos.xlsx'),
    [string]$OutFile = (Join-Path $PSScriptRoot '..\data\team-names.json'),
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

# Erratas del Excel que impedirían encontrar la traducción si el nombre llega bien escrito.
$Normalize = @{
    "Modremoth's Canopy" = "Mordremoth's Canopy"   # el dragón anciano es Mordremoth
}

$Languages = [ordered]@{
    'Inglés (English)'  = 'en'
    'Español'           = 'es'
    'Francés (Français)' = 'fr'
    'Alemán (Deutsch)'  = 'de'
}

if (-not (Test-Path $Xlsx)) { throw "No existe el Excel: $Xlsx" }

# Un .xlsx es un ZIP: se copia a temporal y se expande sin tocar el original.
$work = Join-Path ([System.IO.Path]::GetTempPath()) ("gw2lurker-xlsx-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory $work -Force | Out-Null
try {
    $copy = Join-Path $work 'equipos.zip'
    Copy-Item $Xlsx $copy -Force
    Expand-Archive -Path $copy -DestinationPath (Join-Path $work 'x') -Force
    $root = Join-Path $work 'x'

    # Cadenas compartidas: las celdas de texto apuntan aquí por índice.
    $shared = @()
    $sharedPath = Join-Path $root 'xl\sharedStrings.xml'
    if (Test-Path $sharedPath) {
        [xml]$ss = Get-Content $sharedPath -Raw -Encoding UTF8
        foreach ($si in $ss.SelectNodes("//*[local-name()='si']")) {
            $shared += (($si.SelectNodes(".//*[local-name()='t']") | ForEach-Object { $_.InnerText }) -join '')
        }
    }

    $sheetPath = Join-Path $root 'xl\worksheets\sheet1.xml'
    if (-not (Test-Path $sheetPath)) { throw "El Excel no tiene xl\worksheets\sheet1.xml" }
    [xml]$sheet = Get-Content $sheetPath -Raw -Encoding UTF8

    # Filas -> columnas. La referencia de celda (A1, B1…) da la columna.
    $rows = @()
    foreach ($row in $sheet.SelectNodes("//*[local-name()='row']")) {
        $cells = @{}
        foreach ($cell in $row.SelectNodes("*[local-name()='c']")) {
            $ref = $cell.GetAttribute('r')
            $column = ($ref -replace '\d', '')
            $type = $cell.GetAttribute('t')
            $valueNode = $cell.SelectSingleNode("*[local-name()='v']")
            if ($null -eq $valueNode) { continue }
            if ($type -eq 's') {
                $text = $shared[[int]$valueNode.InnerText]
            } else {
                $text = $valueNode.InnerText
            }
            $cells[$column] = ([string]$text).Trim()
        }
        if ($cells.Count -gt 0) { $rows += , $cells }
    }
    if ($rows.Count -lt 2) { throw "El Excel no tiene filas de datos (solo $($rows.Count))" }

    # Cabecera -> índice de columna por idioma. Si cambia el orden de las columnas, se adapta.
    $header = $rows[0]
    $columnOf = @{}
    foreach ($column in $header.Keys) {
        $title = $header[$column]
        foreach ($name in $Languages.Keys) {
            if ($title -eq $name -or $title -like "$($name.Split(' ')[0])*") {
                $columnOf[$Languages[$name]] = $column
            }
        }
    }
    foreach ($code in 'en', 'es', 'fr', 'de') {
        if (-not $columnOf.ContainsKey($code)) {
            throw "No encuentro la columna de '$code' en la cabecera: $($header.Values -join ' | ')"
        }
    }

    $names = [ordered]@{}
    $normalized = @()
    foreach ($cells in $rows[1..($rows.Count - 1)]) {
        $english = [string]$cells[$columnOf['en']]
        if (-not $english) { continue }

        if ($Normalize.ContainsKey($english)) {
            $fixed = $Normalize[$english]
            $normalized += "$english -> $fixed"
            $english = $fixed
        }

        $entry = [ordered]@{}
        foreach ($code in 'es', 'fr', 'de') {
            $value = [string]$cells[$columnOf[$code]]
            if ($value) { $entry[$code] = $value }
        }
        if ($entry.Count -gt 0) {
            $entry['source'] = 'spreadsheet'
            $names[$english] = $entry
        }
    }

    # Comparar con los equipos en uso, para avisar de lo que no encaja.
    $teamsFile = Join-Path $PSScriptRoot '..\data\wvw-teams.json'
    $inUse = @()
    if (Test-Path $teamsFile) {
        $inUse = (Get-Content $teamsFile -Raw -Encoding UTF8 | ConvertFrom-Json).teams
    }
    $matched = @()
    $missing = @()
    foreach ($team in $inUse) {
        if ($names.Contains($team.name)) { $matched += $team.name } else { $missing += $team.name }
    }
    $unused = @($names.Keys | Where-Object { $inUse.name -notcontains $_ })

    $payload = [ordered]@{
        note      = 'Traducciones de nombres de equipo indexadas por el nombre ingles. Las genera scripts/merge_team_names.ps1 desde data/equipos.xlsx; se resuelven por nombre, no por id, porque los nombres rotan cada temporada y los ids no.'
        source    = 'data/equipos.xlsx'
        updated_at = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
        normalized = $normalized
        names     = $names
    }
    $json = $payload | ConvertTo-Json -Depth 6

    Write-Host "Equipos en el Excel: $($names.Count)"
    Write-Host "Coinciden con los 33 en uso: $($matched.Count)" -ForegroundColor Green
    if ($missing.Count -gt 0) {
        Write-Host "En uso y sin traduccion ($($missing.Count)): $($missing -join ', ')" -ForegroundColor Yellow
    }
    if ($unused.Count -gt 0) {
        Write-Host "En el Excel pero no en uso ($($unused.Count)): $($unused -join ', ')" -ForegroundColor Yellow
    }
    if ($normalized.Count -gt 0) {
        Write-Host "Normalizados: $($normalized -join '; ')"
    }

    if ($DryRun) {
        Write-Host 'DryRun: no se escribe nada.' -ForegroundColor Yellow
    } else {
        [System.IO.File]::WriteAllText($OutFile, $json, (New-Object System.Text.UTF8Encoding($false)))
        Write-Host "Escrito: $OutFile" -ForegroundColor Green
    }
} finally {
    Remove-Item $work -Recurse -Force -ErrorAction SilentlyContinue
}
