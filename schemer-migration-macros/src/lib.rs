extern crate proc_macro;

use collect::MigrationData;
use proc_macro::TokenStream;
use proc_macro2::{Ident, Span, TokenStream as TokenStream2};
use quote::quote;
use syn::{parse::Parse, LitStr, Token};

mod collect;

type BoxError = Box<dyn std::error::Error>;

struct PathCrate {
    crate_name: Option<LitStr>,
    path: LitStr,
}

impl Parse for PathCrate {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let path: LitStr = input.parse()?;
        if input.peek(Token![=>]) {
            input.parse::<Token![=>]>()?;
            let crate_name: LitStr = input.parse()?;

            Ok(Self {
                crate_name: Some(crate_name),
                path,
            })
        } else {
            Ok(Self {
                crate_name: None,
                path,
            })
        }
    }
}

#[proc_macro]
pub fn include_migrations_dir(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as PathCrate);

    let path = collect::resolve_path(input.path.value(), input.path.span()).expect("Error");
    let migrations = collect::read_migrations(path).expect("Error");

    expand_migrations(
        input
            .crate_name
            .map(|v| Ident::new(&v.value(), v.span()))
            .unwrap_or_else(|| Ident::new("mtapp", Span::call_site())),
        migrations.iter(),
    )
    .into()
}

fn expand_migrations<'a>(
    crate_name: Ident,
    migrations: impl Iterator<Item = &'a collect::MigrationData>,
) -> TokenStream2 {
    let mut blocks = Vec::new();
    let mut idents = Vec::new();

    for migration in migrations {
        let (block, ident) = expand_migration(crate_name.clone(), migration);
        blocks.push(block);
        idents.push(ident);
    }

    if blocks.len() > 0 {
        quote!({
            #(#blocks)*
            let res: Vec<Box<dyn #crate_name::Migration>> = vec![
                #(
                    Box::new(#idents),
                )*
            ];
            Some(res)
        })
    } else {
        quote!(None)
    }
}

fn expand_migration(crate_name: Ident, m: &collect::MigrationData) -> (TokenStream2, Ident) {
    let ident = quote::format_ident!("__Migration_{}", &m.name);

    let MigrationData {
        name,
        description,
        dependencies,
        up,
        down,
    } = m;
    let dependencies = dependencies.iter().map(|d| d.to_string());

    (
        quote!(
            struct #ident;

            impl #crate_name::Migration for #ident{
                fn name(&self) -> std::borrow::Cow<'static, str> {#name.into()}
                fn description(&self) -> &'static str {#description}
                fn dependencies(&self) -> std::collections::HashSet<#crate_name::MigrationId> {
                    let mut set = std::collections::HashSet::new();
                    #(set.insert(#crate_name::MigrationId::try_from(#dependencies).expect("Invalid deps"));)*
                    set
                }
                fn up(&self) -> std::borrow::Cow<'static, str> {#up.into()}
                fn down(&self) -> std::borrow::Cow<'static, str> {#down.into()}
            };

        ),
        ident,
    )
}
