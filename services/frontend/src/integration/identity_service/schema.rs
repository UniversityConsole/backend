use async_graphql::{Context, Enum, InputObject, Object, ServerError, SimpleObject, ID};
use identity_service::pb::GetPermissionsInput;
use thiserror::Error;
use tracing_futures::Instrument;

use super::IdentityServiceRef;

#[derive(Clone)]
pub struct UserAccount {
    pub id: ID,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Clone, SimpleObject)]
pub struct RenderedPolicyStatement {
    pub access_kind: AccessKind,
    pub paths: Vec<String>,
}

#[derive(Clone, InputObject)]
pub struct InputPolicyStatement {
    pub access_kind: AccessKind,
    pub paths: Vec<String>,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
pub enum AccessKind {
    Query,
    Mutation,
}

#[derive(Clone, SimpleObject)]
pub struct AuthenticationOutput {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Clone, SimpleObject)]
pub struct GenerateAccessTokenOutput {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(InputObject)]
pub struct CreateAccountParams {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
}

#[derive(Clone, SimpleObject)]
pub struct CreateAccountOutput {
    pub account_id: String,
}

#[derive(Enum, Copy, Clone, Eq, PartialEq)]
#[graphql(remote = "identity_service::pb::AccountState")]
pub enum AccountState {
    PendingActivation,
    Active,
    Deactivated,
}

#[derive(Debug, Error)]
pub enum GraphQLError {
    #[error("Permission denied.")]
    PermissionDenied,

    #[error("An internal error occurred.")]
    Internal,

    #[error(transparent)]
    Operation(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[Object]
impl UserAccount {
    async fn id(&self) -> &ID {
        &self.id
    }

    async fn email(&self) -> &String {
        &self.email
    }

    async fn first_name(&self) -> &String {
        &self.first_name
    }

    async fn last_name(&self) -> &String {
        &self.last_name
    }

    #[tracing::instrument(skip_all)]
    async fn policy_statements(&self, ctx: &Context<'_>) -> Result<Vec<RenderedPolicyStatement>, GraphQLError> {
        let mut identity_service_client = ctx.data_unchecked::<IdentityServiceRef>().clone();
        let request = tonic::Request::new(GetPermissionsInput {
            account_id: self.id.to_string(),
        });
        let output = identity_service_client
            .get_permissions(request)
            .instrument(tracing::info_span!("identity_service::get_permissions"))
            .await
            .map_err(|e| {
                tracing::error!(error = ?&e, "GetPermissions failed.");
                GraphQLError::Operation(e.into())
            })?
            .into_inner();

        let statements = output
            .permissions_document
            .map(|doc| doc.statements.into_iter())
            .ok_or(GraphQLError::Internal)?;


        Ok(statements
            .map(|stmt| {
                use identity_service::pb::policy_statement::AccessKind as ProtobufAccessKind;
                let access_kind = if stmt.access_kind == ProtobufAccessKind::Mutation as i32 {
                    AccessKind::Mutation
                } else {
                    AccessKind::Query
                };

                RenderedPolicyStatement {
                    paths: stmt.paths,
                    access_kind,
                }
            })
            .collect())
    }
}

impl From<GraphQLError> for ServerError {
    fn from(e: GraphQLError) -> Self {
        ServerError::new(e.to_string(), None)
    }
}
