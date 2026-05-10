pub mod layout;

use std::sync::Arc;
use std::time::{Duration, Instant};
use crate::state::AppState;
use crate::state::EffectType;
use crate::engine::effect::apply_effect;
use crate::engine::strobe::is_strobe_visible;
use layout::BlockLayout;

pub const OUTPUT_WIDTH: usize = 90;
pub const OUTPUT_HEIGHT: usize = 720;
pub const BLOCK_GAP: usize = 2;

pub fn run_render_loop(state: Arc<AppState>) {
    let target_interval = Duration::from_micros(16_667);

    loop {
        let start = Instant::now();
        render_frame(&state);
        let elapsed = start.elapsed();
        if elapsed < target_interval {
            std::thread::sleep(target_interval - elapsed);
        }
    }
}

fn render_frame(state: &AppState) {
    let lighting = state.lighting.read();
    let fixture_count = lighting.config.fixture_count.min(lighting.fixtures.len());
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64();

    let mut buffer = vec![0u8; OUTPUT_WIDTH * OUTPUT_HEIGHT * 4];

    for i in 0..fixture_count {
        let fixture = &lighting.fixtures[i];
        let layout = BlockLayout::for_fixture(i, fixture_count);

        let r_a = fixture.tilt_encoded();
        let g_a = fixture.pan_encoded();
        fill_block(&mut buffer, layout.a_x, layout.a_y, layout.block_w, layout.block_h, r_a, g_a, 0, 255);

        let effective_color = if lighting.effect.effect_type != EffectType::None {
            apply_effect(
                &fixture.color,
                &lighting.effect,
                i,
                fixture_count,
                now_secs,
            )
        } else {
            fixture.color.clone()
        };

        let strobe_visible = is_strobe_visible(
            fixture.strobe_on,
            fixture.strobe_speed,
            now_secs,
        );

        if !fixture.is_on {
            fill_block(&mut buffer, layout.b_x, layout.b_y, layout.block_w, layout.block_h, 0, 0, 0, 255);
        } else if strobe_visible {
            let (r, g, b) = effective_color.to_rgb_u8(fixture.dimmer);
            fill_block(&mut buffer, layout.b_x, layout.b_y, layout.block_w, layout.block_h, r, g, b, 255);
        } else {
            fill_block(&mut buffer, layout.b_x, layout.b_y, layout.block_w, layout.block_h, 0, 0, 0, 255);
        }
    }

    let mut fb = state.frame_buffer.write();
    fb.copy_from_slice(&buffer);
}

fn fill_block(buffer: &mut [u8], bx: usize, by: usize, w: usize, h: usize, r: u8, g: u8, b: u8, a: u8) {
    for dy in 0..h {
        for dx in 0..w {
            let px = bx + dx;
            let py = by + dy;
            if px < OUTPUT_WIDTH && py < OUTPUT_HEIGHT {
                let idx = (py * OUTPUT_WIDTH + px) * 4;
                buffer[idx] = r;
                buffer[idx + 1] = g;
                buffer[idx + 2] = b;
                buffer[idx + 3] = a;
            }
        }
    }
}
