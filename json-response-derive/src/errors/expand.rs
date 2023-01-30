use super::types::JsonErrors;
use crate::ctxt::Ctxt;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{punctuated::Punctuated, token::Comma, Data, DataEnum, Variant};

pub fn expand_derive(input: &syn::DeriveInput) -> Result<TokenStream, Vec<syn::Error>> {
    let ctxt = Ctxt::new();
    let qoute = match &input.data {
        Data::Enum(DataEnum { variants, .. }) => expand_derive_enum(input, variants, &ctxt),
        _ => {
            ctxt.error_spanned_by(input, "Expected `enum`");
            None
        }
    };
    let qoute = match qoute {
        Some(cont) => cont,
        None => return Err(ctxt.check().unwrap_err()),
    };
    ctxt.check()?;
    Ok(qoute)
}

fn expand_derive_enum(
    input: &syn::DeriveInput,
    variants: &Punctuated<Variant, Comma>,
    ctxt: &Ctxt,
) -> Option<TokenStream> {
    let name = &input.ident;
    let json_errors = match JsonErrors::from_variants(name.clone(), variants, ctxt) {
        Some(json_errors) => json_errors,
        None => return None,
    };
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let gen = quote! {
        impl #impl_generics json_response::__private::IntoResponse for #name #ty_generics #where_clause {
            fn into_response(self) -> json_response::__private::Response {
                match self{
                    #json_errors
                }
            }
        }
    };
    Some(gen)
}
