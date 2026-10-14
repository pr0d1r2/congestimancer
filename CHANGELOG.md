# Changelog

All notable changes to congestimancer are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and versions follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Version ladder: an even minor (0.2, 0.4, ...) is a stable rung; an odd minor
is a development series, not for production.

## [Unreleased]

### Added

- `UnitError`: every parse error names the input and the form expected.
- Numbers in every value take `.` as the decimal separator, whatever the
  locale, and a value past 64 bits is an error, never wrapped.
- `Bytes` and `parse_bytes`: sizes like `10GB` or `1.5GiB`; `KB` is 1000,
  `KiB` is 1024, units are case insensitive, fractions round down.
- `Rate` and `parse_rate`: a size per second, like `50MB/s`.
- `Percent` and `parse_percent`: 0 to 100 with an optional `%`, like `12.5%`.
- `parse_duration`: durations like `1h30m` or `500ms`, units `d h m s ms`
  largest first; a bare number is an error.
- `Celsius` and `parse_celsius`: temperatures like `85C` or `72.5C`.
- `ClockWindow`, `ClockTime` and `parse_clock_window`: daily windows like
  `22:00-06:00` in local time, wrapping midnight when start is after end.
- `Count` and `parse_count`: whole numbers of things; a decimal is an error.
- `Decimal` and `parse_decimal`: load-like thresholds that take decimals,
  like `0.5`; no threshold of any kind is negative.
- `Name` and `parse_name`: safe identifiers, 1 to 64 of `A-Z a-z 0-9 . _ -`,
  never a leading `-`, never `.` or `..`.
- Every value renders back as the input that parses to it (`Display`, and
  `render_duration` for durations): `1KiB`, `12.5%`, `1h30m`, `22:00-06:00`.
- `SignalId`: one signal per gate switch in `docs/CONGESTION.md`, named by
  its switch (`io-busy`); `--all-load`, `--all-run-queue`, `--all-mem` and
  `--needs-mem` read the signal of their plain switch.
- `Reading`, `Measure`, `Reason` and `Level`: a signal reads a value with
  its unit (rendered as its switch takes it: `12.5%`, `50MB/s`, `72.5C`),
  `warming up` before a rate has two samples, or `unsupported: <reason>`.
- `Source` and `Clock` traits, and `SystemClock`: every reading is taken at
  a monotonic instant, and every interval is measured on a `Clock`.
- `Signal`, `Sample`, `Sampled`, `Sampler` and `Counter`: rates and
  interval percents come from two samples of a running total; the first
  sample, and any sample after a counter goes back, reads `warming up`,
  never zero and never negative.
- `Root`: the directory kernel files are read under, `/` on the live
  machine or a fixture tree; `..` never climbs above it.
- `SignalSet` and `refresh`: only the signals the requested gates read are
  refreshed; the others are never touched.
- `Trend` and `Trends`: each signal reads `rising`, `falling` or `flat`
  from its last two values.
- `parse_psi`, `Psi`, `PsiLine` and `PsiError`: one parser for Linux
  pressure stall text (`/proc/pressure/*` and cgroup `*.pressure`); a bad
  line is named in the error.
