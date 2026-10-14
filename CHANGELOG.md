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
