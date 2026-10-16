#!/usr/bin/env bash
#
# bench (src/wait:V10, nix:V25): cgmr's own cost stays inside its budget.
#
#   nix/guard/bench.sh [CGMR]    default: a fresh release build
#
# ROUND CPU: `cgmr run --load 0.001` never passes, so for SECONDS it does
# nothing but check rounds every 10ms; its user + system CPU (bash `times`,
# children) divided by the rounds it had time for must stay <= 5 ms.
# RSS: the same waiting process, sampled with `ps -o rss=` every 100ms;
# the peak must stay <= 10 MB.
# COLD START: wall time of `cgmr check --load 1`, the best of RUNS, must
# stay <= 20 ms. Best, not mean: a busy machine slows a run, it never
# speeds one up, so the minimum is the cost cgmr itself adds.
#
# Measured on a release build: a debug build is not what users run.
set -euo pipefail

seconds=2
interval_ms=10
runs=10
round_budget_us=5000
rss_budget_kib=10240
start_budget_us=20000

if [ "$#" -ge 1 ]; then
  cgmr=$1
else
  cargo build --release --locked --quiet --bin cgmr
  cgmr=target/release/cgmr
fi
[ -x "$cgmr" ] || {
  echo "bench: no cgmr binary at $cgmr -- nothing was measured. This is a failure, not a pass." >&2
  exit 1
}
unset CGMR_FIXTURE_ROOT
scratch=$(mktemp -d)
trap 'rm -rf "$scratch"' EXIT

# Microseconds since the epoch, from bash's own clock.
now_us() {
  local t=${EPOCHREALTIME/[.,]/}
  printf '%s' "$t"
}

# Children's user + system CPU so far, in microseconds, from `times`. Run
# in this shell, never in a pipe: a subshell has no children of its own.
# The ps and sleep calls that sample RSS count too, so the round cost is
# overstated, never understated.
children_cpu_us() {
  times >"$scratch/times"
  awk 'NR == 2 {
    n = 0
    for (i = 1; i <= 2; i++) { split($i, p, /[ms]/); n += p[1] * 60 + p[2] }
    printf "%d", n * 1000000
  }' "$scratch/times"
}

status=0
over() {
  echo "bench: $1 (src/wait:V10)" >&2
  status=1
}

# Cold start.
best=
for _ in $(seq "$runs"); do
  t0=$(now_us)
  "$cgmr" check --load 1 >/dev/null 2>&1 || true
  t=$(($(now_us) - t0))
  if [ -z "$best" ] || [ "$t" -lt "$best" ]; then best=$t; fi
done
echo "bench: cold start ${best}us (budget ${start_budget_us}us)"
[ "$best" -le "$start_budget_us" ] || over "cold start of cgmr check --load 1 took ${best}us, over ${start_budget_us}us"

# Round CPU and RSS, from one waiting process.
children_cpu_us >"$scratch/cpu0"
cpu0=$(<"$scratch/cpu0")
"$cgmr" run --load 0.001 --interval "${interval_ms}ms" --timeout "${seconds}s" >/dev/null 2>&1 &
pid=$!
peak=0
while kill -0 "$pid" 2>/dev/null; do
  rss=$(ps -o rss= -p "$pid" 2>/dev/null | tr -d ' ' || true)
  if [ -n "$rss" ] && [ "$rss" -gt "$peak" ]; then peak=$rss; fi
  sleep 0.1
done
code=0
wait "$pid" || code=$?
[ "$code" -eq 124 ] || over "the waiting run exited $code, not 124: it did not wait the whole time"
children_cpu_us >"$scratch/cpu1"
cpu=$(($(<"$scratch/cpu1") - cpu0))
rounds=$((seconds * 1000 / interval_ms))
per_round=$((cpu / rounds))
echo "bench: round CPU ${per_round}us over ${rounds} rounds (budget ${round_budget_us}us)"
echo "bench: peak RSS ${peak}KiB (budget ${rss_budget_kib}KiB)"
[ "$per_round" -le "$round_budget_us" ] || over "one --load round used ${per_round}us CPU, over ${round_budget_us}us"
[ "$peak" -le "$rss_budget_kib" ] || over "resident memory peaked at ${peak}KiB, over ${rss_budget_kib}KiB"

exit "$status"
