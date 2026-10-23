# Integration

Ways to put `cgmr` in front of heavy work: git hooks, CI, build tools and
coding agents. Every example uses switches from
[CONGESTION.md](CONGESTION.md); [CLI.md](CLI.md) covers exit codes.

## Getting cgmr

```sh
cargo install congestimancer      # once published; installs `cgmr`
nix run github:pr0d1r2/congestimancer -- status
```

In another flake, pin a tag and add the overlay, so `cgmr` is on the dev
shell's PATH:

```nix
{
  # Pin a release tag; the overlay adds `cgmr` to pkgs.
  inputs.congestimancer.url = "github:pr0d1r2/congestimancer/<tag>";

  outputs = { nixpkgs, congestimancer, ... }:
    let
      system = "aarch64-darwin";
      pkgs = import nixpkgs {
        inherit system;
        overlays = [ congestimancer.overlays.default ];
      };
    in
    {
      devShells.${system}.default = pkgs.mkShell { packages = [ pkgs.cgmr ]; };
    };
}
```

## A project profile

Put the gates in the repository once, in a `.cgmr` at its root, and call
them by name. A project file may not use `--when`.

```text
# .cgmr
[heavy]
--load 1
--needs-mem 2GiB
--timeout 10m
--or-background
--heartbeat 30s
```

```sh
cgmr --profile heavy -- cargo test
```

`--or-background` means a busy machine delays the step, then runs it at
background priority, instead of failing it after ten minutes.

`--load 1` suits a single long step. It is too tight for a hook that runs
several heavy steps, see
[Several heavy steps in one hook](#several-heavy-steps-in-one-hook).
A project profile shared across platforms should also avoid
`--mem-pressure` and the other PSI gates: they exit 69 on macOS.

## hk

Wrap only the steps that are heavy; a fast lint gains nothing from
waiting.

```pkl
["coverage"] {
  glob = List("**/*.rs")
  check = "cgmr --profile heavy -- cargo llvm-cov --workspace"
}
```

### Several heavy steps in one hook

The load average lags by about a minute and counts the hook's own steps. A
tight `--load` gate is therefore never satisfied once the first steps have
started: each later step waits for load that its own siblings cause.
Measured on an 8-core machine, same range:

| profile | wall |
| --- | --- |
| ungated | 42s |
| `--load 0.9 --mem 2GiB --slots 3 --timeout 1m --or-background` | 133s |
| `--load 1.5 --mem 2GiB --slots 3 --fifo --timeout 30s --or-background` | 40s |

With the tight gate every heavy step waited the full timeout, then ran at
background priority. Within one hook:

- Pace the steps with `--slots` and `--fifo`; slots count the hook's own
  steps exactly, with no lag.
- Keep the load gate for real overload only, well above 1 per CPU.
- Pair it with `--timeout` and `--or-background` so nothing hangs.
- `cgmr advise` reads the same lagging load, so it may size `--jobs` down
  to a fraction of the cores mid-hook.

```text
# .cgmr
[hook]
--load 1.5
--needs-mem 2GiB
--slots 3
--fifo
--timeout 30s
--or-background
```

## lefthook

```yaml
pre-push:
  commands:
    test:
      run: cgmr --profile heavy -- cargo test --workspace
```

With several heavy commands in one hook, use a pacing profile as in
[Several heavy steps in one hook](#several-heavy-steps-in-one-hook).

## Make, xargs and other parallel runners

Ask how many jobs fit, then size the run to it:

```sh
jobs=$(cgmr advise --jobs 16 --needs-mem 1GiB | sed 's/^jobs=\([0-9]*\).*/\1/')
make -j"${jobs:-1}"
```

Or gate each job and let slots bound the fan-out across processes:

```sh
ls tests/*.sh | xargs -P 16 -I{} cgmr --slots 4 --slot-key e2e --fifo -- {}
```

## CI

On a shared self-hosted runner, several jobs land on one machine. Slots and
a start rate stop them all starting their heavy phase at once:

```yaml
- name: Integration tests
  run: cgmr --slots 2 --slot-key integration --spacing 30s --timeout 30m -- make integration
```

On hosted runners, which give each job its own machine, record the
machine's state rather than gate on it:

```yaml
- name: Machine state
  run: cgmr status --json
```

## Coding agents

An agent should ask before it fans out, not after the machine stalls:

```sh
cgmr advise --jobs 8 --json
# {"schema_version":1,"jobs":3,"limit":"mem","action":"reduce","blocking":[]}
```

- `action` is `run`, `reduce` or `wait`; `jobs` is how many fit now.
- `cgmr check --json --load 1 --needs-mem 2GiB` answers yes or no for one
  task, with every gate's verdict.
- `cgmr status --json` gives every reading, for an agent that wants to
  decide itself.
- Wrap each spawned build in `cgmr -- ...` with the same gates, so a burst
  of starts is spread out by `--spacing` or `--fifo` instead of all passing
  in the same second.

## Dogfood: a tool's own gate

A repository that builds cgmr (or any gate tool) should not let a compile
error lock its own push. This repository's pattern is a shim that builds
cgmr from the working tree and falls back to running the step ungated:

```sh
# nix/guard/cgmr.sh [SWITCHES...] -- CMD...
case "${CGMR_DOGFOOD:-1}" in
0 | false | no) exec "${cmd[@]}" ;;
esac
if ! cargo build --quiet --bin cgmr; then
  echo "cgmr does not build; running ungated" >&2
  exec "${cmd[@]}"
fi
exec target/debug/cgmr "${switches[@]}" -- "${cmd[@]}"
```

Its hk steps call `bash nix/guard/cgmr.sh --profile heavy -- <step>`, and
`CGMR_DOGFOOD=0` turns the gating off for one run. The wave-build budget
script asks `cgmr advise --profile build` through the same shim.
