# VRC Lighting Controller

VRChat 舞台世界的外置灯光控制程序。通过 90x720 像素画布（仅颜色区）编码灯光参数（角度 + 颜色），经由 NDI / 窗口捕获 / Spout2 推流到 VRChat 世界，Udon 脚本采样像素还原灯光状态。

## 功能特性

- **演出模式（默认）**：键盘/触屏友好的现场布局，10 推子 + 10 旋钮 + 颜色预设（QWERT/ASDFG）+ 状态快照（ZXCVBNM）+ 数字键 1~0 灯具开关；支持"按钮模式"按住=亮，松开=灭
- **备用 UI**：旧版多面板控制台，支持 XY 触摸板、色轮、Cue 列表、效果库、群组等高级编辑
- **响应式 UI**：4 档断点，桌面键鼠优先 / 平板 / 手机均可操作
- **多灯控制**：1-32 个灯具，可编组同步
- **水平/垂直控制**：XY 触摸板 + 旋钮，±180° 范围
- **RGB 颜色**：色轮选色 + 亮度推子
- **频闪引擎**：软件层帧级频闪，速度可调
- **Effect Engine**：内置 5 种效果（颜色循环 / 扫描 / 脉冲 / 波浪 / 随机）
- **Cue 列表**：保存场景快照，一键触发，可设淡入淡出
- **多人协作**：内嵌 Web 服务器，任意设备浏览器即可远程控制
- **状态持久化**：自动保存到本地 JSON，重启恢复
- **跨平台**：Windows / macOS / Linux

## 技术栈

- **后端**：Rust + Tauri 2 + axum (WebSocket)
- **前端**：Vue 3 + TypeScript + Pinia
- **输出**：90x720 RGBA 帧合成（仅颜色区） + NDI + Spout2（Spout2 仅 Windows）

## 快速开始

### 前置条件

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://www.rust-lang.org/tools/install) >= 1.70
- (可选) [NDI 6 SDK](https://ndi.video/tools/ndi-sdk/) + Runtime — NDI 输出需要；编译时需 SDK 头文件（Windows 默认 `C:\Program Files\NDI\NDI 6 SDK`），运行时需将 `Bin\x64` 加入 PATH
- (Windows 可选) Spout2 — 首次构建会自动联网克隆 Spout2 源码（约 1-2 分钟，仅一次）；已本地克隆则设置 `SPOUT2_DIR` 环境变量指向仓库路径即可离线构建

### 安装依赖

```bash
pnpm install
```

### 开发模式

```bash
pnpm tauri dev
```

### 构建发布版

```bash
pnpm tauri build
```

构建产物位于 `src-tauri/target/release/bundle/`。

## 远程控制

启动程序后，同一局域网内的设备在浏览器中访问：

```
http://<你的IP>:9000
```

即可打开完整的灯控台界面，支持多人同时控制。

## 使用文档

每个按钮、每个面板的详细说明，请查看 [docs/usage-guide.md](docs/usage-guide.md)。

## 输出协议

详见 [docs/protocol.md](docs/protocol.md)。

### 简要说明

每灯占 2 个色块（块宽 42，高度按灯数动态填满 720），输出画布 90x720（仅保留色块区域）：
- **Block A**（角度）：R = Tilt 编码，G = Pan 编码
- **Block B**（颜色）：RGB 直接映射灯光颜色 × 亮度

### VRChat 端 Udon 采样

```csharp
// 灯 i 的采样坐标（按 OUTPUT_WIDTH=90, OUTPUT_HEIGHT=720, gap=2, block_w=42）
int blockH = (int)Mathf.Floor((720f - (N + 1) * 2f) / N);
float centerY = 2f + i * (blockH + 2f) + blockH / 2f;
Vector2 blockA_uv = new Vector2(23f / 90f, 1f - centerY / 720f);
Vector2 blockB_uv = new Vector2(67f / 90f, 1f - centerY / 720f);

// 读取颜色
Color angleColor = renderTexture.GetPixel(blockA_uv);
Color lightColor = renderTexture.GetPixel(blockB_uv);

// 解码角度
float tilt = angleColor.r * 360f - 180f;
float pan  = angleColor.g * 360f - 180f;
```

## 项目结构

```
├── src/                    # Vue 3 前端
│   ├── components/         # UI 组件（XYPad, ColorWheel, FaderStrip 等）
│   ├── layouts/            # 布局组件
│   ├── stores/             # Pinia 状态管理
│   ├── api/                # 后端通信适配器（Tauri IPC / WebSocket）
│   └── types/              # TypeScript 类型定义
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── state/          # 状态管理（灯具/编组/预设）
│       ├── engine/         # 引擎（效果/频闪/Cue）
│       ├── render/         # 帧合成器
│       ├── output/         # NDI 输出
│       ├── server/         # Web 服务器 + WebSocket
│       └── persistence.rs  # 状态持久化
└── docs/                   # 文档
```

## 许可证

MIT
