# `tryexpand`

[![Crates.io](https://img.shields.io/crates/v/tryexpand)](https://crates.io/crates/tryexpand)
[![Crates.io](https://img.shields.io/crates/d/tryexpand)](https://crates.io/crates/tryexpand)
[![Crates.io](https://img.shields.io/crates/l/tryexpand)](https://crates.io/crates/tryexpand)
[![docs.rs](https://docs.rs/tryexpand/badge.svg)](https://docs.rs/tryexpand/)

Similar to [trybuild], but allows you to test how declarative or procedural macros are expanded.

----

## Documentation

Please refer to the documentation on [docs.rs](https://docs.rs/tryexpand).

## Usage

Install Rust and [`cargo expand`].

Add to your crate's Cargo.toml:

```toml
[dev-dependencies]
tryexpand = "0.1.0"
```

Under your crate's `tests/` directory, create `tests.rs` file containing the following code:

```rust
#[test]
pub fn pass() {
    tryexpand::expand(["tests/expand_pass/*.rs"]);
    // or:
    tryexpand::expand_args(["tests/expand_pass/*.rs"], ["--features", "test-feature"]);
}

#[test]
pub fn fail() {
    tryexpand::expand_fail(["tests/expand_fail/*.rs"]);
    // or:
    tryexpand::expand_args_fail(["tests/expand_fail/*.rs"], ["--features", "test-feature"]);
}
```

Populate the `tests/expand_pass/`/`tests/expand_fail/` directories with Rust source files.
Each source file is a macro expansion test case.

See [tests/macro-tests](tests/macro-tests) and [tests/proc-macro-tests](tests/proc-macro-tests) for the reference.

## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our [code of conduct](https://www.rust-lang.org/conduct.html),  
and the process for submitting pull requests to us.

## Versioning

We use [SemVer](http://semver.org/) for versioning. For the versions available, see the [tags on this repository](https://github.com/regexident/cargo-modules/tags).

## License

This project is licensed under the [**MIT**][mit-license] or [**Apache-2.0**][apache-license] – see the [LICENSE-MIT.md](LICENSE-MIT.md)/[LICENSE-APACHE.md](LICENSE-APACHE.md) files for details.

[trybuild]: https://github.com/dtolnay/trybuild
[`cargo expand`]: https://github.com/dtolnay/cargo-expand
[mit-license]: https://www.tldrlegal.com/license/mit-license
[apache-license]: https://www.tldrlegal.com/l/apache-license-2-0-apache-2-0

## Provenance

The `tryexpand` crate originated as a fork of [eupn](https://github.com/eupn)'s `macrotest` ([crates.io](https://crates.io/crates/macrotest)).
