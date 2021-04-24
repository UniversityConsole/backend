mod operation;
mod service;
mod token_utils;

extern crate proc_macro;
use proc_macro::TokenStream;

use operation::{create_operation_details, Operation};
use service::{create_service_client, ServiceDefinition};
use syn::parse_macro_input;

#[proc_macro]
pub fn service(input: TokenStream) -> TokenStream {
    let definition = parse_macro_input!(input as ServiceDefinition);
    create_service_client(&definition)
}

#[proc_macro]
pub fn operation(input: TokenStream) -> TokenStream {
    let op = parse_macro_input!(input as Operation);
    create_operation_details(&op)
}
