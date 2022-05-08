use std::lazy::SyncLazy;

use service_core::resource_access::{AccessKind, PolicyStatement};

use crate::permissions::helper::compose_statement;

/// Permissions given to anonymous entities.
pub static ANONYMOUS_PERMISSIONS: SyncLazy<Vec<PolicyStatement>> = SyncLazy::new(|| {
    const ALLOWED_MUTATIONS: [&str; 1] = ["authenticate(email: *, password: *)::*"];

    vec![compose_statement(AccessKind::Mutation, ALLOWED_MUTATIONS)]
});
