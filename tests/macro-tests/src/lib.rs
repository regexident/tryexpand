#[macro_export]
macro_rules! test_vec {
    () => {
        Vec::new()
    };
    ( $( $x:expr ),+ ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

#[cfg(feature = "test-feature")]
#[cfg_attr(feature = "test-feature", macro_export)]
macro_rules! test_feature_vec {
    () => {
        Vec::new()
    };
    ( $( $x:expr ),+ ) => {
        {
            let mut temp_vec = Vec::new();
            $(
                temp_vec.push($x);
            )*
            temp_vec
        }
    };
}

#[macro_export]
macro_rules! expand_to_valid_code {
    () => {
        let x = 1 + 2;
    };
}

#[cfg(feature = "test-feature")]
#[cfg_attr(feature = "test-feature", macro_export)]
macro_rules! feature_expand_to_valid_code {
    () => {
        let x = 1 + 2;
    };
}

#[macro_export]
macro_rules! expand_to_invalid_code {
    () => {
        let x = 1 + "2";
    };
}

#[cfg(feature = "test-feature")]
#[cfg_attr(feature = "test-feature", macro_export)]
macro_rules! feature_expand_to_invalid_code {
    () => {
        let x = 1 + "2";
    };
}
