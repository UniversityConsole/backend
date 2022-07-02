use std::sync::LazyLock;

use service_core::resource_access::{AccessKind, PolicyStatement};

use crate::permissions::helper::compose_statement;

/// Permissions given to anonymous entities.
pub static ANONYMOUS_PERMISSIONS: LazyLock<Vec<PolicyStatement>> = LazyLock::new(|| {
    const ALLOWED_MUTATIONS: [&str; 1] = ["authenticate(email: *, password: *)::*"];

    vec![compose_statement(AccessKind::Mutation, ALLOWED_MUTATIONS)]
});
