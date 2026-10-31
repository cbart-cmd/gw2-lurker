#!/usr/bin/env bash
# Guarda respuestas reales de la API como fixtures de test.
# Uso: ./scripts/capture_fixtures.sh <WORLD_ID> ["Nombre de guild a buscar"]
set -euo pipefail
WORLD="${1:?uso: capture_fixtures.sh <WORLD_ID> [guild]}"; GUILD="${2:-}"
OUT="crates/api/tests/fixtures"; API="https://api.guildwars2.com/v2"
UA="wvw-watch-dev/0.0 (fixture capture)"; TS=$(date -u +%Y%m%dT%H%M%SZ)
mkdir -p "$OUT"
get() { curl -fsS -A "$UA" "$1" -o "$2" && echo "OK  $2 ($(wc -c <"$2") bytes)"; }
get "$API/worlds?ids=all&lang=es"                   "$OUT/worlds.json"
get "$API/wvw/objectives?ids=all&lang=es"           "$OUT/objectives_es.json"
get "$API/wvw/matches?world=$WORLD"                 "$OUT/match_${TS}.json"
get "$API/wvw/matches/overview?world=$WORLD"        "$OUT/match_overview_${TS}.json"
# Rectángulos de los mapas de WvW (ids tomados del propio match; ADR-011)
IDS=$(python3 -c 'import json,sys;print(",".join(str(m["id"]) for m in json.load(open(sys.argv[1]))["maps"]))' "$OUT/match_${TS}.json")
get "$API/maps?ids=$IDS&lang=es"                    "$OUT/maps_wvw.json"
if [ -n "$GUILD" ]; then
  enc=$(python3 -c 'import sys,urllib.parse;print(urllib.parse.quote(sys.argv[1]))' "$GUILD")
  get "$API/guild/search?name=$enc"                 "$OUT/guild_search_${TS}.json"
fi
echo "Consejo: repite en 5-10 min para tener dos snapshots del mismo match (necesario para tests de diff)."
