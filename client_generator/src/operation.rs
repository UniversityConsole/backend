extern crate proc_macro;

use std::default::Default;

use quote::quote;
use syn::parse::{Error, Result};
use syn::punctuated::Punctuated;
use syn::token::Colon2;
use syn::{Expr, ExprStruct, Ident, Path, PathSegment};

use crate::token_utils;

pub struct Operation {
    pub name: String,
    pub documentation: Option<String>,
    pub input: Option<Path>,
    pub output: Option<Path>,
    pub error: Path,
}

#[non_exhaustive]
pub enum OperationFnParam {
    Name,
    Documentation,
    Input,
    Output,
    Error,
}

pub struct InvalidOperationParam<'a>(&'a Ident);

impl Default for Operation {
    fn default() -> Self {
        Operation {
            name: String::default(),
            documentation: None,
            input: None,
            output: None,
            error: Path {
                leading_colon: None,
                segments: Punctuated::<PathSegment, Colon2>::default(),
            },
        }
    }
}

impl OperationFnParam {
    pub fn from_ident(ident: &Ident) -> std::result::Result<Self, InvalidOperationParam> {
        let ident_str = &ident.to_string();
        match ident_str.as_str() {
            "name" => Ok(OperationFnParam::Name),
            "documentation" => Ok(OperationFnParam::Documentation),
            "input" => Ok(OperationFnParam::Input),
            "output" => Ok(OperationFnParam::Output),
            "error" => Ok(OperationFnParam::Error),
            _ => Err(InvalidOperationParam(&ident)),
        }
    }
}

pub fn from_expr(expr: &Expr) -> Result<Operation> {
    let mut op = Operation::default();

    // TODO Find a way not to convert back to TokenStream
    let token_stream = quote! { #expr };
    let expr: ExprStruct = syn::parse2(token_stream)?;

    for field in &expr.fields {
        let name = token_utils::member_as_ident(&field.member)?;
        let name =
            OperationFnParam::from_ident(&name).map_err(|e| Error::new(e.0.span(), "unknown operation parameter"))?;
        match name {
            OperationFnParam::Name => {
                op.name = token_utils::as_str(&field.expr)?;
            }
            OperationFnParam::Documentation => {
                op.documentation = Some(token_utils::as_str(&field.expr)?);
            }
            OperationFnParam::Input => {
                op.input = Some(token_utils::as_path(&field.expr)?.path.clone());
            }
            OperationFnParam::Output => {
                op.output = Some(token_utils::as_path(&field.expr)?.path.clone());
            }
            OperationFnParam::Error => {
                op.error = token_utils::as_path(&field.expr)?.path.clone();
            }
        }
    }

    Ok(op)
}
