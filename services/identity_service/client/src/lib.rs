/*
#[macro_use]
use client_generator::service;
use identity_service_commons::{CreateAccountError, CreateAccountInput, CreateAccountOutput};
use identity_service_commons::{UpdateAccountError, UpdateAccountInput, UpdateAccountOutput};
use identity_service_commons::{UpdateCredentialsError, UpdateCredentialsInput};

service!(Service {
    name: "IdentityService",
    http_scope: "/identity_service",
    documentation: "Service for managing the user accounts.",
    operations: [
        Operation {
            name: "CreateAccount",
            input: CreateAccountInput,
            output: CreateAccountOutput,
            error: CreateAccountError,
            documentation: "Create a new user account.",
        },
        Operation {
            name: "UpdateAccount",
            input: UpdateAccountInput,
            output: UpdateAccountOutput,
            error: UpdateAccountError,
            documentation: "Update an existing user account",
        },
        Operation {
            name: "UpdateCredentials",
            input: UpdateCredentialsInput,
            error: UpdateCredentialsError,
            documentation: "Update the credentials for an existing user account",
        },
    ],
});
 */
