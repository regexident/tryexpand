#[macro_use]
extern crate proc_macro_tests;

#[derive(MyDerive)]
struct Test;

#[derive(MyFeatureDerive)]
struct TestFeature;
