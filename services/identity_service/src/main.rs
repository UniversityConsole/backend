#![feature(never_type, let_else, once_cell)]

extern crate core;

mod context;
mod operations;
mod permissions;
mod user_account;
mod utils;

use context::Context;
use identity_service::pb::identity_service_server::{IdentityService, IdentityServiceServer};
use identity_service::pb::{
    AuthenticateInput, AuthenticateOutput, AuthorizeInput, AuthorizeOutput, CreateAccountInput, CreateAccountOutput,
    DescribeAccountInput, DescribeAccountOutput, GenerateAccessTokenInput, GenerateAccessTokenOutput,
    GetPermissionsInput, GetPermissionsOutput, ListAccountsInput, ListAccountsOutput, UpdatePermissionsInput,
    UpdatePermissionsOutput,
};
use log::LevelFilter;
use memcache::Url;
use operations::authorize::authorize;
use operations::create_account::create_account;
use operations::describe_account::describe_account;
use operations::get_permissions::get_permissions;
use operations::list_accounts::list_accounts;
use operations::update_permissions::update_permissions;
use simple_logger::SimpleLogger;
use thiserror::Error;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::context::ContextKey;
use crate::operations::authenticate::authenticate;
use crate::operations::generate_access_token::generate_access_token;
use crate::user_account::ddb_repository::DdbAccountsRepository;
use crate::user_account::AccountsRepository;
use crate::utils::memcache::MemcacheConnPool;

trait ThreadSafeAccountsRepository: AccountsRepository + Send + Sync {}
impl<T: AccountsRepository + Send + Sync> ThreadSafeAccountsRepository for T {}


struct IdentityServiceImpl<T: ThreadSafeAccountsRepository> {
    pub ctx: Context,
    pub refresh_token_cache: MemcacheConnPool,
    pub accounts_repository: T,
}

#[derive(Debug, Error)]
enum ServiceInitError {
    #[error("Context value {0} is missing from environment.")]
    MissingContextValue(ContextKey),

    #[error("Invalid URL: {0}.")]
    InvalidUrl(String),

    #[error("Creating an r2d2 connection pool failed: {0}.")]
    ConnectionPool(r2d2::Error),
}

impl<T: ThreadSafeAccountsRepository> IdentityServiceImpl<T> {
    fn new(ctx: Context, accounts_repository: T) -> Result<Self, ServiceInitError> {
        let endpoint = Url::parse(ctx.refresh_token_cache.as_ref())
            .map_err(|_| ServiceInitError::InvalidUrl(ctx.refresh_token_cache.clone()))?;
        let connection_manager = memcache::ConnectionManager::new(endpoint);
        let refresh_token_cache =
            MemcacheConnPool::new(connection_manager).map_err(ServiceInitError::ConnectionPool)?;

        Ok(Self {
            ctx,
            refresh_token_cache,
            accounts_repository,
        })
    }
}

#[tonic::async_trait]
impl<T: 'static + ThreadSafeAccountsRepository> IdentityService for IdentityServiceImpl<T> {
    async fn create_account(
        &self,
        request: Request<CreateAccountInput>,
    ) -> Result<Response<CreateAccountOutput>, Status> {
        create_account(&self.accounts_repository, request.into_inner())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn describe_account(
        &self,
        request: Request<DescribeAccountInput>,
    ) -> Result<Response<DescribeAccountOutput>, Status> {
        describe_account(&self.accounts_repository, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn list_accounts(&self, request: Request<ListAccountsInput>) -> Result<Response<ListAccountsOutput>, Status> {
        list_accounts(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn update_permissions(
        &self,
        request: Request<UpdatePermissionsInput>,
    ) -> Result<Response<UpdatePermissionsOutput>, Status> {
        update_permissions(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn get_permissions(
        &self,
        request: Request<GetPermissionsInput>,
    ) -> Result<Response<GetPermissionsOutput>, Status> {
        get_permissions(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn authorize(&self, request: Request<AuthorizeInput>) -> Result<Response<AuthorizeOutput>, Status> {
        authorize(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn authenticate(
        &self,
        mut request: Request<AuthenticateInput>,
    ) -> Result<Response<AuthenticateOutput>, Status> {
        authenticate(
            &self.ctx,
            &self.ctx.dynamodb_adapter,
            &self.refresh_token_cache,
            request.get_mut(),
        )
        .await
        .map(Response::new)
        .map_err(|err| err.into())
    }

    async fn generate_access_token(
        &self,
        mut request: Request<GenerateAccessTokenInput>,
    ) -> Result<Response<GenerateAccessTokenOutput>, Status> {
        generate_access_token(
            &self.ctx,
            &self.ctx.dynamodb_adapter,
            &self.refresh_token_cache,
            request.get_mut(),
        )
        .await
        .map(Response::new)
        .map_err(|err| err.into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(LevelFilter::Warn)
        .with_module_level(module_path!(), LevelFilter::Debug)
        .init()
        .unwrap();

    let addr = "0.0.0.0:8080".parse().unwrap();
    let ctx = Context::from_env().await;
    let accounts_repository = DdbAccountsRepository::new(ctx.dynamodb_adapter.clone(), ctx.accounts_table_name.clone());
    let identity_service = IdentityServiceImpl::new(ctx, accounts_repository)?;
    let server = IdentityServiceServer::new(identity_service);

    Server::builder().add_service(server).serve(addr).await?;

    Ok(())
}
