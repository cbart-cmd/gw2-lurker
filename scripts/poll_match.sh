#!/usr/bin/env bash
# Spike T-02: mide con qué frecuencia cambia realmente /v2/wvw/matches y qué caché declara.
# Uso: ./scripts/poll_match.sh <WORLD_ID> [intervalo_s=10] [duracion_min=30]
set -euo pipefail
WORLD="${1:?uso: poll_match.sh <WORLD_ID> [intervalo] [duracion_min]}"
INT="${2:-10}"; DUR="${3:-30}"
URL="https://api.guildwars2.com/v2/wvw/matches?world=$WORLD"
DIR="spike-logs"; mkdir -p "$DIR/bodies"
TS=$(date -u +%Y%m%dT%H%M%SZ); CSV="$DIR/poll_$TS.csv"
echo "utc,http,bytes,sha256,changed,age,cache_control,etag,last_modified" > "$CSV"
prev=""; end=$(( $(date +%s) + DUR*60 ))
while [ "$(date +%s)" -lt "$end" ]; do
  now=$(date -u +%Y-%m-%dT%H:%M:%SZ); hdr=$(mktemp); body=$(mktemp)
  code=$(curl -sS -A "wvw-watch-dev/0.0 (spike)" -D "$hdr" -o "$body" -w '%{http_code}' "$URL" || echo 000)
  bytes=$(wc -c <"$body"); h=$(sha256sum "$body" | cut -d' ' -f1)
  g() { grep -i "^$1:" "$hdr" | head -1 | cut -d: -f2- | tr -d '\r,' | sed 's/^ //' || true; }
  changed=0; [ "$h" != "$prev" ] && [ "$code" = "200" ] && { changed=1; cp "$body" "$DIR/bodies/match_${now//:/}.json"; prev="$h"; }
  echo "$now,$code,$bytes,$h,$changed,$(g age),$(g cache-control),$(g etag),$(g last-modified)" | tee -a "$CSV"
  rm -f "$hdr" "$body"; sleep "$INT"
done
echo "Listo: $CSV  (los cuerpos distintos están en $DIR/bodies/ y sirven de fixtures)"
