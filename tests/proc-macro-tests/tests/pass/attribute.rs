#[macro_use]
extern crate proc_macro_tests;

#[my_proc_macro_attribute]
struct Test;

#[my_feature_proc_macro_attribute]
struct TestFeature;
