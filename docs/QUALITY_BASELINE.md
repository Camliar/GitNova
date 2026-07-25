# MVP Quality Baseline

TASK-0028 固化本地可重复运行的质量门槛，不把尚未运行的平台验证写成已通过。TASK-0029 增加跨平台 CI 与受控发布流程；只有真实 runner 成功后，对应平台才可标记为已验证。

## Automated gates

| Gate | Command | Current contract |
| --- | --- | --- |
| Protocol/Host/privacy | `npm run check` | Schema 一致；Desktop 无直接 network API/debug console；CSP 拒绝远程 HTTP origin；UI tests 与 production build |
| Core correctness | `cargo test --workspace` | Git read/mutation、GitHub/PR/Squash Trace、transport 与 protocol contracts |
| Rust hygiene | `cargo fmt --all -- --check` and `cargo clippy --workspace --all-targets -- -D warnings` | formatting and zero warnings |
| Native host | `pnpm --filter @gitnova/desktop tauri build --no-bundle` | 当前平台 release binary |
| Desktop bundle | `npm run bundle:desktop` | 当前平台安装包及独立 Core sidecar |

Core contract tests 启动 20 个真实 `gitnova-core` 进程。排序后的 p95 必须不超过 500ms；最大样本另设 2s anti-flake ceiling。该值是开发机/测试 runner 的 regression gate，不是所有硬件的市场承诺。

## Network and sensitive data

Desktop CSP 使用 `default-src 'self'` 且没有 remote HTTP allowlist；React production source 禁止 `fetch`、XHR、WebSocket 和 EventSource。GitHub 网络访问只经过 Core-owned `gh api`，并在 Connect、Open PR、remote commit patch 和 Squash Trace 动作触发前说明范围。Core transport stderr 只输出固定错误分类，不输出底层 error detail、protocol body、path、token 或 diff。

## Platform matrix

| Target | Local verification in TASK-0028 | Required before release |
| --- | --- | --- |
| macOS x86_64 | Full Rust/frontend/Tauri release, ad-hoc signed `.app` with Core sidecar and `.dmg` bundle | CI universal build plus production signed/notarized release approval |
| macOS arm64 | Not run locally | native CI test and universal signed bundle |
| Windows x64 | Not run locally | native CI test, process cleanup and signed installer smoke test |
| Linux x64 | Not run locally | native CI test and package smoke test |

WSL、Remote SSH 与 Dev Container 由 TASK-0030 的 remote Core launcher 验证。
