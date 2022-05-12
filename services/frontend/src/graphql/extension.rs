use std::sync::Arc;

use async_graphql::{extensions, ServerError};
use identity_service::pb::AuthorizeInput;
use service_core::resource_access::graphql_interop::parser::from_document;
use tracing_futures::Instrument;

use crate::integration::identity_service::schema::GraphQLError;
use crate::integration::identity_service::IdentityServiceRef;
use crate::schema::authorization::Authorization;
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
    #[tracing::instrument(skip_all)]
    async fn parse_query(
        &self,
        ctx: &extensions::ExtensionContext<'_>,
        query: &str,
        variables: &async_graphql::Variables,
        next: extensions::NextParseQuery<'_>,
    ) -> async_graphql::ServerResult<async_graphql_parser::types::ExecutableDocument> {
        let document = next.run(ctx, query, variables).await?;

        let mut access_requests = from_document(&document).map_err(|e| ServerError::new(e.to_string(), None))?;
        // FIXME Add support for multi-operation documents.
        let access_request = access_requests
            .pop()
            .ok_or_else(|| ServerError::new("No access request was compiled.", None))?;

        let account_id = ctx
            .data_unchecked::<Option<Authorization>>()
            .as_ref()
            .map(|v| v.claims.sub.clone());

        let mut identity_service_client = ctx.data_unchecked::<IdentityServiceRef>().clone();
        let request = tonic::Request::new(AuthorizeInput {
            account_id,
            access_request: Some(access_request.into()),
        });
        let output = identity_service_client
            .authorize(request)
            .instrument(tracing::info_span!("identity_service::authorize"))
            .await
            .map_err(|e| {
                tracing::error!(error = ?&e, "Authorize failed.");
                ServerError::from(GraphQLError::Internal)
            })?
            .into_inner();
        if !output.permission_granted {
            return Err(GraphQLError::PermissionDenied.into());
        }

        Ok(document)
    }
}
