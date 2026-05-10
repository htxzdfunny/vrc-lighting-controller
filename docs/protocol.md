# VRC Lighting Controller - 输出像素协议

## 概述

本软件通过 **90x720** 像素画布（仅保留有颜色区）输出灯光控制数据。每个灯具占用 **2 个色块**，VRChat 端的 Udon 脚本通过采样视频播放器渲染纹理上的对应像素点来读取灯光参数。

## 画布规格

- 分辨率：**90 x 720**（仅包含色块区域，无大面积黑边）
- 背景色：黑色 (0, 0, 0)
- 帧率：60fps
- 输出方式：NDI / 窗口捕获 / Spout2

## 色块布局

每灯 2 块，块间留 **2px** 黑色间隔，**块宽固定 42px**，块高按灯数量动态填满 720。

纵向排列，灯 i 的布局：

```
OUTPUT_WIDTH = 90, OUTPUT_HEIGHT = 720
block_h = floor((OUTPUT_HEIGHT - (N + 1) * 2) / N)
Block A (角度):  x = 2,   y = 2 + i * (block_h + 2),  size = 42 x block_h
Block B (颜色):  x = 46,  y = 2 + i * (block_h + 2),  size = 42 x block_h
```

## 编码规则

### Block A — Pan/Tilt 角度

整块填充同一纯色。

| 通道 | 含义 | 编码公式 | 范围 |
|------|------|----------|------|
| R | Tilt (垂直扫描) | `round((tilt + 180) / 360 * 255)` | 0-255 → -180°~+180° |
| G | Pan (水平扫描) | `round((pan + 180) / 360 * 255)` | 0-255 → -180°~+180° |
| B | 保留 | 固定 0 | — |

**解码公式（Udon 端）：**
```
tilt = R / 255.0 * 360.0 - 180.0
pan  = G / 255.0 * 360.0 - 180.0
```

### Block B — 颜色 RGB

整块填充同一纯色。颜色已经乘以 Dimmer 系数。

| 通道 | 含义 |
|------|------|
| R | 红色分量 (0-255) |
| G | 绿色分量 (0-255) |
| B | 蓝色分量 (0-255) |

当灯具处于频闪状态时，Block B 会按频闪频率在「正常颜色」和「黑色 (0,0,0)」之间切换。

## 采样坐标表

对于 Udon 脚本，建议采样每个色块的**中心像素**以获得最稳定的读数：

```
center_y = gap + i * (block_h + gap) + floor(block_h / 2)
Block A 中心 x = gap + floor(block_w / 2) = 2 + 21 = 23
Block B 中心 x = gap + block_w + gap + floor(block_w / 2) = 2 + 42 + 2 + 21 = 67

灯 i 的 Block A 中心: (23, center_y)
灯 i 的 Block B 中心: (67, center_y)
```

## UV 坐标 (归一化)

```
灯 i 的 Block A 中心 UV: (23.0 / OUTPUT_WIDTH, 1.0 - center_y / OUTPUT_HEIGHT)
灯 i 的 Block B 中心 UV: (67.0 / OUTPUT_WIDTH, 1.0 - center_y / OUTPUT_HEIGHT)
```

按默认 OUTPUT_WIDTH=90, OUTPUT_HEIGHT=720：
```
Block A UV: (23.0 / 90.0, 1.0 - center_y / 720.0)
Block B UV: (67.0 / 90.0, 1.0 - center_y / 720.0)
```

注意：UV 坐标 Y 轴翻转（纹理坐标系 Y=0 在底部）。

## WebSocket API

连接到 `ws://<host>:9000/ws` 后，服务端会自动推送完整状态。

客户端发送命令格式：
```json
{
  "type": "set_fixture_color",
  "data": {
    "fixture_id": 0,
    "r": 1.0,
    "g": 0.0,
    "b": 0.0
  }
}
```

可用命令类型：`set_fixture_color`, `set_fixture_position`, `set_fixture_dimmer`, `set_fixture_strobe`, `select_fixtures`, `get_state`
