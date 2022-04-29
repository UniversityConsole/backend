use service_core::resource_access::string_interop::compiler::from_string;

use crate::user_account::RenderedPolicyStatement;

pub(crate) fn validate_resource_paths<'a>(
    statements: impl IntoIterator<Item = &'a RenderedPolicyStatement>,
) -> Result<(), (usize, usize)> {
    for (stmt_idx, stmt) in statements.into_iter().enumerate() {
        for (path_idx, path) in stmt.paths.iter().enumerate() {
            from_string(&path).map_err(|_| (stmt_idx, path_idx))?;
        }
    }

    Ok(())
}
