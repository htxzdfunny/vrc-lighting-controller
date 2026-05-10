use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

impl Default for Color {
    fn default() -> Self {
        Self {
            r: 1.0,
            g: 1.0,
            b: 1.0,
        }
    }
}

impl Color {
    pub fn to_rgb_u8(&self, dimmer: f64) -> (u8, u8, u8) {
        let clamp = |v: f64| (v * dimmer).clamp(0.0, 1.0);
        (
            (clamp(self.r) * 255.0).round() as u8,
            (clamp(self.g) * 255.0).round() as u8,
            (clamp(self.b) * 255.0).round() as u8,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fixture {
    pub id: usize,
    pub name: String,
    pub pan: f64,
    pub tilt: f64,
    pub color: Color,
    pub dimmer: f64,
    pub strobe_on: bool,
    pub strobe_speed: f64,
}

impl Fixture {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            name: format!("Light {}", id + 1),
            pan: 0.0,
            tilt: 0.0,
            color: Color::default(),
            dimmer: 1.0,
            strobe_on: false,
            strobe_speed: 5.0,
        }
    }

    pub fn pan_encoded(&self) -> u8 {
        ((self.pan + 180.0) / 360.0 * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8
    }

    pub fn tilt_encoded(&self) -> u8 {
        ((self.tilt + 180.0) / 360.0 * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8
    }

    pub fn color_rgb_u8(&self) -> (u8, u8, u8) {
        self.color.to_rgb_u8(self.dimmer)
    }
}
