use crate::state::{EffectType, EffectState};
use crate::state::fixture::Color;

pub fn apply_effect(
    base_color: &Color,
    effect: &EffectState,
    fixture_index: usize,
    total_fixtures: usize,
    time_secs: f64,
) -> Color {
    if effect.effect_type == EffectType::None {
        return base_color.clone();
    }

    let speed = effect.speed;

    let phase_per_fixture = if total_fixtures > 1 {
        effect.phase_offset * (fixture_index as f64) / (total_fixtures as f64 - 1.0)
    } else {
        0.0
    };

    let t = time_secs * speed + phase_per_fixture;
    let intensity = effect.intensity;

    match effect.effect_type {
        EffectType::ColorCycle => {
            let hue = (t * 360.0) % 360.0;
            let (r, g, b) = hsl_to_rgb(hue, 1.0, 0.5);
            Color {
                r: lerp(base_color.r, r, intensity),
                g: lerp(base_color.g, g, intensity),
                b: lerp(base_color.b, b, intensity),
            }
        }
        EffectType::Scan => {
            let wave = ((t * std::f64::consts::TAU).sin() * 0.5 + 0.5) * intensity;
            Color {
                r: base_color.r * wave,
                g: base_color.g * wave,
                b: base_color.b * wave,
            }
        }
        EffectType::Pulse => {
            let pulse = ((t * std::f64::consts::TAU).sin().abs()) * intensity;
            Color {
                r: base_color.r * (1.0 - intensity + pulse),
                g: base_color.g * (1.0 - intensity + pulse),
                b: base_color.b * (1.0 - intensity + pulse),
            }
        }
        EffectType::Wave => {
            let wave = (t * std::f64::consts::TAU).sin() * 0.5 + 0.5;
            Color {
                r: base_color.r * lerp(1.0 - intensity, 1.0, wave),
                g: base_color.g * lerp(1.0 - intensity, 1.0, wave),
                b: base_color.b * lerp(1.0 - intensity, 1.0, wave),
            }
        }
        EffectType::Random => {
            let seed = (t * 7.0).floor() as u64;
            let hash = simple_hash(seed, fixture_index as u64);
            let brightness = (hash as f64 / u64::MAX as f64) * intensity;
            Color {
                r: base_color.r * (1.0 - intensity + brightness),
                g: base_color.g * (1.0 - intensity + brightness),
                b: base_color.b * (1.0 - intensity + brightness),
            }
        }
        EffectType::None => base_color.clone(),
    }
}

fn lerp(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (f64, f64, f64) {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (r + m, g + m, b + m)
}

fn simple_hash(a: u64, b: u64) -> u64 {
    let mut h = a.wrapping_mul(6364136223846793005).wrapping_add(b);
    h = h ^ (h >> 33);
    h = h.wrapping_mul(0xff51afd7ed558ccd);
    h = h ^ (h >> 33);
    h
}
