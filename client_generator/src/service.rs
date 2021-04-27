extern crate proc_macro;

use crate::operation::Operation;
use crate::operation::{create_empty_struct, OpTypeKind};
use crate::token_utils;
use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::spanned::Spanned;
use syn::{Expr, ExprStruct, Ident};

pub struct ServiceDefinition {
    pub service_name: String,
    pub scope: String,
    pub documentation: Option<String>,
    pub operations: Vec<Operation>,
}

enum ServiceFnParam {
    Name,
    HttpScope,
    Documentation,
    Operations,
}

struct InvalidServiceParam<'a>(&'a Ident);

impl ServiceFnParam {
    fn from_ident(value: &Ident) -> std::result::Result<Self, InvalidServiceParam> {
        let value_str = &value.to_string();
        match value_str.as_str() {
            "name" => Ok(ServiceFnParam::Name),
            "http_scope" => Ok(ServiceFnParam::HttpScope),
            "documentation" => Ok(ServiceFnParam::Documentation),
            "operations" => Ok(ServiceFnParam::Operations),
            _ => Err(InvalidServiceParam(&value)),
        }
    }
}

impl Parse for ServiceDefinition {
    fn parse(input: ParseStream) -> Result<Self> {
        let params: ExprStruct = input.parse()?;

        let mut service_name = String::from("");
        let mut http_scope = String::from("");
        let mut documentation = None;
        let mut operations = vec![];

        for field in params.fields.iter() {
            let name = token_utils::member_as_ident(&field.member)?;
            let name = ServiceFnParam::from_ident(&name)
                .map_err(|e| Error::new(e.0.span(), "unknown service parameter"))?;

            match &name {
                &ServiceFnParam::Name => {
                    service_name = token_utils::as_str(&field.expr)?;
                }
                &ServiceFnParam::HttpScope => {
                    http_scope = token_utils::as_str(&field.expr)?;
                }
                &ServiceFnParam::Documentation => {
                    documentation = Some(token_utils::as_str(&field.expr)?);
                }
                &ServiceFnParam::Operations => {
                    operations = parse_operations(&field.expr)?;
                }
            }
        }

        Ok(ServiceDefinition {
            service_name,
            scope: http_scope,
            documentation,
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
            "service does not declare any operations",
        ));
    }

    let mut ops = Vec::<Operation>::new();
    for item in items.elems.iter() {
        let op = crate::operation::from_expr(&item)?;
        ops.push(op);
    }

    Ok(ops)
}

pub fn create_service_client(definition: &ServiceDefinition) -> TokenStream {
    let ServiceDefinition {
        service_name,
        scope,
        operations,
        documentation,
    } = definition;
    let service_client_name = format_ident!("{}Client", service_name);
    let operations_clients = operations.iter().map(create_op_client);
    let documentation = if let Some(doc_str) = &documentation {
        quote! {
            #[doc=#doc_str]
        }
    } else {
        let default_doc = format!("Client for {}", &service_name);
        quote! {
            #[doc=#default_doc]
        }
    };

    let all_kinds: [OpTypeKind; 2] = [OpTypeKind::Input, OpTypeKind::Error];
    let mut empty_structs = vec![];
    for op in operations {
        empty_structs.extend(all_kinds.iter().map(|kind| create_empty_struct(&op, &kind)));
    }

    let service_client = quote! {
        #documentation
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
    let op_input = if let Some(input_ty) = &op.input {
        quote! { input: &#input_ty }
    } else {
        proc_macro2::TokenStream::new()
    };
    let op_doc = if let Some(doc_str) = &op.documentation {
        quote! {
            #[doc = #doc_str]
        }
    } else {
        let default_doc = format!("Execute service operatin {}", &op.name);
        quote! {
            #[doc = #default_doc]
        }
    };

    quote! {
        #op_doc
        pub async fn #op_fn_name(#op_input) -> #op_result {

        }
    }
}

fn get_op_result(op: &Operation) -> proc_macro2::TokenStream {
    let output = if let Some(ty) = &op.output {
        quote! { #ty }
    } else {
        quote! { () }
    };
    let error = op.error_type();
    quote! {
        std::result::Result<#output, #error>
    }
}
