# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

Please make sure to add your changes to the appropriate categories:

- `Added`: for new functionality
- `Changed`: for changes in existing functionality
- `Deprecated`: for soon-to-be removed functionality
- `Removed`: for removed functionality
- `Fixed`: for fixed bugs
- `Performance`: for performance-relevant changes
- `Security`: for security-relevant changes
- `Other`: for everything else

## [Unreleased]

### Added

- Added `Options` type.
- Added `expand_opts()` & `expand_opts_fail()` (replacing `expand_args()` & `expand_args_fail()`).

### Changed

- n/a

### Deprecated

- n/a

### Removed

- Removed `expand_args()` & `expand_args_fail()` (in favor of `expand_opts()` & `expand_opts_fail()`).

### Fixed

- Named errors (i.e. `error[E…]: …`) are now properly detected and included in error snapshots.
- Generated `.extended.rs` files obtained from failures no longer include Rust prelude, etc.
- No longer crashes when encountering an unexpectedly empty stdout/stderr, but reports an error instead.
- No longer reports updated snapshots for snapshots that were overwritten, but actually unchanged.

### Performance

- Tests now properly share a single target directory, speeding up compilation times.

### Security

- n/a

### Other

- n/a

## [0.2.0] - 2024-01-12

### Changed

- On failure two snapshots are now getting generated:
  - the output from `stdout` gets saved to a `.expanded.rs` file
  - the output from `stderr` gets saved to a `.error.txt` file

### Removed

- Removed `scopeguard` crate from project's dependencies
- Removed `serde_json` crate from project's dependencies

## [0.1.0] - 2024-01-12

Initial release.
