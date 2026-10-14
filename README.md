# congestimancer

<!-- BEGIN badges -->
<!-- END badges -->

`cgmr` holds a command until the machine has room for it: load, memory,
disk, IO and other congestion gates must pass first.

```sh
cgmr --load 2 --mem 2GB -- make -j8   # wait, then run
cgmr check --load 1 --disk 5GB        # one-shot check for scripts
cgmr advise --jobs 8 --json           # how much work fits now
```

Status: in development; nothing is published yet.

- [Congestion methods](docs/CONGESTION.md): every signal, gate and strategy.
- [Spec](SPEC.md): the rules the code is built against.
- [Agents](AGENTS.md): how coding agents work in this repository.

## License

MIT, see [LICENSE](LICENSE).
