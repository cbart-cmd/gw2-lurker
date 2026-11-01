<#
.SYNOPSIS
  Regenera la parte automática de modelo.md: mapa de ficheros, items, tests y APIs que usa cada uno.
.DESCRIPTION
  Lee los `//!` y `///` del código. Lo que no tiene `///` se describe con
  scripts/modelo-descripciones.txt (una línea `ruta|Tipo::item => descripción` por item).
  Conserva lo que haya en modelo.md por encima del marcador <!-- GENERADO --> (índice escrito a mano).
  Uso: pwsh -File scripts/gen_modelo.ps1 [-ListMissing]
  -ListMissing: no escribe nada; lista las claves de items sin descripción (para completar el .txt).
#>
param([switch]$ListMissing)
$ErrorActionPreference = 'Stop'
$root = Split-Path $PSScriptRoot -Parent
Set-Location $root
$out = Join-Path $root 'modelo.md'
$marker = '<!-- GENERADO: no editar debajo; se rehace con scripts/gen_modelo.ps1 -->'

# Descripciones a mano de los items sin `///`.
$extra = @{}
$extraPath = Join-Path $PSScriptRoot 'modelo-descripciones.txt'
if (Test-Path $extraPath) {
    foreach ($l in Get-Content $extraPath -Encoding utf8) {
        if ($l -match '^\s*(#|$)') { continue }
        $k, $v = $l -split ' => ', 2
        if ($v) { $extra[$k.Trim()] = $v.Trim() }
    }
}
$script:missing = [System.Collections.Generic.List[string]]::new()
$script:usedKeys = [System.Collections.Generic.HashSet[string]]::new()

function Clip([string]$s, [int]$n = 260) {
    $s = ($s -replace '\s+', ' ').Trim()
    if ($s.Length -gt $n) { $s.Substring(0, $n).TrimEnd() + '…' } else { $s }
}

# Primer párrafo de un bloque de comentarios ya sin prefijo.
function FirstParagraph([string[]]$lines, [int]$n = 260) {
    $acc = @()
    foreach ($l in $lines) {
        if ($l.Trim() -eq '' -and $acc.Count) { break }
        if ($l.Trim() -ne '') { $acc += $l.Trim() }
    }
    Clip ($acc -join ' ') $n
}

function Describe-Rust([string]$path) {
    $lines = Get-Content $path -Encoding utf8
    $sb = [System.Text.StringBuilder]::new()
    $loc = $lines.Count

    # Documentación del módulo.
    $modDoc = @()
    foreach ($l in $lines) {
        if ($l -match '^\s*//!\s?(.*)$') { $modDoc += $Matches[1] }
        elseif ($modDoc.Count -or $l.Trim() -eq '' -or $l -match '^\s*#!\[') { if ($modDoc.Count -and $l -notmatch '^\s*//!') { break } }
        else { break }
    }

    # Separar el módulo de tests (siempre al final por convención).
    $testStart = $loc
    for ($i = 0; $i -lt $loc - 1; $i++) {
        if ($lines[$i] -match '^\s*#\[cfg\(test\)\]' -and $lines[$i + 1] -match '^\s*(pub\s+)?mod\s+\w+\s*\{') { $testStart = $i; break }
    }
    $prod = if ($testStart -gt 0) { $lines[0..($testStart - 1)] } else { @() }
    $prodText = $prod -join "`n"

    [void]$sb.AppendLine("### ``$($path -replace '\\','/')`` ($loc líneas)")
    [void]$sb.AppendLine()
    if ($modDoc.Count) { [void]$sb.AppendLine((FirstParagraph $modDoc 600)); [void]$sb.AppendLine() }

    # Items de producción con su doc.
    $items = @()
    $doc = @()
    $inExtern = $false
    $externLib = ''
    $externFns = @()
    $relPath = $path -replace '\\', '/'
    $owner = ''
    $isTest = $false
    for ($i = 0; $i -lt $prod.Count; $i++) {
        $l = $prod[$i]
        if ($l -match '^\s*#\[link\(name\s*=\s*"([^"]+)"') { $externLib = $Matches[1] }
        if ($l -match '^\s*(unsafe\s+)?extern\s+"[^"]+"\s*\{') { $inExtern = $true; continue }
        if ($inExtern) {
            if ($l -match '^\s*\}') { $inExtern = $false; continue }
            if ($l -match '^\s*(?:pub\s+)?fn\s+(\w+)') { $externFns += "$($Matches[1]) ($externLib)" }
            continue
        }
        if ($l -match '^\s*///\s?(.*)$') { $doc += $Matches[1]; continue }
        if ($l -match '^\s*#\[test\]') { $isTest = $true; continue }
        if ($l -match '^\s*#\[') { continue }
        $indent = ($l -replace '^(\s*).*', '$1').Length
        $t = $l.Trim()
        if ($indent -le 4 -and $t -cmatch '^(pub(?:\([^)]*\))?\s+)?((?:const\s+|async\s+|unsafe\s+|extern\s+"[^"]*"\s+)*)(fn|struct|enum|trait|mod|type|static|const|impl|macro_rules!)\s*(.*)$') {
            $vis = if ($Matches[1]) { 'pub ' } else { '' }
            $kind = $Matches[3]; $rest = $Matches[4]
            $name = switch ($kind) {
                'impl' { (($rest -replace '\s*\{.*$', '') -replace '\s*where.*$', '') }
                default { ($rest -replace '^(\w+).*$', '$1') }
            }
            if ($kind -eq 'fn') {
                $sig = $rest -replace '\s*\{\s*$', ''
                if ($sig -notmatch '\)') { $sig = ($sig -replace '\($', '') + '(…)' }
                $name = Clip $sig 110
            }
            if ($indent -eq 0) {
                $owner = if ($kind -in 'impl', 'trait') {
                    # `impl<T> Trait for Type` → `Type`; `impl Type` → `Type`.
                    $r = ($rest -replace '^<[^>]*>\s*', '') -replace '\s*(\{|where).*$', ''
                    if ($r -match '\bfor\s+(.+)$') { $r = $Matches[1] }
                    ($r -replace '<.*$', '').Trim()
                } else { '' }
            }
            $short = if ($kind -eq 'impl') { '' } else { $rest -replace '^(\w+).*$', '$1' }
            $key = if ($indent -gt 0 -and $owner) { "$relPath|$owner::$short" } else { "$relPath|$short" }
            $skip = ($kind -in 'const', 'static' -and $indent -gt 0 -and -not $doc.Count) -or ($kind -eq 'mod' -and $t -match ';$' -and -not $doc.Count -and $false)
            if (-not $skip -and -not $isTest -and -not ($kind -eq 'fn' -and $name -match '^fmt\(' )) {
                $d = if ($doc.Count) { FirstParagraph $doc 200 } else { '' }
                if (-not $d -and $kind -notin 'impl', 'mod') {
                    if ($extra.ContainsKey($key)) { $d = $extra[$key]; [void]$script:usedKeys.Add($key) }
                    else { $script:missing.Add("$key`t$kind $name") }
                }
                $prefix = if ($indent -gt 0) { '  - ' } else { '- ' }
                $items += "$prefix``$vis$kind $name``" + $(if ($d) { " — $d" } else { '' })
            }
        }
        if ($t -ne '') { $doc = @(); $isTest = $false }
    }
    if ($items.Count) {
        [void]$sb.AppendLine('**Contenido:**')
        [void]$sb.AppendLine()
        $items | ForEach-Object { [void]$sb.AppendLine($_) }
        [void]$sb.AppendLine()
    }

    # Dependencias internas.
    $deps = [ordered]@{}
    foreach ($m in [regex]::Matches($prodText, '\b(crate|super|gw2lurker_core|gw2lurker_api)::(\w+)')) {
        $k = "$($m.Groups[1].Value)::$($m.Groups[2].Value)"; $deps[$k] = 1
    }
    if ($deps.Count) { [void]$sb.AppendLine("**Usa (interno):** " + (($deps.Keys | Sort-Object | ForEach-Object { "``$_``" }) -join ', ')); [void]$sb.AppendLine() }

    # APIs externas.
    $apis = [ordered]@{}
    foreach ($m in [regex]::Matches($prodText, '\bnexus::(?:[a-z_]+::)*[A-Za-z_]+')) { $apis[$m.Value] = 1 }
    foreach ($m in [regex]::Matches($prodText, '"(/v2/[^"\s]*)')) { $apis["GW2 API $($m.Groups[1].Value)"] = 1 }
    foreach ($m in [regex]::Matches($prodText, 'https?://[^"\s)>]+')) { $apis[$m.Value] = 1 }
    foreach ($m in [regex]::Matches($prodText, '\b(?:std::)?fs::(\w+)')) { $apis["disco: fs::$($m.Groups[1].Value)"] = 1 }
    foreach ($m in [regex]::Matches($prodText, '\b(?:std::)?thread::(\w+)')) { $apis["hilos: thread::$($m.Groups[1].Value)"] = 1 }
    foreach ($m in [regex]::Matches($prodText, '\bimgui::(\w+)')) { $apis["imgui::$($m.Groups[1].Value)"] = 1 }
    foreach ($m in [regex]::Matches($prodText, '\bserde_json::(\w+)')) { $apis["serde_json::$($m.Groups[1].Value)"] = 1 }
    foreach ($f in $externFns) { $apis["Win32: $f"] = 1 }
    if ($prodText -match '\bui\s*:\s*&\s*(?:nexus::imgui::)?Ui\b') { $apis['imgui (dibuja con &Ui)'] = 1 }
    if ($apis.Count) { [void]$sb.AppendLine("**APIs externas:** " + (($apis.Keys | ForEach-Object { "``$_``" }) -join ', ')); [void]$sb.AppendLine() }

    # Tests.
    $tests = @()
    for ($i = $testStart; $i -lt $loc - 1; $i++) {
        if ($lines[$i] -match '^\s*#\[test\]') {
            for ($j = $i + 1; $j -lt [Math]::Min($i + 4, $loc); $j++) {
                if ($lines[$j] -match '^\s*fn\s+(\w+)') { $tests += $Matches[1]; break }
            }
        }
    }
    if ($path -match '[\\/]tests?[\\/]|tests\.rs$') {
        for ($i = 0; $i -lt $loc - 1; $i++) {
            if ($lines[$i] -match '^\s*#\[test\]' -and $lines[$i + 1] -match '^\s*fn\s+(\w+)' -and $tests -notcontains $Matches[1]) { $tests += $Matches[1] }
        }
    }
    if ($tests.Count) { [void]$sb.AppendLine("**Tests ($($tests.Count)):** " + (($tests | ForEach-Object { "``$_``" }) -join ', ')); [void]$sb.AppendLine() }
    $sb.ToString()
}

function Describe-Other([string]$path) {
    $p = $path -replace '\\', '/'
    $lines = Get-Content $path -Encoding utf8 -TotalCount 40 -ErrorAction SilentlyContinue
    $d = ''
    if ($p -match '\.md$') {
        $h = ($lines | Where-Object { $_ -match '^#\s' } | Select-Object -First 1) -replace '^#\s*', ''
        $body = $lines | Where-Object { $_ -notmatch '^#' -and $_ -notmatch '^\s*[-|>`]' } 
        $d = (Clip $h 100) + ' — ' + (FirstParagraph @($body | Where-Object { $_.Trim() -ne '' } | Select-Object -First 3) 220)
    } elseif ($p -match '\.(ps1|sh|bat)$') {
        $syn = $lines | Select-String -Pattern '^\s*\.SYNOPSIS' -Context 0, 3
        if ($syn) {
            $acc = @(); foreach ($x in $syn.Context.PostContext) { if ($x -match '^\s*\.') { break }; $acc += $x }
            $d = FirstParagraph $acc 220
        }
        else {
            $c = $lines | Where-Object { $_ -match '^\s*(#|::|rem\s)' -and $_ -notmatch '^#!' } | ForEach-Object { $_ -replace '^\s*(#|::|rem)\s?', '' }
            $d = FirstParagraph $c 220
        }
    } elseif ($p -match '\.(toml|yml|yaml|txt)$') {
        $c = $lines | Where-Object { $_ -match '^\s*#' } | ForEach-Object { $_ -replace '^\s*#\s?', '' }
        $d = FirstParagraph $c 220
    }
    "- ``$p``" + $(if ($d -and $d -ne ' — ') { " — $d" } else { '' })
}

$sections = [System.Text.StringBuilder]::new()
[void]$sections.AppendLine($marker)
[void]$sections.AppendLine()
[void]$sections.AppendLine("Generado el $(Get-Date -Format 'yyyy-MM-dd HH:mm') desde el commit ``$(git rev-parse --short HEAD)``.")
[void]$sections.AppendLine()

$files = git ls-files
foreach ($crate in 'core', 'api', 'addon') {
    [void]$sections.AppendLine("## Crate ``$crate``")
    [void]$sections.AppendLine()
    $files | Where-Object { $_ -like "crates/$crate/*" -and $_ -notlike '*.rs' -and $_ -notlike '*/fixtures/*' } | ForEach-Object { [void]$sections.AppendLine((Describe-Other $_)) }
    $fx = @($files | Where-Object { $_ -like "crates/$crate/*/fixtures/*" })
    if ($fx.Count) { [void]$sections.AppendLine("- Fixtures de tests ($($fx.Count)): " + (($fx | ForEach-Object { '`' + (Split-Path $_ -Leaf) + '`' }) -join ', ')) }
    [void]$sections.AppendLine()
    $files | Where-Object { $_ -like "crates/$crate/*.rs" } | Sort-Object | ForEach-Object { [void]$sections.AppendLine((Describe-Rust $_)) }
}

[void]$sections.AppendLine('## Scripts, specs, docs y configuración')
[void]$sections.AppendLine()
$files | Where-Object { $_ -notlike 'crates/*' -and $_ -notlike 'docs/historial/*' -and $_ -notlike 'data/*' } | Sort-Object | ForEach-Object { [void]$sections.AppendLine((Describe-Other $_)) }

if ($ListMissing) {
    $script:missing
    "sin descripción: $($script:missing.Count)"
    $extra.Keys | Where-Object { -not $script:usedKeys.Contains($_) } | ForEach-Object { "clave obsoleta: $_" }
    return
}

$head = ''
if (Test-Path $out) {
    $all = Get-Content $out -Raw -Encoding utf8
    $at = $all.IndexOf($marker)
    if ($at -ge 0) { $head = $all.Substring(0, $at) } else { $head = $all.TrimEnd() + "`n`n" }
}
[System.IO.File]::WriteAllText($out, $head + $sections.ToString(), [System.Text.UTF8Encoding]::new($false))
"modelo.md: $([Math]::Round((Get-Item $out).Length / 1KB)) KB · items sin descripción: $($script:missing.Count)"
$stale = @($extra.Keys | Where-Object { -not $script:usedKeys.Contains($_) })
if ($stale.Count) { "claves obsoletas en modelo-descripciones.txt: $($stale -join ', ')" }
