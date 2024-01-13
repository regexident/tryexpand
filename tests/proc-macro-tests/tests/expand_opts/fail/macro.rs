#[macro_use]
extern crate proc_macro_tests;

pub fn main() {
    my_proc_macro_panics! { struct Test; }
}
