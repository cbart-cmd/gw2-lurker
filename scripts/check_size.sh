#!/usr/bin/env bash
# Uso: check_size.sh ruta/a/addon.dll  (LIMIT_BYTES por defecto 2.5 MB)
set -euo pipefail
dll="${1:?ruta a la DLL}"; limit="${LIMIT_BYTES:-2621440}"; target="${TARGET_BYTES:-1572864}"
size=$(stat -c%s "$dll")
echo "Tamaño: $size bytes (objetivo $target, límite $limit)"
[ "$size" -le "$limit" ] || { echo "ERROR: supera el límite"; exit 1; }
[ "$size" -le "$target" ] || echo "AVISO: supera el objetivo"
