#!/usr/bin/env bash
#
# workers (nix:V27): how many build workers a wave round may start now
# without congesting this machine, and how many cargo jobs each may use.
# A stand-in for `cgmr advise` until it exists.
#
#   nix/guard/workers.sh [WANTED]    prints: workers=N jobs=J
#
# CPU: two free logical CPUs per worker, after the 1-minute load average.
# Memory: 2 GiB of available memory per worker (a cargo build of this
# workspace peaks well under that). Never more than WANTED, never below 1:
# a round always makes progress, one node at a time if it must.
set -euo pipefail

wanted=${1:-9}

case "$(uname -s)" in
Darwin)
  cpus=$(sysctl -n hw.ncpu)
  load=$(sysctl -n vm.loadavg | awk '{ print $2 }')
  page=$(sysctl -n hw.pagesize)
  avail_kib=$(vm_stat | awk -v p="$page" '
    /Pages free/ || /Pages inactive/ || /Pages speculative/ { gsub(/\./, "", $NF); n += $NF }
    END { printf "%d", n * p / 1024 }')
  ;;
Linux)
  cpus=$(nproc)
  load=$(awk '{ print $1 }' /proc/loadavg)
  avail_kib=$(awk '/^MemAvailable:/ { print $2 }' /proc/meminfo)
  ;;
*)
  echo "workers: unsupported OS $(uname -s)" >&2
  exit 69
  ;;
esac

awk -v c="$cpus" -v l="$load" -v m="$avail_kib" -v w="$wanted" 'BEGIN {
  by_cpu = int((c - l) / 2)
  by_mem = int(m / (2 * 1024 * 1024))
  n = w
  if (by_cpu < n) n = by_cpu
  if (by_mem < n) n = by_mem
  if (n < 1) n = 1
  jobs = int(c / n)
  if (jobs < 1) jobs = 1
  printf "workers=%d jobs=%d\n", n, jobs
}'
