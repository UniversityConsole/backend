use std::error::Error;

pub trait ServiceError: Error {
    #[must_use]
    fn http_code(&self) -> u16;
}
