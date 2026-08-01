# Rust OS 第一次启动学习教程设计

## 目标

把已经批准的第一次 Rust OS 启动技术方案改写成一篇学习者可以独立阅读、从上到下亲手输入的 Obsidian 教程。学习者不需要依赖导师逐条发送命令，也不需要先读项目内部实施计划；教程本身必须完整说明每一步做什么、为什么这样做、输入什么、预期看到什么，以及失败时如何只排查当前边界。

教程最终引导学习者在 Apple Silicon macOS 上执行 `cargo run`，通过 QEMU 启动 x86_64 BIOS 镜像，并看到内核通过 COM1 串口输出：

```text
Rust OS: kernel entered
```

教程发布时只表示“教程已准备”，不能把尚未执行的实验写成已经成功。

## 读者与写作假设

读者已经接触过 Java、C++ 和 Rust，理解 Rust 所有权、生命周期、trait 与基本并发概念，但对 Cargo 工程组织、裸机环境和操作系统启动链缺少系统经验。

正文采用以下语言策略：

- 首先使用日常语言建立直觉，再给出准确术语。
- 优先联系普通 Java/Rust 程序、线程、Tokio runtime 和网络服务的既有经验。
- 每个类比必须说明失效边界，不能用类比代替真实机制。
- 新术语第一次出现时立即解释，不要求读者先查术语表才能继续。
- 不用“显然”“很简单”“照抄即可”等会掩盖前置知识的表达。

## 交付文件

### 新建教程

正式教程保存到现有 Rust OS Vault：

`/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/02-内核实验/01-第一次启动 Rust 内核.md`

`02-内核实验` 是现有 `Rust OS` Vault 内的新目录，不创建新的 Vault，也不在项目仓库或桌面保存正式副本。

### 更新导航

教程生成后同时更新：

- `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/00-学习主页.md`
- `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/01-基础认知/10-手写内核学习路线.md`

学习主页提供正向入口；学习路线提供从“串口输出”阶段进入实验的链接；教程使用 WikiLink 返回相关基础章节，形成双向导航。

## 单篇教程结构

教程保持为一个文件，读者按章节顺序向下执行，不拆成环境、Cargo、kernel、QEMU 等多篇文件。文件内部使用清晰标题和目录式入口降低长文导航成本。

正文依次包含：

1. **实验结果与边界**
   - 最终会看到什么。
   - 这一行输出能证明什么、不能证明什么。
   - 本实验明确不实现的内容。
2. **先看完整故事**
   - 使用 Mermaid 展示 `cargo run`、kernel ELF、BIOS 镜像、runner、QEMU、bootloader、kernel 与 COM1 的数据流。
   - 明确哪些阶段在 macOS，哪些阶段在 QEMU 客体。
3. **最终目录地图**
   - 展示完整目录树。
   - 用一张职责表说明每个文件存在的原因。
4. **实验 0：检查当前起点**
   - 检查工作目录、Git 状态、Homebrew、Rust 与 QEMU。
   - 指出当前根 `Cargo.toml` 的旧成员、误放 `[toolchain]` 与拼写错误，但不要求读者一次理解全部配置。
5. **实验 1：准备 QEMU 与固定工具链**
   - 安装并验证 QEMU。
   - 完整写出 `rust-toolchain.toml`。
   - 逐项解释 dated nightly、profile、components、target、`rust-src`、`llvm-tools` 与交叉编译。
6. **实验 2：建立两个运行地点不同的 package**
   - 完整写出根 `Cargo.toml`、临时宿主 runner、`kernel/Cargo.toml` 与最小 kernel 入口。
   - 分别编译并用 `file` 观察 arm64 Mach-O 与 x86-64 ELF。
   - 解释 workspace、package、binary、宿主和客体的边界。
7. **实验 3：理解 `no_std`、`no_main` 与内核入口**
   - 完整解释 crate 属性、`entry_point!`、`BootInfo`、`!`、panic handler 与自旋循环。
   - 对比普通程序的 `main` 调用前提与裸机入口契约。
8. **实验 4：从 kernel ELF 生成 BIOS 镜像**
   - 完整写出 `.cargo/config.toml`、根 artifact build dependency 和 `build.rs`。
   - 逐行解释 `bindeps`、`OUT_DIR`、`CARGO_BIN_FILE_KERNEL_kernel`、`BiosBoot` 与 `BIOS_PATH`。
   - 验证非空 `bios.img`。
9. **实验 5：用宿主 runner 启动 QEMU**
   - 完整写出最终 `src/main.rs`。
   - 逐项解释 TCG、磁盘、串口、monitor、无图形显示和禁止自动重启参数。
   - 说明第一次无内核输出时的正常现象与退出方式。
10. **实验 6：写出第一条内核串口消息**
    - 完整加入 `uart_16550` 依赖与最终 kernel 代码。
    - 解释 COM1、`0x3F8`、硬件端口所有权的 `unsafe` 边界、`fmt::Write` 与 panic 输出。
    - 运行 QEMU 并观察固定消息。
11. **最终验收**
    - rustfmt、分别面向宿主和客体的 Clippy、完整构建、产物架构和人工启动检查。
    - 确认 Git 没有跟踪 `target/` 或镜像。
12. **错误速查与复盘**
    - 按“首个失败阶段”组织排错，而不是罗列无边界的常见报错。
    - 给出五个理解问题及参考答案。
    - 列出下一实验：亲手替换 `uart_16550`，但不在本教程实现。
13. **最终文件全集**
    - 集中展示完成后所有配置与源码，供读者核对遗漏。
    - 这一节是核对区，不代替前面的逐步输入与解释。

## 每一步的固定教学模板

每个可执行步骤按相同顺序书写：

1. **本步目标：**一句话说明完成后能观察到什么。
2. **先理解：**只讲本步新增概念，并说明它与上一阶段的连接。
3. **为什么需要：**解释缺少这一层时，启动链会停在哪里。
4. **你来输入：**给出准确路径和完整文件内容；修改已有文件时显示修改后的完整内容。
5. **逐段解释：**配置按键值解释，Rust 代码按逻辑块或关键行解释。
6. **运行：**给出可以直接执行的完整命令。
7. **预期现象：**写出稳定输出或明确的输出特征，不伪造 hash、安装版本等动态值。
8. **它证明了什么：**限制结论范围，防止把“能启动”理解为“已经有完整操作系统”。
9. **如果失败：**只列本步直接相关的检查项，并告诉读者应保留哪段原始错误。
10. **停下来确认：**用一至两个问题检查理解；答案紧接在可折叠或独立的参考区域中，不阻止读者继续阅读。

## 代码与配置呈现规则

- 所有需要学习者输入的文件都给出完整内容，不使用 `...`、`TODO` 或“其余保持不变”。
- 同一文件多次演进时，每次都展示该阶段的完整版本，并说明相比上一步改变了什么。
- 命令默认从 `/Users/xulei/.dev/os` 执行；需要其他目录时明确写出。
- 使用 `cargo run --package rust-os-runner`、`cargo build --package kernel --target x86_64-unknown-none` 等目标明确的命令，避免读者不知道 Cargo 在构建哪个 package。
- 动态输出只描述可验证特征。例如 QEMU 版本要求以 `QEMU emulator version` 开头，不预先虚构具体版本号。
- `bootloader` 与 `bootloader_api` 精确固定为 `=0.11.17`；BIOS-only 构建关闭默认 UEFI feature。
- 工具链写 `nightly-2026-07-27`，组件写 `rustfmt`、`clippy`、`rust-src`、`llvm-tools`，target 写 `x86_64-unknown-none`。
- 第一阶段使用 `uart_16550 = "=0.4.0"`，并明确标记为下一实验将替换的暂时黑盒。
- 不复制参考仓库实现，不加入 UEFI、OVMF、自动 QEMU 退出、中断、内存管理、进程或文件系统代码。

## 知识链接

教程至少链接到以下已有章节：

- `[[01-基础认知/01-操作系统全景与启动过程#组件卡片 14：固件、bootloader 与内核启动|固件、bootloader 与内核入口]]`
- `[[01-基础认知/09-Rust内核启动与工具链#组件卡片 15：no_std、unsafe、ABI、ELF、linker 与 cross compilation|Rust 内核启动与工具链]]`
- `[[01-基础认知/10-手写内核学习路线#推荐的手写内核顺序|手写内核总体路线]]`
- `[[01-基础认知/04-println如何走到终端#运行故事一：println! 如何走到终端|普通 stdout 与内核串口输出的区别]]`
- `[[01-基础认知/05-文件、设备与IO|设备、驱动与 I/O]]`

同一 Vault 内全部使用 WikiLink，不创建跨 Vault WikiLink。

## 状态与后续更新

教程首次生成时顶部明确显示：

- 教程状态：已准备。
- 实验状态：尚未执行。

只有学习者真实运行并看到固定消息后，才把实验状态改为已完成，并记录实际 Rust、Cargo、QEMU 版本与实现 commit。理解检查的勾选状态也只能在学习者完成复述后更新。

## 验收标准

教程生成完成必须满足：

1. 文件位于 `Rust OS` iCloud Vault 的 `02-内核实验` 目录。
2. 学习者只阅读这一篇正文，就能获得从环境检查到首次串口输出所需的全部路径、配置、代码和命令。
3. 每个步骤都包含“做什么、为什么、完整输入、运行、预期、结论边界、失败检查”。
4. 所有阶段性文件与最终文件在文档内部一致，依赖版本、函数名、环境变量名和 QEMU 参数不互相矛盾。
5. 教程不宣称实验已经成功，也不修改项目中的核心配置或代码。
6. 学习主页、学习路线和教程之间存在可用 WikiLink。
7. 没有 `TBD`、`TODO`、省略号代码或要求读者自行补齐的配置。
8. Markdown 围栏、Mermaid、标题结构与 Obsidian callout 能正常解析。

## 技术依据

- 已批准的第一次启动设计：`docs/superpowers/specs/2026-08-01-rust-os-first-boot-design.md`
- 已核对的逐步实施计划：`docs/superpowers/plans/2026-08-01-rust-os-first-boot.md`
- rust-osdev bootloader 0.11.17 官方 basic 示例：<https://github.com/rust-osdev/bootloader/tree/v0.11.17/examples/basic>
- Cargo artifact dependencies：<https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies>
- QEMU invocation：<https://qemu.readthedocs.io/en/master/system/invocation.html>
