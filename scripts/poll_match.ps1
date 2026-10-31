# Spike T-02 (Windows / PowerShell): mide la frecuencia real de cambio de /v2/wvw/matches.
# Uso: .\scripts\poll_match.ps1 -World 1234 [-IntervalSec 10] [-DurationMin 30]
param([Parameter(Mandatory)][int]$World, [int]$IntervalSec = 10, [int]$DurationMin = 30)
$url = "https://api.guildwars2.com/v2/wvw/matches?world=$World"
$dir = "spike-logs"; New-Item -ItemType Directory -Force -Path "$dir/bodies" | Out-Null
$ts = (Get-Date).ToUniversalTime().ToString("yyyyMMddTHHmmssZ"); $csv = "$dir/poll_$ts.csv"
"utc,http,bytes,sha256,changed,age,cache_control,etag,last_modified" | Out-File $csv -Encoding utf8
$prev = ""; $end = (Get-Date).AddMinutes($DurationMin)
while ((Get-Date) -lt $end) {
  $now = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
  $code = 0; $bytes = 0; $hash = ""; $changed = 0; $age = ""; $cc = ""; $etag = ""; $lm = ""
  try {
    $r = Invoke-WebRequest -UseBasicParsing -UserAgent "wvw-watch-dev/0.0 (spike)" -Uri $url
    $code = [int]$r.StatusCode; $bytes = $r.RawContentLength
    $sha = [System.Security.Cryptography.SHA256]::Create()
    $hash = ([BitConverter]::ToString($sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($r.Content))) -replace '-', '').ToLower()
    $age = "$($r.Headers['Age'])"; $cc = ("$($r.Headers['Cache-Control'])" -replace ',', ';')
    $etag = "$($r.Headers['ETag'])"; $lm = ("$($r.Headers['Last-Modified'])" -replace ',', '')
    if ($hash -ne $prev) {
      $changed = 1; $prev = $hash
      Set-Content -Path "$dir/bodies/match_$($now -replace ':','').json" -Value $r.Content -Encoding utf8
    }
  } catch {
    if ($_.Exception.Response) { $code = [int]$_.Exception.Response.StatusCode } else { $code = 0 }
  }
  $line = "$now,$code,$bytes,$hash,$changed,$age,$cc,$etag,$lm"
  $line | Tee-Object -FilePath $csv -Append
  Start-Sleep -Seconds $IntervalSec
}
"Listo: $csv  (los cuerpos distintos están en $dir/bodies/ y sirven de fixtures)"
