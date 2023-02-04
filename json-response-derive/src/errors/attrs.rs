use crate::ctxt::Ctxt;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{Attribute, Expr, ExprTuple, Lit, Path};

#[derive(Clone)]
pub enum ExprRight {
    Lit(Lit),
    Path(Path),
}

#[derive(Clone)]
pub struct Attr {
    pub ident: String,
    pub left: Path,
    pub right: ExprRight,
    pub optional: bool,
}

#[derive(Clone)]
pub struct Attrs(Vec<Attr>);

impl Attrs {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn from_attr(attr: &Attribute, ctxt: &Ctxt) -> Option<Self> {
        let attrs_tuple = match syn::parse2::<ExprTuple>(attr.tokens.to_owned()) {
            Ok(tuple) => tuple,
            Err(error) => {
                ctxt.syn_error(error);
                return None;
            }
        };
        let attrs = attrs_tuple.elems.into_iter().filter_map(|attr| {
            if let Expr::Assign(expr) = attr {
                if let (Expr::Path(left), Expr::Lit(right)) =
                    (expr.left.as_ref(), expr.right.as_ref())
                {
                    Some(Attr {
                        ident: left.path.get_ident().unwrap().to_string(),
                        left: left.path.clone(),
                        right: ExprRight::Lit(right.lit.clone()),
                        optional: false,
                    })
                } else if let (Expr::Path(left), Expr::Path(right)) =
                    (expr.left.as_ref(), expr.right.as_ref())
                {
                    Some(Attr {
                        ident: left.path.get_ident().unwrap().to_string(),
                        left: left.path.clone(),
                        right: ExprRight::Path(right.path.clone()),
                        optional: false,
                    })
                } else {
                    ctxt.error_spanned_by(expr, "Assignments should be in form of `var = value`.");
                    None
                }
            } else {
                ctxt.error_spanned_by(
                    attr,
                    "Only assignments are allowed to be used in error attributes.",
                );
                None
            }
        });
        Some(Self(attrs.collect()))
    }

    pub(crate) fn set_optional(&mut self, key: &str) {
        for mut attr in &mut self.0 {
            if attr.ident == key {
                attr.optional = true
            }
        }
    }

    pub(crate) fn mut_inner(&mut self) -> &mut Vec<Attr> {
        &mut self.0
    }

    pub(crate) fn expand_unzip(&self) -> (Vec<Path>, Vec<TokenStream>) {
        let vect: &Vec<(Path, TokenStream)> = &self
            .0
            .iter()
            .map(|attr| {
                let right = &attr.right;
                let mut lit = match right {
                    ExprRight::Lit(value) => match value {
                        Lit::Str(_) => quote! {#value.into()},
                        _ => quote! {#value},
                    },
                    ExprRight::Path(path) => quote! {#path},
                };
                if attr.optional {
                    lit = quote! {Some(#lit)}
                }
                (attr.left.clone(), lit)
            })
            .collect();
        let (fields, values): (Vec<Path>, Vec<TokenStream>) = vect.clone().into_iter().unzip();
        (fields, values)
    }

    pub(crate) fn utoipa_expand_unzip(&self) -> (Vec<Path>, Vec<TokenStream>) {
        self.0
            .iter()
            .map(|attr| {
                let right = &attr.right;
                let lit = match right {
                    ExprRight::Lit(value) => {
                        quote! {#value}
                    }
                    ExprRight::Path(path) => quote! {#path.as_u16()},
                };
                (attr.left.clone(), lit)
            })
            .collect::<Vec<(Path, TokenStream)>>()
            .into_iter()
            .unzip()
    }
}
