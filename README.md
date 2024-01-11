# `tryexpand`

[![Crates.io](https://img.shields.io/crates/v/tryexpand)](https://crates.io/crates/tryexpand)
![MSRV 1.56](https://img.shields.io/badge/MSRV-1.56-orange.svg)
[![docs.rs](https://docs.rs/tryexpand/badge.svg)](https://docs.rs/tryexpand/)
[![Crates.io](https://img.shields.io/crates/d/tryexpand)](https://crates.io/crates/tryexpand)
[![Crates.io](https://img.shields.io/crates/l/tryexpand)](https://crates.io/crates/tryexpand)

Similar to [trybuild], but allows you to test how declarative or procedural macros are expanded.

*Minimal Supported Rust Version: 1.56*

----

## Documentation

Please refer to the [documentation](https://docs.rs/tryexpand).

## Example

Install nightly rust and [`cargo expand`].

Add to your crate's Cargo.toml:

```toml
[dev-dependencies]
tryexpand = "1"
```

Under your crate's `tests/` directory, create `tests.rs` file containing the following code:

```rust
#[test]
pub fn pass() {
    tryexpand::expand("tests/expand/*.rs");
}
```

Populate the `tests/expand/` directory with rust source files. Each source file is a macro expansion test case.

See [test-project](test-project) and [test-procmacro-project](test-procmacro-project) for the reference.

[trybuild]: https://github.com/dtolnay/trybuild
[`cargo expand`]: https://github.com/dtolnay/cargo-expand
