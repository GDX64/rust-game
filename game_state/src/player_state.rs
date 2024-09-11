use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct PlayerState {
    pub name: String,
    pub id: u64,
}
