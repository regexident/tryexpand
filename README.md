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

Then under your crate's `tests/` directory, create `tests.rs` file containing the following code:

```rust
// Use `expand()` or `expand_opts()` to assert
// successful expansion (ignoring type-checking of the expansion):
#[test]
pub fn expand_pass() {
    tryexpand::expand(
        // One or more glob patterns:
        ["tests/expand/pass/*.rs"]
    );
    // or if you need to pass additional CLI arguments:
    tryexpand::expand_opts(
        // One or more glob patterns:
        ["tests/expand/pass/*.rs"],
        // Arguments to pass to `cargo expand` command:
        ["--features", "test-feature"]
    );
}

// Use `expand_checked()` or `expand_opts_checked()` to
// assert successful expansion and(!) type-checking of the expansion:
#[test]
pub fn expand_checked_pass() {
    tryexpand::expand_checked(
        // One or more glob patterns:
        ["tests/expand/checked_pass/*.rs"]
    );
    // or if you need to pass additional CLI arguments:
    tryexpand::expand_opts_checked(
        // One or more glob patterns:
        ["tests/expand/checked_pass/*.rs"],
        // Arguments to pass to `cargo expand` command:
        ["--features", "test-feature"]
    );
}

// Use `expand_fail()` or `expand_opts_fail()` to assert
// unsuccessful expansion (ignoring type-checking of the expansion):
#[test]
pub fn expand_fail() {
    tryexpand::expand_fail(
        // One or more glob patterns:
        ["tests/expand/fail/*.rs"]
    );
    // or if you need to pass additional CLI arguments:
    tryexpand::expand_opts_fail(
        // One or more glob patterns:
        ["tests/expand/fail/*.rs"],
        // Arguments to pass to `cargo expand` command:
        ["--features", "test-feature"]
    );
}
```

Next populate the `tests/expand/pass/`, `tests/expand/checked_pass/` and `tests/expand/fail/` directories with Rust source files.

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

Each `tryexpand` test will invoke the `cargo expand` command on each of the source files that matches the glob pattern and will compare the expansion result with the corresponding `*.expand.out.rs` file.

If the environment variable `TRYEXPAND=overwrite` is provided (e.g. `$ TRYEXPAND=overwrite cargo test`), then `*.expand.out.rs` snapshot files will
be created, or overwritten, if one already exists. Snapshot files should get checked into version control.

Hand-writing `*.expand.out.rs` files is not recommended.

Possible test outcomes are:

- **Pass**: expansion succeeded and the result is the same as in the `.expand.out.rs` file.
- **Failure**: expansion failed, is missing or was different from the existing `.expand.out.rs` file content.

### Performance considerations

When working with multiple expansion test files, it is recommended to specify wildcard (`*.rs`) instead of doing a multiple calls to the `expand` functions for individual files.

Usage of wildcards for multiple files will group them under a single temporary crate for which dependencies will be built a single time. In contrast, calling `expand` functions for each source file will create multiple temporary crates and that will reduce performance as dependencies will be build for each of the temporary crates.

[More info](https://en.wikipedia.org/wiki/Glob_(programming)) on how glob patterns work.

See [tests/macro-tests](tests/macro-tests) and [tests/proc-macro-tests](tests/proc-macro-tests) as a reference.

### Debugging

For each `expand()`-like method call within your tests a temporary and uniquely named Rust project will get generated within `$CARGO_TARGET_DIR/target/tests/`.
By default these projects will get deleted upon test completion (regardless of the outcome).
If you wish to take a look at the actual code/projects being expanded you can provide `TRYEXPAND_KEEP_ARTIFACTS=1` (e.g. `$ TRYEXPAND_KEEP_ARTIFACTS=1 cargo test`) and `tryexpand` will skip the cleanup.

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
