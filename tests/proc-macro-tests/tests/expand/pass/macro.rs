#[macro_use]
extern crate proc_macro_tests;

pub fn main() {
    my_proc_macro! { struct Test; }
    #[cfg(feature = "test-feature")]
    my_feature_proc_macro! { struct Test; }
}
