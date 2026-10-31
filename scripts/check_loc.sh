#!/usr/bin/env bash
# Falla si algún .rs supera MAX líneas; avisa a partir de WARN.
set -euo pipefail
MAX="${MAX:-800}"; WARN="${WARN:-500}"; fail=0
while IFS= read -r -d '' f; do
  n=$(wc -l < "$f")
  if   [ "$n" -gt "$MAX"  ]; then echo "ERROR $n líneas (> $MAX): $f"; fail=1
  elif [ "$n" -gt "$WARN" ]; then echo "AVISO $n líneas (> $WARN): $f"; fi
done < <(find . -name '*.rs' -not -path './target/*' -print0)
exit $fail
