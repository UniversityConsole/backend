use tonic::{Code, Status};

use crate::operation_error::OperationError;

#[derive(Debug, thiserror::Error)]
pub enum EndpointError<E: OperationError + 'static> {
    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal service error")]
    Internal,

    #[error("operation error: {0}")]
    Operation(#[from] E),
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

impl<E: OperationError> From<EndpointError<E>> for Status {
    fn from(e: EndpointError<E>) -> Self {
        Status::new(e.code(), e.to_string())
    }
}

impl<E: OperationError> EndpointError<E> {
    pub fn validation(msg: impl Into<String>) -> Self {
        EndpointError::Validation(msg.into())
    }

    pub fn internal() -> Self {
        EndpointError::Internal
    }

    pub fn operation(e: impl Into<E>) -> Self {
        EndpointError::Operation(e.into())
    }
}
