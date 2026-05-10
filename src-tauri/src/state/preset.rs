use serde::{Deserialize, Serialize};
use super::fixture::Color;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPreset {
    pub id: String,
    pub name: String,
    pub color: Color,
}
