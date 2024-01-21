macro_rules! produce_valid_code {
    () => {
        let _ = 2 + 2;
    };
}

fn main() {
    produce_valid_code!();
}
