#!/usr/bin/env bash
#
# workers (nix:V27): how many build workers a wave round may start now
# without congesting this machine, and how many cargo jobs each may use.
#
#   nix/guard/workers.sh [WANTED]    prints: workers=N jobs=J
#
# The repository's own cgmr answers (.:V1): `cgmr advise --profile build
# --jobs WANTED` through nix/guard/cgmr.sh, the `build` profile in .cgmr
# sizing a worker at two logical CPUs and 2 GiB of available memory.
# When cgmr cannot answer (it does not build, or CGMR_DOGFOOD is off), the
# same sizing is measured here: two free logical CPUs per worker after the
# 1-minute load average, 2 GiB of available memory per worker.
# Never more than WANTED, never below 1: a round always makes progress,
# one node at a time if it must.
set -euo pipefail

wanted=${1:-9}
here="${BASH_SOURCE[0]%/*}"

case "$(uname -s)" in
Darwin)
  cpus=$(sysctl -n hw.ncpu)
  ;;
Linux)
  cpus=$(nproc)
  ;;
*)
  echo "workers: unsupported OS $(uname -s)" >&2
  exit 69
  ;;
esac

# Workers that fit, measured without cgmr.
measured() {
  local load avail_kib
  case "$(uname -s)" in
  Darwin)
    load=$(sysctl -n vm.loadavg | awk '{ print $2 }')
    local page
    page=$(sysctl -n hw.pagesize)
    avail_kib=$(vm_stat | awk -v p="$page" '
      /Pages free/ || /Pages inactive/ || /Pages speculative/ { gsub(/\./, "", $NF); n += $NF }
      END { printf "%d", n * p / 1024 }')
    ;;
  Linux)
    load=$(awk '{ print $1 }' /proc/loadavg)
    avail_kib=$(awk '/^MemAvailable:/ { print $2 }' /proc/meminfo)
    ;;
  esac
  awk -v c="$cpus" -v l="$load" -v m="$avail_kib" 'BEGIN {
    by_cpu = int((c - l) / 2)
    by_mem = int(m / (2 * 1024 * 1024))
    printf "%d", (by_cpu < by_mem ? by_cpu : by_mem)
  }'
}

if advice=$(bash "$here/cgmr.sh" advise --quiet --profile build --jobs "$wanted" 2>/dev/null); then
  fit=$(sed -n 's/^jobs=\([0-9]*\) .*/\1/p' <<<"$advice")
fi
fit=${fit:-$(measured)}

awk -v c="$cpus" -v f="$fit" -v w="$wanted" 'BEGIN {
  n = (f < w ? f : w)
  if (n < 1) n = 1
  jobs = int(c / n)
  if (jobs < 1) jobs = 1
  printf "workers=%d jobs=%d\n", n, jobs
}'
