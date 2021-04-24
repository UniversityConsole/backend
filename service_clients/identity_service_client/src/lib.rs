use client_generator::operation;
use client_generator::service;

service!(
    IdentityService,
    "/identity_service",
    [
        CreateAccount,
        UpdateProfile,
        UpdateCredentials,
        DeactivateAccount
    ]
);

operation!(CreateAccount, CreateAccountInput, CreateAccountOutput, void);

pub struct CreateAccountInput {
    pub email: String,
    pub name: String,
}

pub struct CreateAccountOutput {
    pub account_id: String,
}
