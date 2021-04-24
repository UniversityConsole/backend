extern crate proc_macro;

use crate::token_utils;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Error, Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Expr, Token};

pub struct Operation {
    pub name: String,
    pub input: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
}

enum EmptyStructKind {
    Input,
    Output,
    Error,
}

impl Parse for Operation {
    fn parse(input: ParseStream) -> Result<Self> {
        let params = Punctuated::<Expr, Token![,]>::parse_separated_nonempty(&input)?;
        if params.len() != 4 {
            return Err(Error::new(
                input.span(),
                "expected arguments: operation_name, input, output, error",
            ));
        }

        let operation_name = token_utils::get_identifier(&params[0])?;
        let input = {
            let ident = token_utils::get_identifier(&params[1])?;
            if ident == "void" {
                None
            } else {
                Some(ident)
            }
        };
        let output = {
            let ident = token_utils::get_identifier(&params[2])?;
            if ident == "void" {
                None
            } else {
                Some(ident)
            }
        };
        let error = {
            let ident = token_utils::get_identifier(&params[3])?;
            if ident == "void" {
                None
            } else {
                Some(ident)
            }
        };

        Ok(Operation {
            name: operation_name,
            input,
            output,
            error,
        })
    }
}

pub fn create_operation_details(op: &Operation) -> TokenStream {
    let Operation {
        name,
        input,
        output,
        error,
    } = &op;
    let op_detail_name = format_ident!("Op{}Detail", &name);
    let (empty_input_name, empty_input) = create_op_empty_struct(&op, EmptyStructKind::Input);
    let (empty_output_name, empty_output) = create_op_empty_struct(&op, EmptyStructKind::Output);
    let (empty_error_name, empty_error) = create_op_empty_struct(&op, EmptyStructKind::Error);

    let input_type = if input == &None {
        empty_input_name
    } else {
        format_ident!("{}", input.as_ref().unwrap())
    };
    let output_type = if output == &None {
        empty_output_name
    } else {
        format_ident!("{}", output.as_ref().unwrap())
    };
    let error_type = if error == &None {
        empty_error_name
    } else {
        format_ident!("{}", error.as_ref().unwrap())
    };

    let op_detail = quote! {
        struct #op_detail_name {}

        impl #op_detail_name {
            type Input = #input_type;
            type Output = #output_type;
            type Error = #error_type;

            pub fn operation_name() -> &'static str {
                #name
            }
        }

        #empty_input
        #empty_output
        #empty_error
    };

    TokenStream::from(op_detail)
}

fn create_op_empty_struct(
    op: &Operation,
    kind: EmptyStructKind,
) -> (proc_macro2::Ident, proc_macro2::TokenStream) {
    let (suffix, need_empty_type) = match kind {
        EmptyStructKind::Input => ("Input", op.input == None),
        EmptyStructKind::Output => ("Output", op.output == None),
        EmptyStructKind::Error => ("Error", op.error == None),
    };
    let name = format_ident!("{}Empty{}", &op.name, suffix);

    if !need_empty_type {
        return (name, proc_macro2::TokenStream::new());
    }

    let expanded = quote! {
        struct #name {}
    };
    (name, expanded)
}
