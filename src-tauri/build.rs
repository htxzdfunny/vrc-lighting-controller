use std::path::Path;

fn main() {
    // Ensure ../dist exists (project root's dist/) so include_dir! doesn't fail.
    // The frontend build (pnpm build) populates it before a full release build.
    let dist = Path::new("../dist");
    if !dist.exists() {
        std::fs::create_dir_all(dist).expect("failed to create dist dir");
    }

    // Embed the short git commit hash at compile time
    let git_hash = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into());
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    tauri_build::build()
}
