#!/usr/bin/env bash
# Uso: check_size.sh ruta/a/addon.dll  (LIMIT_BYTES por defecto 2.5 MB)
# Objetivo 2.2 MB desde ADR-013 (2026-10-31): el addon no cabía en 1.5 MB con las vistas.
set -euo pipefail
dll="${1:?ruta a la DLL}"; limit="${LIMIT_BYTES:-2621440}"; target="${TARGET_BYTES:-2306867}"
size=$(stat -c%s "$dll")
echo "Tamaño: $size bytes (objetivo $target, límite $limit)"
[ "$size" -le "$limit" ] || { echo "ERROR: supera el límite"; exit 1; }
[ "$size" -le "$target" ] || echo "AVISO: supera el objetivo"
