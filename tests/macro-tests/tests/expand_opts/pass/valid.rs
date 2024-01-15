#[macro_use]
extern crate macro_tests;

pub fn main() {
    let zero = test_feature_vec![];
    let one = test_feature_vec![1];
    let many = test_feature_vec![1, 2, 3];
}
