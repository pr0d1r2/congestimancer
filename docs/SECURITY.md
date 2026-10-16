# Security

cgmr runs commands for you, keeps lock files on disk and can reach other
machines over ssh. This page lists where those touch untrusted input and
what cgmr does about each. Report a vulnerability through a private GitHub
security advisory on this repository rather than a public issue.

## `--when` runs a shell command

`--when CMD` runs `CMD` through `sh -c` each round and passes when it exits
0. That is arbitrary code execution by design, so cgmr only takes it from
places you control:

- The command line, where you typed it.
- The user profile file (`$XDG_CONFIG_HOME/cgmr/profiles`, else
  `~/.config/cgmr/profiles`).

It is refused everywhere else:

- `CGMR_WHEN` is ignored with a warning: an inherited environment must not
  be able to run commands.
- A project `.cgmr` file that sets `--when` is a usage error (exit 64)
  naming the file and line. A cloned repository is not trusted to run
  anything when you merely `cd` into it, the same rule direnv applies.

`--when-file PATH` only tests whether a file exists and never runs it.

## The lock directory

Slots, FIFO tickets, rate buckets and memory reservations live in a per-user
directory: `$XDG_RUNTIME_DIR/cgmr`, else `$TMPDIR/cgmr`, else
`/tmp/cgmr-<uid>`.

- cgmr creates it with mode 0700.
- If it already exists and is owned by another user, or its mode is wider
  than 0700, cgmr refuses to use it and exits 69 with the reason. This stops
  another user from planting the directory, or a symlink to it, in a shared
  `/tmp` to read or block your locks.
- Locks are `flock`s on files inside it. The kernel releases them when the
  holding process dies, so a crash never leaves a slot taken.
- Only the started command inherits the lock file descriptors; cgmr's own
  helpers (the `--when` shell, ssh) never do.

## Names used in paths

A slot key (`--slot-key`), a profile name and any other name cgmr turns into
a file name must be 1 to 64 characters of `[A-Za-z0-9._-]`, must not start
with `-`, and must not be `.` or `..`. Anything else is a usage error
(exit 64). When `--slots` has no key, the command's base name is used, and
it must pass the same rule. So no name can escape the lock directory or be
read as an option.

## `--host` and ssh

`--host H` evaluates the gates on another machine by running `cgmr check
--json` there through your own `ssh`:

- ssh runs with `-o BatchMode=yes`, so it never stops to ask for a password
  or a host key; it fails the round instead.
- A host that starts with `-` is refused (exit 64), and the host is passed
  after `--` in ssh's argv, so it can never be read as an ssh option such as
  `-oProxyCommand=...`.
- Each gate argument is a separate argv word, quoted for the remote shell,
  so a threshold cannot inject a command there.
- Only the gates run remotely. Slots, reservations and the command itself
  stay on the local machine.
- The answer must carry a `schema_version` cgmr supports, else exit 69.

## Network

The only connections cgmr opens are the ones you ask for: `--rtt` makes a
TCP connect to `--rtt-host`, and `--host` runs your `ssh`. It never phones
home, and its build needs no network once the locked crates are fetched.

## Unsafe code

`unsafe` is denied crate-wide. Only `src/sys`, the OS facade, may allow it,
every `unsafe` block there carries a `// SAFETY:` comment, and a gate step
fails the build on `allow(unsafe_code)` anywhere else.

## The command's environment

Before `exec`, cgmr removes its own `CGMR_*` variables from the command's
environment (except the held-slot marker), so a nested `cgmr` does not
inherit gates, profiles or a fixture root by accident.
