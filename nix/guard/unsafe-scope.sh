#!/usr/bin/env bash
#
# unsafe-scope (nix:V11): `allow(unsafe_code)` may appear only under
# src/sys/, the OS facade. Everywhere else the crate-level `deny` stands.
#
#   nix/guard/unsafe-scope.sh
set -euo pipefail

if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  echo "unsafe-scope: not inside a git work tree -- nothing was checked. This is a failure, not a pass." >&2
  exit 1
fi

found=$(git grep -n -I 'allow(unsafe_code)' -- '*.rs' ':!src/sys/**' || true)
if [ -n "$found" ]; then
  printf '%s\n' "$found" >&2
  echo "unsafe-scope: allow(unsafe_code) outside src/sys/. Move the unsafe call behind a safe function in src/sys (nix:V11)." >&2
  exit 1
fi
