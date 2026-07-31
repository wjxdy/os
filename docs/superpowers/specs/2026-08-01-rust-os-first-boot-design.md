# Rust OS 第一次启动里程碑设计

## 目标

在 Apple Silicon macOS 上执行一条 Cargo 命令，通过 QEMU 启动一个 x86_64 裸机 Rust 内核，并从串口观察到学习者亲手写下的固定消息：

```text
Rust OS: kernel entered
```

这一结果只证明本次构建产生的启动镜像能够经过 bootloader 进入指定内核入口，并且最小串口输出路径可用；它不证明中断、内存管理、调度或其他设备已经实现。

## 教学原则

- 学习者亲手输入所有课程代码和配置；导师不一次性生成完整工程。
- 每次只引入一个新文件或一个新概念，输入前先解释它解决什么问题。
- 每一步都先运行并观察，再用学习者自己的话解释结果。
- 配置文件也是程序行为的一部分，但只在当前里程碑确实需要时引入。
- bootloader 第一阶段作为有明确契约的基础设施使用；后续再单独学习它替内核完成了哪些启动工作。
- 第一条启动链路成功前，不加入中断、页表、堆、进程、文件系统或图形输出。

## 技术路线

- 主机：Apple Silicon macOS。
- 客体体系结构：x86_64。
- 虚拟机：`qemu-system-x86_64`，使用软件模拟即可，不要求客体与主机指令集相同。
- 启动方式：`rust-osdev/bootloader` 0.11 系列的 BIOS 镜像。
- 内核目标：Rust 官方 `x86_64-unknown-none` target，不创建自定义 target JSON。
- Rust：固定到一个实际构建验证通过的 dated nightly，并安装 `llvm-tools-preview`、`rustfmt`、`clippy` 和 `x86_64-unknown-none`。
- 输出：第一阶段使用 `uart_16550` 完成串口输出；下一里程碑再用亲手实现的端口 I/O 替换它。

选择 BIOS 而不是 UEFI，是为了避免第一次启动额外引入 OVMF 固件；选择现成 bootloader 而不是手写 MBR，是为了先聚焦“内核入口与裸机执行环境”。

## 工程结构

第一里程碑最终只保留理解启动链路所需的结构：

```text
os/
├── .cargo/
│   └── config.toml          # 仅启用 Cargo artifact dependency
├── Cargo.toml               # 顶层 runner、workspace 与镜像构建依赖
├── Cargo.lock               # 固定依赖解析结果
├── rust-toolchain.toml      # 固定 nightly、组件和裸机 target
├── build.rs                 # 把 kernel ELF 交给 bootloader 生成 BIOS 镜像
├── src/
│   └── main.rs              # 启动 QEMU 的宿主程序
└── kernel/
    ├── Cargo.toml           # 裸机内核 package
    └── src/
        └── main.rs          # no_std/no_main、入口、panic 与串口输出
```

旧计划中的 `labs` 和 `freestanding` 不进入本里程碑。当前根 `Cargo.toml` 中误放的 `[toolchain]` 配置会在第一步校正；工具链配置只保留在 `rust-toolchain.toml`。

## 组件职责

### `kernel`

`kernel` 编译为 `x86_64-unknown-none` ELF。它不依赖宿主 macOS，也没有普通进程提供的标准库和 `main` 启动环境。

内核入口通过 `bootloader_api::entry_point!` 注册，签名接收 bootloader 传入的 `BootInfo`。第一里程碑只验证入口确实被调用，不读取或修改内存区域信息。

### 顶层 runner

顶层 package 是运行在 macOS 上的普通 Rust 程序，不是内核。它承担两项工作：

1. 构建阶段取得 `kernel` ELF，并调用 bootloader 创建 BIOS 磁盘镜像。
2. 运行阶段启动 `qemu-system-x86_64`，把串口连接到当前终端。

将宿主 runner 与裸机 kernel 分开，可以明确区分“帮助制作和启动镜像的普通程序”与“进入虚拟机后运行的内核”。

### bootloader

bootloader 负责建立第一阶段不手写的启动条件，包括从 BIOS 启动镜像、进入 x86_64 内核所需的执行环境、装载内核 ELF、准备栈和传递 `BootInfo`。内核不把 bootloader 当作自身的一部分，只依赖双方约定的入口契约。

### 串口

内核通过 COM1 串口把字节发送给 QEMU，QEMU 再把这些字节转发到 macOS 终端。第一阶段使用 `uart_16550` 降低首次启动变量数量；下一阶段读取其必要行为后，用最小自有驱动替换。

## 启动数据流

```text
cargo run
  → Cargo 为 x86_64-unknown-none 构建 kernel ELF
  → build.rs 使用 bootloader 生成 BIOS 磁盘镜像
  → 顶层 runner 启动 qemu-system-x86_64
  → QEMU 固件从镜像启动 bootloader
  → bootloader 装载 kernel ELF 并调用内核入口
  → kernel 初始化串口并发送固定消息
  → QEMU 将串口字节显示在当前终端
```

## 课程步骤

1. 检查并校正当前两个 TOML 文件的职责边界。
2. 安装 QEMU，确认可执行文件与版本。
3. 选择、固定并验证 dated nightly 及所需组件。
4. 创建 `kernel` package，只验证裸机 ELF 可以编译。
5. 加入 `no_std`、`no_main`、入口函数和 panic handler，并解释普通 `main` 为什么不可用。
6. 创建 BIOS 镜像构建层，验证镜像文件确实生成。
7. 创建宿主 runner，启动 QEMU 并连接串口。
8. 在内核中写出固定消息，完成第一次可观察启动。
9. 进行口头检查并记录验证命令，然后提交实现。

每一步失败时只排查当前新增的边界，不同时修改工具链、镜像构建和内核代码。

## 错误处理与诊断

- 工具缺失：明确指出缺少的是 QEMU、Rust 组件还是 target，不把环境错误误判为内核错误。
- 内核编译失败：只检查 `kernel` package 和裸机 target，不启动镜像构建。
- 镜像构建失败：记录实际 kernel ELF 路径、bootloader 版本和构建脚本错误。
- QEMU 无输出：依次确认 QEMU 是否启动、镜像是否被使用、内核入口是否到达、串口参数是否连接；不直接加入更多打印设施。
- 内核 panic：panic handler 尝试写入串口后停机，不能依赖 `std`、文件或宿主终端 API。
- QEMU 需要人工停止属于第一里程碑允许的行为；自动退出机制放到后续测试里程碑。

## 验收标准

必须同时满足：

1. `cargo build` 能构建顶层 runner、kernel ELF 和 BIOS 镜像。
2. `cargo run` 能启动 `qemu-system-x86_64`。
3. 当前终端中能看到且只依赖内核串口路径产生的 `Rust OS: kernel entered`。
4. 学习者能够解释：
   - 为什么内核使用 `no_std` 和 `no_main`；
   - 顶层 runner 与 kernel 分别运行在哪里；
   - bootloader 在控制权交给内核前建立了哪些最小条件；
   - 串口输出为什么能出现在 macOS 终端；
   - Apple Silicon 主机为什么可以通过 QEMU 运行 x86_64 客体。
5. Git 只包含源文件、锁文件和必要配置，不提交 `target`、磁盘镜像或其他构建产物。

## 明确不在本阶段处理

- UEFI 和 OVMF。
- 手写 BIOS、MBR 或 bootloader。
- VGA/framebuffer 图形输出。
- IDT、异常和中断。
- 物理页分配、页表和内核堆。
- 用户态、进程、系统调用、fd 和文件系统。
- 自动化 QEMU 成功/失败退出。
- 从参考仓库复制实现。

## 官方参考

- rust-osdev bootloader：<https://github.com/rust-osdev/bootloader>
- 官方基础示例：<https://github.com/rust-osdev/bootloader/tree/main/examples/basic>
- bootloader 文档：<https://docs.rs/bootloader/0.11.17/bootloader/>
- bootloader_api 文档：<https://docs.rs/bootloader_api/0.11.17/bootloader_api/>
