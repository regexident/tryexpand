#[macro_use]
extern crate macro_tests;
pub fn main() {
    let zero = Vec::new();
    let one = {
        let mut temp_vec = Vec::new();
        temp_vec.push(1);
        temp_vec
    };
    let many = {
        let mut temp_vec = Vec::new();
        temp_vec.push(1);
        temp_vec.push(2);
        temp_vec.push(3);
        temp_vec
    };
}
