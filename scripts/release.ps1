# Publica gw2_lurker.dll en el repositorio de distribucion compartido (T-34, RF-53).
#
# Uso:
#   pwsh -File scripts/release.ps1 -DryRun     # compila y valida, sin tocar nada remoto
#   pwsh -File scripts/release.ps1             # publica de verdad
#   pwsh -File scripts/release.ps1 -VerifyOnly # solo comprueba la URL publicada
#
# Reglas que aplica (docs/RELEASE.md), en orden:
#   1. la DLL debe empezar por "MZ" y respetar el limite de tamano,
#   2. el sidecar .md5 lleva 32 hex en minusculas SIN BOM y SIN salto de linea,
#   3. el blob preparado no puede ser un puntero LFS (git cat-file -s == tamano local),
#   4. solo se tocan los ficheros gw2_lurker.* del repo hermano,
#   5. la release se crea con --latest=false y se retienen solo la actual y la anterior,
#   6. se descarga la URL exacta de update_link y se comparan bytes y MD5.

[CmdletBinding()]
param(
    [switch]$DryRun,
    [switch]$VerifyOnly,
    # Empty on purpose: the host is already windows-msvc and a plain build keeps the DLL in
    # `target\release\`, like the sibling projects. Pass `-Target <triple>` to override.
    [string]$Target = '',
    [string]$DistRepo = 'github.com/cbart-cmd/gw2addonsreleases',
    [string]$LocalDistRepo = ''
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$RepoRoot = Split-Path $PSScriptRoot -Parent
$DllName = 'gw2_lurker.dll'
$Md5Name = "$DllName.md5"
$UpdateUrl = 'https://raw.githubusercontent.com/cbart-cmd/gw2addonsreleases/main/gw2_lurker.dll'
$GitAttributesLine = 'gw2_lurker.dll -filter -diff -merge -text'

if (-not $LocalDistRepo) {
    $LocalDistRepo = Join-Path (Split-Path $RepoRoot -Parent) 'gw2addonsreleases'
}

function Step($text) { Write-Host "`n== $text" -ForegroundColor Cyan }
function Ok($text) { Write-Host "   ok: $text" -ForegroundColor Green }
function Warn($text) { Write-Host "   aviso: $text" -ForegroundColor Yellow }
function Fail($text) { Write-Host "   ERROR: $text" -ForegroundColor Red; throw $text }

function Get-WorkspaceVersion {
    $manifest = Join-Path $RepoRoot 'Cargo.toml'
    $match = [regex]::Match((Get-Content -Raw $manifest), '(?m)^version = "([^"]+)"')
    if (-not $match.Success) { Fail "no encuentro [workspace.package] version en $manifest" }
    return $match.Groups[1].Value
}

function Get-Md5([string]$path) {
    (Get-FileHash -Algorithm MD5 -Path $path).Hash.ToLowerInvariant()
}

function Assert-Mz([string]$path) {
    $head = [System.IO.File]::ReadAllBytes($path)[0..1]
    if ($head[0] -ne 0x4D -or $head[1] -ne 0x5A) { Fail "$path no empieza por MZ" }
    Ok "MZ presente"
}

# Bytes reales, sin que PowerShell intente interpretar la respuesta como texto.
function Save-Remote([string]$url, [string]$destination) {
    $curl = Get-Command curl.exe -ErrorAction SilentlyContinue
    if ($curl) {
        & $curl.Source --fail --silent --show-error --location --output $destination $url
        if ($LASTEXITCODE -ne 0) { Fail "curl fallo al descargar $url (codigo $LASTEXITCODE)" }
    } else {
        Invoke-WebRequest -Uri $url -OutFile $destination
    }
}

function Test-PublishedDll([string]$expectedPath) {
    $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("gw2lurker-verify-" + [guid]::NewGuid().ToString('N') + '.bin')
    try {
        Save-Remote $UpdateUrl $tmp
        Assert-Mz $tmp
        $local = Get-Md5 $expectedPath
        $remote = Get-Md5 $tmp
        $localSize = (Get-Item $expectedPath).Length
        $remoteSize = (Get-Item $tmp).Length
        if ($localSize -ne $remoteSize) { Fail "tamano distinto: local $localSize, publicado $remoteSize (puntero LFS o cache)" }
        if ($local -ne $remote) { Fail "MD5 distinto: local $local, publicado $remote (cache de CDN: reintenta en unos minutos)" }
        Ok "update_link devuelve $remoteSize bytes con MD5 $remote"
    } finally {
        Remove-Item $tmp -ErrorAction SilentlyContinue
    }
}

# Empty `-Target` keeps the plain `target\release` layout, like the sibling projects.
$BuildArgs = @('build', '--release', '-p', 'gw2lurker-addon')
if ($Target) { $BuildArgs += @('--target', $Target) }
$OutDir = if ($Target) { Join-Path $RepoRoot "target\$Target\release" } else { Join-Path $RepoRoot 'target\release' }
$version = Get-WorkspaceVersion
$tag = "gw2-lurker-v$version"
Step "Version $version (tag $tag)"

if ($VerifyOnly) {
    Test-PublishedDll (Join-Path $OutDir $DllName)
    return
}

Step 'Compilando release'
Push-Location $RepoRoot
try {
    & cargo @BuildArgs
    if ($LASTEXITCODE -ne 0) { Fail 'cargo build fallo' }
} finally {
    Pop-Location
}
$dll = Join-Path $OutDir $DllName
if (-not (Test-Path $dll)) { Fail "no existe $dll" }
$size = (Get-Item $dll).Length
Ok "DLL de $size bytes"

Assert-Mz $dll
$md5 = Get-Md5 $dll
Ok "MD5 $md5"

$bash = @('C:\Program Files\Git\bin\bash.exe', 'C:\Program Files\Git\usr\bin\bash.exe') |
    Where-Object { Test-Path $_ } | Select-Object -First 1
if ($bash) {
    & $bash -c "./scripts/check_size.sh '$($dll -replace '\\', '/')'"
    if ($LASTEXITCODE -ne 0) { Fail 'check_size.sh fallo' }
} else {
    Warn 'sin bash: no puedo ejecutar check_size.sh'
}

if ($DryRun) {
    Step 'DryRun: nada remoto'
    Write-Host "   se copiaria a $LocalDistRepo\$DllName ($size bytes) y $Md5Name ($md5)"
    Write-Host "   release $tag en $DistRepo con --latest=false"
    return
}

if (-not (Test-Path (Join-Path $LocalDistRepo '.git'))) {
    Fail "no existe el clon del repo de distribucion en $LocalDistRepo"
}

Step 'Comprobando el repo de distribucion'
$existing = & git -C $LocalDistRepo tag --list $tag
if ($existing) {
    $head = & git -C $LocalDistRepo rev-parse HEAD
    $tagCommit = & git -C $LocalDistRepo rev-list -n 1 $tag
    if ($head -ne $tagCommit) { Fail "el tag $tag ya existe en el repo de distribucion" }
    Warn "el tag $tag ya existe en HEAD: republico sus adjuntos"
}

Step 'Copiando DLL y MD5'
Copy-Item $dll (Join-Path $LocalDistRepo $DllName) -Force
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText((Join-Path $LocalDistRepo $Md5Name), $md5, $utf8NoBom)
$sidecar = [System.IO.File]::ReadAllBytes((Join-Path $LocalDistRepo $Md5Name))
if ($sidecar.Length -ne 32) { Fail "el .md5 mide $($sidecar.Length) bytes; deben ser 32 sin BOM ni salto" }
Ok "$Md5Name tiene 32 bytes exactos"

$gitattributes = Join-Path $LocalDistRepo '.gitattributes'
$attrText = if (Test-Path $gitattributes) { Get-Content -Raw $gitattributes } else { '' }
if ($attrText -notmatch [regex]::Escape($GitAttributesLine)) {
    $addition = if ($attrText.Length -gt 0 -and -not $attrText.EndsWith("`n")) { "`n$GitAttributesLine`n" } else { "$GitAttributesLine`n" }
    [System.IO.File]::AppendAllText($gitattributes, $addition, $utf8NoBom)
    Ok 'excepcion LFS anadida a .gitattributes'
} else {
    Ok 'excepcion LFS ya presente'
}

Step 'Verificando el blob preparado'
& git -C $LocalDistRepo add -- $DllName $Md5Name .gitattributes
if ($LASTEXITCODE -ne 0) { Fail 'git add fallo' }
$blobSize = (& git -C $LocalDistRepo cat-file -s ":$DllName") -as [int]
if ($blobSize -ne $size) { Fail "el blob preparado mide $blobSize bytes y la DLL ${size}: es un puntero LFS" }
Ok "el blob preparado mide $size bytes"
$attrs = & git -C $LocalDistRepo check-attr filter diff merge text -- $DllName
if ($attrs -match ':(set|unspecified|value)') { Fail "atributos git inesperados:`n$attrs" }
Ok 'atributos filter/diff/merge/text en unset'

Step 'Commit y push (solo nuestros ficheros)'
& git -C $LocalDistRepo commit -m "gw2_lurker v$version" -- $DllName $Md5Name .gitattributes
if ($LASTEXITCODE -ne 0) { Warn 'nada que commitear (mismo contenido)' }
& git -C $LocalDistRepo push origin HEAD:main
if ($LASTEXITCODE -ne 0) { Fail 'git push fallo' }
Ok 'repo de distribucion actualizado'

Step "Release $tag"
$notes = "gw2_lurker.dll v$version`n`nMD5: $md5"
& gh release view $tag --repo $DistRepo 2>$null | Out-Null
if ($LASTEXITCODE -eq 0) {
    & gh release upload $tag (Join-Path $LocalDistRepo $DllName) (Join-Path $LocalDistRepo $Md5Name) --repo $DistRepo --clobber
} else {
    & gh release create $tag (Join-Path $LocalDistRepo $DllName) (Join-Path $LocalDistRepo $Md5Name) `
        --repo $DistRepo --target main --latest=false --title $tag --notes $notes
}
if ($LASTEXITCODE -ne 0) { Fail 'gh release fallo' }
Ok "release $tag publicada"

Step 'Retencion: actual + anterior'
$releases = & gh release list --repo $DistRepo --limit 50 --json tagName,createdAt |
    ConvertFrom-Json | Where-Object { $_.tagName -like 'gw2-lurker-v*' } |
    Sort-Object { [datetime]$_.createdAt } -Descending
$obsolete = $releases | Select-Object -Skip 2
foreach ($release in $obsolete) {
    & gh release delete $release.tagName --repo $DistRepo --yes --cleanup-tag
    if ($LASTEXITCODE -eq 0) { Ok "borrada $($release.tagName)" } else { Warn "no pude borrar $($release.tagName)" }
}
if (-not $obsolete) { Ok 'solo hay dos releases nuestras' }

Step 'Verificando la descarga real'
Test-PublishedDll $dll

Write-Host "`nPublicado gw2_lurker.dll v$version (MD5 $md5)" -ForegroundColor Green
