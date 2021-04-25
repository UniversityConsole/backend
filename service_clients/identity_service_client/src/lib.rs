use client_generator::service;

service!(
    IdentityService,
    "/identity_service",
    [(
        CreateAccount,
        CreateAccountInput,
        CreateAccountOutput,
        CreateAccountError
    ),]
);

pub struct CreateAccountInput {
    pub email: String,
    pub name: String,
}

pub struct CreateAccountOutput {
    pub account_id: String,
}

enum CreateAccountError {
    DuplicateAccount,
}
