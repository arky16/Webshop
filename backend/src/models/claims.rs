use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims{
    pub sub: String,
    pub role: String,
    pub exp: u64
}