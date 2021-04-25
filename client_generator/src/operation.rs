extern crate proc_macro;

use crate::token_utils;
use quote::{format_ident, quote, IdentFragment};
use syn::parse::{Error, Result};
use syn::spanned::Spanned;
use syn::ExprTuple;

pub struct Operation {
    pub name: String,
    pub input: Option<String>,
    pub output: Option<String>,
    pub error: Option<String>,
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

    pub(crate) fn input_type(&self) -> proc_macro2::Ident {
        if self.needs_empty_type(OpTypeKind::Input) {
            self.empty_type_name(OpTypeKind::Input)
        } else {
            format_ident!("{}", self.input.as_ref().unwrap())
        }
    }

    pub(crate) fn output_type(&self) -> proc_macro2::Ident {
        if self.needs_empty_type(OpTypeKind::Output) {
            self.empty_type_name(OpTypeKind::Output)
        } else {
            format_ident!("{}", self.output.as_ref().unwrap())
        }
    }

    pub(crate) fn error_type(&self) -> proc_macro2::Ident {
        if self.needs_empty_type(OpTypeKind::Error) {
            self.empty_type_name(OpTypeKind::Error)
        } else {
            format_ident!("{}", self.error.as_ref().unwrap())
        }
    }

    fn type_for_kind(&self, kind: OpTypeKind) -> proc_macro2::Ident {
        match kind {
            OpTypeKind::Input => self.input_type(),
            OpTypeKind::Output => self.output_type(),
            OpTypeKind::Error => self.error_type(),
        }
    }
}

pub fn from_tuple_expr(expr: &ExprTuple) -> Result<Operation> {
    if expr.elems.len() != 4 {
        return Err(Error::new(
            expr.span(),
            "expected tuple (name, input_type, output_type, error_type)",
        ));
    }

    let params = &expr.elems;

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

pub fn create_empty_struct(op: &Operation, kind: &OpTypeKind) -> proc_macro2::TokenStream {
    if !op.needs_empty_type(kind.clone()) {
        return proc_macro2::TokenStream::new();
    }

    let name = op.type_for_kind(kind.clone());
    quote! {
        struct #name {}
    }
}
