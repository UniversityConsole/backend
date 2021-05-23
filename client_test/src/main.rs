use identity_service_client::IdentityServiceClient;
use identity_service_commons::DescribeAccountInput;

type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client =
        IdentityServiceClient::new("https://fr6f16fdx9.execute-api.eu-west-1.amazonaws.com");
    let account = client
        .describe_account(DescribeAccountInput {
            account_id: uuid::Uuid::parse_str("8f4e9f94-1470-4c2f-9160-c9560eff3cf0").unwrap(),
        })
        .await?;

    println!("{}", account.account.email);

    Ok(())
}
