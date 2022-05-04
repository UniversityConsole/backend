use std::sync::Arc;

use async_graphql::{extensions, ServerError};

use crate::resource_access::graphql_interop::parser::from_document;

pub struct Authorizer;

impl extensions::ExtensionFactory for Authorizer {
    fn create(&self) -> Arc<dyn extensions::Extension> {
        Arc::new(AuthorizerExtension::default())
    }
}

#[derive(Default)]
pub struct AuthorizerExtension;

#[async_trait::async_trait]
impl extensions::Extension for AuthorizerExtension {
    async fn parse_query(
        &self,
        ctx: &extensions::ExtensionContext<'_>,
        query: &str,
        variables: &async_graphql::Variables,
        next: extensions::NextParseQuery<'_>,
    ) -> async_graphql::ServerResult<async_graphql_parser::types::ExecutableDocument> {
        let document = async_graphql_parser::parse_query(&query)?;
        let mut access_requests = from_document(&document).map_err(|e| ServerError::new(e.to_string(), None))?;
        // FIXME Add support for multi-operation documents.
        let access_request = access_requests
            .pop()
            .ok_or_else(|| ServerError::new("No access request was compiled.", None))?;

        println!("access_request: {:?}", access_request);

        next.run(ctx, query, variables).await
    }
}