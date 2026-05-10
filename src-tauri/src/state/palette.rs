use serde::{Deserialize, Serialize};
use super::fixture::{Color, Fixture};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaletteSlot {
    pub color: Option<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StateSnapshot {
    pub label: String,
    pub fixtures: Vec<Fixture>,
}
