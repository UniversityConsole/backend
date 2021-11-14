mod context;
mod operations;
mod svc;

use context::Context;
use log::LevelFilter;
use operations::create_account::create_account;
use operations::describe_account::describe_account;
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
        create_account(&self.ctx, request.get_ref())
            .await
            .map(|output| Response::new(output))
            .map_err(|err| err.into())
    }

    async fn describe_account(
        &self,
        request: Request<DescribeAccountInput>,
    ) -> Result<Response<DescribeAccountOutput>, Status> {
        describe_account(&self.ctx, request.get_ref())
            .await
            .map(Response::new)
            .map_err(|err| err.into())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    SimpleLogger::new()
        .with_level(LevelFilter::Info)
        .with_module_level(module_path!(), LevelFilter::Debug)
        .init()
        .unwrap();

    let addr = "[::1]:8080".parse().unwrap();
    let ctx = Context::from_env();
    let identity_service = IdentityServiceImpl { ctx };
    let server = IdentityServiceServer::new(identity_service);

    Server::builder().add_service(server).serve(addr).await?;

    Ok(())
}
