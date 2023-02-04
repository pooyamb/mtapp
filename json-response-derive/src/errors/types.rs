use super::attrs::Attrs;
use crate::ctxt::Ctxt;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use quote::{ToTokens, TokenStreamExt};
use syn::{punctuated::Punctuated, token::Comma, Attribute, Ident, Variant};

/// Type of error enum variants supported
pub enum JsonErrorKind {
    /// A request error, without any extra information.
    /// Is represented as a variant with no value
    NaiveRequest,
    /// A request error, with any extra information
    /// Is represented as a tuple variant with one field
    Request,
    /// An internal error, with any extra information
    /// Is represented as a variant with no value
    NaiveInternal,
    /// An internal error, with any extra information
    /// Is represented as a tuple variant with one field
    Internal,
}

pub struct JsonError {
    ident: Ident,
    kind: JsonErrorKind,
    attrs: Attrs,
}

impl JsonError {
    pub(crate) fn new_internal_error(
        variant: &Variant,
        _attr: &Attribute,
        ctxt: &Ctxt,
    ) -> Option<Self> {
        match variant.fields.len() {
            0 => Some(JsonError {
                ident: variant.ident.clone(),
                kind: JsonErrorKind::NaiveInternal,
                attrs: Attrs::new(),
            }),
            1 => Some(JsonError {
                ident: variant.ident.clone(),
                kind: JsonErrorKind::Internal,
                attrs: Attrs::new(),
            }),
            _ => {
                ctxt.error_spanned_by(
                    variant,
                    "Tuple variants with more than one fields are not supported",
                );
                None
            }
        }
    }

    pub(crate) fn new_request_error(
        variant: &Variant,
        attr: &Attribute,
        ctxt: &Ctxt,
    ) -> Option<Self> {
        let mut attrs = match Attrs::from_attr(attr, ctxt) {
            Some(attrs) => attrs,
            None => return None,
        };
        attrs.set_optional("message");
        let allowed_fields = ["status", "code", "message"];
        for attr in attrs.mut_inner().iter_mut() {
            if !allowed_fields.contains(&attr.ident.as_str()) {
                ctxt.error_spanned_by(
                    &attr.left,
                    format!("Unknown attribute {}", attr.left.get_ident().unwrap()),
                )
            }
            if attr.ident == "message" {
                attr.optional = true;
            }
        }

        match variant.fields.len() {
            0 => Some(JsonError {
                ident: variant.ident.clone(),
                kind: JsonErrorKind::NaiveRequest,
                attrs,
            }),
            1 => Some(JsonError {
                ident: variant.ident.clone(),
                kind: JsonErrorKind::Request,
                attrs,
            }),
            _ => {
                ctxt.error_spanned_by(variant, "Only one error can be wrapped in each variant");
                None
            }
        }
    }

    pub(crate) fn from_variant(variant: &Variant, ctxt: &Ctxt) -> Option<Self> {
        for attr in &variant.attrs {
            let ident = &attr.path.get_ident();
            if let Some(ident) = ident {
                let ident_string = ident.to_string();
                if ident_string == "request_error" {
                    if let Some(json_error) = Self::new_request_error(variant, attr, ctxt) {
                        return Some(json_error);
                    }
                } else if ident_string == "internal_error" {
                    if let Some(json_error) = Self::new_internal_error(variant, attr, ctxt) {
                        return Some(json_error);
                    }
                }
            }
        }
        ctxt.error_spanned_by(
            variant.ident.clone(),
            "All enum variants should have either a request_error or an internal_error attribute",
        );
        None
    }

    pub(crate) fn expand_match_condition(&self, type_ident: &Ident) -> TokenStream {
        let kind = &self.ident;
        match self.kind {
            JsonErrorKind::NaiveRequest => {
                let (fields, values) = self.attrs.expand_unzip();
                quote! {
                    #type_ident::#kind => json_response::JsonError{
                        #(#fields: #values ,)*
                        content: (),
                        ..json_response::JsonError::default()
                    }.into_response()
                }
            }
            JsonErrorKind::Request => {
                let (fields, values) = self.attrs.expand_unzip();
                quote! {
                    #type_ident::#kind(err) => json_response::JsonError{
                        #(#fields: #values),*,
                        content: err,
                        ..json_response::JsonError::default()
                    }.into_response()
                }
            }
            JsonErrorKind::NaiveInternal => quote! {
                #type_ident::#kind => {
                    /// Log the error
                    json_response::__private::error!(
                        "{}::{}",
                        stringify!(#type_ident),
                        stringify!(#kind)
                    );

                    json_response::JsonError{
                        status: json_response::__private::StatusCode::INTERNAL_SERVER_ERROR,
                        code: "50000 internal-error".into(),
                        content: (),
                        ..json_response::JsonError::default()
                    }.into_response()
                }
            },
            JsonErrorKind::Internal => quote! {
                #type_ident::#kind(err) => {
                    /// Log the error
                    json_response::__private::error!(
                        "{}::{} {}",
                        stringify!(#type_ident),
                        stringify!(#kind),
                        err
                    );

                    json_response::JsonError{
                        status: json_response::__private::StatusCode::INTERNAL_SERVER_ERROR,
                        code: "50000 internal-error".into(),
                        content: (),
                        ..json_response::JsonError::default()
                    }.into_response()
                }
            },
        }
    }

    pub(crate) fn expand_utoipa_response(&self, name: &Ident) -> TokenStream {
        match self.kind {
            JsonErrorKind::NaiveRequest => {
                let (key, value) = self.attrs.utoipa_expand_unzip();
                quote! {
                        (
                            stringify!(#name),
                            json_response::__private::utoipa::ResponseBuilder::new()
                                .content(
                                    "application/json",
                                    json_response::__private::utoipa::ContentBuilder::new()
                                        .schema(
                                            json_response::__private::utoipa::ObjectBuilder::new()
                                                #(.property(
                                                    stringify!(#key),
                                                    json_response::__private::utoipa::ObjectBuilder::new()
                                                        .schema_type(json_response::__private::utoipa::SchemaType::Integer)
                                                        .enum_values(Some([#value]))
                                                        .example(Some(#value.into())),
                                                ))*
                                                .build(),
                                        )
                                        .build()
                                        .into(),
                                )
                                .build()
                                .into(),
                        )
                }
            }
            JsonErrorKind::Request => {
                let (fields, values) = self.attrs.utoipa_expand_unzip();
                quote! {
                    (
                        stringify!(#name),
                        json_response::__private::utoipa::ResponseBuilder::new()
                            .content(
                                "application/json",
                                json_response::__private::utoipa::ContentBuilder::new()
                                    .schema(
                                        json_response::__private::utoipa::ObjectBuilder::new()
                                            #(.property(
                                                stringify!(#fields),
                                                json_response::__private::utoipa::ObjectBuilder::new()
                                                    .schema_type(json_response::__private::utoipa::SchemaType::Integer)
                                                    .enum_values(Some([#values]))
                                                    .example(Some(#values.into())),
                                            ))*
                                            .property(
                                                "content",
                                                json_response::__private::utoipa::ObjectBuilder::new(),
                                            )
                                            .build(),
                                    )
                                    .build()
                                    .into(),
                            )
                            .build()
                            .into(),
                    )
                }
            }
            _ => {
                quote! {
                    (
                        "InternalError",
                        json_response::__private::utoipa::ResponseBuilder::new()
                            .content(
                                "application/json",
                                json_response::__private::utoipa::ContentBuilder::new()
                                    .schema(
                                        json_response::__private::utoipa::ObjectBuilder::new()
                                            .property(
                                                "status",
                                                json_response::__private::utoipa::ObjectBuilder::new()
                                                    .schema_type(json_response::__private::utoipa::SchemaType::Integer)
                                                    .enum_values(Some([500]))
                                                    .example(Some(500.into())),
                                            )
                                            .property(
                                                "code",
                                                json_response::__private::utoipa::ObjectBuilder::new()
                                                    .schema_type(json_response::__private::utoipa::SchemaType::String)
                                                    .enum_values(Some(["50000 internal-error"]))
                                                    .example(Some("50000 internal-error".into())),
                                            )
                                            .property(
                                                "content",
                                                json_response::__private::utoipa::ObjectBuilder::new(),
                                            )
                                            .build(),
                                    )
                                    .build()
                                    .into(),
                            )
                            .build()
                            .into(),
                    )
                }
            }
        }
    }
}

pub struct JsonErrors {
    ident: Ident,
    errors: Vec<JsonError>,
}

impl JsonErrors {
    pub(crate) fn from_variants(
        ident: Ident,
        variants: &Punctuated<Variant, Comma>,
        ctxt: &Ctxt,
    ) -> Option<Self> {
        let mut ret = Vec::new();
        for variant in variants.iter() {
            match JsonError::from_variant(variant, ctxt) {
                Some(err) => ret.push(err),
                None => return None,
            }
        }
        Some(Self { ident, errors: ret })
    }

    pub(crate) fn into_utoipa_expand(self) -> JsonErrorUtoipa {
        JsonErrorUtoipa {
            ident: self.ident,
            errors: self.errors,
        }
    }
}

impl ToTokens for JsonErrors {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for err_type in &self.errors {
            let cond = err_type.expand_match_condition(&self.ident);
            let gen = quote!(#cond,);
            tokens.append_all(gen);
        }
    }
}

pub struct JsonErrorUtoipa {
    ident: Ident,
    errors: Vec<JsonError>,
}

impl ToTokens for JsonErrorUtoipa {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for err_type in &self.errors {
            let name = Ident::new(
                &format!("{}{}", self.ident, err_type.ident),
                Span::call_site(),
            );
            let response = err_type.expand_utoipa_response(&name);
            let gen = quote!(
                pub struct #name;
                impl json_response::__private::utoipa::ToResponse<'static> for #name {
                    fn response() -> (&'static str, json_response::__private::utoipa::RefOr<json_response::__private::utoipa::Response>) {
                        #response
                    }
                }
            );
            tokens.append_all(gen);
        }
    }
}
