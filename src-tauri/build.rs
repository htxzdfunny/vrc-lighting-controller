use std::path::Path;

fn main() {
    // Ensure ../dist exists (project root's dist/) so include_dir! doesn't fail.
    // The frontend build (pnpm build) populates it before a full release build.
    let dist = Path::new("../dist");
    if !dist.exists() {
        std::fs::create_dir_all(dist).expect("failed to create dist dir");
    }

    tauri_build::build()
}
