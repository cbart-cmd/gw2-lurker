# Empaqueta el indice de nombres de clan y lo publica en el repositorio de distribucion (RF-20c).
#
# Uso:
#   pwsh -File scripts/pack-guild-index.ps1                    # solo empaqueta y verifica
#   pwsh -File scripts/pack-guild-index.ps1 -Publish           # ademas lo sube al repo hermano
#   pwsh -File scripts/pack-guild-index.ps1 -Json <ruta>       # usar otro guild-names.json
#
# Los dos ficheros viven en `data/`, junto al JSON del que salen:
#   data/guild-names.json          el indice de trabajo (se versiona: lo mantiene el jugador)
#   data/gw2_lurker_guilds.zip     lo que descarga el addon
#   data/gw2_lurker_guilds.zip.md5 su hash
# El nombre del ZIP esta fijado en `crates/addon/src/guild_index.rs` (`DEFAULT_URL`): si cambia
# aqui, cambia alli.
#
# Reglas que aplica, en orden:
#   1. el JSON tiene que existir, ser un objeto `id -> "[TAG] Nombre"` y no estar vacio,
#   2. el sidecar .md5 lleva 32 hex en minusculas SIN BOM y SIN salto de linea,
#   3. se relee el ZIP y se comprueba que dentro esta el JSON y que el MD5 cuadra,
#   4. solo se tocan los ficheros gw2_lurker_guilds.* del repo hermano,
#   5. se descarga la URL publicada y se comparan bytes y MD5 (con -Publish).

[CmdletBinding()]
param(
    [switch]$Publish,
    [string]$Json = '',
    [string]$OutDir = '',
    [string]$LocalDistRepo = ''
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$RepoRoot = Split-Path $PSScriptRoot -Parent
$ZipName = 'gw2_lurker_guilds.zip'
$EntryName = 'guild-names.json'
$Md5Name = "$ZipName.md5"
$PublicUrl = "https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/$ZipName"

if (-not $Json) {
    # El de trabajo está en el repo, junto al ZIP; si aún no existe se usa el que tiene el jugador
    # en la carpeta del addon (el del repo manda: no se copia al revés).
    $inRepo = Join-Path $RepoRoot "data\$EntryName"
    if (Test-Path $inRepo) {
        $Json = $inRepo
    } else {
        $documents = [Environment]::GetFolderPath('MyDocuments')
        $Json = Join-Path $documents "Guild Wars 2\addons\gw2_lurker\$EntryName"
    }
}
if (-not $OutDir) { $OutDir = Join-Path $RepoRoot 'data' }
if (-not $LocalDistRepo) {
    $LocalDistRepo = Join-Path (Split-Path $RepoRoot -Parent) 'gw2addonsreleases'
}

function Step($text) { Write-Host "`n== $text" -ForegroundColor Cyan }
function Ok($text) { Write-Host "   ok: $text" -ForegroundColor Green }
function Fail($text) { Write-Host "   ERROR: $text" -ForegroundColor Red; throw $text }

function Get-Md5([string]$path) {
    (Get-FileHash -Algorithm MD5 -Path $path).Hash.ToLowerInvariant()
}

Step 'Comprobando el indice de origen'
if (-not (Test-Path $Json)) { Fail "no existe $Json (generalo con scripts/guild-search.ps1 -BuildIndex)" }
$raw = [System.IO.File]::ReadAllText($Json)
$index = $raw | ConvertFrom-Json -AsHashtable
$count = @($index.Keys).Count
if ($count -eq 0) { Fail "$Json no tiene ningun nombre" }
$bad = @($index.GetEnumerator() | Where-Object { -not $_.Key -or -not $_.Value } | Select-Object -First 3)
if ($bad.Count -gt 0) { Fail "hay entradas sin id o sin nombre en $Json" }
Ok "$count nombres en $Json"

Step 'Empaquetando'
$staging = Join-Path ([System.IO.Path]::GetTempPath()) ("gw2lurker-index-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $staging | Out-Null
try {
    Copy-Item $Json (Join-Path $staging $EntryName)
    if (-not (Test-Path $OutDir)) { New-Item -ItemType Directory -Path $OutDir | Out-Null }
    $zip = Join-Path $OutDir $ZipName
    if (Test-Path $zip) { Remove-Item $zip -Force }
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    [System.IO.Compression.ZipFile]::CreateFromDirectory(
        $staging,
        $zip,
        [System.IO.Compression.CompressionLevel]::Optimal,
        $false
    )
} finally {
    Remove-Item $staging -Recurse -Force -ErrorAction SilentlyContinue
}
Ok "$ZipName generado ($((Get-Item $zip).Length) bytes)"

Step 'Escribiendo y verificando el .md5'
$md5 = Get-Md5 $zip
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText((Join-Path $OutDir $Md5Name), $md5, $utf8NoBom)
$sidecar = [System.IO.File]::ReadAllBytes((Join-Path $OutDir $Md5Name))
if ($sidecar.Length -ne 32) { Fail "el .md5 mide $($sidecar.Length) bytes (deberian ser 32)" }
if ($md5 -cne $md5.ToLowerInvariant()) { Fail 'el .md5 tiene mayusculas' }
Ok "$Md5Name = $md5"

Step 'Releyendo el ZIP publicado'
Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [System.IO.Compression.ZipFile]::OpenRead($zip)
try {
    $entry = $archive.Entries | Where-Object { $_.FullName -eq $EntryName }
    if (-not $entry) { Fail "el ZIP no trae $EntryName en la raiz" }
    $stream = $entry.Open()
    $reader = New-Object System.IO.StreamReader($stream)
    $inside = $reader.ReadToEnd()
    $reader.Dispose()
} finally {
    $archive.Dispose()
}
$insideCount = @(($inside | ConvertFrom-Json -AsHashtable).Keys).Count
if ($insideCount -ne $count) { Fail "el ZIP trae $insideCount nombres y el origen $count" }
Ok "$EntryName dentro del ZIP con $insideCount nombres"

if (-not $Publish) {
    Write-Host "`nSin -Publish: no se toca nada remoto." -ForegroundColor Yellow
    Write-Host "   los dos ficheros estan en $OutDir"
    exit 0
}

Step 'Comprobando el repo de distribucion'
if (-not (Test-Path (Join-Path $LocalDistRepo '.git'))) {
    Fail "no existe el clon del repo de distribucion en $LocalDistRepo"
}
$others = & git -C $LocalDistRepo status --porcelain |
    Where-Object { $_ -match '\S' } |
    Where-Object { $_ -notmatch "gw2_lurker_guilds\.(zip|zip\.md5)$" }
if ($others) {
    Fail "hay cambios ajenos en el repo de distribucion:`n$($others -join "`n")"
}
Ok 'solo estan nuestros ficheros'

Step 'Copiando al repo hermano'
Copy-Item $zip (Join-Path $LocalDistRepo $ZipName) -Force
[System.IO.File]::WriteAllText((Join-Path $LocalDistRepo $Md5Name), $md5, $utf8NoBom)

# La excepcion de LFS tiene que existir ANTES del `git add`, o el repo guarda un puntero de
# texto y el addon descarga 130 bytes en vez del ZIP (lo mismo que con la DLL).
$gitAttributes = Join-Path $LocalDistRepo '.gitattributes'
$lines = @()
if (Test-Path $gitAttributes) { $lines = @(Get-Content $gitAttributes) }
$want = "$ZipName -filter -diff -merge -text"
if (-not ($lines | Where-Object { $_.Trim() -eq $want })) {
    $lines += $want
    [System.IO.File]::WriteAllText($gitAttributes, (($lines -join "`n") + "`n"), $utf8NoBom)
    Ok "excepcion de LFS anadida para $ZipName"
}

& git -C $LocalDistRepo add -- $ZipName $Md5Name
if (Test-Path $gitAttributes) { & git -C $LocalDistRepo add -- '.gitattributes' }

$blobSize = [int](& git -C $LocalDistRepo cat-file -s ":$ZipName")
$localSize = (Get-Item (Join-Path $LocalDistRepo $ZipName)).Length
if ($blobSize -ne $localSize) {
    Fail "el blob preparado mide $blobSize y el fichero $localSize (puntero LFS)"
}
Ok "blob de $blobSize bytes (no es un puntero LFS)"

$message = "gw2_lurker_guilds v$count nombres ($md5)"
& git -C $LocalDistRepo commit -m $message -- $ZipName $Md5Name | Out-Host
if ($LASTEXITCODE -ne 0) { Fail 'no se pudo hacer el commit en el repo de distribucion' }
& git -C $LocalDistRepo push origin main | Out-Host
if ($LASTEXITCODE -ne 0) {
    Fail "no se pudo empujar a origin (¿alguien mas publico? haz 'git -C $LocalDistRepo pull --rebase' y repite)"
}
Ok 'repo de distribucion actualizado'

Step 'Verificando la descarga real'
$fetched = Invoke-WebRequest -Uri $PublicUrl -UseBasicParsing
$bytes = $fetched.Content
if ($bytes -is [string]) { $bytes = [System.Text.Encoding]::Latin1.GetBytes($bytes) }
$hash = [System.Security.Cryptography.MD5]::Create().ComputeHash($bytes)
$remoteMd5 = ([System.BitConverter]::ToString($hash) -replace '-', '').ToLowerInvariant()
if ($remoteMd5 -ne $md5) {
    Fail "la URL publicada devuelve MD5 $remoteMd5 y el local es $md5 (¿cache de raw.githubusercontent?)"
}
Ok "$PublicUrl devuelve $($bytes.Length) bytes con MD5 $remoteMd5"
