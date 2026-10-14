#!/usr/bin/env bash
#
# Module size (nix:V16): every tracked Rust file under src/ and dev/src/ has
# at most 500 code lines and at most 1000 lines in total. Code lines are the
# non-blank lines that are not `//` comments. Tests live in sibling tests.rs
# files (nix:V6), so a module's tests never count against it.
#
#   nix/guard/module-size.sh
set -euo pipefail

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "module-size: not inside a git work tree -- the file list comes from git, so nothing was checked. This is a failure, not a pass." >&2
  exit 1
fi

status=0
while IFS= read -r file; do
  [ -n "$file" ] || continue
  total=$(wc -l <"$file" | tr -d ' ')
  code=$(grep -cvE '^[[:space:]]*($|//)' "$file" || true)
  if [ "$code" -gt 500 ]; then
    echo "module-size: $file has $code code lines (limit 500). Split the module." >&2
    status=1
  fi
  if [ "$total" -gt 1000 ]; then
    echo "module-size: $file has $total lines (limit 1000). Split the module." >&2
    status=1
  fi
done < <(git ls-files 'src/*.rs' 'src/**/*.rs' 'dev/src/*.rs' 'dev/src/**/*.rs')
exit "$status"
