#!/usr/bin/env bash
# Falla si la versión del workspace, las dependencias internas de los crates y Cargo.lock no
# coinciden. Lo llama scripts/validate.ps1 como paso `version` (AGENTS.md §3).
set -uo pipefail
root=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
cd "$root" || exit 1

version=$(sed -n 's/^version = "\([0-9][^"]*\)"$/\1/p' Cargo.toml | head -n1)
if [ -z "$version" ]; then
  echo "ERROR no encuentro la version del workspace en Cargo.toml"
  exit 1
fi

fail=0; checked=0
for f in crates/*/Cargo.toml; do
  while IFS= read -r line; do
    name=${line%% *}
    dep=$(printf '%s\n' "$line" | sed -n 's/.*version = "\([^"]*\)".*/\1/p')
    [ -n "$dep" ] || continue
    checked=$((checked + 1))
    if [ "$dep" != "$version" ]; then
      echo "ERROR $f: $name pide $dep"
      fail=1
    fi
  done < <(grep -E '^gw2lurker-(core|api|addon)[[:space:]]*=' "$f" || true)
done

while IFS='|' read -r name lock_version; do
  [ -n "$name" ] || continue
  checked=$((checked + 1))
  if [ "$lock_version" != "$version" ]; then
    echo "ERROR Cargo.lock: $name esta en $lock_version"
    fail=1
  fi
done < <(awk -F'"' '/^name = "gw2lurker-/ { name=$2; want=1; next }
                     want == 1 && /^version = / { print name "|" $2; want=0 }' Cargo.lock)

if [ "$fail" -ne 0 ]; then
  echo "Versiones: desajuste con $version (scripts/bump-version.ps1 -Version $version)"
  exit 1
fi
echo "Versiones: $version en $checked sitios"
