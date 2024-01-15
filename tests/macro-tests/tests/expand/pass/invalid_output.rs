#[macro_use]
extern crate macro_tests;

pub fn main() {
    // `cargo expand` is not able to detect the error
    // in the code expanded from this macro.
    //
    // As such running `tryexpand::expand()` on it will succeed,
    // while running `tryexpand::expand_checked()` will fail.
    expand_to_invalid_code!();
}
