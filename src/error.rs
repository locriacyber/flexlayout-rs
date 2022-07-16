use crate::Id;

/// item not in context
#[derive(Debug, Clone)]
pub struct ItemNotFound(pub Id);
