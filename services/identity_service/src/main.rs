#![feature(never_type, let_else)]
#![allow(dead_code)]

extern crate core;

mod context;
mod operations;
mod svc;
mod user_account;
mod utils;

use context::Context;
use log::LevelFilter;
use memcache::Url;
use operations::authorize::authorize;
use operations::create_account::create_account;
use operations::describe_account::describe_account;
use operations::get_permissions::get_permissions;
use operations::list_accounts::list_accounts;
use operations::update_permissions::update_permissions;
use simple_logger::SimpleLogger;
use svc::identity_service_server::{IdentityService, IdentityServiceServer};
use svc::{CreateAccountInput, CreateAccountOutput, DescribeAccountInput, DescribeAccountOutput};
use thiserror::Error;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use crate::context::ContextKey;
use crate::operations::authenticate::authenticate;
use crate::utils::memcache::MemcacheConnPool;

struct IdentityServiceImpl {
    pub ctx: Context,
    pub refresh_token_cache: MemcacheConnPool,
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

impl IdentityServiceImpl {
    fn new(ctx: Context) -> Result<Self, ServiceInitError> {
        let endpoint = Url::parse(ctx.refresh_token_cache.as_ref())
            .map_err(|_| ServiceInitError::InvalidUrl(ctx.refresh_token_cache.clone()))?;
        let connection_manager = memcache::ConnectionManager::new(endpoint);
        let refresh_token_cache =
            MemcacheConnPool::new(connection_manager).map_err(|e| ServiceInitError::ConnectionPool(e))?;

        Ok(Self {
            ctx,
            refresh_token_cache,
        })
    }
}

#[tonic::async_trait]
impl IdentityService for IdentityServiceImpl {
    async fn create_account(
        &self,
        mut request: Request<CreateAccountInput>,
    ) -> Result<Response<CreateAccountOutput>, Status> {
        create_account(&self.ctx, &self.ctx.dynamodb_adapter, request.get_mut())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn describe_account(
        &self,
        request: Request<DescribeAccountInput>,
    ) -> Result<Response<DescribeAccountOutput>, Status> {
        describe_account(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn list_accounts(
        &self,
        request: Request<svc::ListAccountsInput>,
    ) -> Result<Response<svc::ListAccountsOutput>, Status> {
        list_accounts(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn update_permissions(
        &self,
        request: Request<svc::UpdatePermissionsInput>,
    ) -> Result<Response<svc::UpdatePermissionsOutput>, Status> {
        update_permissions(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn get_permissions(
        &self,
        request: Request<svc::GetPermissionsInput>,
    ) -> Result<Response<svc::GetPermissionsOutput>, Status> {
        get_permissions(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn authorize(&self, request: Request<svc::AuthorizeInput>) -> Result<Response<svc::AuthorizeOutput>, Status> {
        authorize(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }

    async fn authenticate(
        &self,
        mut request: Request<svc::AuthenticateInput>,
    ) -> Result<Response<svc::AuthenticateOutput>, Status> {
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
    let identity_service = IdentityServiceImpl::new(ctx)?;
    let server = IdentityServiceServer::new(identity_service);

    Server::builder().add_service(server).serve(addr).await?;

    Ok(())
}
