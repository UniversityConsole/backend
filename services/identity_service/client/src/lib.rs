#[macro_use]
use client_generator::service;
use commons::{CreateAccountError, CreateAccountInput, CreateAccountOutput};

service!(Service {
    name: "IdentityService",
    http_scope: "/identity_service",
    documentation: "Service for managing the user accounts and groups.",
    operations: [Operation {
        name: "CreateAccount",
        input: CreateAccountInput,
        error: CreateAccountError,
        documentation: "Create a new user account.",
    },],
});
