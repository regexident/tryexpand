pub struct TypeThatRequiresNoFeature;

#[cfg(feature = "test-feature")]
pub struct TypeThatRequiresTestFeature;
