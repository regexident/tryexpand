#[macro_use]
extern crate macro_tests;
pub fn main() {
    {
        let mut temp_vec = Vec::new();
        temp_vec.push(1);
        temp_vec
    };
}