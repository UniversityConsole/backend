use thiserror::Error;
use types::Expression;

use crate::resource_access::types::AppendNodeError;

pub mod compiler;
mod parser;
mod string_literal;
pub mod types;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Unknown error.")]
    Unknown,
}

#[derive(Error, Debug)]
pub enum CompileError {
    #[error("Cannot match fields under an any-match.")]
    CannotAppendToAny,

    #[error("Unknown.")]
    Unknown,
}

impl From<AppendNodeError> for CompileError {
    fn from(err: AppendNodeError) -> Self {
        match err {
            AppendNodeError::CannotAppendToAny => CompileError::CannotAppendToAny,
        }
    }
}

pub fn parse(input: &str) -> Result<Expression, ParseError> {
    let (_, expression) = parser::path_set(input).map_err(|_| ParseError::Unknown)?;
    Ok(expression)
}
