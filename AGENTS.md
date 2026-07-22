# Rust OS 项目规则

## 学习文档存放

- 本项目的学习教程、学习计划、学习笔记和概念讲解统一保存到独立 Obsidian Vault：`/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/`。
- 不要再把 Rust OS 项目专用学习文档写入通用 Vault `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/项目学习/`。
- 项目自身的 README、接口文档、设计文档、内部实施计划和项目记忆仍保存在项目仓库中。

## 知识边界与链接

- Rust OS 专用内容放入 `Rust OS` Vault；可复用于其他项目的 Rust、Tokio、Java、网络等通用知识继续留在 `项目学习` Vault，避免复制出两份正文。
- 两个 Vault 之间使用带明确 `vault` 和 `file` 参数的 `obsidian://open` Markdown 链接；同一 Vault 内继续优先使用 WikiLink。
- 创建或更新 Rust OS 学习文档前，先检查 `Rust OS` Vault 中的已有文档；存在前置知识、后续内容或同一主题时，必须建立 WikiLink。
- 若跨 Vault 引用通用知识，应同时检查通用笔记是否需要增加返回 `Rust OS` Vault 的链接。
- 不要随意修改 Vault 名称 `Rust OS` 或 `项目学习`；改名会使已有跨 Vault URI 失效。

## 当前入口

- 学习主页：`/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/00-学习主页.md`
- 基础认知地图：`/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/01-基础认知/Rust 操作系统基础认知地图.md`
