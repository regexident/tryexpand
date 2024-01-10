#[macro_use]
extern crate macro_tests;

pub fn main() {
    test_vec![1, 2, 3];
    test_feature_vec![1, 2, 3];

    invalid![];
}
