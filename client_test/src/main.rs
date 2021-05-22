use identity_service_client::IdentityServiceClient;
use identity_service_commons::ListAccountsInput;
use std::default::Default;

type Error = Box<dyn std::error::Error>;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let client =
        IdentityServiceClient::new("https://fr6f16fdx9.execute-api.eu-west-1.amazonaws.com");
    let accounts = client
        .list_accounts(ListAccountsInput {
            starting_token: Some("bla".to_string()),
            ..ListAccountsInput::default()
        })
        .await?;

    println!("{:#?}", accounts);

    Ok(())
}
