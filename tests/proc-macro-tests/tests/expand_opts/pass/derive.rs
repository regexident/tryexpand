#[macro_use]
extern crate proc_macro_tests;

#[derive(MyDerive)]
struct Test;

#[cfg_attr(feature = "test-feature", derive(MyFeatureDerive))]
struct TestFeature;
