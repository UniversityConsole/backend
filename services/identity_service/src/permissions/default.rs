use std::lazy::SyncLazy;

use service_core::resource_access::{AccessKind, PolicyStatement};

use crate::permissions::helper::compose_statement;


/// Permissions given to authenticated entities by default.
pub static DEFAULT_PERMISSIONS: SyncLazy<Vec<PolicyStatement>> = SyncLazy::new(|| {
    const ALLOWED_MUTATIONS: [&str; 1] = ["generateAccessToken(refreshToken: *)::*"];

    vec![compose_statement(AccessKind::Mutation, ALLOWED_MUTATIONS)]
});
