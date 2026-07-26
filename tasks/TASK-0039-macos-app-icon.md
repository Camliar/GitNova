# TASK-0039: macOS App Icon Repair

- **Status:** Done
- **Priority:** P1
- **Owner:** Codex
- **Branch:** `codex/task-039-macos-icon`
- **Dependencies:** completed TASK-0038 baseline (`9aeba1b`)

## Goal

修复安装后的 macOS 应用图标出现白色方形底板和图形垂直错位的问题，确保各平台 bundle 图标均由同一个正方形品牌源生成。

## Scope

- 从 `assets/icons/gitnova-mark.svg` 重新生成 Desktop PNG、ICNS 与 ICO 图标。
- 在 Tauri bundle 配置中显式声明 macOS ICNS 图标。
- 扩展 delivery 检查，验证 macOS ICNS 存在、源 PNG 为正方形且品牌图形覆盖完整画布。
- 生成并验证新的 Intel macOS DMG 安装包。

## Non-goals

- 重新设计 GitNova 品牌标志、颜色或字标。
- Apple Developer ID 签名、公证或发布渠道配置。
- 修改 Desktop、Core、Provider、Squash Trace 或 AI Assist 业务逻辑。

## Deliverables

- [x] corrected cross-platform Desktop icon assets
- [x] explicit macOS ICNS bundle configuration
- [x] icon regression checks
- [x] verified replacement macOS DMG

## Review Checklist

- [x] macOS Finder/Dock no longer receives the malformed PNG canvas.
- [x] Windows ICO still contains required resolutions.
- [x] application icon remains derived from the checked-in GitNova mark.
- [x] delivery checks and native macOS bundle succeed.

## Done Definition

- [x] 自主 Review 无阻塞项，状态 Done，提交推送并快进合并 main。

## Verification

- `node scripts/check-delivery.mjs`
- Desktop TypeScript check, 63 tests and production frontend build
- Native x86_64 macOS Tauri DMG bundle with ad-hoc application signature
- `hdiutil verify target/release/bundle/dmg/GitNova_0.1.0_x64.dmg`
