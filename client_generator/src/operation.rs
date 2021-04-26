extern crate proc_macro;

use crate::token_utils;
use quote::{format_ident, quote, IdentFragment};
use std::default::Default;
use syn::parse::{Error, Result};
use syn::{Expr, ExprStruct, Ident, Path};

#[derive(Default)]
pub struct Operation {
    pub name: String,
    pub documentation: Option<String>,
    pub input: Option<Path>,
    pub output: Option<Path>,
    pub error: Option<Path>,
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

#[derive(Copy, Clone)]
pub enum OpTypeKind {
    Input,
    Output,
    Error,
}

impl ToString for OpTypeKind {
    fn to_string(&self) -> String {
        match self {
            Self::Input => "Input",
            Self::Output => "Output",
            Self::Error => "Error",
        }
        .to_string()
    }
}

impl IdentFragment for OpTypeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl Operation {
    fn empty_type_name(&self, kind: OpTypeKind) -> proc_macro2::Ident {
        let suffix = match kind {
            OpTypeKind::Input => "Input",
            OpTypeKind::Output => "Output",
            OpTypeKind::Error => "Error",
        };
        format_ident!("{}Empty{}", &self.name, suffix)
    }

    fn needs_empty_type(&self, kind: OpTypeKind) -> bool {
        match kind {
            OpTypeKind::Input => self.input.is_none(),
            OpTypeKind::Output => self.output.is_none(),
            OpTypeKind::Error => self.output.is_none(),
        }
    }

    pub(crate) fn input_type(&self) -> Path {
        if self.needs_empty_type(OpTypeKind::Input) {
            self.empty_type_name(OpTypeKind::Input).into()
        } else {
            self.input.clone().unwrap()
        }
    }

    pub(crate) fn output_type(&self) -> Path {
        if self.needs_empty_type(OpTypeKind::Output) {
            self.empty_type_name(OpTypeKind::Output).into()
        } else {
            self.output.clone().unwrap()
        }
    }

    pub(crate) fn error_type(&self) -> Path {
        if self.needs_empty_type(OpTypeKind::Error) {
            self.empty_type_name(OpTypeKind::Error).into()
        } else {
            self.error.clone().unwrap()
        }
    }

    fn type_for_kind(&self, kind: OpTypeKind) -> Path {
        match kind {
            OpTypeKind::Input => self.input_type(),
            OpTypeKind::Output => self.output_type(),
            OpTypeKind::Error => self.error_type(),
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
        let name = OperationFnParam::from_ident(&name)
            .map_err(|e| Error::new(e.0.span(), "unknown operation parameter"))?;
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
                op.error = Some(token_utils::as_path(&field.expr)?.path.clone());
            }
        }
    }

    Ok(op)
}

pub fn create_empty_struct(op: &Operation, kind: &OpTypeKind) -> proc_macro2::TokenStream {
    if !op.needs_empty_type(kind.clone()) {
        return proc_macro2::TokenStream::new();
    }

    let name = op.type_for_kind(kind.clone());
    quote! {
        struct #name {}
    }
}
