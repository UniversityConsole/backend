use std::sync::Arc;

use async_graphql::extensions;

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
        let _document = async_graphql_parser::parse_query(&query)?;
        // let _access_requests = super::graphql_interop::from_document(&document)
        //     .map_err(|e| ServerError::new(e.to_string(), None))?;
        next.run(ctx, query, variables).await
    }
}
