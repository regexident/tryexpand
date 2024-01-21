pub fn main() {}

#[test]
fn passing_test() {}

#[test]
#[should_panic]
fn should_panic() {
    panic!("Expected failure");
}
