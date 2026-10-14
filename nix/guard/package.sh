#!/usr/bin/env bash
#
# package (nix:V24): the published .crate must build from its own contents
# and carry every path Cargo.toml's must-package names.
#
#   nix/guard/package.sh
set -euo pipefail

listed=$(cargo package --locked --allow-dirty --list -p congestimancer)
status=0
while IFS= read -r path; do
  [ -n "$path" ] || continue
  if ! grep -qxF "$path" <<<"$listed"; then
    echo "package: $path is in must-package but not in the .crate; check exclude in Cargo.toml (nix:V24)." >&2
    status=1
  fi
done < <(awk '
  /^\[package\.metadata\.congestimancer\]/ { on = 1; next }
  /^\[/ { on = 0 }
  on && /"/ { gsub(/[",[:space:]]/, ""); if ($0 != "" && $0 !~ /=/) print }
' Cargo.toml)
[ "$status" -eq 0 ] || exit 1

# Build and test from the unpacked tarball, not the repository.
cargo package --locked --allow-dirty -p congestimancer --quiet
