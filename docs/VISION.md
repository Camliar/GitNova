# Vision

## 愿景

让每位开发者都能理解版本历史中的意图，而不仅是执行 Git 命令。即使 Squash Merge 压平了主分支历史，PR 原始 commits、逐 commit 变更与最终 squash commit 之间的故事仍应当可追溯。

GitNova 坚持三条产品原则：

1. **Local First**：核心体验离线可用，仓库和衍生数据默认留在本机。
2. **Insight over ceremony**：用清晰的上下文帮助判断，不增加流程负担。
3. **One Core, many Hosts**：业务语义只在 `gitnova-core` 中实现，多宿主体验保持一致。

## 目标用户

- 希望用可视化方式理解 Git 的个人开发者。
- 在大型或长期仓库中追踪变更意图的团队成员。
- 需要在桌面应用与 IDE 之间保持一致体验的高级用户。

## 产品承诺

GitNova 不建设中心服务器，不要求上传仓库，不把 Host 变成业务层，也不替代 System Git。架构保障见[架构说明](ARCHITECTURE.md)，产品边界见[产品需求](PRODUCT_REQUIREMENTS.md)，可验证指标见[非功能需求](NON_FUNCTIONAL.md)。
