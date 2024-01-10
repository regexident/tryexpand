extern crate proc_macro;
use proc_macro::TokenStream;

use quote::{format_ident, quote};
use syn::{parse_macro_input, DeriveInput};

#[cfg(feature = "test-feature")]
#[proc_macro]
pub fn my_feature_proc_macro(input: TokenStream) -> TokenStream {
    my_proc_macro_impl(input)
}

/// Example of [function-like procedural macro][1].
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#function-like-procedural-macros
#[proc_macro]
pub fn my_proc_macro(input: TokenStream) -> TokenStream {
    my_proc_macro_impl(input)
}

fn my_proc_macro_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = format_ident!("{}ProcMacro", input.ident);

    let tokens = quote! {
        #input

        struct #ident;
    };

    tokens.into()
}

#[cfg(feature = "test-feature")]
#[proc_macro_derive(MyFeatureDerive)]
pub fn my_feature_proc_macro_derive(input: TokenStream) -> TokenStream {
    my_proc_macro_derive_impl(input)
}

/// Example of user-defined [derive mode macro][1]
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#derive-mode-macros
#[proc_macro_derive(MyDerive)]
pub fn my_proc_macro_derive(input: TokenStream) -> TokenStream {
    my_proc_macro_derive_impl(input)
}

fn my_proc_macro_derive_impl(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = format_ident!("{}ProcMacroDerive", input.ident);

    let tokens = quote! {
        struct #ident;
    };

    tokens.into()
}

#[cfg(feature = "test-feature")]
#[proc_macro_attribute]
pub fn my_feature_proc_macro_attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    my_proc_macro_attribute_impl(args, input)
}

/// Example of user-defined [procedural macro attribute][1].
///
/// [1]: https://doc.rust-lang.org/reference/procedural-macros.html#attribute-macros
#[proc_macro_attribute]
pub fn my_proc_macro_attribute(args: TokenStream, input: TokenStream) -> TokenStream {
    my_proc_macro_attribute_impl(args, input)
}

fn my_proc_macro_attribute_impl(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = format_ident!("{}ProcMacroAttribute", input.ident);

    let tokens = quote! {
        #input

        struct #ident;
    };

    tokens.into()
}

#[proc_macro]
pub fn my_proc_macro_panics(_input: TokenStream) -> TokenStream {
    panic!("test")
}

#[proc_macro_derive(MyDerivePanics)]
pub fn my_proc_macro_derive_panics(_input: TokenStream) -> TokenStream {
    panic!("test")
}

#[proc_macro_attribute]
pub fn my_proc_macro_attribute_panics(_args: TokenStream, _input: TokenStream) -> TokenStream {
    panic!("test")
}
