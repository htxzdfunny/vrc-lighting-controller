import fs from "node:fs";
import path from "node:path";
import process from "node:process";
import { fileURLToPath } from "node:url";

// Runs as Tauri `beforeBundleCommand`, i.e. AFTER cargo build and BEFORE the bundler.
// Stages all native libraries the bundler can't infer so the resulting installer is
// fully self-contained on end-user machines.
//
//   - Spout: built by `rust-spout2` during cargo build → copied from target/<profile>/.
//   - NDI:   shipped as a system runtime; copied from the dev machine's NDI SDK or PATH.
//            Redistributing the NDI runtime requires complying with the NDI redistribution
//            license terms (https://ndi.video/sdk/eula/).

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const rootDir = path.resolve(__dirname, "..");
const tauriDir = path.join(rootDir, "src-tauri");
const runtimeRoot = path.join(tauriDir, "resources", "runtime", "win-x64");
const spoutTarget = path.join(runtimeRoot, "spout");
const ndiTarget = path.join(runtimeRoot, "ndi");

const SPOUT_DLL_NAMES = ["Spout.dll", "SpoutLibrary.dll"];
const NDI_DLL_NAME = "Processing.NDI.Lib.x64.dll";

function ensureDir(dir) {
  fs.mkdirSync(dir, { recursive: true });
}

function clearDllFiles(dir) {
  if (!fs.existsSync(dir)) return;
  for (const name of fs.readdirSync(dir)) {
    const full = path.join(dir, name);
    if (fs.statSync(full).isFile() && name.toLowerCase().endsWith(".dll")) {
      fs.rmSync(full, { force: true });
    }
  }
}

function detectProfile() {
  const explicit = process.env.VRC_BUNDLE_PROFILE;
  if (explicit) return explicit;
  if (process.env.TAURI_ENV_DEBUG === "true") return "debug";
  return "release";
}

function pickTargetDir(profile) {
  const candidates = [
    process.env.CARGO_TARGET_DIR
      ? path.join(process.env.CARGO_TARGET_DIR, profile)
      : null,
    path.join(tauriDir, "target", profile),
    path.join(rootDir, "target", profile),
  ].filter(Boolean);
  for (const dir of candidates) {
    if (fs.existsSync(dir) && fs.statSync(dir).isDirectory()) return dir;
  }
  return null;
}

function unique(values) {
  const seen = new Set();
  const out = [];
  for (const v of values) {
    if (!v) continue;
    const key = v.toLowerCase();
    if (seen.has(key)) continue;
    seen.add(key);
    out.push(v);
  }
  return out;
}

function ndiDirsFromEnv() {
  const explicit = [
    process.env.VRC_RUNTIME_NDI_DIR,
    process.env.NDI_RUNTIME_DIR,
  ];
  const sdkRoots = [
    process.env.NDI_SDK_DIR,
    process.env.NDI_RUNTIME_DIR_V6,
    process.env.NDI_RUNTIME_DIR_V5,
    process.env.NDI_RUNTIME_DIR_V4,
  ].filter(Boolean);
  const fromRoots = sdkRoots.flatMap((root) => [
    path.join(root, "Bin", "x64"),
    path.join(root, "x64"),
    root,
  ]);
  const programFiles = [
    process.env["ProgramFiles"],
    process.env["ProgramFiles(x86)"],
    "C:\\Program Files",
    "C:\\Program Files (x86)",
  ].filter(Boolean);
  const common = programFiles.flatMap((pf) => [
    path.join(pf, "NDI", "NDI 6 SDK", "Bin", "x64"),
    path.join(pf, "NDI", "NDI 5 SDK", "Bin", "x64"),
    path.join(pf, "NDI", "NDI 4 SDK", "Bin", "x64"),
    path.join(pf, "NDI", "NDI 6 Runtime", "v6"),
    path.join(pf, "NDI", "NDI 5 Runtime", "v5"),
    path.join(pf, "NDI", "NDI 4 Runtime", "v4"),
  ]);
  return unique([...explicit, ...fromRoots, ...common]);
}

function pathDirs() {
  const raw = process.env.PATH || process.env.Path || "";
  return raw.split(path.delimiter).filter(Boolean);
}

function findNdiDll() {
  const candidateDirs = [...ndiDirsFromEnv(), ...pathDirs()];
  for (const dir of candidateDirs) {
    if (!dir) continue;
    const full = path.join(dir, NDI_DLL_NAME);
    try {
      if (fs.existsSync(full) && fs.statSync(full).isFile()) return full;
    } catch {
      // ignore probe failures (invalid PATH entries, permission errors)
    }
  }
  return null;
}

if (process.platform !== "win32") {
  console.log("[runtime-deps] Non-Windows host detected; nothing to stage.");
  process.exit(0);
}

const profile = detectProfile();
const targetDir = pickTargetDir(profile);

ensureDir(spoutTarget);
ensureDir(ndiTarget);
clearDllFiles(spoutTarget);
clearDllFiles(ndiTarget);

if (!targetDir) {
  console.error(
    `[runtime-deps] 找不到 cargo 输出目录（profile=${profile}）。请先执行 cargo/tauri build。`
  );
  process.exit(1);
}

const missingSpout = [];
for (const name of SPOUT_DLL_NAMES) {
  const src = path.join(targetDir, name);
  if (!fs.existsSync(src)) {
    missingSpout.push(name);
    continue;
  }
  fs.copyFileSync(src, path.join(spoutTarget, name));
}

if (missingSpout.length > 0) {
  console.error(
    `[runtime-deps] 在 ${targetDir} 找不到 Spout 产物: ${missingSpout.join(", ")}。\n` +
      "通常意味着 rust-spout2 构建失败，请检查 cargo build 输出。"
  );
  process.exit(1);
}

const ndiSource = findNdiDll();
if (!ndiSource) {
  console.error(
    `[runtime-deps] 找不到 ${NDI_DLL_NAME}。请先安装 NDI Tools/SDK，或设置环境变量:\n` +
      "  - NDI_SDK_DIR        指向 NDI SDK 根目录\n" +
      "  - VRC_RUNTIME_NDI_DIR 直接指向包含该 DLL 的目录\n"
  );
  process.exit(1);
}
fs.copyFileSync(ndiSource, path.join(ndiTarget, NDI_DLL_NAME));

const manifest = {
  generatedAt: new Date().toISOString(),
  profile,
  spout: {
    source: targetDir,
    copied: SPOUT_DLL_NAMES,
  },
  ndi: {
    source: ndiSource,
    copied: [NDI_DLL_NAME],
  },
};
fs.writeFileSync(
  path.join(runtimeRoot, "runtime-manifest.json"),
  `${JSON.stringify(manifest, null, 2)}\n`,
  "utf8"
);

console.log(`[runtime-deps] Profile: ${profile}`);
console.log(`[runtime-deps] Spout source: ${targetDir}`);
console.log(`[runtime-deps] Spout copied -> ${spoutTarget}`);
console.log(`[runtime-deps] NDI source: ${ndiSource}`);
console.log(`[runtime-deps] NDI copied  -> ${ndiTarget}`);
