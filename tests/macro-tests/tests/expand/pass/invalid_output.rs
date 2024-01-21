pub fn main() {
    // `cargo expand` is not able to detect the error
    // in the code expanded from this macro.
    //
    // As such running `cargo expand` on it will succeed,
    // while running `cargo check` will fail.
    macro_tests::expand_to_invalid_code!();
}
