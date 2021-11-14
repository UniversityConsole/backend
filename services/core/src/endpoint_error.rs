use crate::operation_error::OperationError;
use std::error::Error;
use std::fmt::Display;
use strum::AsRefStr;
use tonic::Code;
use tonic::Status;

#[derive(Debug, AsRefStr)]
pub enum EndpointError<E: OperationError> {
    Validation(String),
    Internal,
    Operation(E),
}

impl<E: OperationError> OperationError for EndpointError<E> {
    fn code(&self) -> tonic::Code {
        match self {
            EndpointError::Validation(_) => Code::InvalidArgument,
            EndpointError::Internal => Code::Internal,
            EndpointError::Operation(e) => e.code(),
        }
    }
}

impl<E: OperationError> Error for EndpointError<E> {}

impl<E: OperationError> Display for EndpointError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let kind: &str = self.as_ref();
        let msg = match self {
            EndpointError::Validation(msg) => msg.clone(),
            EndpointError::Internal => String::from("Internal server error."),
            EndpointError::Operation(err) => err.to_string(),
        };

        write!(f, "{}: {}", kind, msg)
    }
}

impl<E: OperationError> Into<Status> for EndpointError<E> {
    fn into(self) -> Status {
        Status::new(self.code(), self.to_string())
    }
}
