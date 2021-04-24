extern crate proc_macro;
use proc_macro::TokenStream;

use crate::operation::Operation;
use crate::token_utils;
use quote::{format_ident, quote};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

pub struct ServiceDefinition {
    pub service_name: String,
    pub scope: String,
    pub operations: Vec<Operation>,
}

impl Parse for ServiceDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let params = Punctuated::<Expr, Token![,]>::parse_separated_nonempty(&input)?;
        if params.len() != 3 {
            return Err(Error::new(
                input.span(),
                "expected arguments: service_name, scope, operations",
            ));
        }

        let service_name = token_utils::get_identifier(&params[0])?;
        let scope = token_utils::get_str(&params[1])?;
        let operations = vec![];

        Ok(ServiceDefinition {
            service_name,
            scope,
            operations,
        })
    }
}

pub fn create_service_client(definition: &ServiceDefinition) -> TokenStream {
    let ServiceDefinition {
        service_name,
        scope,
        operations: _,
    } = definition;
    let service_client_name = format_ident!("{}Client", service_name);

    let service_client = quote! {
        struct #service_client_name {}

        impl #service_client_name {
            pub fn service_name() -> &'static str {
                #service_name
            }

            pub fn scope() -> &'static str {
                #scope
            }
        }
    };

    TokenStream::from(service_client)
}
