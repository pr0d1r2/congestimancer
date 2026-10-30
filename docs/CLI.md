# CLI guide

`cgmr` has one job: hold a command until the machine has room for it. This
page covers the verbs, how switches and environment variables combine, and
what each exit code means. Every gate switch and what it measures is in
[CONGESTION.md](CONGESTION.md).

## Verbs

| Verb | What it does |
| --- | --- |
| `run` (default) | Wait until every gate passes, then `exec` the command after `--`. Without a command, exit 0 once the gates pass. |
| `check` | Evaluate every gate once and exit: 0 all pass, 1 one fails. Never waits, except one extra sample for rates. |
| `status [--json]` | Print every signal this machine supports with its current value. |
| `advise [--jobs J] [--json]` | Say how many parallel jobs fit now, what limits them, and whether to run, reduce or wait. |
| `profiles` | List the profiles in force: names, the files they come from, and their switches. |
| `completions <shell>` | Print a completion script for bash, elvish, fish, powershell or zsh. |
| `man` | Print the man page (roff). |

The command always comes after `--`, so its own arguments never mix with
cgmr's:

```sh
cgmr --load 2 -- make -j8          # wait, then exec make
cgmr --load 2 && make              # no command: wait, then exit 0
cgmr check --load 1 --disk 5GB     # one-shot, for scripts and agents
cgmr status --json                 # every reading, for agents
cgmr advise --jobs 8 --needs-mem 2GB --json
cgmr profiles
cgmr completions zsh > ~/.zfunc/_cgmr
cgmr man > ~/.local/share/man/man1/cgmr.1
```

`cgmr <verb> --help` shows the switches and examples for that verb.

## Switches, environment and profiles

- Every switch `--foo-bar` also reads `CGMR_FOO_BAR`, except `--when`,
  which runs a shell command and is never taken from the environment.
- Precedence, highest first: the flag, the environment variable, the
  profile, the built-in default.
- Environment values use the same text as the flag. A switch without a
  value takes `1`, `true` or `yes` for on and `0`, `false` or `no` for
  off; anything else is a usage error. A switch that repeats
  (`--max-procs`, `--max-user-procs`) takes a comma-separated list:
  `CGMR_MAX_PROCS="rustc*=4,cargo=2"`.
- A single-value switch given twice is a usage error.
- An unknown `CGMR_*` variable gets a warning naming the nearest known one,
  and cgmr carries on. `--quiet` hides warnings, never errors.
- `--profile NAME` takes the switches of a named profile from the nearest
  `.cgmr` file or the user profile file; see CONGESTION.md for the format
  and lookup.

## Machine-readable output

`check --json`, `status --json` and `advise --json` print one JSON object
with a `schema_version` field (currently 1). Field names are stable; a
breaking change raises the version.
`status --json` adds a `learned` list (`key`, `peak_bytes`) once
`--needs-mem auto` has measured a key.

## Exit codes

| Code | Meaning |
| --- | --- |
| 0 | Every gate passed and there is no command. `check`: every gate passed. |
| 1 | `check`: a gate failed. |
| 64 | Usage error: an unknown switch, a value that does not parse, a bad profile. |
| 69 | A gate is unsupported on this machine, or cgmr cannot work here (an unsafe lock directory). |
| 124 | `--timeout` ran out before the gates passed. |
| 130, 143 | SIGINT or SIGTERM arrived while waiting. cgmr keeps the default disposition, so the shell sees death by signal and the command never runs. |
| N | Once the command runs, cgmr has `exec`ed it: the exit code is the command's own. |

`advise` exits 0 whatever it advises; 69 means a requested gate is
unsupported here.

### The exit-code collision

After `exec`, cgmr is gone and the exit code belongs to the command. A
command that itself exits 64, 69 or 124 therefore looks exactly like a cgmr
usage error, an unsupported gate or a timeout. To tell them apart:

- Run `cgmr check` with the same gates first; its 64 or 69 is always
  cgmr's.
- Or keep the gating and the work separate: `cgmr --load 2 && make`, so
  cgmr's exit code and the command's never share a process.
- With `--govern`, cgmr stays the parent and still exits with the
  command's code, so the same collision applies.
