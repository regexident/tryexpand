macro_rules! produce_invalid_code {
    () => {
        let _ = 2 + "2";
    };
}

fn main() {
    produce_invalid_code!();
}
