mod parser;
mod string_literal;
mod types;

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("Unknown error.")]
    Unknown,
}

pub fn _parse(input: &str) -> Result<types::Expression<'_>, ParseError> {
    let (_, expression) = parser::path_set(input).map_err(|_| ParseError::Unknown)?;
    Ok(expression)
}