# UI Guideline

## 原则

1. **先解释，再操作：** 明确展示上下文、影响范围和下一步。
2. **渐进披露：** 默认保持简洁，高级信息按需展开。
3. **一致语义：** Host 视觉可适配平台，但 Core 结果和术语一致。
4. **本地可感知：** 清晰标识本地、网络、运行中和只读状态。

## 基础视觉

使用[品牌指南](BRANDING.md)中的颜色；正文优先系统字体栈，代码、哈希和路径使用等宽字体。以 4 px 为基础间距单位，常用间距为 8/12/16/24/32 px。避免为装饰牺牲信息密度和可读性。

## 交互

- 所有功能可通过键盘完成，焦点环始终可见。
- 危险或不可逆操作必须显示对象、影响并二次确认。
- 任务超过 400 ms 时提供进行中反馈；支持取消时必须呈现取消入口。
- 错误说明发生了什么、哪些内容未改变以及用户可以做什么。
- 空状态应解释原因并提供一个明确下一步。

## Desktop 仓库工作台

- 已打开仓库使用紧凑顶部栏、固定仓库侧栏和单一主视图，Local Changes、All Commits、Pull Requests、Settings 不得同时纵向堆叠。
- 仓库身份在顶部只展示一次；顶部选择器管理最近仓库。分支上下文只在左侧 Branches/Remotes/Tags 树展示，当前本地分支加粗、着色并带当前标记，不在顶部重复分支选择器。
- commit timeline 优先单行密度：graph、refs、message、author、SHA、时间横向排列，refs 位于 message 前，点击整行打开下方 Commit/Changes 标签详情；常规行高固定为 28 px，内容从顶部开始排列，不能因可用高度增加而拉伸，选中态覆盖整行。
- commit Changes 先显示纵向 changed-file 列表，点击文件名才按需加载单文件 diff；加载或失败不得清空 timeline、commit metadata 和文件列表。
- Squash Trace 从所选 timeline commit 的显式检查动作进入。确认关联后先显示紧凑的 PR ordered original commits 与 `originals → final commit` 关系；点击 original commit 后再显示 Commit/Changes，点击 changed-file 后才加载该文件 patch。没有关联或 Provider 失败时，普通本地 Commit/Changes 始终保留。
- Local Changes 将 Core 状态明确分成 Unstaged 与 Staged 列表，混合状态文件在两个范围分别出现；点击对应范围内的文件名直接打开 diff，不另设 View 按钮或范围徽章。commit composer 位于同一工作流底部；AI 只作为 commit composer 内的显式渐进披露操作。
- AI Provider、模型、端点及排除路径属于 Settings，不在 history、PR 或常驻导航中重复展示。

## 可访问性与跨 Host

目标为 WCAG 2.2 AA。文本对比度、触控目标、缩放、屏幕阅读器名称和减少动态效果均需验证。共享设计令牌，不强制各 Host 像素一致；遵守其原生导航与命令习惯。质量要求见[非功能需求](NON_FUNCTIONAL.md)，统一术语见[词汇表](GLOSSARY.md)。
