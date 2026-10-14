congestimancer

Tool for load awareness in the system that allow to calculate machine congestion from:
- unix load
- memory pressure
- io load to advise
- potentially more

To:
- pause or perform other operations
- create mechanical workflows based on resource availability
- advise agents on actions.

Write in Rust.
Use atomic git commits per topic for human readability.

We should have nix flake dev shell loaded with direnv.
When we have basic shell - ask me to reload and continue session.

Guardrails is a first thing and we should:
- use pklith and nix-hk
- use sherd for SPEC-DAG federation and to do sherd wave builds in parallel subagents
- use microlith for spec operations
- use itok for context limiting
- use characterminator to keep characters at bays
Inspire all scrutinized checks from other our Rust repos which apply.
We can dogfood this project in guardrails as well.
Use 1-to-1 unit coverage with floor and ceiling aspiring towards 100%.
Work in TDD: RED, GREEN, REFACTOR cycles.
Guardrails should be installed when entering dev shell.
When we have guardrails ready - ask me to reload and continue session.

# cgmr CLI with swiches:

cgmr --load 2 -- echo example command # wait for load to be below two per logical CPU thread
cgml --all-load 10 -- echo example command # wait for total system load to be below 10

cgmr --mem 2GB -- echo example command # wait for have at least 2GB free memory per processing thread
cgmr --all-mem 10GB -- echo example command # wait for system to have at least 10GB free memory

cgmr --disk 5GB -- echo example command # wait to have 5GB disk space in the working directory

cgmr --io-read 1MB/s -- echo example command # wait for IO reads to be below 1MB/s
cgmr --io-write 1GB/s -- echo example command # wait for IO writes to be below 1GB/s
cgmr --io 100MB/s -- echo example command # wait for total IO reads+writes to be below 100MB/s

Switch can be mixed together as and:
cgmr --load 1 --mem 1GB --disk 2GB --io-read 100MB/s --io-write 100MB/s -- example command
