pub use db_adapters::tag_adapter::TagWithName as TagVisible;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TagCreateRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TagUpdateRequest {
    pub name: String,
}
