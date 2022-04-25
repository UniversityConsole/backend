#![feature(never_type)]

mod context;
mod operations;
mod svc;
mod user_account;

use context::Context;
use log::LevelFilter;
use operations::create_account::create_account;
use operations::describe_account::describe_account;
use operations::get_permissions::get_permissions;
use operations::list_accounts::list_accounts;
use operations::update_permissions::update_permissions;
use simple_logger::SimpleLogger;
use svc::identity_service_server::{IdentityService, IdentityServiceServer};
use svc::{CreateAccountInput, CreateAccountOutput, DescribeAccountInput, DescribeAccountOutput};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

struct IdentityServiceImpl {
    pub ctx: Context,
}

#[tonic::async_trait]
impl IdentityService for IdentityServiceImpl {
    async fn create_account(
        &self,
        request: Request<CreateAccountInput>,
    ) -> Result<Response<CreateAccountOutput>, Status> {
        create_account(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
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
    let identity_service = IdentityServiceImpl { ctx };
    let server = IdentityServiceServer::new(identity_service);

    Server::builder().add_service(server).serve(addr).await?;

    Ok(())
}
