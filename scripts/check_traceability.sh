#!/usr/bin/env bash
# Trazabilidad requisito -> prueba (T-03b).
#
# - ERROR si un test usa un id de requisito que no existe en la spec (nombre inventado
#   o errata): eso siempre es un fallo.
# - INFORME de los requisitos que no tienen ni test ni fila en manual-tests.md.
#   No falla por ellos: el proyecto está a medias y la lista sirve de hoja de ruta.
# - Con --strict sí falla si falta cobertura, para usarlo antes de publicar.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

strict=0
[ "${1:-}" = "--strict" ] && strict=1

spec=specs/01-requirements.md
manual=specs/manual-tests.md
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT

# Ids definidos en la spec: **RF-03b (M)** o | RNF-02 | ...
# Se descartan los comodines de la matriz de trazabilidad (RF-03*).
grep -oE '(RNF|RF)-[0-9]+[a-z]?\*?' "$spec" | grep -v '\*$' | tr '[:upper:]' '[:lower:]' | sort -u > "$tmp/defined"

# Ids usados en nombres de test: fn rf03b_..., fn rnf02_...
grep -rhoE 'fn (rf|rnf)[0-9]+[a-z]?_' crates --include='*.rs' \
  | sed -E 's/^fn //' | sed -E 's/_$//' \
  | sed -E 's/^rf([0-9])/rf-\1/; s/^rnf([0-9])/rnf-\1/' \
  | tr '[:upper:]' '[:lower:]' | sort -u > "$tmp/tested"

# Ids citados en la lista de pruebas manuales
if [ -f "$manual" ]; then
  grep -oE '(RNF|RF)-[0-9]+[a-z]?' "$manual" | tr '[:upper:]' '[:lower:]' | sort -u > "$tmp/manual"
else
  : > "$tmp/manual"
fi

cat "$tmp/tested" "$tmp/manual" | sort -u > "$tmp/covered"

unknown=$(comm -13 "$tmp/defined" "$tmp/tested" || true)
missing=$(comm -23 "$tmp/defined" "$tmp/covered" || true)

total=$(wc -l < "$tmp/defined" | tr -d ' ')
with_test=$(comm -12 "$tmp/defined" "$tmp/tested" | wc -l | tr -d ' ')
with_manual=$(comm -12 "$tmp/defined" "$tmp/manual" | wc -l | tr -d ' ')
gaps=$(printf '%s' "$missing" | grep -c . || true)

echo "Trazabilidad: $total requisitos · $with_test con test · $with_manual con prueba manual · $gaps sin cubrir"

if [ -n "$unknown" ]; then
  echo "ERROR: tests que citan requisitos inexistentes:" >&2
  printf '  %s\n' $unknown >&2
  exit 1
fi

if [ -n "$missing" ]; then
  echo "Sin cubrir todavía:"
  printf '  %s\n' $missing
fi

if [ "$strict" = "1" ] && [ -n "$missing" ]; then
  echo "ERROR: --strict y hay requisitos sin cubrir" >&2
  exit 1
fi

exit 0
