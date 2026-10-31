<#
.SYNOPSIS
  Compila (opcional) e instala el addon gw2_lurker.dll en la carpeta addons de Guild Wars 2.

.DESCRIPTION
  Copia gw2_lurker.dll a <juego>\addons, que es donde Nexus busca los addons. La carpeta de
  datos del addon (<juego>\addons\gw2_lurker) la crea el propio addon con su settings.json,
  no hay que tocarla.

  La DLL se verifica antes y despues de copiar: tamano, cabecera MZ y MD5. Un HTTP 200 o un
  fichero de 130 bytes (puntero Git LFS) no son una DLL.

  Tambien dice **que version es**: se lee del `Cargo.toml` del repo y se comprueba que esa
  cadena este dentro de la DLL (`env!("CARGO_PKG_VERSION")` la mete en el binario). Si no esta,
  la DLL se compilo antes de cambiar la version y el script avisa.

.EXAMPLE
  .\install.ps1                 # copia la DLL ya compilada
  .\install.ps1 -Build          # ejecuta tests, compila en release y copia
  .\install.ps1 -DryRun         # dice que haria, sin copiar nada
  .\install.ps1 -Dest "D:\Games\Guild Wars 2\addons"
#>
param(
    [string]$Dest = '',
    [switch]$Build,
    [switch]$DryRun
)

$ErrorActionPreference = 'Stop'
$ProgressPreference = 'SilentlyContinue'

$Name = 'gw2_lurker.dll'
$Source = Join-Path $PSScriptRoot 'target\release\gw2_lurker.dll'

function Get-GameAddonsDir {
    # El registro es la fuente buena; la ruta conocida queda como respaldo.
    $keys = @(
        'HKLM:\SOFTWARE\ArenaNet\Guild Wars 2',
        'HKLM:\SOFTWARE\WOW6432Node\ArenaNet\Guild Wars 2'
    )
    foreach ($key in $keys) {
        if (-not (Test-Path $key)) { continue }
        $entry = Get-ItemProperty $key -ErrorAction SilentlyContinue
        foreach ($candidate in @($entry.Install64Path, $entry.InstallPath)) {
            if ($candidate -and (Test-Path (Join-Path $candidate 'Gw2-64.exe'))) {
                return (Join-Path $candidate 'addons')
            }
        }
    }
    return 'C:\Program Files\Guild Wars 2\addons'
}

function Get-Md5([string]$path) {
    # .NET en vez de Get-FileHash: al lanzar el script desde cmd con 'powershell' (5.1) el
    # autoload del modulo Utility falla en esta maquina y Get-FileHash no se reconoce.
    $md5 = [System.Security.Cryptography.MD5]::Create()
    try {
        $stream = [System.IO.File]::OpenRead($path)
        try {
            $bytes = $md5.ComputeHash($stream)
        } finally {
            $stream.Dispose()
        }
    } finally {
        $md5.Dispose()
    }
    return ([System.BitConverter]::ToString($bytes) -replace '-', '').ToLowerInvariant()
}

function Test-Dll([string]$path) {
    if (-not (Test-Path $path)) { throw "No existe $path" }
    $info = Get-Item $path
    if ($info.Length -lt 65536) {
        throw "$path mide $($info.Length) bytes: parece un puntero de Git LFS, no una DLL."
    }
    $head = [System.IO.File]::ReadAllBytes($path)[0..1]
    if ($head[0] -ne 0x4D -or $head[1] -ne 0x5A) { throw "$path no empieza por MZ." }
    return $info
}

function Get-ManifestVersion {
    # El numero que manda es el del manifiesto del repo, no el nombre del fichero.
    $manifest = Join-Path $PSScriptRoot 'Cargo.toml'
    if (-not (Test-Path $manifest)) { return '' }
    $match = [regex]::Match((Get-Content $manifest -Raw), '(?m)^version\s*=\s*"([^"]+)"')
    if ($match.Success) { return $match.Groups[1].Value }
    return ''
}

function Test-VersionInDll([string]$path, [string]$version) {
    # `env!("CARGO_PKG_VERSION")` mete la version como cadena en el binario, asi que buscarla
    # dentro de la DLL dice si esa DLL salio de esa version. `GetEncoding(28591)` es latin-1 y
    # existe tambien en PowerShell 5.1, que es el que lanza install.bat.
    if (-not $version) { return $false }
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $text = [System.Text.Encoding]::GetEncoding(28591).GetString($bytes)
    return $text.Contains($version)
}

if (-not $Dest) { $Dest = Get-GameAddonsDir }

if ($Build) {
    $env:Path += ";$env:USERPROFILE\.cargo\bin"
    Push-Location $PSScriptRoot
    try {
        Write-Host '== cargo test ==' -ForegroundColor Cyan
        cargo test --workspace
        if ($LASTEXITCODE -ne 0) { throw 'Los tests han fallado: no se instala nada.' }
        Write-Host '== cargo build --release ==' -ForegroundColor Cyan
        cargo build --release -p gw2lurker-addon
        if ($LASTEXITCODE -ne 0) { throw 'cargo build ha fallado.' }
    } finally {
        Pop-Location
    }
}

$info = Test-Dll $Source
$md5 = Get-Md5 $Source
$target = Join-Path $Dest $Name
$version = Get-ManifestVersion
$versionNote = if ($version) { ", version $version" } else { '' }

Write-Host "Origen : $Source ($($info.Length) bytes, MD5 $md5$versionNote)"
Write-Host "Destino: $target"

if (-not $version) {
    Write-Host 'AVISO: no he podido leer la version de Cargo.toml.' -ForegroundColor Yellow
} elseif (-not (Test-VersionInDll $Source $version)) {
    Write-Host "AVISO: la DLL no contiene la version $version, asi que se compilo antes de subirla. Usa -Build para recompilar." -ForegroundColor Yellow
}

# Que habia instalado: la version no se puede leer de una DLL ajena (no lleva recurso de
# version), pero si el MD5 y la fecha, que es lo que dice si cambia algo.
if (Test-Path $target) {
    $previous = Get-Item $target
    $previousMd5 = Get-Md5 $target
    if ($previousMd5 -eq $md5) {
        Write-Host "Ya estaba: misma DLL instalada desde $($previous.LastWriteTime)." -ForegroundColor Yellow
    } else {
        Write-Host "Antes  : $($previous.Length) bytes, MD5 $previousMd5, $($previous.LastWriteTime)"
    }
}

if (-not (Test-Path $Dest)) {
    Write-Host "ERROR: no existe la carpeta de destino: $Dest. Usa -Dest para indicar la carpeta addons del juego." -ForegroundColor Red
    exit 1
}
if ($DryRun) {
    Write-Host 'DryRun: no se copia nada.' -ForegroundColor Yellow
    if ($version) { Write-Host "Version: $version (leida de Cargo.toml)" }
    Write-Host "Ajustes (Documentos): $([Environment]::GetFolderPath('MyDocuments'))\Guild Wars 2\addons\gw2_lurker\settings.json"
    return
}
if (Get-Process -Name 'Gw2-64' -ErrorAction SilentlyContinue) {
    Write-Host 'ERROR: Guild Wars 2 esta abierto y mantiene la DLL bloqueada. Cierra el juego y vuelve a ejecutar el script.' -ForegroundColor Red
    exit 1
}

function Copy-Addon {
    Copy-Item -Path $Source -Destination $target -Force
}

try {
    Copy-Addon
} catch [System.UnauthorizedAccessException] {
    # Program Files exige permisos de administrador: se relanza el script elevado.
    $principal = [Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()
    if ($principal.IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) { throw }
    Write-Host "Se necesitan permisos de administrador para escribir en $Dest. Relanzando..." -ForegroundColor Yellow
    $argList = "-NoProfile -ExecutionPolicy Bypass -File `"$PSCommandPath`" -Dest `"$Dest`""
    Start-Process -FilePath (Get-Process -Id $PID).Path -ArgumentList $argList -Verb RunAs -Wait
    return
}

# Verificar lo copiado, no lo que creiamos haber copiado.
$copied = Test-Dll $target
$copiedMd5 = Get-Md5 $target
if ($copiedMd5 -ne $md5) {
    throw "El fichero copiado no coincide (MD5 $copiedMd5 frente a $md5)."
}

Write-Host "Copiado: $($copied.FullName) ($($copied.Length) bytes, MD5 $copiedMd5$versionNote, $($copied.LastWriteTime))" -ForegroundColor Green
if ($version -and -not (Test-VersionInDll $target $version)) {
    Write-Host "AVISO: lo instalado no dice la version $version. Algo no cuadra: revisa el build." -ForegroundColor Yellow
}
$documents = [Environment]::GetFolderPath('MyDocuments')
Write-Host "Ajustes del addon en: $documents\Guild Wars 2\addons\gw2_lurker\settings.json"
Write-Host "Abre GW2 y entra en Mundo contra Mundo. El log esta en $(Join-Path $Dest 'Nexus\Nexus.log')"
