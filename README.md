# `tryexpand`

[![Crates.io](https://img.shields.io/crates/v/tryexpand)](https://crates.io/crates/tryexpand)
[![Crates.io](https://img.shields.io/crates/d/tryexpand)](https://crates.io/crates/tryexpand)
[![Crates.io](https://img.shields.io/crates/l/tryexpand)](https://crates.io/crates/tryexpand)
[![docs.rs](https://docs.rs/tryexpand/badge.svg)](https://docs.rs/tryexpand/)

Similar to [trybuild](https://crates.io/crates/trybuild), but allows you to test how declarative or procedural macros are expanded.

----

## Documentation

Please refer to the documentation on [docs.rs](https://docs.rs/tryexpand).

## Requirements

`tryexpand` requires [cargo-expand](https://crates.io/crates/cargo-expand) to be installed.

## Usage

### Installation

Add `tryexpand` to your project as a dev-dependency by running

```terminal
cargo install --dev tryexpand
```

### Writing tests

Then under your crate's `tests/` directory, create `tests.rs` file containing calls to `tryexpand::expand()` and populate the `tests/expand/pass/`, `tests/expand/checked_pass/` and `tests/expand/fail/` directories with corresponding Rust source files under test.

### Test actions

The `tryexpand` crate exposes the following functions test actions:

#### cargo expand

The `tryexpand::expand(…)` function runs `cargo expand` for each test file and snapshot the results:

```rust
tryexpand::expand(
    // ...
)
.expect_pass();
```

If you wish to also perform additional snapshot tests for all successfully expanded files you can do so via an additional call to either of these:

- `.and_check()` to run `cargo check` for each test file and snapshot the results:

  ```rust
  tryexpand::expand(
      // ...
  )
  .and_check()
  .expect_pass();
  ```

- `.and_run()` to run `cargo run` for each test file and snapshot the results:

  ```rust
  tryexpand::expand(
      // ...
  )
  .and_run()
  .expect_pass();
  ```

- `.and_run_tests()` to run `cargo test` for each test file and snapshot the results:

  ```rust
  tryexpand::expand(
      // ...
  )
  .and_run_tests()
  .expect_pass();
  ```

#### cargo check

The `tryexpand::check(…)` function runs `cargo check` for each test file and snapshot the results:

```rust
tryexpand::check(
    // ...
)
.expect_pass();
```

#### cargo run

The `tryexpand::run(…)` function runs `cargo run` for each test file and snapshot the results:

```rust
tryexpand::run(
    // ...
)
.expect_pass();
```

#### cargo test

The `tryexpand::run_tests(…)` function runs `cargo test` for each test file and snapshot the results:

```rust
tryexpand::run_tests(
    // ...
)
.expect_pass();
```

#### Pass

The base of each `tryexpand` test suite is the corresponding [test action](#test-actions) function (we're using `tryexpand::expand(…)` here, but this applies to all actions), which you pass a list of file paths (or glob patterns) to:

```rust
#[test]
pub fn pass() {
    tryexpand::expand(
        ["tests/expand/pass/*.rs"]
    ).expect_pass();

    // or its short-hand (by default `.expect_pass()` is implied):

    tryexpand::expand(
        ["tests/expand/pass/*.rs"]
    );
}
```

By default `tryexpand`'s [test action](#test-actions) functions assert matched test files to pass their tests.

#### Fail

If instead you want to write tests for a failure's diagnostics, then ou can do so via an additional call to `.expect_fail()` (we're using `tryexpand::expand(…)` here, but this applies to all actions):

```rust
#[test]
pub fn fail() {
    tryexpand::expand(
        ["tests/expand/fail/*.rs"]
    ).expect_fail();
}
```

#### CLI arguments

Additionally you can specify arguments to pass to the `cargo` command:

```rust
#[test]
tryexpand::expand(
    // ...
)
// ...
.args(["--features", "test-feature"])
.expect_pass();
```

#### CLI env vars

As well as environment variables to set for the `cargo` command:

```rust
tryexpand::expand(
    // ...
)
// ...
.envs([("MY_ENV", "my env var value")])
.expect_pass();
```

### Running tests

The test can be run with:

```terminal
cargo test
```

While it is possible to run parallel tests it is recommended to run them serially:

```terminal
cargo test -- --test-threads=1
```

For debugging purposes you may want to see the output for all tests, not just the failing ones:

```terminal
cargo test -- --no-capture
```

Each `tryexpand` test will invoke the `cargo expand` command (as well as any of the optional follow-up commands: `cargo check`, `cargo run`, `cargo test`) on each of the source files that matches the glob pattern and will compare the expansion result with the corresponding `*.out.rs`, `*.out.txt` or `*.err.txt` snapshot files.

If the environment variable `TRYEXPAND=overwrite` is provided (e.g. `$ TRYEXPAND=overwrite cargo test`), then snapshot files will be created, or overwritten, if one already exists. Snapshot files should get checked into version control.

Hand-writing snapshot files is not recommended.

### Performance considerations

When working with multiple expansion test files, it is recommended to specify wildcard (`*.rs`) instead of doing a multiple calls to the `expand` functions for individual files.

Usage of wildcards for multiple files will group them under a single temporary crate for which dependencies will be built a single time. In contrast, calling `expand` functions for each source file will create multiple temporary crates and that will reduce performance as dependencies will be build for each of the temporary crates.

[More info](https://en.wikipedia.org/wiki/Glob_(programming)) on how glob patterns work.

See [tests/macro-tests](tests/macro-tests) and [tests/proc-macro-tests](tests/proc-macro-tests) as a reference.

### Reliability considerations

Since each rustc/cargo release might make changes to the emitted diagnostics
it is recommended to run `tryexpand` tests using a [pinned](https://rust-lang.github.io/rustup/overrides.html) [toolchain](https://rust-lang.github.io/rustup/concepts/toolchains.html), e.g.:

```terminal
cargo +1.76.0 test <OPTIONS>
```

### Debugging

#### `TRYEXPAND_KEEP_ARTIFACTS`

For each `expand()`-like method call within your tests a temporary and uniquely named Rust project will get generated within `$CARGO_TARGET_DIR/target/tests/`.
By default these projects will get deleted upon test completion (regardless of the outcome).
If you wish to take a look at the actual code/projects being expanded you can provide `TRYEXPAND_KEEP_ARTIFACTS=1` (e.g. `$ TRYEXPAND_KEEP_ARTIFACTS=1 cargo test`) and `tryexpand` will skip the cleanup.

#### `TRYEXPAND_TRUNCATE_OUTPUT`

By default `tryexpand` truncates console output that's longer than 100 lines.
If you wish to temporarily turn this behavior you can provide `TRYEXPAND_TRUNCATE_OUTPUT=0` (e.g. `$ TRYEXPAND_TRUNCATE_OUTPUT=0 cargo test`) and `tryexpand` will produce the full console output.

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our [code of conduct](https://www.rust-lang.org/conduct.html),  
and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/regexident/cargo-modules/tags).

## License

This project is licensed under the [**MIT**][mit-license] or [**Apache-2.0**][apache-license] – see the [LICENSE-MIT.md](LICENSE-MIT.md)/[LICENSE-APACHE.md](LICENSE-APACHE.md) files for details.

## Provenance

The `tryexpand` crate originated as a fork of [eupn](https://github.com/eupn)'s `macrotest` ([crates.io](https://crates.io/crates/macrotest)).

[mit-license]: https://www.tldrlegal.com/license/mit-license
[apache-license]: https://www.tldrlegal.com/l/apache-license-2-0-apache-2-0
