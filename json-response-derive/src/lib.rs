extern crate proc_macro;
use proc_macro::TokenStream;
mod ctxt;
mod errors;
use quote::quote;

#[proc_macro_derive(ApiError, attributes(request_error, internal_error))]
pub fn derive_api_error(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as syn::DeriveInput);

    errors::expand_derive(&input)
        .unwrap_or_else(to_compile_errors)
        .into()
}

fn to_compile_errors(errors: Vec<syn::Error>) -> proc_macro2::TokenStream {
    let compile_errors = errors.iter().map(syn::Error::to_compile_error);
    quote!(#(#compile_errors)*)
}
