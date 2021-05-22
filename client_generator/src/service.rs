extern crate proc_macro;

use crate::operation::Operation;
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

    let service_client = quote! {
        use service_client_runtime::{OperationError, ServiceClient};

        #documentation
        pub struct #service_client_name {
            client: ServiceClient,
        }

        impl #service_client_name {
            pub fn from_env() -> Result<Self, std::env::VarError> {
                Ok(#service_client_name {
                    client: ServiceClient::from_env(#service_client_name::service_name())?,
                })
            }

            pub fn new(endpoint: &str) -> Self {
                #service_client_name {
                    client: ServiceClient {
                        endpoint: String::from(endpoint),
                        service_name: String::from(#service_client_name::service_name()),
                    },
                }
            }

            pub fn service_name() -> &'static str {
                #service_name
            }

            pub fn scope() -> &'static str {
                #scope
            }

            #(#operations_clients)*
        }
    };

    TokenStream::from(service_client)
}

fn create_op_client(op: &Operation) -> proc_macro2::TokenStream {
    let op_fn_name = format_ident!("{}", op.name.to_case(Case::Snake));
    let op_result = get_op_result(&op);
    let op_name = &op.name;
    let op_input = &op.input.as_ref().map_or(
        proc_macro2::TokenStream::new(),
        |input_ty| quote!(input: #input_ty),
    );
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
        pub async fn #op_fn_name(&self, #op_input) -> #op_result {
            self.client.call_service(#op_name, input).await
        }
    }
}

fn get_op_result(op: &Operation) -> proc_macro2::TokenStream {
    let output = &op.output.as_ref().map_or(quote!(()), |ty| quote!(#ty));
    let error = &op.error;
    quote! {
        std::result::Result<#output, OperationError<#error>>
    }
}
