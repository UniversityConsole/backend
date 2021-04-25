extern crate proc_macro;

use crate::operation::Operation;
use crate::operation::{create_empty_struct, OpTypeKind};
use crate::token_utils;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
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
        let operations = parse_operations(&params[2])?;

        Ok(ServiceDefinition {
            service_name,
            scope,
            operations,
        })
    }
}

fn parse_operations(expr: &Expr) -> Result<Vec<Operation>> {
    let items = {
        if let Expr::Array(arr) = &expr {
            arr
        } else {
            return Err(Error::new(expr.span(), "expected array"));
        }
    };
    if items.elems.len() == 0 {
        return Err(Error::new(
            expr.span(),
            "service does not expose any operations",
        ));
    }

    let mut ops = Vec::<Operation>::new();
    for item in items.elems.iter() {
        let op_expr = {
            if let Expr::Tuple(tup) = &item {
                tup
            } else {
                return Err(Error::new(item.span(), "operation must be a tuple"));
            }
        };

        let op = crate::operation::from_tuple_expr(&op_expr)?;
        ops.push(op);
    }

    Ok(ops)
}

pub fn create_service_client(definition: &ServiceDefinition) -> TokenStream {
    let ServiceDefinition {
        service_name,
        scope,
        operations,
    } = definition;
    let service_client_name = format_ident!("{}Client", service_name);
    let operations_clients = operations.iter().map(create_op_client);

    let all_kinds: [OpTypeKind; 3] = [OpTypeKind::Input, OpTypeKind::Output, OpTypeKind::Error];
    let mut empty_structs = vec![];
    for op in operations {
        empty_structs.extend(all_kinds.iter().map(|kind| create_empty_struct(&op, &kind)));
    }

    let service_client = quote! {
        struct #service_client_name {}

        impl #service_client_name {
            pub fn service_name() -> &'static str {
                #service_name
            }

            pub fn scope() -> &'static str {
                #scope
            }

            #(#operations_clients)*
        }

        #(#empty_structs)*
    };

    TokenStream::from(service_client)
}

fn create_op_client(op: &Operation) -> proc_macro2::TokenStream {
    let op_fn_name = format_ident!("{}", op.name.to_case(Case::Snake));
    let op_result = get_op_result(&op);
    let op_input = op.input_type();
    let op_output = op.output_type();
    quote! {
        pub async fn #op_fn_name(_: &#op_input) -> #op_result {
            Ok(#op_output {})
        }
    }
}

fn get_op_result(op: &Operation) -> proc_macro2::TokenStream {
    let output = op.output_type();
    let error = op.error_type();
    quote! {
        std::result::Result<#output, #error>
    }
}
