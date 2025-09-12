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

- n/a

### Changed

- Updated dependencies:
  - `cargo_metadata` from `0.20.0` to `0.22.0`
- Bumped MSRV from `0.82.0` to `0.86.0`.

### Deprecated

- n/a

### Removed

- n/a

### Fixed

- n/a

### Performance

- n/a

### Security

- n/a

### Other

- n/a

## [0.11.0] - 2025-06-25

### Changed

- Updated dependencies:
  - `cargo_metadata` from `0.19.2` to `0.20.0`
- Bumped MSRV from `0.78.0` to `0.82.0`.

## [0.10.0] - 2025-05-10

### Added

- Added support for Rust edition "2024"

### Changed

- Updated dependencies:
  - `thiserror` from `1.0.58` to `2.0.0`
  - `cargo_metadata` from `0.18.1` to `0.19.2`
  - `cargo_toml` from `0.20.0` to `0.22.0`

## [0.9.2] - 2024-11-02

### Added

- Re-introduced overriding of `CARGO_TARGET_DIR`, but using parent crate's `CARGO_TARGET_DIR`.

### Fixed

- Fixed a bug (by no longer passing `--tests` to `cargo check`) that would sometimes cause errors of individual files to get wrongly reported all bundled together.

### Performance

- Reversed the minor performance regression introduced in `0.9.1`.

## [0.9.1] - 2024-11-01

### Added

- Added `TRYEXPAND_DEBUG_LOG` CLI option for turning on debug logging (use with `-- --nocapture` for optimal results).

### Changed

- Stopped overriding `CARGO_TARGET_DIR` (you can still opt-in via `.env("CARGO_TARGET_DIR", "…")`) (see ###Fixed).

### Deprecated

- n/a

### Removed

- n/a

### Fixed

- Fixed a bug (by no longer overriding `CARGO_TARGET_DIR`) that would sometimes cause errors of individual files to get wrongly reported all bundled together.

### Performance

- Due to no longer using the same `CARGO_TARGET_DIR` for all tests (even across test groups) there may be some (hopefully minor) performance regressions for some projects.

## [0.9.0] - 2024-11-01

### Added

- Added `struct BuildTestSuite`.
- Added `struct ExpandTestSuite`.
- Added `fn check(…) -> BuildTestSuite` crate-level function.
- Added `fn run(…) -> BuildTestSuite` crate-level function.
- Added `fn run_tests(…) -> BuildTestSuite` crate-level function.

### Changed

- Changed return type of `fn expand(…)` crate-level function from `-> TestSuite` to `-> ExpandTestSuite`.
- Updated dependencies:
  - `regex` from `1.10.3` to `1.11.1`

### Removed

- Removed `struct TestSuite` (by marking it as `pub(crate)`).

## [0.8.4] - 2024-05-24

### Added

- Added `TRYEXPAND_TRUNCATE_OUTPUT` CLI option for turning off truncation of console output.

## [0.8.3] - 2024-05-05

### Changed

- Have `cargo check/run/test` always run without terminal colors (i.e. `--color "never"`).

### Fixed

- Fixed bug that would write stdout's output to snapshot file with '.err.txt' extension instead of '.out.txt'.

## [0.8.2] - 2024-05-05

### Changed

- Updated dependencies:
  - `cargo_toml` from `0.19.2` to `0.20.0`
- Changed location of generated test project's cargo config file from './.cargo/config' to './.cargo/config.toml'.

### Changed

## [0.8.1] - 2024-03-21

- Updated dependencies:
  - `yansi` from `0.5.1` to `1.0.1`
  - `cargo_toml` from `0.19.1` to `0.19.2`
  - `thiserror` from `1.0.57` to `1.0.58`

## [0.8.0] - 2024-02-28

### Changed

- Updated dependencies:
  - `cargo_toml` from `0.18.0` to `0.19.1`
  - `serde` from `1.0.195` to `1.0.197`
  - `thiserror` from `1.0.56` to `1.0.57`
- Bumped MSRV from `0.70.0` to `0.74.0`.

### Fixed

- Fixed redundant logging of expanded code as both, "EXPANDED:" and "OUTPUT:".

## [0.7.1] - 2024-01-26

### Changed

- Restricted logging of snapshot blocks to 100 lines per block.

### Fixed

- Fixed bug where unexpected failure was getting reported, but no actual error included in the log.

## [0.7.0] - 2024-01-22

### Added

- Added support for running test files (i.e. `cargo run`).
- Added support for testing test files (i.e. `cargo test`).
- Added `struct TestSuite`
  - with `.arg()`/`.args()` builder-style methods for providing args.
  - with `.env()`/`.envs()` builder-style methods for providing envs.
  - with `.skip_overwrite()` builder-style method for suppressing snapshot writing.
  - with `.and_check()` builder-style method for running `cargo check` for successful expansions.
  - with `.and_run_tests()` builder-style method for running `cargo test` for successful expansions.
  - with `.and_run()` builder-style method for running `cargo run` for successful expansions.
  - with `.expect_pass()`/`.expect_fail()` builder-style methods for asserting passes/failures.

### Changed

- Changed visibility of `crate::Options` to `pub(crate)`

### Removed

- `fn expand_fail()`
- `fn expand_opts()`
- `fn expand_opts_fail()`
- `fn expand_checking_fail()`
- `fn expand_opts_checking()`
- `fn expand_opts_checking_fail()`

## [0.6.0] - 2024-01-16

### Added

- Added support for checking (i.e. `cargo check`) successful expansions via `expand_checking()` and `expand_opts_checking()`.
- Added field `skip_overwrite: bool` to `Options` for selectively suppressing snapshots.

### Changed

- Updated dependencies:
  - `serde` from `1.0.105` to `1.0.194`
- Changed file extensions:
  - from `.expand.out.rs` to `.out.rs`
  - from `.expand.err.txt` to `.err.txt`

### Fixed

- External dependencies of the crate are now properly mirrored by the test projects.
- Features of the crate are now properly mirrored by the test projects.

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
