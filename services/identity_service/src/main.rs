#![feature(never_type)]

mod context;
mod operations;
mod svc;
mod user_account;

use context::Context;
use log::LevelFilter;
use operations::create_account::create_account;
use operations::describe_account::describe_account;
use operations::list_accounts::list_accounts;
use simple_logger::SimpleLogger;
use svc::identity_service_server::IdentityService;
use svc::identity_service_server::IdentityServiceServer;
use svc::CreateAccountInput;
use svc::CreateAccountOutput;
use svc::DescribeAccountInput;
use svc::DescribeAccountOutput;
use tonic::transport::Server;
use tonic::Request;
use tonic::Response;
use tonic::Status;

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
        log::debug!("got ListAccounts request: {:?}.", &request);

        list_accounts(&self.ctx, &self.ctx.dynamodb_adapter, request.get_ref())
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
