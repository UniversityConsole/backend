use lambda_http::{handler, IntoResponse, Request, RequestExt};
use lambda_http::lambda_runtime::{self, Context};

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    lambda_runtime::run(handler(hello)).await?;
    Ok(())
}

async fn hello(request: Request, _: Context) -> Result<impl IntoResponse, Error> {
    Ok(format!(
        "hello {}",
        request
            .query_string_parameters()
            .get("name")
            .unwrap_or_else(|| "stranger")
    ))
}
