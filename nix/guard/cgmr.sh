#!/usr/bin/env bash
#
# cgmr shim (.:V1, nix:T12): run this repository's own cgmr, built from the
# working tree, so the gate dogfoods the tool it ships.
#
#   nix/guard/cgmr.sh [SWITCHES...] -- CMD...   wait for the gates, run CMD
#   nix/guard/cgmr.sh VERB [SWITCHES...]        any other verb (advise, ...)
#
# CGMR_DOGFOOD=0|false|no turns the gating off: CMD runs at once.
# A cgmr that does not build never locks the gate: the shim says so and
# runs CMD ungated (a verb with no CMD exits 69 instead, as cgmr does for
# "cannot work here"). Profiles come from the repository's .cgmr.
set -euo pipefail

here="${BASH_SOURCE[0]%/*}"
root=$(cd "$here/../.." && pwd)

switches=()
cmd=()
while [ "$#" -gt 0 ]; do
  if [ "$1" = -- ]; then
    shift
    cmd=("$@")
    break
  fi
  switches+=("$1")
  shift
done

# Run CMD without cgmr because of $1, or exit 69 when there is no CMD.
ungated() {
  if [ "${#cmd[@]}" -eq 0 ]; then
    echo "cgmr.sh: $1, and there is no command to run without it." >&2
    exit 69
  fi
  exec "${cmd[@]}"
}

case "${CGMR_DOGFOOD:-1}" in
0 | false | no) ungated "CGMR_DOGFOOD is off" ;;
esac

# Its own target dir: cargo-test runs in parallel and spawns
# target/debug/cgmr, which a relink here would pull from under it.
target="$root/target/cgmr-shim"
if ! cargo build --quiet --manifest-path "$root/Cargo.toml" --target-dir "$target" --bin cgmr >&2; then
  [ "${#cmd[@]}" -eq 0 ] || echo "cgmr.sh: cgmr does not build; running ungated (.:V1)." >&2
  ungated "cgmr does not build"
fi

cgmr="$target/debug/cgmr"
if [ "${#cmd[@]}" -eq 0 ]; then
  exec "$cgmr" "${switches[@]}"
fi
exec "$cgmr" "${switches[@]}" -- "${cmd[@]}"
