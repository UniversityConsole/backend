#[macro_use]
use client_generator::service;

service!(Service {
    name: "IdentityService",
    http_scope: "/identity_service",
    documentation: "Service for managing the user accounts and groups.",
    operations: [Operation {
        name: "CreateAccount",
        input: CreateAccountInput,
        output: CreateAccountOutput,
        error: CreateAccountError,
        documentation: "Create a new user account.",
    },],
});

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
