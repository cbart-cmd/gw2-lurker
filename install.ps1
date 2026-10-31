<#
.SYNOPSIS
  Compila (opcional) e instala el addon gw2_lurker.dll en la carpeta addons de Guild Wars 2.

.DESCRIPTION
  Copia gw2_lurker.dll a <juego>\addons, que es donde Nexus busca los addons. La carpeta de
  datos del addon (<juego>\addons\gw2_lurker) la crea el propio addon con su settings.json,
  no hay que tocarla.

  La DLL se verifica antes y despues de copiar: tamano, cabecera MZ y MD5. Un HTTP 200 o un
  fichero de 130 bytes (puntero Git LFS) no son una DLL.

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

Write-Host "Origen : $Source ($($info.Length) bytes, MD5 $md5)"
Write-Host "Destino: $target"

if (-not (Test-Path $Dest)) {
    Write-Host "ERROR: no existe la carpeta de destino: $Dest. Usa -Dest para indicar la carpeta addons del juego." -ForegroundColor Red
    exit 1
}
if ($DryRun) {
    Write-Host 'DryRun: no se copia nada.' -ForegroundColor Yellow
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

Write-Host "Copiado: $($copied.FullName) ($($copied.Length) bytes, MD5 $copiedMd5, $($copied.LastWriteTime))" -ForegroundColor Green
Write-Host "Ajustes del addon en: $(Join-Path $Dest 'gw2_lurker\settings.json')"
Write-Host "Abre GW2 y entra en Mundo contra Mundo. El log esta en $(Join-Path $Dest 'Nexus\Nexus.log')"
