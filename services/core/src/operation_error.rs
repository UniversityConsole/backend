use std::error::Error;

use tonic::Code;

/// Trait to be implemented by errors returned by the different operations of services.
pub trait OperationError: Error {
    /// gRPC code corresponding to this error.
    fn code(&self) -> Code;
}

impl OperationError for ! {
    fn code(&self) -> Code {
        panic!()
    }
}
