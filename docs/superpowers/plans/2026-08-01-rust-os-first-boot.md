# Rust OS 第一次可观察启动 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 在 Apple Silicon macOS 上执行 `cargo run`，通过 QEMU 的 x86_64 BIOS 启动链进入学习者亲手编写的 Rust 裸机入口，并在当前终端看到 `Rust OS: kernel entered`。

**Architecture:** 工程分为运行在 macOS 上的顶层 runner 和编译为 `x86_64-unknown-none` 的 `kernel`。Cargo artifact dependency 把 kernel ELF 交给 `rust-osdev/bootloader` 生成 BIOS 磁盘镜像，runner 再用 QEMU 软件模拟 x86_64 机器，并把客体 COM1 串口连接到宿主终端。

**Tech Stack:** Rust dated nightly、Cargo resolver 3 与 artifact dependencies、Rust 2024 edition、`x86_64-unknown-none`、`bootloader = 0.11.17`、`bootloader_api = 0.11.17`、`uart_16550 = 0.4.0`、Homebrew QEMU、BIOS 启动。

---

## 教练式执行约束

本计划是课程地图，不是让 Agent 一次性生成完整工程的授权。实际执行时遵守以下规则：

- 学习者亲手输入所有核心代码和配置；导师一次只展开一个步骤。
- 每个文件在输入前，先用一句话说明它属于宿主、构建阶段还是客体内核。
- 每次只新增一个边界，然后立即运行命令观察结果。
- 学习者复述当前步骤后才进入下一步；答案只要求概念正确，不要求背术语。
- 失败时先保留完整报错，只排查本步骤新增的文件，不同时改工具链、镜像构建和内核代码。
- Agent 可以核对命令输出、审查学习者输入和更新学习记录，但不代替学习者输入本计划中的核心工程文件。
- 当前工作树已有 `AGENTS.md`、`PROJECT_PROGRESS.md`、`PROJECT_TODO.md` 三项未提交改动；每次只能显式暂存本任务列出的文件，不使用 `git add .`。

## 范围边界

本计划只接通以下路径：

```text
cargo run
  -> macOS Cargo 构建 x86_64 kernel ELF
  -> build.rs 生成 BIOS 磁盘镜像
  -> macOS runner 启动 qemu-system-x86_64
  -> QEMU/BIOS 启动 bootloader
  -> bootloader 装载 ELF 并调用 kernel_main
  -> kernel 写 COM1
  -> QEMU 把 COM1 转发到当前终端
```

本计划不加入 UEFI、OVMF、自定义 target JSON、手写 bootloader、VGA/framebuffer、中断、页表、堆、进程、系统调用、fd、文件系统或自动退出 QEMU。

## 最终文件结构与职责

```text
os/
├── .cargo/
│   └── config.toml              # 只启用 Cargo artifact dependency
├── .gitignore                   # 忽略构建产物、镜像和本地 worktree
├── Cargo.toml                   # macOS runner package、workspace、镜像构建依赖
├── Cargo.lock                   # 固定本次实际解析的依赖版本
├── rust-toolchain.toml          # 固定 dated nightly、组件和裸机 target
├── build.rs                     # 用 kernel ELF 生成 BIOS 磁盘镜像
├── src/
│   └── main.rs                  # 运行在 macOS 上并启动 QEMU
└── kernel/
    ├── Cargo.toml               # x86_64 裸机内核 package
    └── src/
        └── main.rs              # no_std/no_main、入口、panic 和串口输出
```

文件边界固定如下：

- `src/main.rs` 可以使用 `std` 和 `std::process::Command`，因为它是 macOS 普通进程。
- `kernel/src/main.rs` 只能依赖裸机可用能力；它不调用 macOS，也没有普通进程的 `main` 环境。
- `build.rs` 运行在构建阶段，消费 kernel ELF，产出 BIOS 镜像，并把镜像路径交给 runner。
- `.cargo/config.toml` 只负责开启目前仍需 nightly 的 artifact dependency，不放 QEMU 参数或 target 默认值。

## 版本策略

- 工具链固定为 `nightly-2026-07-27`，不写浮动的 `nightly`。该日期的 Rust 官方发布清单已经确认 Apple Silicon 宿主所需组件与 `x86_64-unknown-none` 标准库均可用。
- `bootloader` 与 `bootloader_api` 同时精确固定为 `=0.11.17`，避免二者契约随补丁版本漂移。
- `uart_16550` 精确固定为 `=0.4.0`；下一里程碑再读取并替换它的端口 I/O 行为。
- 如果 Task 1 的 rustup 组件安装失败，必须停止并保留原始错误；不能私自换成当天 nightly 或删除组件。先核对该 dated nightly 的发行状态，再以一次独立计划修订更换日期。
- QEMU 由 Homebrew 安装；安装后的实际首行版本输出会记录到实验笔记，不在计划中猜测版本号。

## Task 1：建立可复现的宿主工具环境

**Files:**
- Modify: `rust-toolchain.toml`
- Modify: `.gitignore`

- [ ] **Step 1：确认仓库基线，不清理已有改动**

Run:

```bash
pwd
git status --short --branch
brew --version | head -n 1
command -v qemu-system-x86_64 || true
```

Expected:

- `pwd` 输出 `/Users/xulei/.dev/os`。
- 分支为 `main`；允许看到已有的 `AGENTS.md`、`PROJECT_PROGRESS.md`、`PROJECT_TODO.md` 修改。
- Homebrew 可用。
- 当前大概率找不到 `qemu-system-x86_64`；找不到表示工具未安装，不是内核错误。

- [ ] **Step 2：安装 QEMU**

Run:

```bash
brew install qemu
```

Expected: Homebrew 成功安装或报告 `qemu` 已经是最新可用版本，没有 `Error` 结尾。

- [ ] **Step 3：确认需要的模拟器程序存在**

Run:

```bash
command -v qemu-system-x86_64
qemu-system-x86_64 --version | head -n 1
```

Expected:

- 第一条输出 Homebrew 下的实际可执行文件绝对路径。
- 第二条以 `QEMU emulator version` 开头。

- [ ] **Step 4：把项目工具链切换为固定日期的 nightly**

Replace `rust-toolchain.toml` with:

```toml
[toolchain]
channel = "nightly-2026-07-27"
profile = "minimal"
components = ["clippy", "rustfmt", "rust-src", "llvm-tools"]
targets = ["x86_64-unknown-none"]
```

这里同时需要两个容易混淆的组件：项目自己的 kernel 使用 Rust 已提供的 `x86_64-unknown-none` 标准目标；bootloader 构建内部仍会为早期 BIOS stage 使用 `-Z build-std=core`，所以需要 `rust-src`。`llvm-tools` 是 rustup 接受的组件名，它映射到发布包 `llvm-tools-preview`，供 bootloader 镜像构建过程使用 `llvm-objcopy` 等工具。

- [ ] **Step 5：让 rustup 安装并确认被固定的工具链**

Run:

```bash
rustup show active-toolchain
rustc --version --verbose
cargo --version
```

Expected:

- 首次运行会下载 `nightly-2026-07-27-aarch64-apple-darwin` 及声明的组件和 target。
- active toolchain 是由 `/Users/xulei/.dev/os/rust-toolchain.toml` 覆盖得到的 dated nightly。
- `rustc` 的 `host` 仍是 `aarch64-apple-darwin`；切换工具链没有把 Mac 变成 x86_64。

- [ ] **Step 6：分别检查“宿主组件”和“客体 target”**

Run:

```bash
rustup component list --installed | rg 'clippy|rustfmt|rust-src|llvm-tools'
rustup target list --installed | rg 'aarch64-apple-darwin|x86_64-unknown-none'
```

Expected:

- 组件输出包含 `clippy`、`rustfmt`、`rust-src` 和 `llvm-tools` 对应的 installed 项。
- target 输出同时包含宿主 `aarch64-apple-darwin` 与客体 `x86_64-unknown-none`。

- [ ] **Step 7：补全构建产物忽略规则**

Replace `.gitignore` with:

```gitignore
.worktrees/
/target/
*.img
.DS_Store
```

- [ ] **Step 8：做第一次概念检查**

学习者用自己的话回答：

1. `x86_64-unknown-none` 是“生成哪种代码”的规则，还是“运行代码的虚拟机”？
2. QEMU 为什么是另一个独立工具？

Pass condition: 能说明 Rust target 负责生成 x86_64 裸机产物，QEMU 负责模拟一台能够执行该产物的机器；两者不是同一层。

- [ ] **Step 9：只提交工具环境文件**

Run:

```bash
git add .gitignore rust-toolchain.toml
git diff --cached --check
git diff --cached --name-only
git commit -m "chore: prepare Rust OS toolchain"
```

Expected:

- staged 文件只有 `.gitignore` 和 `rust-toolchain.toml`。
- `git diff --cached --check` 无输出。
- commit 成功；三份已有项目管理文件仍保持未暂存。

## Task 2：亲眼区分 macOS runner 与 x86_64 裸机 kernel

**Files:**
- Replace: `Cargo.toml`
- Create: `src/main.rs`
- Create: `kernel/Cargo.toml`
- Create: `kernel/src/main.rs`
- Generate: `Cargo.lock`

- [ ] **Step 1：创建两个 package 所需目录**

Run:

```bash
mkdir -p src kernel/src
```

Expected: 命令无输出，`src/` 与 `kernel/src/` 存在。

- [ ] **Step 2：把旧虚拟 workspace 改为带宿主 package 的 workspace**

Replace `Cargo.toml` with:

```toml
[workspace]
resolver = "3"
members = ["kernel"]

[package]
name = "rust-os-runner"
version = "0.1.0"
edition = "2024"
publish = false
```

解释边界：同一个文件同时声明 workspace 和根 package。workspace 负责组织成员；`[package]` 让根目录本身成为真正运行在 macOS 上的 runner。旧的 `[toolchain]` 被彻底删除，因为工具链只属于 `rust-toolchain.toml`。

- [ ] **Step 3：先写一个只证明宿主 package 可运行的 runner**

Create `src/main.rs`:

```rust
fn main() {
    println!("host runner: macOS process");
}
```

- [ ] **Step 4：声明裸机 kernel package**

Create `kernel/Cargo.toml`:

```toml
[package]
name = "kernel"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
bootloader_api = "=0.11.17"
```

- [ ] **Step 5：写出最小的裸机入口与 panic 停机路径**

Create `kernel/src/main.rs`:

```rust
#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {
        core::hint::spin_loop();
    }
}
```

解释边界：

- `#![no_std]` 表示不依赖建立在现成操作系统服务上的 `std`，但仍可使用 `core`。
- `#![no_main]` 关闭普通应用的 Rust 入口假设。
- `entry_point!` 按 bootloader 契约注册真正的内核入口，并让入口接收 `BootInfo`。
- 返回类型 `!` 表示入口不会返回给一个不存在的普通调用者。
- 当前循环只是可控停机点；它不是调度器，也不是 Tokio runtime。

- [ ] **Step 6：运行宿主 package**

Run:

```bash
cargo run --package rust-os-runner
```

Expected: 最后一行是：

```text
host runner: macOS process
```

这一行来自 macOS 普通进程，尚未启动 QEMU，也不是内核输出。此时 kernel 文件已经存在只是为了让 workspace 结构有效，Cargo 运行的 package 仍由 `--package rust-os-runner` 明确选择。

- [ ] **Step 7：只构建 kernel ELF，不制作镜像**

Run:

```bash
cargo build --package kernel --target x86_64-unknown-none
```

Expected: 以 `Finished dev` 结束，并生成 `target/x86_64-unknown-none/debug/kernel`。

- [ ] **Step 8：直接观察两个 package 的架构差异**

Run:

```bash
file target/debug/rust-os-runner
file target/x86_64-unknown-none/debug/kernel
```

Expected:

- runner 是面向 macOS 的 `Mach-O 64-bit executable arm64`。
- kernel 输出中同时包含 `ELF 64-bit` 与 `x86-64`。

这两个结果是本阶段最重要的边界证据：Cargo 在同一个 workspace 中生成了两个运行地点完全不同的程序。

- [ ] **Step 9：做第二次概念检查**

学习者用自己的话回答：为什么根 `src/main.rs` 可以使用普通 `main` 和 `println!`，而 `kernel/src/main.rs` 要使用 `no_std`、`no_main` 与 bootloader 入口？

Pass condition: 能说明 runner 已经由 macOS 创建成普通进程；kernel 将在尚无自有操作系统服务的虚拟机中接管控制权，两者的启动前提不同。

- [ ] **Step 10：提交两个 package 的最小可编译边界**

Run:

```bash
git add Cargo.toml Cargo.lock src/main.rs kernel/Cargo.toml kernel/src/main.rs
git diff --cached --check
git diff --cached --name-only
git commit -m "feat: add host runner and bare kernel"
```

Expected: staged 文件只包含本 Task 列出的五个路径；commit 成功。

## Task 3：把 kernel ELF 变成 BIOS 可启动镜像

**Files:**
- Create: `.cargo/config.toml`
- Modify: `Cargo.toml`
- Create: `build.rs`
- Update: `Cargo.lock`

- [ ] **Step 1：只为 artifact dependency 开启 Cargo nightly 能力**

Run:

```bash
mkdir -p .cargo
```

Create `.cargo/config.toml`:

```toml
[unstable]
bindeps = true
```

这里的 `bindeps` 是 binary artifact dependencies：顶层 package 需要的不是把 `kernel` 当普通 Rust 库链接进 runner，而是取得它编译出的独立二进制文件。

- [ ] **Step 2：声明镜像构建依赖与 kernel artifact**

Replace `Cargo.toml` with:

```toml
[workspace]
resolver = "3"
members = ["kernel"]

[package]
name = "rust-os-runner"
version = "0.1.0"
edition = "2024"
publish = false

[build-dependencies]
bootloader = { version = "=0.11.17", default-features = false, features = ["bios"] }
kernel = { path = "kernel", artifact = "bin", target = "x86_64-unknown-none" }
```

解释边界：`bootloader` 是供 `build.rs` 调用的宿主构建库；关闭默认 feature 后只构建本里程碑需要的 BIOS 路径，不额外构建 UEFI。`kernel` 这一项要求 Cargo 额外产出一个 x86_64 裸机 binary artifact，而不是 Rust 库依赖。

- [ ] **Step 3：写 BIOS 镜像构建脚本**

Create `build.rs`:

```rust
use std::path::PathBuf;

fn main() {
    let out_dir = PathBuf::from(
        std::env::var_os("OUT_DIR").expect("Cargo did not provide OUT_DIR"),
    );
    let kernel = PathBuf::from(
        std::env::var_os("CARGO_BIN_FILE_KERNEL_kernel")
            .expect("Cargo did not provide the kernel binary artifact"),
    );

    let bios_path = out_dir.join("bios.img");
    bootloader::BiosBoot::new(&kernel)
        .create_disk_image(&bios_path)
        .expect("failed to create BIOS disk image");

    println!("cargo::rustc-env=BIOS_PATH={}", bios_path.display());
}
```

逐行边界：

- `OUT_DIR` 是 Cargo 给本次 build script 的构建产物目录。
- `CARGO_BIN_FILE_KERNEL_kernel` 由 artifact dependency 自动提供：前半个 `KERNEL` 来自依赖名，后半个 `kernel` 来自 binary 名。
- `BiosBoot::new` 消费 ELF 路径，`create_disk_image` 产出 BIOS 镜像。
- `cargo::rustc-env` 把镜像绝对路径嵌入随后编译的 runner；它不是给 kernel 设置环境变量。

- [ ] **Step 4：构建 runner，并让 build script 生成镜像**

Run:

```bash
cargo build --package rust-os-runner
```

Expected: Cargo 构建 kernel artifact、bootloader 构建依赖与 runner，最后以 `Finished dev` 结束。

- [ ] **Step 5：证明 BIOS 镜像真实存在且非空**

Run:

```bash
find target/debug/build -path '*/out/bios.img' -type f -size +0 -print
```

Expected: 至少输出一条以 `/out/bios.img` 结尾的路径。中间目录中的 Cargo hash 每次可能不同；只检查文件存在、非空和文件名正确。

- [ ] **Step 6：做第三次概念检查**

学习者按顺序说出这三个制品的区别：

1. `target/x86_64-unknown-none/debug/kernel`；
2. Cargo build-script 的 `OUT_DIR` 中生成的 `bios.img`；
3. `target/debug/rust-os-runner`。

Pass condition: 能分别说出“待装载的内核 ELF”“包含启动链的磁盘镜像”“在 macOS 上负责启动 QEMU 的程序”。

- [ ] **Step 7：提交镜像构建层**

Run:

```bash
git add .cargo/config.toml Cargo.toml Cargo.lock build.rs
git diff --cached --check
git diff --cached --name-only
git commit -m "build: create bootable BIOS image"
```

Expected: staged 文件只有 `.cargo/config.toml`、`Cargo.toml`、`Cargo.lock`、`build.rs`；commit 成功，`bios.img` 不在 Git 中。

## Task 4：让宿主 runner 启动 QEMU

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1：把临时宿主输出替换为 QEMU 启动程序**

Replace `src/main.rs` with:

```rust
use std::process::Command;

fn main() {
    let bios_path = env!("BIOS_PATH");
    let drive = format!("format=raw,file={bios_path}");

    let status = Command::new("qemu-system-x86_64")
        .arg("-accel")
        .arg("tcg")
        .arg("-drive")
        .arg(drive)
        .arg("-serial")
        .arg("mon:stdio")
        .arg("-display")
        .arg("none")
        .arg("-no-reboot")
        .status()
        .expect("failed to start qemu-system-x86_64");

    assert!(status.success(), "QEMU exited with status {status}");
}
```

参数职责：

- `-accel tcg` 明确使用软件模拟，让 Apple Silicon 宿主执行 x86_64 客体指令。
- `-drive` 后面的 raw 磁盘参数把 Task 3 的 BIOS 镜像作为客体磁盘；其中的实际路径由 `env!("BIOS_PATH")` 取得。
- `-serial mon:stdio` 把客体串口与 QEMU monitor 复用到当前终端。
- `-display none` 不打开一个暂时没有用途的图形窗口。
- `-no-reboot` 避免客体故障后自动重启而掩盖问题。

- [ ] **Step 2：格式化并检查宿主代码**

Run:

```bash
cargo fmt --all
cargo clippy --package rust-os-runner --no-deps -- -D warnings
```

Expected: 两条命令成功，Clippy 以 `Finished dev` 结束且没有 warning。

- [ ] **Step 3：第一次启动 QEMU，此时不要期待内核文字**

Run:

```bash
cargo run --package rust-os-runner
```

Expected:

- Cargo 显示正在运行 `target/debug/rust-os-runner`。
- QEMU 没有报告找不到镜像或无法启动。
- 因为 kernel 还没有串口写入，终端可能在启动后保持安静；“安静”不是本里程碑最终成功，只证明 runner 没有立即启动失败。

退出方式：按一次 `Ctrl+A`，松开后再按 `X`。这是 QEMU `mon:stdio` 的退出组合，不是客体内核实现的关机功能。

- [ ] **Step 4：做第四次概念检查**

学习者用自己的话回答：Apple Silicon 为什么能运行这个 x86_64 客体？

Pass condition: 能说明 runner 本身仍是 arm64 macOS 程序；QEMU 的 TCG 读取并模拟 x86_64 指令，速度可能比同架构硬件虚拟化慢，但客体不要求和宿主使用同一指令集。

- [ ] **Step 5：提交 QEMU runner**

Run:

```bash
git add src/main.rs
git diff --cached --check
git diff --cached --name-only
git commit -m "feat: launch BIOS image in QEMU"
```

Expected: staged 文件只有 `src/main.rs`；commit 成功。

## Task 5：让 kernel 通过 COM1 产生第一条可见消息

**Files:**
- Modify: `kernel/Cargo.toml`
- Modify: `kernel/src/main.rs`
- Update: `Cargo.lock`

- [ ] **Step 1：只加入本里程碑需要的串口依赖**

Replace `kernel/Cargo.toml` with:

```toml
[package]
name = "kernel"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
bootloader_api = "=0.11.17"
uart_16550 = "=0.4.0"
```

- [ ] **Step 2：先确认增加依赖没有破坏裸机编译**

Run:

```bash
cargo build --package kernel --target x86_64-unknown-none
```

Expected: 以 `Finished dev` 结束。此时行为还没变化，因为源码尚未调用串口。

- [ ] **Step 3：初始化 COM1，并在入口写出固定消息**

Replace `kernel/src/main.rs` with:

```rust
#![no_std]
#![no_main]

use bootloader_api::{entry_point, BootInfo};
use core::{fmt::Write, panic::PanicInfo};

entry_point!(kernel_main);

fn serial_port() -> uart_16550::SerialPort {
    let mut port = unsafe { uart_16550::SerialPort::new(0x3F8) };
    port.init();
    port
}

fn kernel_main(_boot_info: &'static mut BootInfo) -> ! {
    let mut serial = serial_port();
    writeln!(serial, "Rust OS: kernel entered").expect("failed to write to COM1");

    loop {
        core::hint::spin_loop();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    let _ = writeln!(serial_port(), "KERNEL PANIC: {info}");

    loop {
        core::hint::spin_loop();
    }
}
```

解释边界：

- `0x3F8` 是这条传统 PC/QEMU 路径中 COM1 的 I/O 端口基址。
- `unsafe` 只包住“声明独占控制这个硬件端口”的边界；Rust 无法自行证明端口存在或没有另一个所有者。
- `fmt::Write` 让 `writeln!` 把格式化结果交给 `SerialPort`，没有使用文件、fd 或标准输出。
- `panic_handler` 不能把错误交给 macOS；它尽力通过同一串口报告，然后停在循环中。
- `spin_loop` 只是自旋提示，不会主动让出 CPU，也不是线程休眠。

- [ ] **Step 4：格式化并分别检查客体与宿主**

Run:

```bash
cargo fmt --all
cargo clippy --package kernel --target x86_64-unknown-none --no-deps -- -D warnings
cargo clippy --package rust-os-runner --no-deps -- -D warnings
cargo build
```

Expected:

- rustfmt 无错误。
- 两次 Clippy 都没有 warning。
- 根目录 `cargo build` 成功构建 runner、kernel artifact 和 BIOS 镜像。

- [ ] **Step 5：完成第一次可观察启动**

Run:

```bash
cargo run
```

Expected: 当前终端明确出现下面这一行：

```text
Rust OS: kernel entered
```

看到后用 `Ctrl+A`，松开，再按 `X` 退出 QEMU。不得因为看到了 bootloader 自身日志就判定成功；验收标记只能是学习者在 `kernel_main` 中写下的这条固定消息。

- [ ] **Step 6：证明构建产物没有混进 Git**

Run:

```bash
git status --short
git ls-files | rg '(^|/)target/|\.img$' || true
```

Expected:

- `git status --short` 只显示源码、锁文件或原有项目管理文件的改动。
- 第二条命令无输出，说明已跟踪文件中没有 `target/` 或磁盘镜像。

- [ ] **Step 7：提交第一条内核输出**

Run:

```bash
git add Cargo.lock kernel/Cargo.toml kernel/src/main.rs
git diff --cached --check
git diff --cached --name-only
git commit -m "feat: print first kernel serial message"
```

Expected: staged 文件只有 `Cargo.lock`、`kernel/Cargo.toml`、`kernel/src/main.rs`；commit 成功。

## Task 6：完成验收与口头回路

**Files:**
- No source changes

- [ ] **Step 1：执行最终机械检查**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --package kernel --target x86_64-unknown-none --no-deps -- -D warnings
cargo clippy --package rust-os-runner --no-deps -- -D warnings
cargo build
git diff --check
```

Expected: 所有命令退出码为 0；`git diff --check` 无输出。

- [ ] **Step 2：再次核对三个制品**

Run:

```bash
file target/debug/rust-os-runner
file target/x86_64-unknown-none/debug/kernel
find target/debug/build -path '*/out/bios.img' -type f -size +0 -print
```

Expected:

- arm64 macOS runner 存在。
- x86-64 ELF kernel 存在。
- 至少一个非空 BIOS 镜像存在。

- [ ] **Step 3：执行最终人工启动验收**

Run:

```bash
cargo run
```

Expected: 终端出现且完整匹配 `Rust OS: kernel entered`。观察后按 `Ctrl+A`，松开，再按 `X` 退出。

- [ ] **Step 4：完成五问口头检查**

学习者用自己的话回答：

1. 为什么 kernel 使用 `no_std`？
2. 为什么 kernel 使用 `no_main`，又是谁最终调用 `kernel_main`？
3. runner、`build.rs`、bootloader、kernel 分别运行在什么阶段或位置？
4. kernel 的字符串为什么最后出现在 macOS 终端，而它并没有调用 macOS 的 stdout？
5. Apple Silicon 为什么能够运行 x86_64 kernel？

Pass conditions:

- `no_std`：kernel 不能预设文件、线程、Socket 等宿主 OS 服务已经存在；仍可用 `core`。
- `no_main`：普通应用入口环境不存在；bootloader 按 `entry_point!` 建立的契约转入 `kernel_main`。
- 组件位置：runner 与 `build.rs` 在 macOS；bootloader 与 kernel 在 QEMU 客体启动链中；`build.rs` 只在编译阶段运行。
- 串口路径：kernel 写 COM1 端口，QEMU 模拟该设备并把字节转发到宿主终端。
- 跨架构：Rust 交叉编译生成 x86_64 指令，QEMU TCG 在 arm64 宿主上软件模拟这些指令。

若任何一问说不清，只回到对应 Task 的单个边界重新观察，不增加新代码。

## Task 7：把实验结果接回 Rust OS 知识网络

**Files:**
- Create: `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/02-内核实验/01-第一次启动 Rust 内核.md`
- Modify: `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/00-学习主页.md`
- Modify: `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/01-基础认知/10-手写内核学习路线.md`
- Modify: `PROJECT_PROGRESS.md`
- Modify: `PROJECT_TODO.md`

- [ ] **Step 1：采集要记录的真实环境证据**

Run:

```bash
rustc --version
cargo --version
qemu-system-x86_64 --version | head -n 1
git log -1 --oneline
```

Expected: 四条命令都输出实际值。项目记忆必须使用这些真实输出，不猜测 QEMU 版本，也不把计划编写时的 commit 当成实验完成 commit。

- [ ] **Step 2：在现有 Rust OS Vault 中创建实验记录**

Create `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/02-内核实验/01-第一次启动 Rust 内核.md` with:

````markdown
# 实验 01：第一次启动 Rust 内核

## 结果

在 Apple Silicon macOS 上，Cargo 构建了 x86_64 裸机 kernel ELF，bootloader 构建层把它制作为 BIOS 磁盘镜像，macOS runner 再通过 QEMU TCG 启动该镜像。内核入口最终通过 COM1 输出：

```text
Rust OS: kernel entered
```

这次结果只证明实际测试到的 BIOS 启动链、内核入口与最小串口输出路径已经接通；它不证明中断、内存管理、进程或文件系统已经存在。

## 已验证环境

- 宿主：Apple Silicon macOS。
- Rust channel：`nightly-2026-07-27`。
- 内核编译目标：`x86_64-unknown-none`。
- 客体执行方式：Homebrew QEMU 的 x86_64 system emulator，使用 TCG 软件模拟。
- 精确 Rust、Cargo、QEMU 版本与实现 commit 保存在仓库 `PROJECT_PROGRESS.md`。

## 三个制品

- `target/debug/rust-os-runner`：运行在 macOS 上的 arm64 普通进程，负责启动 QEMU。
- `target/x86_64-unknown-none/debug/kernel`：等待 bootloader 装载的 x86-64 ELF 内核。
- `bios.img`：包含 BIOS 启动链并携带 kernel ELF 的磁盘镜像。

## 我现在能解释

- [x] `no_std` 与“完全没有库”不是一回事，kernel 仍可使用 `core`。
- [x] `no_main` 是因为普通应用入口的前提不存在。
- [x] `build.rs` 在宿主构建阶段运行，不是内核的一部分。
- [x] 内核写 COM1，QEMU 把串口字节转发到 macOS 终端。
- [x] 交叉编译负责生成 x86_64 指令，QEMU TCG 负责在 arm64 主机上执行其模拟。

## 相关基础

- [[01-基础认知/01-操作系统全景与启动过程#组件卡片 14：固件、bootloader 与内核启动|启动链]]
- [[01-基础认知/09-Rust内核启动与工具链#组件卡片 15：no_std、unsafe、ABI、ELF、linker 与 cross compilation|Rust 裸机工具链]]
- [[01-基础认知/10-手写内核学习路线#推荐的手写内核顺序|手写内核路线]]
- [[01-基础认知/04-println如何走到终端#运行故事一：println! 如何走到终端|普通输出与内核串口输出的区别]]
````

- [ ] **Step 3：用已完成实验的真实入口更新学习主页**

Replace `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/00-学习主页.md` with:

```markdown
这个 Vault 专门保存 Rust OS 的学习资料、内核实验记录与设计决策。通用 Rust、Tokio 和网络知识继续保存在“项目学习” Vault。

## 从这里开始

1. [[01-基础认知/00-章节导航|按主题查阅基础认知]]
2. [[02-内核实验/01-第一次启动 Rust 内核|查看第一次内核启动实验]]
3. 后续学习继续采用“一个概念、一个极小实验、一次复述”的节奏。

## 动手实验

1. [[02-内核实验/01-第一次启动 Rust 内核|实验 01：第一次启动 Rust 内核]]

## Vault 边界

- 这里保存 Rust OS 项目专用知识。
- 可复用于其他项目的 Rust/Tokio 知识留在通用 Vault，使用跨 Vault 链接跳转。
- [打开通用知识 Vault“项目学习”](obsidian://open?vault=%E9%A1%B9%E7%9B%AE%E5%AD%A6%E4%B9%A0)。

## 当前阶段

实验 01 已完成；下一步是亲手实现最小 COM1 串口驱动，替换第一阶段使用的 `uart_16550`。
```

- [ ] **Step 4：为学习路线增加反向入口**

在 `/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/01-基础认知/10-手写内核学习路线.md` 的分隔线 `---` 之前加入：

```markdown
## 对应实验

- [[02-内核实验/01-第一次启动 Rust 内核|实验 01：第一次启动 Rust 内核]]
```

- [ ] **Step 5：更新项目记忆，但不把它混入内核实现提交**

Update `PROJECT_PROGRESS.md` so that it records all of the following facts using the real versions and commit from Step 1:

- 当前阶段为“第一次启动完成，准备手写最小串口驱动”。
- `cargo run` 已在 QEMU 中观察到 `Rust OS: kernel entered`。
- dated nightly、QEMU 版本和实现 commit 的真实值。
- runner、kernel ELF 与 BIOS 镜像三层边界已通过 `file` 和实际启动验证。

Update `PROJECT_TODO.md` so that:

- “第一次启动”相关项已完成。
- 唯一紧邻的下一步是设计并实现最小 COM1 端口 I/O 驱动，以替换 `uart_16550`。
- 中断、内存管理、进程与文件系统仍保留为后续事项，不提前展开。

- [ ] **Step 6：检查 Vault 链接与最终仓库状态**

Run:

```bash
rg -n '实验 01：第一次启动 Rust 内核' '/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/00-学习主页.md' '/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/01-基础认知/10-手写内核学习路线.md' '/Users/xulei/Library/Mobile Documents/iCloud~md~obsidian/Documents/Rust OS/02-内核实验/01-第一次启动 Rust 内核.md'
```

Expected:

- 三个 Vault 文件都能检索到实验标题，形成主页、路线与实验记录之间的双向入口。

- [ ] **Step 7：单独提交项目记忆**

Run:

```bash
git add PROJECT_PROGRESS.md PROJECT_TODO.md
git diff --cached --check
git diff --cached --name-only
git commit -m "docs: record first Rust kernel boot"
```

Expected: staged 文件只有 `PROJECT_PROGRESS.md` 与 `PROJECT_TODO.md`；实验实现文件没有混入文档提交。

- [ ] **Step 8：检查最终仓库状态**

Run:

```bash
git status --short --branch
git ls-files | rg '(^|/)target/|\.img$' || true
```

Expected:

- Git 中只剩执行本计划前就存在且本计划未获授权处理的改动。
- 第二条命令无输出；Git 没有跟踪 `target/` 或 `.img`。

## 故障隔离表

| 首个失败点 | 当前只检查 | 暂时不要改 |
|---|---|---|
| `rustup` 安装失败 | dated nightly 名称、组件名、target 名 | Cargo、kernel、QEMU 参数 |
| `cargo build -p kernel` 失败 | `kernel/Cargo.toml`、`kernel/src/main.rs`、裸机 target | `build.rs`、runner、QEMU |
| artifact 变量缺失 | `.cargo/config.toml`、根 `Cargo.toml` 的依赖名与 binary 名 | 串口代码、QEMU |
| `bios.img` 未生成 | `build.rs`、kernel ELF 路径、bootloader 版本 | runner、串口 |
| QEMU 进程起不来 | QEMU 安装、`BIOS_PATH`、`-drive` 参数 | kernel 业务代码 |
| QEMU 启动但无固定消息 | 镜像是否更新、入口是否到达、COM1 初始化、串口转发参数 | 中断、VGA、堆、文件系统 |
| 出现 `KERNEL PANIC` | panic 后紧邻信息与本次新增 kernel 代码 | 宿主 stdout 或网络设置 |

## 完成定义

只有同时满足以下条件，才能宣布本计划完成：

1. `cargo fmt --all -- --check`、两次目标明确的 Clippy 与 `cargo build` 全部成功。
2. `file` 证明 runner 是 arm64 Mach-O，kernel 是 x86-64 ELF。
3. 非空 BIOS 镜像由 `build.rs` 生成。
4. `cargo run` 启动 QEMU，终端出现精确消息 `Rust OS: kernel entered`。
5. 学习者通过五问口头检查，能够讲清构建阶段、宿主、客体和串口数据流。
6. Git 未跟踪 `target/`、`.img` 或其他生成物，并且每个实现 commit 只包含指定文件。
7. Rust OS Vault 中存在实验记录，主页和学习路线都有 WikiLink 入口，真实 Rust/QEMU/commit 版本已经写入。

## 官方参考

- rust-osdev bootloader：<https://github.com/rust-osdev/bootloader>
- bootloader 0.11.17 官方 basic 示例：<https://github.com/rust-osdev/bootloader/tree/v0.11.17/examples/basic>
- bootloader 0.11.17 文档：<https://docs.rs/bootloader/0.11.17/bootloader/>
- bootloader_api 0.11.17 文档：<https://docs.rs/bootloader_api/0.11.17/bootloader_api/>
- Cargo artifact dependencies：<https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#artifact-dependencies>
- Rust `nightly-2026-07-27` 发布清单：<https://static.rust-lang.org/dist/2026-07-27/channel-rust-nightly.toml>
- QEMU invocation 文档：<https://qemu.readthedocs.io/en/master/system/invocation.html>
