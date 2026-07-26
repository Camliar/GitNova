# Brand Assets

本目录包含 GitNova Foundation 阶段的占位品牌资产：

- [`logo/gitnova-logo.svg`](logo/gitnova-logo.svg)：标准横向组合
- [`logo/gitnova-logo.png`](logo/gitnova-logo.png)：标准组合 PNG 导出
- [`icons/gitnova-mark.svg`](icons/gitnova-mark.svg)：紧凑图标
- [`icons/gitnova-mark.png`](icons/gitnova-mark.png)：紧凑图标 PNG 导出
- [`brand/tokens.json`](brand/tokens.json)：品牌颜色与基础尺寸令牌

使用前阅读[品牌指南](../docs/BRANDING.md)。SVG 是源文件，PNG 为方便预览的 2× 导出；当前资产均为占位版。

Desktop 平台图标必须从正方形源文件 `icons/gitnova-mark.svg` 生成。使用 Tauri CLI 的 `tauri icon` 生成 PNG、ICNS 与 ICO，禁止手工缩放或把图形放入带额外空白的画布。
