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
