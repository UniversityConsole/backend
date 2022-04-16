use super::resource_path::Path;

pub struct ResourceAccess {
    pub kind: AccessKind,
    pub path: Path,
}

pub enum AccessKind {
    Query,
    Mutate,
    Subscribe,
}
