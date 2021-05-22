use client_generator::service;
use identity_service_commons::{CreateAccountError, CreateAccountInput, CreateAccountOutput};
use identity_service_commons::{DescribeAccountError, DescribeAccountInput, DescribeAccountOutput};
use identity_service_commons::{ListAccountsError, ListAccountsInput, ListAccountsOutput};

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
            name: "ListAccounts",
            input: ListAccountsInput,
            output: ListAccountsOutput,
            error: ListAccountsError,
            documentation: "List all existing user accounts.",
        },
        Operation {
            name: "DescribeAccount",
            input: DescribeAccountInput,
            output: DescribeAccountOutput,
            error: DescribeAccountError,
            documentation: "Describe an existing account, given its unique identifier.",
        },
    ],
});
