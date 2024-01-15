#[macro_use]
extern crate macro_tests;

pub fn main() {
    let zero = test_vec![];
    let one = test_vec![1];
    let many = test_vec![1, 2, 3];
}
