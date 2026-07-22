# Rust OS 基础认知地图章节拆分 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 把独立 Obsidian Vault `Rust OS` 中的单文件基础认知地图无损拆成一个章节导航、12 个顺序章节和一个相关知识入口，并重建主页、章节间及跨 Vault 链接。

**Architecture:** 先通过 Finder 从受 macOS 权限保护的 Vault 取得只读快照，再用临时 Node.js 脚本按原有二级标题机械拆分、移动卡片 14、重写跨文件 WikiLink 并生成统一页脚。新文件写回 Vault 后先做 round-trip 哈希、结构和实际跳转验证；只有全部通过才删除旧总文档。项目仓库只保存设计、计划、规则和项目记忆，不保存 Obsidian 正文副本。

**Tech Stack:** Markdown、Obsidian WikiLink、Obsidian URI、Finder/Obsidian、Node.js、`rg`、`awk`、`shasum`、`cmp`、`apply_patch`

---

## 文件结构与职责

**Vault 中创建：**

- `Rust OS/01-基础认知/00-章节导航.md`：基础认知唯一入口、快速路线和完整目录。
- `Rust OS/01-基础认知/01-操作系统全景与启动过程.md`：系统分层、启动故事、卡片 01 和 14。
- `Rust OS/01-基础认知/02-执行、调度与内核边界.md`：执行、调度、系统调用、中断、卡片 02–05。
- `Rust OS/01-基础认知/03-内存管理.md`：内存与卡片 06–07。
- `Rust OS/01-基础认知/04-println如何走到终端.md`：第一个运行故事。
- `Rust OS/01-基础认知/05-文件、设备与IO.md`：文件、设备、I/O 与卡片 08–09。
- `Rust OS/01-基础认知/06-Tokio网络请求如何等待与醒来.md`：第二个运行故事。
- `Rust OS/01-基础认知/07-网络栈.md`：网络与卡片 10。
- `Rust OS/01-基础认知/08-并发、进程通信与用户空间.md`：并发、IPC、用户空间与卡片 11–13。
- `Rust OS/01-基础认知/09-Rust内核启动与工具链.md`：Rust 裸机词汇与卡片 15。
- `Rust OS/01-基础认知/10-手写内核学习路线.md`：八阶段实现顺序。
- `Rust OS/01-基础认知/11-术语表.md`：70 项术语。
- `Rust OS/01-基础认知/12-自测题.md`：11 道题及答案。
- `Rust OS/01-基础认知/99-相关知识入口.md`：跨 Vault 延伸阅读。

**Vault 中修改：**

- `Rust OS/00-学习主页.md`：入口改到章节导航。
- `项目学习/Tokio 中的 Task、Future 与线程.md`：返回链接改到 Tokio 运行故事。
- `项目学习/进程管道、缓冲区与 flush.md`：两处返回链接改到 IPC 卡片。
- `项目学习/Rust 的 to_owned、clone 与字符串所有权.md`：返回链接改到内存章节原标题。

**Vault 中删除：**

- `Rust OS/01-基础认知/Rust 操作系统基础认知地图.md`：仅在新结构全部验收后删除。

**仓库中修改：**

- `AGENTS.md`：当前入口改为章节导航。
- `PROJECT_PROGRESS.md`：记录拆分完成和验收证据。
- `PROJECT_TODO.md`：阅读任务改为从章节导航开始。
- `docs/superpowers/specs/2026-07-22-rust-os-foundations-chapter-split-design.md`：状态改为已实施。
- `docs/superpowers/specs/2026-07-21-os-foundations-learning-document-design.md`：顶部现行入口说明改为章节导航。
- `docs/superpowers/plans/2026-07-22-os-foundations-learning-document.md`：顶部现行入口说明改为章节导航。

**临时文件：**

- `/tmp/rust-os-chapter-split/source/`：原文与主页只读快照。
- `/tmp/rust-os-chapter-split/staged/`：准备写回 Vault 的 15 个 Markdown 文件。
- `/tmp/rust-os-chapter-split/roundtrip/`：写回后再复制出的验收快照。
- `/tmp/rust-os-chapter-split/split.mjs`：机械拆分脚本，不提交。
- `/tmp/rust-os-chapter-split/validate.mjs`：结构与 WikiLink 校验脚本，不提交。

### Task 1: 获取原文与基线证据

**Files:**

- Read: `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/00-学习主页.md`
- Read: `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/01-基础认知/Rust 操作系统基础认知地图.md`
- Create: `/tmp/rust-os-chapter-split/source/00-学习主页.md`
- Create: `/tmp/rust-os-chapter-split/source/Rust 操作系统基础认知地图.md`

- [ ] **Step 1: 建立空的临时工作目录**

Run:

```bash
set -euo pipefail
ROOT='/tmp/rust-os-chapter-split'
rm -rf "$ROOT"
mkdir -p "$ROOT/source" "$ROOT/staged" "$ROOT/roundtrip"
test -z "$(find "$ROOT/source" -mindepth 1 -print -quit)"
```

Expected: 命令无输出并以状态 0 结束。

- [ ] **Step 2: 通过 Finder 复制两个源文件，不移动原件**

在 Finder 中按住 Option，把 `Rust OS/00-学习主页.md` 和 `Rust OS/01-基础认知/Rust 操作系统基础认知地图.md` 分别拖到 `/tmp/rust-os-chapter-split/source/`。确认动作显示为复制，且 Vault 原文件仍存在。

- [ ] **Step 3: 验证源快照与基线数量**

Run:

```bash
set -euo pipefail
ROOT='/tmp/rust-os-chapter-split'
HOME_COPY="$ROOT/source/00-学习主页.md"
SOURCE="$ROOT/source/Rust 操作系统基础认知地图.md"
test -s "$HOME_COPY"
test -s "$SOURCE"
test "$(rg -c '^### 组件卡片 (0[1-9]|1[0-5])：' "$SOURCE")" -eq 15
test "$(rg -c '^```mermaid[[:space:]]*$' "$SOURCE")" -eq 4
test "$(rg -c '^> \[!check\]- 第 ([1-9]|1[01]) 题答题要点$' "$SOURCE")" -eq 11
for heading in \
  '## 这篇文档要解决什么' \
  '## 阅读方式' \
  '## 先看整台系统：资源管理与隔离' \
  '## 启动故事：从按下电源到第一个用户进程' \
  '## 执行与调度' \
  '## 内存：地址不是你以为的那块 RAM' \
  '## 运行故事一：println! 如何走到终端' \
  '## 文件与设备' \
  '## 运行故事二：Tokio 网络请求如何等待并醒来' \
  '## 网络' \
  '## 并发、进程通信与用户空间' \
  '## Rust 内核词汇桥梁' \
  '## 推荐的手写内核顺序' \
  '## 术语表' \
  '## 自测题' \
  '## 相关文档'
do
  rg -Fxq "$heading" "$SOURCE"
done
shasum -a 256 "$HOME_COPY" "$SOURCE" | tee "$ROOT/source.sha256"
```

Expected: 所有检查通过，最后输出两个 SHA-256。若基线数量不符，停止执行并检查当前 Vault 内容。

### Task 2: 机械拆分并生成暂存稿

**Files:**

- Create: `/tmp/rust-os-chapter-split/split.mjs`
- Create: `/tmp/rust-os-chapter-split/staged/00-学习主页.md`
- Create: `/tmp/rust-os-chapter-split/staged/01-基础认知/*.md`

- [ ] **Step 1: 用 `apply_patch` 创建拆分脚本**

Create `/tmp/rust-os-chapter-split/split.mjs` with this complete content:

```javascript
import crypto from "node:crypto";
import fs from "node:fs";
import path from "node:path";

const [sourcePath, homePath, stageRoot] = process.argv.slice(2);
if (!sourcePath || !homePath || !stageRoot) {
  throw new Error("usage: node split.mjs SOURCE HOME STAGE_ROOT");
}

const source = fs.readFileSync(sourcePath, "utf8").replace(/\r\n/g, "\n");
const home = fs.readFileSync(homePath, "utf8").replace(/\r\n/g, "\n");
const h2Matches = [...source.matchAll(/^## (.+)$/gm)];
if (h2Matches.length === 0) throw new Error("source has no level-2 headings");

const sections = new Map();
for (let index = 0; index < h2Matches.length; index += 1) {
  const current = h2Matches[index];
  const next = h2Matches[index + 1];
  sections.set(current[1], source.slice(current.index, next?.index ?? source.length).trim());
}
const requiredSections = [
  "这篇文档要解决什么",
  "阅读方式",
  "先看整台系统：资源管理与隔离",
  "启动故事：从按下电源到第一个用户进程",
  "执行与调度",
  "内存：地址不是你以为的那块 RAM",
  "运行故事一：println! 如何走到终端",
  "文件与设备",
  "运行故事二：Tokio 网络请求如何等待并醒来",
  "网络",
  "并发、进程通信与用户空间",
  "Rust 内核词汇桥梁",
  "推荐的手写内核顺序",
  "术语表",
  "自测题",
  "相关文档",
];
for (const heading of requiredSections) {
  if (!sections.has(heading)) throw new Error(`missing section: ${heading}`);
}

const firstLineEnd = source.indexOf("\n");
const preamble = source.slice(firstLineEnd + 1, h2Matches[0].index).trim();
const bridge = sections.get("Rust 内核词汇桥梁");
const card14Heading = "### 组件卡片 14：固件、bootloader 与内核启动";
const card15Heading = "### 组件卡片 15：no_std、unsafe、ABI、ELF、linker 与 cross compilation";
const card14Start = bridge.indexOf(card14Heading);
const card15Start = bridge.indexOf(card15Heading);
if (card14Start < 0 || card15Start <= card14Start) throw new Error("cannot isolate card 14");
const card14 = bridge.slice(card14Start, card15Start).trim();
const bridgeWithoutCard14 = `${bridge.slice(0, card14Start).trimEnd()}\n\n${bridge.slice(card15Start).trimStart()}`;

const chapterDirectory = [
  ["01-操作系统全景与启动过程", "理解操作系统如何管理与隔离资源，以及机器如何把控制权交给内核。"],
  ["02-执行、调度与内核边界", "把程序、进程、线程、Task、调度、系统调用和中断连成一条执行链。"],
  ["03-内存管理", "理解虚拟地址、页表、栈、堆和 Rust 所有权分别负责哪一层安全。"],
  ["04-println如何走到终端", "沿一次输出追踪应用、系统调用、内核、驱动和设备。"],
  ["05-文件、设备与IO", "理解文件描述符、VFS、设备驱动和 I/O 的统一接口。"],
  ["06-Tokio网络请求如何等待与醒来", "沿一次异步网络请求观察 Future、线程、内核和网卡如何协作。"],
  ["07-网络栈", "理解 Socket、内核协议栈、驱动和网卡的职责边界。"],
  ["08-并发、进程通信与用户空间", "理解同步、IPC、init、shell、系统服务和用户空间。"],
  ["09-Rust内核启动与工具链", "把 no_std、unsafe、ABI、ELF、linker 和交叉编译连成因果链。"],
  ["10-手写内核学习路线", "按依赖关系查看八个内核实现阶段。"],
  ["11-术语表", "快速查询 70 个核心词汇及其所属层。"],
  ["12-自测题", "用路径追踪和概念比较检验心智模型。"],
];
const directoryMarkdown = chapterDirectory
  .map(([file, description]) => `- [[${file}]]：${description}`)
  .join("\n");

const documents = new Map([
  ["01-基础认知/00-章节导航.md", [
    "# 基础认知章节导航",
    preamble,
    sections.get("这篇文档要解决什么"),
    sections.get("阅读方式"),
    `## 章节目录\n\n${directoryMarkdown}\n\n- [[99-相关知识入口]]：进入通用 Rust、Tokio 与 Pipe 延伸资料。`,
    "---\n\n返回：[[00-学习主页|学习主页]] · 下一章：[[01-操作系统全景与启动过程]]",
  ].join("\n\n")],
  ["01-基础认知/01-操作系统全景与启动过程.md", [
    "# 操作系统全景与启动过程",
    sections.get("先看整台系统：资源管理与隔离"),
    sections.get("启动故事：从按下电源到第一个用户进程"),
    `## 启动相关组件\n\n${card14}`,
  ].join("\n\n")],
  ["01-基础认知/02-执行、调度与内核边界.md", `# 执行、调度与内核边界\n\n${sections.get("执行与调度")}`],
  ["01-基础认知/03-内存管理.md", `# 内存管理\n\n${sections.get("内存：地址不是你以为的那块 RAM")}`],
  ["01-基础认知/04-println如何走到终端.md", `# println! 如何走到终端\n\n${sections.get("运行故事一：println! 如何走到终端")}`],
  ["01-基础认知/05-文件、设备与IO.md", `# 文件、设备与 I/O\n\n${sections.get("文件与设备")}`],
  ["01-基础认知/06-Tokio网络请求如何等待与醒来.md", `# Tokio 网络请求如何等待与醒来\n\n${sections.get("运行故事二：Tokio 网络请求如何等待并醒来")}`],
  ["01-基础认知/07-网络栈.md", `# 网络栈\n\n${sections.get("网络")}`],
  ["01-基础认知/08-并发、进程通信与用户空间.md", `# 并发、进程通信与用户空间\n\n${sections.get("并发、进程通信与用户空间")}`],
  ["01-基础认知/09-Rust内核启动与工具链.md", `# Rust 内核启动与工具链\n\n${bridgeWithoutCard14}`],
  ["01-基础认知/10-手写内核学习路线.md", `# 手写内核学习路线\n\n${sections.get("推荐的手写内核顺序")}`],
  ["01-基础认知/11-术语表.md", `# 术语表\n\n${sections.get("术语表")}`],
  ["01-基础认知/12-自测题.md", `# 自测题\n\n${sections.get("自测题")}`],
  ["01-基础认知/99-相关知识入口.md", `# 相关知识入口\n\n${sections.get("相关文档")}\n\n---\n\n返回：[[00-章节导航]]`],
]);

const orderedFiles = chapterDirectory.map(([file]) => `01-基础认知/${file}.md`);
for (let index = 0; index < orderedFiles.length; index += 1) {
  const file = orderedFiles[index];
  const previous = orderedFiles[index - 1]?.split("/").at(-1).replace(/\.md$/, "");
  const next = orderedFiles[index + 1]?.split("/").at(-1).replace(/\.md$/, "");
  const links = [
    previous ? `上一章：[[${previous}]]` : null,
    "返回：[[00-章节导航]]",
    next ? `下一章：[[${next}]]` : null,
  ].filter(Boolean);
  documents.set(file, `${documents.get(file)}\n\n---\n\n${links.join(" · ")}`);
}

const headingOwner = new Map();
for (const [file, content] of documents) {
  for (const match of content.matchAll(/^#{2,6} (.+)$/gm)) {
    const heading = match[1].trim();
    if (headingOwner.has(heading)) throw new Error(`duplicate heading: ${heading}`);
    headingOwner.set(heading, file);
  }
}
for (const [file, content] of documents) {
  documents.set(file, content.replace(/\[\[#([^|\]]+)(?:\|([^\]]+))?\]\]/g, (whole, heading, alias) => {
    const targetFile = headingOwner.get(heading);
    if (!targetFile) throw new Error(`unresolved internal heading from ${file}: ${heading}`);
    if (targetFile === file) return whole;
    const targetStem = targetFile.split("/").at(-1).replace(/\.md$/, "");
    return `[[${targetStem}#${heading}|${alias ?? heading}]]`;
  }));
}

const oldHomeLink = /\[\[(?:01-基础认知\/)?Rust 操作系统基础认知地图(?:#[^|\]]+)?(?:\|[^\]]+)?\]\]/g;
if (!(home.match(oldHomeLink) ?? []).length) throw new Error("home has no link to the old map");
documents.set("00-学习主页.md", home.replace(oldHomeLink, "[[01-基础认知/00-章节导航|开始基础认知学习]]"));

fs.rmSync(stageRoot, { recursive: true, force: true });
fs.mkdirSync(stageRoot, { recursive: true });
const manifest = {};
for (const [relativePath, content] of documents) {
  const outputPath = path.join(stageRoot, relativePath);
  fs.mkdirSync(path.dirname(outputPath), { recursive: true });
  const finalContent = `${content.trimEnd()}\n`;
  fs.writeFileSync(outputPath, finalContent, "utf8");
  manifest[relativePath] = crypto.createHash("sha256").update(finalContent).digest("hex");
}
fs.writeFileSync(path.join(path.dirname(stageRoot), "manifest.json"), `${JSON.stringify(manifest, null, 2)}\n`);
console.log(`generated_files=${documents.size}`);
```

- [ ] **Step 2: 运行拆分脚本并检查暂存稿**

Run:

```bash
set -euo pipefail
ROOT='/tmp/rust-os-chapter-split'
node "$ROOT/split.mjs" \
  "$ROOT/source/Rust 操作系统基础认知地图.md" \
  "$ROOT/source/00-学习主页.md" \
  "$ROOT/staged"
test "$(find "$ROOT/staged" -type f -name '*.md' | wc -l | tr -d ' ')" -eq 15
STAGED="$ROOT/staged/01-基础认知"
test "$(rg -g '*.md' -c '^### 组件卡片 (0[1-9]|1[0-5])：' "$STAGED" | awk -F: '{s+=$2} END {print s+0}')" -eq 15
for n in {01..15}; do
  test "$(rg -g '*.md' -c "^### 组件卡片 $n：" "$STAGED" | awk -F: '{s+=$2} END {print s+0}')" -eq 1
done
test "$(rg -g '*.md' -c '^```mermaid[[:space:]]*$' "$STAGED" | awk -F: '{s+=$2} END {print s+0}')" -eq 4
test "$(rg -g '*.md' -c '^> \[!check\]- 第 ([1-9]|1[01]) 题答题要点$' "$STAGED" | awk -F: '{s+=$2} END {print s+0}')" -eq 11
test "$(rg -o 'obsidian://open\?vault=%E9%A1%B9%E7%9B%AE%E5%AD%A6%E4%B9%A0' "$STAGED" -g '*.md' | wc -l | tr -d ' ')" -eq 10
find "$ROOT/staged" -type f -name '*.md' -print | sort
```

Expected: 输出 `generated_files=15`，数量检查全部通过，并列出主页和 14 个基础认知文件。

### Task 3: 把新结构写回 Rust OS Vault，但保留旧总文档

**Files:**

- Create: 14 个 `Rust OS/01-基础认知/*.md` 新文件
- Modify: `Rust OS/00-学习主页.md`
- Preserve: `Rust OS/01-基础认知/Rust 操作系统基础认知地图.md`

- [ ] **Step 1: 通过 Finder 复制新章节**

在 Finder 中并排打开 `/tmp/rust-os-chapter-split/staged/01-基础认知/` 和 `Rust OS/01-基础认知/`。选择来源中的全部 14 个 Markdown 文件并复制到目标，不选择、不覆盖、不删除旧总文档。

Expected: 目标目录同时存在 14 个新文件和 1 个旧总文档。

- [ ] **Step 2: 用暂存主页覆盖 Vault 主页**

把 `/tmp/rust-os-chapter-split/staged/00-学习主页.md` 复制到 `Rust OS` 根目录；Finder 提示同名文件时选择“替换”。

Expected: Obsidian 打开主页后，“开始基础认知学习”指向 `00-章节导航`，旧总文档仍能作为失败恢复副本。

### Task 4: 做 round-trip、结构和本地 WikiLink 验证

**Files:**

- Create: `/tmp/rust-os-chapter-split/roundtrip/00-学习主页.md`
- Create: `/tmp/rust-os-chapter-split/roundtrip/01-基础认知/*.md`
- Create: `/tmp/rust-os-chapter-split/validate.mjs`

- [ ] **Step 1: 从 Vault 复制新文件作为 round-trip 快照**

通过 Finder 把 Vault 中的新主页和 14 个新章节复制到 `/tmp/rust-os-chapter-split/roundtrip/` 的对应层级，不复制旧总文档。

- [ ] **Step 2: 逐文件比较写回内容**

Run:

```bash
set -euo pipefail
ROOT='/tmp/rust-os-chapter-split'
test "$(find "$ROOT/roundtrip" -type f -name '*.md' | wc -l | tr -d ' ')" -eq 15
while IFS= read -r relative; do
  cmp "$ROOT/staged/$relative" "$ROOT/roundtrip/$relative"
done < <(cd "$ROOT/staged" && find . -type f -name '*.md' -print | sed 's#^./##' | sort)
echo 'roundtrip_files=15'
```

Expected: 输出 `roundtrip_files=15`，`cmp` 没有差异。

- [ ] **Step 3: 用 `apply_patch` 创建校验脚本**

Create `/tmp/rust-os-chapter-split/validate.mjs` with this complete content:

```javascript
import fs from "node:fs";
import path from "node:path";

const root = process.argv[2];
if (!root) throw new Error("usage: node validate.mjs SNAPSHOT_ROOT");
function assert(condition, message) {
  if (!condition) throw new Error(message);
}
function walk(directory) {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const fullPath = path.join(directory, entry.name);
    return entry.isDirectory() ? walk(fullPath) : [fullPath];
  });
}

const files = walk(root).filter((file) => file.endsWith(".md"));
assert(files.length === 15, `expected 15 markdown files, got ${files.length}`);
const documents = new Map(files.map((file) => [
  path.relative(root, file).replaceAll(path.sep, "/"),
  fs.readFileSync(file, "utf8"),
]));
const expectedChapters = new Set([
  "00-章节导航.md",
  "01-操作系统全景与启动过程.md",
  "02-执行、调度与内核边界.md",
  "03-内存管理.md",
  "04-println如何走到终端.md",
  "05-文件、设备与IO.md",
  "06-Tokio网络请求如何等待与醒来.md",
  "07-网络栈.md",
  "08-并发、进程通信与用户空间.md",
  "09-Rust内核启动与工具链.md",
  "10-手写内核学习路线.md",
  "11-术语表.md",
  "12-自测题.md",
  "99-相关知识入口.md",
]);
const actualChapters = new Set([...documents.keys()]
  .filter((file) => file.startsWith("01-基础认知/"))
  .map((file) => path.basename(file)));
assert(actualChapters.size === expectedChapters.size, "wrong chapter file count");
for (const file of expectedChapters) assert(actualChapters.has(file), `missing chapter: ${file}`);

const combined = [...documents.values()].join("\n");
const cards = [...combined.matchAll(/^### 组件卡片 (0[1-9]|1[0-5])：/gm)].map((match) => match[1]);
assert(cards.length === 15, `expected 15 cards, got ${cards.length}`);
for (let number = 1; number <= 15; number += 1) {
  const id = String(number).padStart(2, "0");
  assert(cards.filter((card) => card === id).length === 1, `card ${id} missing or duplicated`);
}
const cardMarkers = [
  "> [!summary] 30 秒结论",
  "**1. 它是什么？**",
  "**2. 为什么需要它？如果没有会怎样？**",
  "**3. 它如何工作？**",
  "**4. 可以怎样类比？**",
  "**5. 类比从哪里开始不准确？**",
  "**6. 用 Rust 写内核时会在哪里遇到它？**",
  "**7. 最容易混淆什么？**",
];
for (const [file, content] of documents) {
  const starts = [...content.matchAll(/^### 组件卡片 (0[1-9]|1[0-5])：.*$/gm)];
  for (const start of starts) {
    const followingHeading = /^#{2,3} /gm;
    followingHeading.lastIndex = start.index + start[0].length;
    const end = followingHeading.exec(content)?.index ?? content.length;
    const block = content.slice(start.index, end);
    for (const marker of cardMarkers) {
      assert(block.split(marker).length - 1 === 1, `${file}: card ${start[1]} bad marker ${marker}`);
    }
  }
}

const mermaid = [...combined.matchAll(/^```mermaid\s*$[\s\S]*?^```\s*$/gm)];
assert(mermaid.length === 4, `expected 4 Mermaid blocks, got ${mermaid.length}`);
for (const block of mermaid) {
  assert(/^\s*(flowchart|graph|sequenceDiagram|stateDiagram|classDiagram)/m.test(block[0]), "Mermaid block lacks diagram kind");
}
const glossary = documents.get("01-基础认知/11-术语表.md");
const tableLines = glossary.split("\n").filter((line) => line.trim().startsWith("|"));
assert(tableLines.length - 2 === 70, `expected 70 glossary rows, got ${tableLines.length - 2}`);
const quiz = documents.get("01-基础认知/12-自测题.md");
assert((quiz.match(/^> \[!check\]- 第 ([1-9]|1[01]) 题答题要点$/gm) ?? []).length === 11, "wrong answer count");
const questionSection = quiz.split(/^### 问题$/m)[1]?.split(/^### /m)[0] ?? "";
assert((questionSection.match(/^([1-9]|1[01])\. /gm) ?? []).length === 11, "wrong question count");

const byReference = new Map();
const headings = new Map();
for (const [file, content] of documents) {
  const stem = file.replace(/\.md$/, "");
  for (const reference of new Set([stem, path.basename(stem)])) {
    assert(!byReference.has(reference), `duplicate file reference: ${reference}`);
    byReference.set(reference, file);
  }
  headings.set(file, new Set([...content.matchAll(/^#{1,6} (.+)$/gm)].map((match) => match[1].trim())));
}
for (const [file, content] of documents) {
  for (const match of content.matchAll(/!?\[\[([^\]]+)\]\]/g)) {
    const raw = match[1].split("|")[0];
    const hash = raw.indexOf("#");
    const fileRef = hash === 0 ? "" : (hash < 0 ? raw : raw.slice(0, hash));
    const heading = hash < 0 ? "" : raw.slice(hash + 1);
    const target = fileRef ? byReference.get(fileRef.replace(/\.md$/, "")) : file;
    assert(target, `${file}: unresolved file ${fileRef}`);
    if (heading) assert(headings.get(target).has(heading), `${file}: unresolved heading ${heading}`);
  }
}
const generalLinks = (combined.match(/obsidian:\/\/open\?vault=%E9%A1%B9%E7%9B%AE%E5%AD%A6%E4%B9%A0/g) ?? []).length;
assert(generalLinks === 10, `expected 10 项目学习 links, got ${generalLinks}`);
assert(documents.get("00-学习主页.md").includes("[[01-基础认知/00-章节导航|开始基础认知学习]]"), "home link wrong");
assert(!combined.includes("[[Rust 操作系统基础认知地图"), "old-map WikiLink remains");
console.log(`markdown_files=${files.length}`);
console.log(`component_cards=${cards.length}`);
console.log(`mermaid_blocks=${mermaid.length}`);
console.log(`glossary_terms=${tableLines.length - 2}`);
console.log("questions=11");
console.log(`project_learning_links=${generalLinks}`);
console.log("wikilinks=resolved");
```

- [ ] **Step 4: 运行完整静态验收**

Run:

```bash
set -euo pipefail
node /tmp/rust-os-chapter-split/validate.mjs /tmp/rust-os-chapter-split/roundtrip
```

Expected:

```text
markdown_files=15
component_cards=15
mermaid_blocks=4
glossary_terms=70
questions=11
project_learning_links=10
wikilinks=resolved
```

任何一项失败都在 `staged/` 修正，再重新执行 Task 3 和本 Task；此时不能删除旧总文档。

### Task 5: 更新通用 Vault 的四个返回链接

**Files:**

- Modify: `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/项目学习/Tokio 中的 Task、Future 与线程.md`
- Modify: `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/项目学习/进程管道、缓冲区与 flush.md`
- Modify: `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/项目学习/Rust 的 to_owned、clone 与字符串所有权.md`

- [ ] **Step 1: 记录修改前哈希和旧链接数量**

Run:

```bash
set -euo pipefail
VAULT='/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/项目学习'
shasum -a 256 \
  "$VAULT/Tokio 中的 Task、Future 与线程.md" \
  "$VAULT/进程管道、缓冲区与 flush.md" \
  "$VAULT/Rust 的 to_owned、clone 与字符串所有权.md"
test "$(rg -o 'obsidian://open\?vault=Rust%20OS&file=01-[^)]*Rust%20%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E5%9F%BA%E7%A1%80%E8%AE%A4%E7%9F%A5%E5%9C%B0%E5%9B%BE[^)]*' "$VAULT" -g '*.md' | wc -l | tr -d ' ')" -eq 4
```

Expected: 输出三个哈希，旧总文档 URI 恰好出现 4 次。

- [ ] **Step 2: 用 `apply_patch` 精确替换四处 URI**

把 Tokio 笔记中的一处链接替换为：

```markdown
[Tokio 网络请求如何等待与醒来](obsidian://open?vault=Rust%20OS&file=01-%E5%9F%BA%E7%A1%80%E8%AE%A4%E7%9F%A5%2F06-Tokio%E7%BD%91%E7%BB%9C%E8%AF%B7%E6%B1%82%E5%A6%82%E4%BD%95%E7%AD%89%E5%BE%85%E4%B8%8E%E9%86%92%E6%9D%A5)
```

把 Pipe 笔记中的两处链接替换为：

```markdown
[IPC、管道、消息与共享内存](obsidian://open?vault=Rust%20OS&file=01-%E5%9F%BA%E7%A1%80%E8%AE%A4%E7%9F%A5%2F08-%E5%B9%B6%E5%8F%91%E3%80%81%E8%BF%9B%E7%A8%8B%E9%80%9A%E4%BF%A1%E4%B8%8E%E7%94%A8%E6%88%B7%E7%A9%BA%E9%97%B4%23%E7%BB%84%E4%BB%B6%E5%8D%A1%E7%89%87%2012%EF%BC%9AIPC%E3%80%81%E7%AE%A1%E9%81%93%E3%80%81%E6%B6%88%E6%81%AF%E4%B8%8E%E5%85%B1%E4%BA%AB%E5%86%85%E5%AD%98)
```

把所有权笔记中的一处链接替换为：

```markdown
[Rust 所有权与操作系统内存隔离的边界](obsidian://open?vault=Rust%20OS&file=01-%E5%9F%BA%E7%A1%80%E8%AE%A4%E7%9F%A5%2F03-%E5%86%85%E5%AD%98%E7%AE%A1%E7%90%86%23%E5%86%85%E5%AD%98%EF%BC%9A%E5%9C%B0%E5%9D%80%E4%B8%8D%E6%98%AF%E4%BD%A0%E4%BB%A5%E4%B8%BA%E7%9A%84%E9%82%A3%E5%9D%97%20RAM)
```

保留原有引用块、列表和解释文字，只替换链接标签及 URI。

- [ ] **Step 3: 验证旧 URI 清零、新 URI 数量正确**

Run:

```bash
set -euo pipefail
VAULT='/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/项目学习'
OLD='%2FRust%20%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E5%9F%BA%E7%A1%80%E8%AE%A4%E7%9F%A5%E5%9C%B0%E5%9B%BE'
test "$( (rg -o "$OLD" "$VAULT" -g '*.md' || true) | wc -l | tr -d ' ')" -eq 0
test "$(rg -o '%2F06-Tokio%E7%BD%91%E7%BB%9C%E8%AF%B7%E6%B1%82%E5%A6%82%E4%BD%95%E7%AD%89%E5%BE%85%E4%B8%8E%E9%86%92%E6%9D%A5' "$VAULT" -g '*.md' | wc -l | tr -d ' ')" -eq 1
test "$(rg -o '%2F08-%E5%B9%B6%E5%8F%91[^)]*%23%E7%BB%84%E4%BB%B6%E5%8D%A1%E7%89%87%2012' "$VAULT" -g '*.md' | wc -l | tr -d ' ')" -eq 2
test "$(rg -o '%2F03-%E5%86%85%E5%AD%98%E7%AE%A1%E7%90%86%23%E5%86%85%E5%AD%98' "$VAULT" -g '*.md' | wc -l | tr -d ' ')" -eq 1
```

Expected: 四个数量检查全部通过。

### Task 6: 实际打开链接、检查图表并删除旧总文档

**Files:**

- Verify: `Rust OS/00-学习主页.md` 和 14 个新章节
- Delete: `Rust OS/01-基础认知/Rust 操作系统基础认知地图.md`

- [ ] **Step 1: 在 Obsidian 中验证本地导航**

依次实际点击并核对：

1. `00-学习主页` → “开始基础认知学习” → `基础认知章节导航`。
2. 快速路线卡片 03 → 第 2 章卡片 03 标题。
3. 快速路线卡片 14 → 第 1 章卡片 14 标题。
4. 第 3 章页脚的上一章、返回、下一章。
5. 第 12 章只保留上一章和返回；`99-相关知识入口` 只保留返回。

Expected: 每次点击都到正确文件或标题，没有“未创建笔记”页面。

- [ ] **Step 2: 验证 Mermaid 与跨 Vault 往返**

在 Obsidian 阅读视图确认四幅 Mermaid 都成功渲染。随后分别测试：

1. 第 6 章 → Tokio 深链接 → 通用笔记 → 新第 6 章。
2. 第 8 章 → Pipe 深链接 → 通用笔记 → 第 8 章卡片 12。
3. 第 3 章 → 所有权深链接 → 通用笔记 → 第 3 章原标题。

Expected: 四幅图无渲染错误，六次跨 Vault 跳转到正确页面或锚点。

- [ ] **Step 3: 只有前两步通过后才删除旧总文档**

在 Finder 中把 `Rust OS/01-基础认知/Rust 操作系统基础认知地图.md` 移到废纸篓，不删除任何新章节。

- [ ] **Step 4: 删除后重新取得 round-trip 快照并复验**

清空 `/tmp/rust-os-chapter-split/roundtrip/`，再通过 Finder 复制当前主页和 `01-基础认知` 全部 14 个 Markdown 文件到相同层级。Run:

```bash
set -euo pipefail
ROOT='/tmp/rust-os-chapter-split'
test ! -e "$ROOT/roundtrip/01-基础认知/Rust 操作系统基础认知地图.md"
node "$ROOT/validate.mjs" "$ROOT/roundtrip"
```

Expected: 旧总文档不存在，完整校验仍输出 15 个 Markdown、15 张卡片、4 幅 Mermaid、70 个术语、11 道题、10 个通用 Vault 链接及 WikiLink 全部解析。

### Task 7: 更新仓库规则、历史入口说明与项目记忆

**Files:**

- Modify: `AGENTS.md`
- Modify: `PROJECT_PROGRESS.md`
- Modify: `PROJECT_TODO.md`
- Modify: `docs/superpowers/specs/2026-07-22-rust-os-foundations-chapter-split-design.md`
- Modify: `docs/superpowers/specs/2026-07-21-os-foundations-learning-document-design.md`
- Modify: `docs/superpowers/plans/2026-07-22-os-foundations-learning-document.md`

- [ ] **Step 1: 用 `apply_patch` 更新入口与状态**

执行以下精确语义修改：

- `AGENTS.md`：当前入口替换为 `Rust OS/01-基础认知/00-章节导航.md`；补充原总文档已移除，后续内容应放入相关章节而不是恢复巨型单文件。
- `PROJECT_PROGRESS.md`：记录 14 个导航/章节文件通过结构和跳转验收；15 张卡片、4 幅图、70 项术语、11 道自测全部保留。
- `PROJECT_TODO.md`：第一项改为从 `00-章节导航` 阅读快速路线；保留复述两个运行故事与口头检查。
- 新拆分设计：状态改为“书面规格已通过并实施（2026-07-22）”。
- 旧设计和旧实施计划顶部迁移说明：现行入口改为 `01-基础认知/00-章节导航.md`，原单文件路径只作为历史记录。

`AGENTS.md` 的“当前入口”最终写成：

```markdown
## 当前入口

- 学习主页：`/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/00-学习主页.md`
- 基础认知章节导航：`/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/01-基础认知/00-章节导航.md`
- 原《Rust 操作系统基础认知地图》已按主题拆分并移除；后续内容应更新最相关章节，只有出现职责明确的新主题时才新建章节，不恢复巨型单文件。
```

`PROJECT_PROGRESS.md` 至少加入以下事实行，不使用“应该”“预计”等未验证措辞：

```markdown
- 基础认知资料已拆成 1 个章节导航、12 个顺序章节和 1 个相关知识入口；学习主页已切换到章节导航。
- 拆分后仍包含 15 张组件卡片、4 幅 Mermaid、70 项术语和 11 道自测题；本地 WikiLink 与 6 条跨 Vault 往返测试均通过。
- 2026-07-22：基础认知采用“按学习主题拆分”的章节结构；组件卡片保留在所属主题中，不拆成孤立卡片文件。
```

`PROJECT_TODO.md` 的第一项最终写成：

```markdown
- [ ] 从独立 Obsidian Vault `Rust OS` 的 `00-学习主页` 进入 `01-基础认知/00-章节导航`，按快速路线阅读系统全景、两个运行故事和 15 张组件卡片的“30 秒结论”。
```

- [ ] **Step 2: 验证仓库文本和改动范围**

Run:

```bash
set -euo pipefail
cd /Users/xulei/.dev/os
git diff --check
rg -n '当前.*Rust 操作系统基础认知地图\.md|当前入口.*Rust 操作系统基础认知地图' \
  AGENTS.md PROJECT_PROGRESS.md PROJECT_TODO.md docs/superpowers || true
rg -n '00-章节导航\.md' AGENTS.md PROJECT_PROGRESS.md PROJECT_TODO.md docs/superpowers
git diff -- AGENTS.md PROJECT_PROGRESS.md PROJECT_TODO.md docs/superpowers
```

Expected: 旧入口搜索没有匹配；章节导航出现在当前规则、项目记忆和迁移说明中；`git diff --check` 通过。

- [ ] **Step 3: 提交仓库内的计划内改动**

Run:

```bash
set -euo pipefail
cd /Users/xulei/.dev/os
git add \
  AGENTS.md \
  PROJECT_PROGRESS.md \
  PROJECT_TODO.md \
  docs/superpowers/specs/2026-07-22-rust-os-foundations-chapter-split-design.md \
  docs/superpowers/specs/2026-07-21-os-foundations-learning-document-design.md \
  docs/superpowers/plans/2026-07-22-os-foundations-learning-document.md
git diff --cached --check
git commit -m "docs: record Rust OS chapter split"
```

Expected: 提交成功；本实施计划已在执行前单独提交，不重复暂存。

### Task 8: 最终证据检查与临时文件清理

**Files:**

- Verify: Rust OS Vault、项目学习 Vault、项目仓库
- Delete: `/tmp/rust-os-chapter-split/`

- [ ] **Step 1: 做最终只读验收**

Run:

```bash
set -euo pipefail
ROOT='/tmp/rust-os-chapter-split'
GENERAL='/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/项目学习'
node "$ROOT/validate.mjs" "$ROOT/roundtrip"
test ! -e "$ROOT/roundtrip/01-基础认知/Rust 操作系统基础认知地图.md"
test "$(rg -o 'obsidian://open\?vault=Rust%20OS&file=01-' "$GENERAL" -g '*.md' | wc -l | tr -d ' ')" -eq 4
test "$( (rg -o '%2FRust%20%E6%93%8D%E4%BD%9C%E7%B3%BB%E7%BB%9F%E5%9F%BA%E7%A1%80%E8%AE%A4%E7%9F%A5%E5%9C%B0%E5%9B%BE' "$GENERAL" -g '*.md' || true) | wc -l | tr -d ' ')" -eq 0
cd /Users/xulei/.dev/os
git status --short --branch
test -z "$(git status --porcelain=v1)"
git log -3 --oneline
```

Expected: 静态验收再次通过；通用 Vault 恰好有 4 个新返回链接且没有旧 URI；Git 工作区干净并显示计划与实施提交。

- [ ] **Step 2: 删除临时副本**

Run:

```bash
set -euo pipefail
ROOT='/tmp/rust-os-chapter-split'
rm -rf "$ROOT"
test ! -e "$ROOT"
```

Expected: 临时源快照、脚本、暂存稿和 round-trip 副本全部删除，Vault 和仓库不受影响。
