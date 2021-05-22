use convert_case::{Case, Casing};
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::env::{var, VarError};
use std::fmt::Debug;

#[derive(Debug)]
pub enum OperationError<E: Debug> {
    ClientErr(reqwest::Error),
    ServiceErr(E),
}

pub struct ServiceClient {
    pub endpoint: String,
    pub service_name: String,
}

impl ServiceClient {
    pub fn from_env(service_name: &str) -> Result<Self, VarError> {
        const ENDPOINT_VAR_NAME: &str = "UC_HTTP_ENDPOINT";

        Ok(ServiceClient {
            endpoint: var(ENDPOINT_VAR_NAME)?,
            service_name: String::from(service_name),
        })
    }

    pub async fn call_service<T, U, E>(
        &self,
        operation: &str,
        input: T,
    ) -> Result<U, OperationError<E>>
    where
        T: Serialize,
        U: DeserializeOwned,
        E: DeserializeOwned + std::fmt::Debug,
    {
        const OP_HDR_NAME: &str = "X-Uc-Operation";

        let client = reqwest::Client::new();
        let service_name = self.service_name.to_case(Case::Kebab);
        let req = client
            .post(self.service_endpoint(&service_name))
            .header(OP_HDR_NAME, String::from(operation))
            .json(&input)
            .build()
            .unwrap();
        let res = client
            .execute(req)
            .await
            .map_err(|e| OperationError::ClientErr(e))?;

        if res.status().is_success() {
            Ok(res
                .json::<U>()
                .await
                .map_err(|e| OperationError::ClientErr(e))?)
        } else {
            Err(res
                .json::<E>()
                .await
                .map_err(|e| OperationError::ClientErr(e))
                .map(|e| OperationError::ServiceErr(e))?)
        }
    }

    fn service_endpoint(&self, service_name: &String) -> String {
        format!("{}/{}", &self.endpoint, service_name)
    }
}

impl<E: Debug> std::fmt::Display for OperationError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Operation error.")
    }
}

impl<E: Debug> std::error::Error for OperationError<E> {}
