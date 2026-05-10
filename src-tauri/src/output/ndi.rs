use std::sync::Arc;
use crate::state::{AppState, OutputStateKind};
use crate::render::{OUTPUT_WIDTH, OUTPUT_HEIGHT};

pub fn start_ndi_output(state: Arc<AppState>) {
    std::thread::spawn(move || {
        let ndi = match grafton_ndi::NDI::new() {
            Ok(n) => n,
            Err(e) => {
                let mut lighting = state.lighting.write();
                lighting.output_status.ndi.state = OutputStateKind::Error;
                lighting.output_status.ndi.message = Some(format!("NDI SDK 初始化失败: {}", e));
                drop(lighting);
                state.bump_version();
                return;
            }
        };

        let mut sender: Option<grafton_ndi::Sender> = None;
        let mut current_name = String::new();
        let mut swizzle_buf = vec![0u8; OUTPUT_WIDTH * OUTPUT_HEIGHT * 4];

        loop {
            let (enabled, name, fps) = {
                let lighting = state.lighting.read();
                (
                    lighting.config.ndi_enabled,
                    lighting.config.ndi_name.clone(),
                    lighting.config.output_fps.max(1),
                )
            };

            if enabled {
                if sender.is_none() || current_name != name {
                    sender = None;
                    let opts = grafton_ndi::SenderOptions::builder(&name)
                        .clock_video(false)
                        .build();
                    match grafton_ndi::Sender::new(&ndi, &opts) {
                        Ok(s) => {
                            sender = Some(s);
                            current_name = name.clone();
                            let mut lighting = state.lighting.write();
                            lighting.output_status.ndi.state = OutputStateKind::Active;
                            lighting.output_status.ndi.message = Some(format!("NDI 输出: {}", name));
                            drop(lighting);
                            state.bump_version();
                        }
                        Err(e) => {
                            let mut lighting = state.lighting.write();
                            lighting.output_status.ndi.state = OutputStateKind::Error;
                            lighting.output_status.ndi.message = Some(format!("NDI Sender 创建失败: {}", e));
                            drop(lighting);
                            state.bump_version();
                        }
                    }
                }

                if let Some(ref s) = sender {
                    let valid = {
                        let frame_data = state.frame_buffer.read();
                        if frame_data.len() == OUTPUT_WIDTH * OUTPUT_HEIGHT * 4 {
                            for i in 0..OUTPUT_WIDTH * OUTPUT_HEIGHT {
                                let si = i * 4;
                                swizzle_buf[si] = frame_data[si + 2];     // B
                                swizzle_buf[si + 1] = frame_data[si + 1]; // G
                                swizzle_buf[si + 2] = frame_data[si];     // R
                                swizzle_buf[si + 3] = frame_data[si + 3]; // A
                            }
                            true
                        } else {
                            false
                        }
                    };
                    if valid {
                        let frame = grafton_ndi::VideoFrame::builder()
                            .resolution(OUTPUT_WIDTH as i32, OUTPUT_HEIGHT as i32)
                            .pixel_format(grafton_ndi::PixelFormat::BGRA)
                            .frame_rate(fps as i32, 1)
                            .build()
                            .expect("VideoFrame builder should not fail");
                        let mut frame = frame;
                        frame.data.copy_from_slice(&swizzle_buf);
                        s.send_video(&frame);
                    }

                    // 查询 NDI 接收端连接数（非阻塞），仅在变化时 bump_version
                    if let Ok(count) = s.connection_count(std::time::Duration::from_millis(0)) {
                        let count = count as u64;
                        let mut lighting = state.lighting.write();
                        if lighting.connected_clients.ndi != count {
                            lighting.connected_clients.ndi = count;
                            drop(lighting);
                            state.bump_version();
                        }
                    }
                }
            } else {
                if sender.is_some() {
                    drop(sender);
                    sender = None;
                    current_name.clear();
                }
                let mut lighting = state.lighting.write();
                lighting.output_status.ndi.state = OutputStateKind::Disabled;
                lighting.output_status.ndi.message = Some("NDI 未启用".into());
                lighting.connected_clients.ndi = 0;
                drop(lighting);
                state.bump_version();
            }

            let frame_interval_ms = (1000u64 / fps as u64).max(1);
            std::thread::sleep(std::time::Duration::from_millis(frame_interval_ms));
        }
    });
}
