use std::num::NonZeroU32;
use std::sync::Arc;
use glow::HasContext;
use crate::state::{AppState, OutputStateKind};
use crate::render::{OUTPUT_WIDTH, OUTPUT_HEIGHT};

#[cfg(windows)]
mod windows_gl {
    use windows::Win32::Foundation::{HWND, HINSTANCE, HMODULE, GetLastError};
    use windows::Win32::Graphics::Gdi::{HDC, GetDC, ReleaseDC};
    use windows::Win32::Graphics::OpenGL::{
        HGLRC, wglCreateContext, wglMakeCurrent, wglDeleteContext, wglGetProcAddress,
        ChoosePixelFormat, SetPixelFormat, PIXELFORMATDESCRIPTOR,
        PFD_TYPE_RGBA, PFD_DRAW_TO_WINDOW, PFD_SUPPORT_OPENGL,
        PFD_DOUBLEBUFFER,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        RegisterClassA, CreateWindowExA, DestroyWindow, DefWindowProcA,
        WNDCLASSA, CS_OWNDC,
    };
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, LoadLibraryA, GetProcAddress};

    unsafe extern "system" fn spout_wndproc(
        hwnd: HWND,
        msg: u32,
        wparam: windows::Win32::Foundation::WPARAM,
        lparam: windows::Win32::Foundation::LPARAM,
    ) -> windows::Win32::Foundation::LRESULT {
        DefWindowProcA(hwnd, msg, wparam, lparam)
    }

    pub struct SpoutGl {
        hwnd: HWND,
        hdc: HDC,
        hglrc: HGLRC,
        _h_gl: HMODULE, // keep opengl32.dll loaded
        pub gl: glow::Context,
    }

    impl Drop for SpoutGl {
        fn drop(&mut self) {
            unsafe {
                let _ = wglMakeCurrent(HDC::default(), HGLRC::default());
                let _ = wglDeleteContext(self.hglrc);
                let _ = ReleaseDC(self.hwnd, self.hdc);
                let _ = DestroyWindow(self.hwnd);
            }
        }
    }

    pub unsafe fn create_spout_gl() -> Result<SpoutGl, String> {
        // Step 1: Load opengl32.dll
        let h_gl = LoadLibraryA(windows::core::s!("opengl32.dll"))
            .map_err(|e| format!("opengl32.dll 加载失败: {e}"))?;

        // Step 2: Detach any stale GL context on this thread
        let _ = wglMakeCurrent(HDC::default(), HGLRC::default());

        // Step 3: Register window class with proper WNDPROC
        let class_name = windows::core::s!("SpoutGL");
        let wc = WNDCLASSA {
            style: CS_OWNDC,
            lpfnWndProc: Some(spout_wndproc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: GetModuleHandleA(None).unwrap_or_default().into(),
            hIcon: Default::default(),
            hCursor: Default::default(),
            hbrBackground: Default::default(),
            lpszMenuName: windows::core::s!(""),
            lpszClassName: class_name,
        };
        let atom = RegisterClassA(&wc);
        if atom == 0 {
            let err = GetLastError();
            if err.0 != 1410 {
                return Err(format!("RegisterClassA 失败, GetLastError={}", err.0));
            }
        }

        // Step 4: Create 1x1 hidden window
        let hwnd = CreateWindowExA(
            Default::default(),
            class_name,
            class_name,
            Default::default(),
            0, 0, 1, 1,
            HWND::default(),
            None,
            HINSTANCE::default(),
            None,
        ).map_err(|e| format!("CreateWindowExA 失败: {e}"))?;

        // Step 5: Get device context
        let hdc = GetDC(hwnd);
        if hdc.is_invalid() {
            let _ = DestroyWindow(hwnd);
            return Err("GetDC 返回无效 HDC".into());
        }

        // Step 6: Choose and set pixel format
        let pfd = PIXELFORMATDESCRIPTOR {
            nSize: std::mem::size_of::<PIXELFORMATDESCRIPTOR>() as u16,
            nVersion: 1,
            dwFlags: PFD_DRAW_TO_WINDOW | PFD_SUPPORT_OPENGL | PFD_DOUBLEBUFFER,
            iPixelType: PFD_TYPE_RGBA,
            cColorBits: 32,
            cAlphaBits: 8,
            ..Default::default()
        };
        let pf = ChoosePixelFormat(hdc, &pfd);
        if pf == 0 {
            let _ = ReleaseDC(hwnd, hdc);
            let _ = DestroyWindow(hwnd);
            return Err("ChoosePixelFormat 失败, 没有兼容的像素格式".into());
        }
        if let Err(e) = SetPixelFormat(hdc, pf, &pfd) {
            let _ = ReleaseDC(hwnd, hdc);
            let _ = DestroyWindow(hwnd);
            return Err(format!("SetPixelFormat 失败: {e}"));
        }

        // Step 7: Create and activate GL context
        let hglrc = wglCreateContext(hdc)
            .map_err(|e| format!("wglCreateContext 失败: {e}"))?;

        if let Err(e) = wglMakeCurrent(hdc, hglrc) {
            let _ = wglDeleteContext(hglrc);
            let _ = ReleaseDC(hwnd, hdc);
            let _ = DestroyWindow(hwnd);
            return Err(format!("wglMakeCurrent 失败: {e}"));
        }

        // Step 8: Create glow GL context via GetProcAddress on opengl32.dll
        let gl = {
            let h = h_gl;
            glow::Context::from_loader_function(|name| {
                let c_name = std::ffi::CString::new(name).unwrap();
                // wglGetProcAddress resolves GL 1.2+ and extension entry points.
                // It must be tried first; GetProcAddress only covers GL 1.1 core.
                let ptr = unsafe {
                    match wglGetProcAddress(windows::core::PCSTR(c_name.as_ptr() as _)) {
                        Some(f) => f as *const _,
                        None => std::ptr::null::<std::ffi::c_void>(),
                    }
                };
                if !ptr.is_null() {
                    return ptr;
                }
                // Fall back to GetProcAddress for GL 1.1 core functions.
                unsafe {
                    match GetProcAddress(h, windows::core::PCSTR(c_name.as_ptr() as _)) {
                        Some(f) => f as *const _,
                        None => std::ptr::null(),
                    }
                }
            })
        };

        Ok(SpoutGl { hwnd, hdc, hglrc, _h_gl: h_gl, gl })
    }
}

#[cfg(windows)]
pub fn start_spout_output(state: Arc<AppState>) {
    std::thread::spawn(move || {
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_spout_loop(state);
        }));
        if let Err(e) = result {
            let msg = e.downcast_ref::<String>()
                .map(|s| s.as_str())
                .or_else(|| e.downcast_ref::<&str>().copied())
                .unwrap_or("unknown panic");
            log::error!("[Spout] 线程 panic，已终止: {}", msg);
        }
    });
}

#[cfg(windows)]
fn run_spout_loop(state: Arc<AppState>) {
        let mut spout: Option<rust_spout2::Spout> = None;
        let mut gl: Option<windows_gl::SpoutGl> = None;
        let mut texture_id: u32 = 0;
        let mut sender_created = false;
        let mut setup_done = false;
        let mut active = false;
        let mut current_name = String::new();
        let mut local_buf = vec![0u8; OUTPUT_WIDTH * OUTPUT_HEIGHT * 4];

        loop {
            let (enabled, name, fps) = {
                let lighting = state.lighting.read();
                (
                    lighting.config.spout_enabled,
                    lighting.config.spout_name.clone(),
                    lighting.config.output_fps.max(1),
                )
            };

            let frame_interval_ms = (1000u64 / fps as u64).max(1);

            // Lazy initialization: only attempt setup when enabled and not yet done
            if enabled && !setup_done {
                log::info!("[Spout] 开始初始化, spout_enabled={}, name={}", enabled, name);

                // Step 1: Create Spout library handle
                match rust_spout2::Spout::new() {
                    Some(s) => {
                        log::info!("[Spout] Spout::new() 成功");
                        spout = Some(s);
                    }
                    None => {
                        log::error!("[Spout] Spout::new() 返回 None — Spout2 SDK 未安装或 DLL 缺失");
                        let mut lighting = state.lighting.write();
                        lighting.output_status.spout.state = OutputStateKind::Error;
                        lighting.output_status.spout.message = Some("Spout2 SDK 未安装".into());
                        drop(lighting);
                        state.bump_version();
                        std::thread::sleep(std::time::Duration::from_millis(frame_interval_ms));
                        continue;
                    }
                }

                // Step 2: Create OpenGL context
                match unsafe { windows_gl::create_spout_gl() } {
                    Ok(ctx) => {
                        log::info!("[Spout] OpenGL 上下文创建成功");
                        gl = Some(ctx);
                    }
                    Err(e) => {
                        log::error!("[Spout] OpenGL 上下文创建失败: {}", e);
                        spout = None;
                        let mut lighting = state.lighting.write();
                        lighting.output_status.spout.state = OutputStateKind::Error;
                        lighting.output_status.spout.message = Some(format!("OpenGL: {}", e));
                        drop(lighting);
                        state.bump_version();
                        std::thread::sleep(std::time::Duration::from_millis(frame_interval_ms));
                        continue;
                    }
                }

                // Step 3: Create GL texture
                let gl_ctx = &gl.as_ref().unwrap().gl;
                unsafe {
                    texture_id = gl_ctx.create_texture()
                        .map(|t| (t.0).into())
                        .unwrap_or(0);
                    gl_ctx.bind_texture(glow::TEXTURE_2D, NonZeroU32::new(texture_id).map(glow::NativeTexture));
                    gl_ctx.tex_image_2d(
                        glow::TEXTURE_2D, 0, glow::RGBA as i32,
                        OUTPUT_WIDTH as i32, OUTPUT_HEIGHT as i32, 0,
                        glow::RGBA, glow::UNSIGNED_BYTE,
                        glow::PixelUnpackData::Slice(Some(&local_buf)),
                    );
                    // Without explicit filter params the texture is "incomplete" (default
                    // GL_NEAREST_MIPMAP_LINEAR requires mipmaps we never generate).
                    // An incomplete texture always reads as black, which is the root cause
                    // of the Spout2 black-screen bug.
                    gl_ctx.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, glow::LINEAR as i32);
                    gl_ctx.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, glow::LINEAR as i32);
                    gl_ctx.bind_texture(glow::TEXTURE_2D, None);
                }
                log::info!("[Spout] GL 纹理创建成功, texture_id={}", texture_id);

                // Step 4: Set sender name and create Spout sender
                let spout_ref = spout.as_mut().unwrap();
                let c_name = std::ffi::CString::new(name.as_str()).unwrap();
                unsafe { spout_ref.as_pin_mut().SetSenderName(c_name.as_ptr()) };
                log::info!("[Spout] SetSenderName(\"{}\"), 准备 CreateSender {}x{}", name, OUTPUT_WIDTH, OUTPUT_HEIGHT);
                let ok = unsafe {
                    spout_ref.as_pin_mut().CreateSender(
                        std::ptr::null(),
                        autocxx::c_uint(OUTPUT_WIDTH as u32),
                        autocxx::c_uint(OUTPUT_HEIGHT as u32),
                        autocxx::c_ulong(0),
                    )
                };
                if !ok {
                    log::error!("[Spout] CreateSender 返回 false");
                    spout = None;
                    gl = None;
                    texture_id = 0;
                    let mut lighting = state.lighting.write();
                    lighting.output_status.spout.state = OutputStateKind::Error;
                    lighting.output_status.spout.message = Some("Spout2 CreateSender 失败".into());
                    drop(lighting);
                    state.bump_version();
                    std::thread::sleep(std::time::Duration::from_millis(frame_interval_ms));
                    continue;
                }

                // All setup succeeded
                sender_created = true;
                setup_done = true;
                current_name = name.clone();
                active = true;
                log::info!("[Spout] 初始化全部完成, sender=\"{}\", {}x{}", current_name, OUTPUT_WIDTH, OUTPUT_HEIGHT);
                let mut lighting = state.lighting.write();
                lighting.output_status.spout.state = OutputStateKind::Active;
                lighting.output_status.spout.message = Some(format!("Spout2 输出: {}", current_name));
                drop(lighting);
                state.bump_version();
            }

            if enabled && setup_done {
                let spout_ref = spout.as_mut().unwrap();
                let gl_ctx = &gl.as_ref().unwrap().gl;

                // When sender name changes: release old sender and create new one
                if current_name != name {
                    log::info!("[Spout] sender 名称变更: \"{}\" → \"{}\"", current_name, name);
                    unsafe { spout_ref.as_pin_mut().ReleaseSender(autocxx::c_ulong(0)) };
                    let c_name = std::ffi::CString::new(name.as_str()).unwrap();
                    unsafe { spout_ref.as_pin_mut().SetSenderName(c_name.as_ptr()) };
                    let ok = unsafe {
                        spout_ref.as_pin_mut().CreateSender(
                            std::ptr::null(),
                            autocxx::c_uint(OUTPUT_WIDTH as u32),
                            autocxx::c_uint(OUTPUT_HEIGHT as u32),
                            autocxx::c_ulong(0),
                        )
                    };
                    if !ok {
                        log::error!("[Spout] 重新 CreateSender 失败");
                        active = false;
                        let mut lighting = state.lighting.write();
                        lighting.output_status.spout.state = OutputStateKind::Error;
                        lighting.output_status.spout.message = Some("Spout2 重新创建 Sender 失败".into());
                        drop(lighting);
                        state.bump_version();
                    } else {
                        log::info!("[Spout] 重新 CreateSender 成功: \"{}\"", name);
                        current_name = name.clone();
                        active = true;
                        let mut lighting = state.lighting.write();
                        lighting.output_status.spout.state = OutputStateKind::Active;
                        lighting.output_status.spout.message = Some(format!("Spout2 输出: {}", current_name));
                        drop(lighting);
                        state.bump_version();
                    }
                }

                if active {
                    // Copy frame from shared buffer
                    {
                        let frame_data = state.frame_buffer.read();
                        if frame_data.len() == OUTPUT_WIDTH * OUTPUT_HEIGHT * 4 {
                            local_buf.copy_from_slice(&frame_data);
                        }
                    }

                    // Upload to GL texture and send via Spout2
                    unsafe {
                        gl_ctx.bind_texture(glow::TEXTURE_2D, NonZeroU32::new(texture_id).map(glow::NativeTexture));
                        gl_ctx.tex_image_2d(
                            glow::TEXTURE_2D, 0, glow::RGBA as i32,
                            OUTPUT_WIDTH as i32, OUTPUT_HEIGHT as i32, 0,
                            glow::RGBA, glow::UNSIGNED_BYTE,
                            glow::PixelUnpackData::Slice(Some(&local_buf)),
                        );
                        gl_ctx.bind_texture(glow::TEXTURE_2D, None);
                        // Flush ensures the tex_image_2d upload is committed to the GPU
                        // before Spout2 reads the texture handle.
                        gl_ctx.flush();

                        spout_ref.as_pin_mut().SendTexture(
                            autocxx::c_uint(texture_id),
                            autocxx::c_uint(glow::TEXTURE_2D),
                            autocxx::c_uint(OUTPUT_WIDTH as u32),
                            autocxx::c_uint(OUTPUT_HEIGHT as u32),
                            false,
                            autocxx::c_uint(0),
                        );
                    }
                }
            }

            if !enabled {
                if setup_done {
                    log::info!("[Spout] 禁用, 清理资源");
                    // Explicitly release sender before dropping Spout
                    if sender_created {
                        if let Some(ref mut s) = spout {
                            unsafe { s.as_pin_mut().ReleaseSender(autocxx::c_ulong(0)) };
                        }
                        sender_created = false;
                    }
                    active = false;
                    current_name.clear();

                    // Delete GL texture before dropping GL context
                    if texture_id != 0 {
                        if let Some(ref g) = gl {
                            if let Some(tid) = NonZeroU32::new(texture_id) {
                                unsafe { g.gl.delete_texture(glow::NativeTexture(tid)); }
                            }
                        }
                        texture_id = 0;
                    }

                    gl = None;
                    spout = None;
                    setup_done = false;
                }

                let mut lighting = state.lighting.write();
                lighting.output_status.spout.state = OutputStateKind::Disabled;
                lighting.output_status.spout.message = Some("Spout2 未启用".into());
                drop(lighting);
                state.bump_version();
            }

            std::thread::sleep(std::time::Duration::from_millis(frame_interval_ms));
        }
}

#[cfg(not(windows))]
pub fn start_spout_output(state: Arc<AppState>) {
    std::thread::spawn(move || {
        {
            let mut lighting = state.lighting.write();
            lighting.output_status.spout.state = OutputStateKind::Disabled;
            lighting.output_status.spout.message = Some("当前平台不支持 Spout2".into());
        }
        state.bump_version();
    });
}
