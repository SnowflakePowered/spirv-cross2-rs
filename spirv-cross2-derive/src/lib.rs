mod derive;

use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(CompilerOptions, attributes(option, expand))]
pub fn compiler_options_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    derive::do_derive(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}
