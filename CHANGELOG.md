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

- Support for checking (i.e. `cargo check`) successful expansions via `expand_checking()` and `expand_opts_checking()`.

### Changed

- Updated dependencies:
  - `serde` from `1.0.105` to `1.0.194`
- Changed file extensions:
  - from `.expand.out.rs` to `.out.rs`
  - from `.expand.err.txt` to `.err.txt`

### Deprecated

- n/a

### Removed

- n/a

### Fixed

- External dependencies of the crate are now properly mirrored by the test projects.
- Features of the crate are now properly mirrored by the test projects.

### Performance

- n/a

### Security

- n/a

### Other

- n/a

## [0.5.0] - 2024-01-15

### Changed

- Passing no file patterns is now considered a failure:
  - Calling `expand()` with an empty list of file patterns will fail.
  - Calling `expand_opts()` with an empty list of file patterns will fail.
  - Calling `expand_fail()` with an empty list of file patterns will fail.
  - Calling `expand_opts_fail()` with an empty list of file patterns will fail.
- Passing file patterns that match no files is now considered a failure:
  - Calling `expand()` with a file pattern that matches no files will fail.
  - Calling `expand_opts()` with a file pattern that matches no files will fail.
  - Calling `expand_fail()` with a file pattern that matches no files will fail.
  - Calling `expand_opts_fail()` with a file pattern that matches no files will fail.

## [0.4.0] - 2024-01-15

### Added

- Added `cargo_metadata = "0.18.1"` crate dependency.
- Added support for (virtual/non-virtual) workspaces.

### Changed

- Changed file extension from `.expanded.rs` to `.expand.out.rs` (to match the `expand` command, so we can add others in the future).
- Cargo metadata now gets read via `cargo_metadata` which is more robust than `cargo_toml`.

### Other

- Improved error messages.

## [0.3.0] - 2024-01-13

### Added

- Added `Options` type.
- Added `expand_opts()` & `expand_opts_fail()` (replacing `expand_args()` & `expand_args_fail()`).

### Removed

- Removed `expand_args()` & `expand_args_fail()` (in favor of `expand_opts()` & `expand_opts_fail()`).

### Fixed

- Named errors (i.e. `error[E…]: …`) are now properly detected and included in error snapshots.
- Generated `.extended.rs` files obtained from failures no longer include Rust prelude, etc.
- No longer crashes when encountering an unexpectedly empty stdout/stderr, but reports an error instead.
- No longer reports updated snapshots for snapshots that were overwritten, but actually unchanged.

### Performance

- Tests now properly share a single target directory, speeding up compilation times.

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
