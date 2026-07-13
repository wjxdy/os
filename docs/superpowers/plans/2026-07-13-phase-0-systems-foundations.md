# Phase 0 Systems Foundations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 建立可复现的 Rust 宿主实验工作区，并通过地址对齐、物理页分配、协作式调度、`unsafe` 所有权和 ELF/ABI 实验补齐进入裸机内核前的基础。

**Architecture:** 使用一个最小 Cargo workspace 承载两个 crate：`phase0-labs` 运行在 macOS 上，用测试驱动方式练习可移植的内核算法；`freestanding` 是一个 `no_std` 库，编译到 `x86_64-unknown-none` 并生成 ELF object 供检查。本阶段不安装 QEMU、不创建 bootloader 配置，也不创建内核入口。

**Tech Stack:** Rust 1.96.0 stable、Cargo resolver 3、Rust 2024 edition、`x86_64-unknown-none`、macOS Xcode `llvm-objdump`、内置测试框架、Clippy、rustfmt。

---

## 教练式执行约束

本计划服从已经确认的教练式教学协议：

- 配置、测试、命令和验收标准完整给出。
- 地址对齐、页分配器、调度器和 `OwnedPtr` 的核心实现由学习者亲手完成。
- 导师不得直接代写这些实现；遇到阻塞时按“概念提示 → 方向提示 → 伪代码 → 局部示例”逐级帮助。
- 每项实现通过后，学习者必须解释控制流和不变量，才能提交。
- Agent 只可用于只读审查、测试结果核对和课程资料整理，不得替代学习者完成练习。

## 阶段范围

阶段 0 完成后，学习者应能独立说明：

1. workspace、package、crate、target 和 profile 的区别。
2. 地址对齐为何要求检查零、非 2 的幂和整数溢出。
3. 物理页分配器必须维持哪些集合不变量。
4. 协作式 round-robin 调度器如何进行状态转换。
5. 一个安全 Rust API 包裹 `unsafe` 时，谁负责保证哪些前提。
6. ABI、`#[repr(C)]`、符号、section、ELF 和 `no_std` 分别解决什么问题。

阶段 0 不涉及 QEMU、bootloader、串口、GDT、IDT、页表或真实上下文切换。

## 文件结构

本计划将创建或修改以下文件：

```text
os/
├── .gitignore                         # 构建产物与 macOS 临时文件
├── Cargo.toml                         # workspace 根清单
├── Cargo.lock                         # 固定解析结果
├── rust-toolchain.toml                # 阶段 0 的稳定工具链与裸机 target
├── labs/
│   ├── Cargo.toml                     # 宿主实验 crate
│   ├── src/
│   │   ├── lib.rs                     # 模块出口与 unsafe lint
│   │   ├── address.rs                 # 地址对齐
│   │   ├── frame.rs                   # 物理页分配模拟器
│   │   ├── scheduler.rs               # 协作式 round-robin 状态机
│   │   ├── owned_ptr.rs               # unsafe 所有权边界实验
│   │   └── bin/abi_probe.rs           # repr(C) 布局观察程序
│   └── tests/
│       ├── address_alignment.rs       # 地址对齐黑盒测试
│       ├── frame_allocator.rs         # 页分配器黑盒测试
│       ├── scheduler.rs               # 调度状态机黑盒测试
│       └── owned_ptr.rs               # unsafe 包装与析构测试
├── freestanding/
│   ├── Cargo.toml                     # no_std 裸机库清单
│   └── src/lib.rs                     # 可生成 x86_64 ELF object 的函数
└── docs/lessons/
    ├── 00-cargo-map.md                # Cargo 心智模型实验记录
    ├── 06-abi-elf-no-std.md           # ABI/ELF/no_std 观察记录
    └── phase-0-review.md              # 阶段口试与复盘
```

## Task 1：创建可复现的 Cargo workspace

**Files:**
- Create: `.gitignore`
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `labs/Cargo.toml`
- Create: `labs/src/lib.rs`
- Create: `freestanding/Cargo.toml`
- Create: `freestanding/src/lib.rs`
- Generate: `Cargo.lock`

- [ ] **Step 1：确认当前基线**

Run:

```bash
rustc --version
cargo --version
rustup show active-toolchain
git status --short --branch
```

Expected:

- `rustc 1.96.0` 与 `cargo 1.96.0`。
- 当前工具链为 `stable-aarch64-apple-darwin`。
- Git 分支为 `main`，开始本任务前工作树干净。

- [ ] **Step 2：创建 workspace 根清单**

Create `Cargo.toml`:

```toml
[workspace]
resolver = "3"
members = ["labs", "freestanding"]
```

- [ ] **Step 3：固定阶段 0 工具链**

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "1.96.0"
profile = "minimal"
components = ["clippy", "rustfmt", "rust-src"]
targets = ["x86_64-unknown-none"]
```

进入阶段 1 时，这个文件会在首次 QEMU 启动验证后整体切换为一个明确日期的 nightly；阶段 0 禁止使用浮动 channel。

- [ ] **Step 4：创建忽略规则**

Create `.gitignore`:

```gitignore
/target/
.DS_Store
```

- [ ] **Step 5：创建宿主实验 crate**

Create `labs/Cargo.toml`:

```toml
[package]
name = "phase0-labs"
version = "0.1.0"
edition = "2024"
publish = false

[dependencies]
```

Create `labs/src/lib.rs`:

```rust
#![forbid(unsafe_op_in_unsafe_fn)]
```

- [ ] **Step 6：创建 `no_std` crate**

Create `freestanding/Cargo.toml`:

```toml
[package]
name = "freestanding"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
path = "src/lib.rs"

[dependencies]
```

Create `freestanding/src/lib.rs`:

```rust
#![no_std]

pub fn wrapping_add(left: u64, right: u64) -> u64 {
    left.wrapping_add(right)
}
```

- [ ] **Step 7：让 rustup 安装被固定的组件和 target**

Run:

```bash
rustup show active-toolchain
rustup target list --installed
```

Expected:

- active toolchain 包含 `1.96.0-aarch64-apple-darwin`，并注明由本项目覆盖。
- installed targets 同时包含 `aarch64-apple-darwin` 和 `x86_64-unknown-none`。

- [ ] **Step 8：生成锁文件并验证 workspace**

Run:

```bash
cargo check --workspace
cargo test --workspace
cargo tree --workspace
```

Expected:

- `cargo check` 和 `cargo test` 退出码为 0。
- `cargo tree` 列出 `freestanding v0.1.0` 与 `phase0-labs v0.1.0`。
- 根目录生成 `Cargo.lock`。

- [ ] **Step 9：提交 workspace**

```bash
git add .gitignore Cargo.toml Cargo.lock rust-toolchain.toml labs freestanding
git commit -m "chore: initialize phase zero workspace"
```

## Task 2：建立 Cargo 心智模型

**Files:**
- Create: `docs/lessons/00-cargo-map.md`

- [ ] **Step 1：创建课程记录**

Create `docs/lessons/00-cargo-map.md`:

```markdown
# Cargo 心智模型

## 实验命令

1. `cargo metadata --no-deps --format-version 1`
2. `cargo tree --workspace`
3. `cargo check --workspace`
4. `cargo test --workspace`
5. `cargo test -p phase0-labs`
6. `cargo check -p freestanding --target x86_64-unknown-none`

## 必答问题

1. workspace 与 package 分别解决什么问题？
2. 一个 package 为什么可以包含 library target 和 binary target？
3. `phase0-labs` 为什么在 Rust 源码中写成 `phase0_labs`？
4. `cargo check`、`cargo build` 和 `cargo test` 的产物及用途有什么差异？
5. `--package` 与 `--target` 指向的是同一种概念吗？
6. `Cargo.toml` 与 `Cargo.lock` 分别描述意图还是解析结果？
7. `debug` 与 `release` profile 会影响哪些构建属性？

## 观察结论

本节只记录亲自运行命令后能够从输出证明的结论。每个结论都附对应命令，不抄写定义。
```

- [ ] **Step 2：观察 Cargo 元数据**

Run:

```bash
cargo metadata --no-deps --format-version 1
```

Expected: JSON 的 `packages` 中存在 `phase0-labs` 和 `freestanding`，每个 package 都有自己的 `targets`。

- [ ] **Step 3：分别验证 package 与 compilation target**

Run:

```bash
cargo test -p phase0-labs
cargo check -p freestanding --target x86_64-unknown-none
```

Expected: 两条命令均退出 0；第一条在 macOS host 上运行测试，第二条只为裸机 x86_64 编译。

- [ ] **Step 4：用自己的话回答七个问题**

在 `docs/lessons/00-cargo-map.md` 的“观察结论”中逐条作答。每条回答至少引用一个亲自运行的命令或输出字段。

- [ ] **Step 5：完成口头检查**

导师随机给出一个 Cargo 命令，学习者必须指出：它选择了哪个 package、哪个 target、哪个平台和哪个 profile。回答不清时只回看命令输出，不进入下一课。

- [ ] **Step 6：提交 Cargo 课程记录**

```bash
git add docs/lessons/00-cargo-map.md
git commit -m "docs: record Cargo workspace model"
```

## Task 3：用测试驱动实现地址对齐

**Files:**
- Create: `labs/tests/address_alignment.rs`
- Create: `labs/src/address.rs`
- Modify: `labs/src/lib.rs`

- [ ] **Step 1：先写黑盒测试**

Create `labs/tests/address_alignment.rs`:

```rust
use phase0_labs::address::{AlignError, align_down, align_up, is_aligned};

#[test]
fn aligns_address_down() {
    assert_eq!(align_down(0x1234, 0x1000), Ok(0x1000));
}

#[test]
fn aligns_address_up() {
    assert_eq!(align_up(0x1234, 0x1000), Ok(0x2000));
}

#[test]
fn keeps_an_aligned_address_unchanged() {
    assert_eq!(align_down(0x2000, 0x1000), Ok(0x2000));
    assert_eq!(align_up(0x2000, 0x1000), Ok(0x2000));
}

#[test]
fn detects_alignment() {
    assert_eq!(is_aligned(0x2000, 0x1000), Ok(true));
    assert_eq!(is_aligned(0x2001, 0x1000), Ok(false));
}

#[test]
fn rejects_zero_alignment() {
    assert_eq!(align_up(0x1234, 0), Err(AlignError::Zero));
}

#[test]
fn rejects_non_power_of_two_alignment() {
    assert_eq!(
        align_down(0x1234, 3),
        Err(AlignError::NotPowerOfTwo { alignment: 3 })
    );
}

#[test]
fn reports_align_up_overflow() {
    assert_eq!(
        align_up(usize::MAX, 2),
        Err(AlignError::Overflow {
            address: usize::MAX,
            alignment: 2,
        })
    );
}
```

- [ ] **Step 2：运行测试并确认红灯**

Run:

```bash
cargo test -p phase0-labs --test address_alignment
```

Expected: 编译失败，错误指向 `phase0_labs::address` 尚不存在。失败原因必须是缺少被测接口，而不是测试语法错误。

- [ ] **Step 3：独立定义公开接口**

在 `labs/src/address.rs` 中定义并导出：

```text
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignError {
    Zero,
    NotPowerOfTwo { alignment: usize },
    Overflow { address: usize, alignment: usize },
}

pub fn align_down(address: usize, alignment: usize) -> Result<usize, AlignError>;
pub fn align_up(address: usize, alignment: usize) -> Result<usize, AlignError>;
pub fn is_aligned(address: usize, alignment: usize) -> Result<bool, AlignError>;
```

以上代码块是接口契约的签名表示，不是可直接编译的源码；函数体由学习者在本课中编写。`labs/src/lib.rs` 增加 `pub mod address;`。

- [ ] **Step 4：实现最小正确行为**

实现必须满足：

- 在任何算术前验证 alignment。
- alignment 为 0 时返回 `Zero`。
- alignment 不是 2 的幂时返回 `NotPowerOfTwo`。
- `align_up` 的中间运算溢出时返回 `Overflow`，不能 panic 或回绕。
- 不使用 `unsafe`，不针对测试数据写分支。

- [ ] **Step 5：运行测试并确认绿灯**

Run:

```bash
cargo test -p phase0-labs --test address_alignment
```

Expected: `7 passed; 0 failed`。

- [ ] **Step 6：运行静态检查**

Run:

```bash
cargo fmt --check
cargo clippy -p phase0-labs --all-targets -- -D warnings
```

Expected: 两条命令均退出 0，且没有 warning。

- [ ] **Step 7：解释不变量**

学习者必须解释为什么 2 的幂对齐可用位运算表达、为什么 `align_up` 比 `align_down` 多一个溢出风险，以及错误检查应发生在运算前的原因。

- [ ] **Step 8：提交地址模块**

```bash
git add labs/src/lib.rs labs/src/address.rs labs/tests/address_alignment.rs
git commit -m "feat: add checked address alignment"
```

## Task 4：实现物理页分配模拟器

**Files:**
- Create: `labs/tests/frame_allocator.rs`
- Create: `labs/src/frame.rs`
- Modify: `labs/src/lib.rs`

- [ ] **Step 1：写页分配器黑盒测试**

Create `labs/tests/frame_allocator.rs`:

```rust
use phase0_labs::frame::{
    Frame, FrameAllocator, FrameAllocatorError, PAGE_SIZE,
};

#[test]
fn allocates_aligned_frames_in_ascending_order() {
    let mut allocator = FrameAllocator::new(0x1000, 0x4000).unwrap();

    assert_eq!(allocator.allocate().unwrap().start_address(), 0x1000);
    assert_eq!(allocator.allocate().unwrap().start_address(), 0x2000);
    assert_eq!(allocator.allocate().unwrap().start_address(), 0x3000);
    assert_eq!(allocator.allocate(), None);
}

#[test]
fn reuses_a_deallocated_frame() {
    let mut allocator = FrameAllocator::new(0x1000, 0x3000).unwrap();
    let first = allocator.allocate().unwrap();

    allocator.deallocate(first).unwrap();

    assert_eq!(allocator.allocate(), Some(first));
}

#[test]
fn rejects_unaligned_start() {
    assert!(matches!(
        FrameAllocator::new(0x1001, 0x4000),
        Err(FrameAllocatorError::UnalignedStart { address: 0x1001 })
    ));
}

#[test]
fn rejects_unaligned_end() {
    assert!(matches!(
        FrameAllocator::new(0x1000, 0x4001),
        Err(FrameAllocatorError::UnalignedEnd { address: 0x4001 })
    ));
}

#[test]
fn rejects_empty_or_reversed_range() {
    assert!(matches!(
        FrameAllocator::new(0x2000, 0x2000),
        Err(FrameAllocatorError::InvalidRange {
            start: 0x2000,
            end: 0x2000,
        })
    ));
    assert!(matches!(
        FrameAllocator::new(0x3000, 0x2000),
        Err(FrameAllocatorError::InvalidRange {
            start: 0x3000,
            end: 0x2000,
        })
    ));
}

#[test]
fn rejects_a_foreign_frame() {
    let mut allocator = FrameAllocator::new(0x1000, 0x3000).unwrap();
    let foreign = Frame::from_start_address(0x5000).unwrap();

    assert_eq!(
        allocator.deallocate(foreign),
        Err(FrameAllocatorError::ForeignFrame { address: 0x5000 })
    );
}

#[test]
fn rejects_double_free() {
    let mut allocator = FrameAllocator::new(0x1000, 0x3000).unwrap();
    let frame = allocator.allocate().unwrap();
    allocator.deallocate(frame).unwrap();

    assert_eq!(
        allocator.deallocate(frame),
        Err(FrameAllocatorError::DoubleFree {
            address: frame.start_address(),
        })
    );
}

#[test]
fn page_size_is_four_kibibytes() {
    assert_eq!(PAGE_SIZE, 4096);
}
```

- [ ] **Step 2：运行测试并确认红灯**

Run:

```bash
cargo test -p phase0-labs --test frame_allocator
```

Expected: 编译失败，错误指向 `phase0_labs::frame` 尚不存在。

- [ ] **Step 3：定义公开 API 与状态**

`labs/src/frame.rs` 必须公开：

- `pub const PAGE_SIZE: usize = 4096;`
- `Frame`：派生 `Debug`、`Clone`、`Copy`、`PartialEq` 和 `Eq`，只能表示页对齐的起始地址。
- `Frame::from_start_address(usize) -> Option<Frame>`。
- `Frame::start_address(self) -> usize`。
- `FrameAllocator::new(start, end_exclusive) -> Result<Self, FrameAllocatorError>`。
- `FrameAllocator::allocate(&mut self) -> Option<Frame>`。
- `FrameAllocator::deallocate(&mut self, frame) -> Result<(), FrameAllocatorError>`。
- `FrameAllocatorError` 的五个变体必须与测试完全一致，并派生 `Debug`、`PartialEq` 和 `Eq`。

在 `labs/src/lib.rs` 增加 `pub mod frame;`。

- [ ] **Step 4：独立实现页分配器**

实现必须始终保持：

- 每个 frame 都在 `[start, end_exclusive)` 内且按 `PAGE_SIZE` 对齐。
- 同一 frame 不能同时处于 free 与 allocated 状态。
- 每次成功分配返回唯一 frame。
- 正常初始分配顺序按地址递增。
- 释放后的 frame 在下一次分配时可复用。
- foreign frame 与 double free 返回不同错误。

允许使用 `Vec`、`VecDeque` 或有序集合，但必须说明所选结构如何维护上述不变量。

- [ ] **Step 5：运行页分配器测试**

Run:

```bash
cargo test -p phase0-labs --test frame_allocator
```

Expected: `8 passed; 0 failed`。

- [ ] **Step 6：执行额外边界检查**

Append to `labs/tests/frame_allocator.rs`:

```rust
#[test]
fn supports_a_single_frame_range() {
    let mut allocator = FrameAllocator::new(0x1000, 0x2000).unwrap();

    assert_eq!(allocator.allocate().unwrap().start_address(), 0x1000);
    assert_eq!(allocator.allocate(), None);
}
```

重新运行测试。

Expected: 新测试通过，总测试数变为 `9 passed; 0 failed`。

- [ ] **Step 7：运行格式与 lint**

```bash
cargo fmt --check
cargo clippy -p phase0-labs --all-targets -- -D warnings
```

Expected: 退出码为 0，无 warning。

- [ ] **Step 8：解释状态模型**

学习者必须画出 free、allocated、foreign 三类 frame 的集合关系，并说明为何重复释放会破坏未来分配的唯一性。

- [ ] **Step 9：提交页分配器**

```bash
git add labs/src/lib.rs labs/src/frame.rs labs/tests/frame_allocator.rs
git commit -m "feat: simulate physical frame allocation"
```

## Task 5：实现协作式 round-robin 调度状态机

**Files:**
- Create: `labs/tests/scheduler.rs`
- Create: `labs/src/scheduler.rs`
- Modify: `labs/src/lib.rs`

- [ ] **Step 1：写调度器黑盒测试**

Create `labs/tests/scheduler.rs`:

```rust
use phase0_labs::scheduler::{
    Scheduler, SchedulerError, TaskId, TaskState,
};

#[test]
fn schedules_ready_tasks_round_robin() {
    let mut scheduler = Scheduler::new();
    scheduler.spawn(TaskId::new(1)).unwrap();
    scheduler.spawn(TaskId::new(2)).unwrap();
    scheduler.spawn(TaskId::new(3)).unwrap();

    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(1)));
    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(2)));
    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(3)));
    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(1)));
}

#[test]
fn blocked_task_does_not_run_until_woken() {
    let mut scheduler = Scheduler::new();
    scheduler.spawn(TaskId::new(1)).unwrap();
    scheduler.spawn(TaskId::new(2)).unwrap();
    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(1)));

    assert_eq!(scheduler.block_current(), Ok(TaskId::new(1)));
    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(2)));
    assert_eq!(scheduler.state(TaskId::new(1)), Some(TaskState::Blocked));

    scheduler.wake(TaskId::new(1)).unwrap();
    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(1)));
}

#[test]
fn exited_task_never_runs_again() {
    let mut scheduler = Scheduler::new();
    scheduler.spawn(TaskId::new(1)).unwrap();
    scheduler.spawn(TaskId::new(2)).unwrap();
    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(1)));

    assert_eq!(scheduler.exit_current(), Ok(TaskId::new(1)));
    assert_eq!(scheduler.state(TaskId::new(1)), Some(TaskState::Exited));
    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(2)));
    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(2)));
}

#[test]
fn rejects_duplicate_and_unknown_tasks() {
    let mut scheduler = Scheduler::new();
    scheduler.spawn(TaskId::new(1)).unwrap();

    assert_eq!(
        scheduler.spawn(TaskId::new(1)),
        Err(SchedulerError::DuplicateTask(TaskId::new(1)))
    );
    assert_eq!(
        scheduler.wake(TaskId::new(99)),
        Err(SchedulerError::UnknownTask(TaskId::new(99)))
    );
}

#[test]
fn reports_missing_current_task() {
    let mut scheduler = Scheduler::new();

    assert_eq!(
        scheduler.block_current(),
        Err(SchedulerError::NoCurrentTask)
    );
    assert_eq!(
        scheduler.exit_current(),
        Err(SchedulerError::NoCurrentTask)
    );
}
```

- [ ] **Step 2：运行测试并确认红灯**

```bash
cargo test -p phase0-labs --test scheduler
```

Expected: 编译失败，错误指向 `phase0_labs::scheduler` 尚不存在。

- [ ] **Step 3：定义调度器公开 API**

`labs/src/scheduler.rs` 必须定义：

- `TaskId`：封装 `u64`，提供 `new`，派生 `Debug`、`Clone`、`Copy`、`PartialEq`、`Eq`、`PartialOrd`、`Ord` 和 `Hash`。
- `TaskState::{Ready, Running, Blocked, Exited}`，派生 `Debug`、`Clone`、`Copy`、`PartialEq` 和 `Eq`。
- `SchedulerError::{DuplicateTask(TaskId), UnknownTask(TaskId), NotBlocked(TaskId), NoCurrentTask}`，派生 `Debug`、`Clone`、`Copy`、`PartialEq` 和 `Eq`。
- `Scheduler::new`、`spawn`、`schedule_next`、`block_current`、`wake`、`exit_current` 和 `state`。

签名必须匹配：

```text
pub fn new() -> Self;
pub fn spawn(&mut self, id: TaskId) -> Result<(), SchedulerError>;
pub fn schedule_next(&mut self) -> Option<TaskId>;
pub fn block_current(&mut self) -> Result<TaskId, SchedulerError>;
pub fn wake(&mut self, id: TaskId) -> Result<(), SchedulerError>;
pub fn exit_current(&mut self) -> Result<TaskId, SchedulerError>;
pub fn state(&self, id: TaskId) -> Option<TaskState>;
```

在 `labs/src/lib.rs` 增加 `pub mod scheduler;`。

- [ ] **Step 4：独立实现状态机**

语义必须满足：

- `spawn` 只把新任务加入 Ready 队列。
- `schedule_next` 把仍在 Running 的当前任务放回 Ready 队尾，再选择队首。
- `block_current` 将当前任务转为 Blocked，并清空 current。
- `wake` 只允许把 Blocked 任务转回 Ready。
- `exit_current` 将当前任务永久标为 Exited。
- 任一 TaskId 在任一时刻只能处于一个状态，Ready 队列不得包含重复项。

本实验不创建线程、不使用 Tokio、不依赖 wall-clock 时间。

- [ ] **Step 5：运行调度器测试**

```bash
cargo test -p phase0-labs --test scheduler
```

Expected: `5 passed; 0 failed`。

- [ ] **Step 6：补充非法状态转换测试**

Append to `labs/tests/scheduler.rs`:

```rust
#[test]
fn wake_requires_a_blocked_task() {
    let mut scheduler = Scheduler::new();
    scheduler.spawn(TaskId::new(1)).unwrap();

    assert_eq!(
        scheduler.wake(TaskId::new(1)),
        Err(SchedulerError::NotBlocked(TaskId::new(1)))
    );
    assert_eq!(scheduler.schedule_next(), Some(TaskId::new(1)));
    assert_eq!(
        scheduler.wake(TaskId::new(1)),
        Err(SchedulerError::NotBlocked(TaskId::new(1)))
    );
}
```

重新运行测试并确认通过。

Expected: `6 passed; 0 failed`。

- [ ] **Step 7：运行格式与 lint**

```bash
cargo fmt --check
cargo clippy -p phase0-labs --all-targets -- -D warnings
```

Expected: 退出码为 0，无 warning。

- [ ] **Step 8：解释调度状态机**

学习者必须从一个包含三个 TaskId 的例子手工推演 Ready 队列和 current 的变化，并解释为何这个模拟器还不是真实的上下文切换。

- [ ] **Step 9：提交调度模拟器**

```bash
git add labs/src/lib.rs labs/src/scheduler.rs labs/tests/scheduler.rs
git commit -m "feat: model cooperative round robin scheduling"
```

## Task 6：练习 `unsafe` 所有权边界

**Files:**
- Create: `labs/tests/owned_ptr.rs`
- Create: `labs/src/owned_ptr.rs`
- Modify: `labs/src/lib.rs`

- [ ] **Step 1：先写安全 API 的行为测试**

Create `labs/tests/owned_ptr.rs`:

```rust
use std::cell::Cell;
use std::rc::Rc;

use phase0_labs::owned_ptr::OwnedPtr;

#[test]
fn reads_and_mutates_value() {
    let mut value = OwnedPtr::new(41_u64);

    assert_eq!(*value.get(), 41);
    *value.get_mut() += 1;
    assert_eq!(*value.get(), 42);
}

#[test]
fn owns_non_copy_values() {
    let mut value = OwnedPtr::new(String::from("kernel"));

    value.get_mut().push_str(" lab");
    assert_eq!(value.get(), "kernel lab");
}

#[test]
fn drops_inner_value_exactly_once() {
    struct DropProbe(Rc<Cell<usize>>);

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    let drops = Rc::new(Cell::new(0));
    {
        let _value = OwnedPtr::new(DropProbe(Rc::clone(&drops)));
        assert_eq!(drops.get(), 0);
    }
    assert_eq!(drops.get(), 1);
}
```

- [ ] **Step 2：运行测试并确认红灯**

```bash
cargo test -p phase0-labs --test owned_ptr
```

Expected: 编译失败，错误指向 `phase0_labs::owned_ptr` 尚不存在。

- [ ] **Step 3：定义安全公开接口**

`labs/src/owned_ptr.rs` 定义：

- `OwnedPtr<T>` 内部持有 `NonNull<T>`。
- `OwnedPtr::new(value: T) -> Self`。
- `OwnedPtr::get(&self) -> &T`。
- `OwnedPtr::get_mut(&mut self) -> &mut T`。
- `Drop`，负责恰好一次重建并释放原始 `Box<T>`。

在 `labs/src/lib.rs` 增加 `pub mod owned_ptr;`。

不得为 `OwnedPtr` 实现 `Copy`、`Clone`，也不得手写 `Send` 或 `Sync`。

- [ ] **Step 4：独立实现 `unsafe` 边界**

仅允许使用 `Box::into_raw`、`NonNull` 的引用转换和 `Box::from_raw` 完成实验。每一个 `unsafe` 块前必须有 `// SAFETY:` 注释，分别解释：

- 指针为何非空、对齐且指向有效的 `T`。
- 借用生命周期为何不超过 `OwnedPtr`。
- `&mut self` 为何保证可变引用唯一。
- `Drop` 为何只会重建同一个 allocation 一次。

该类型是用于审查安全契约的教学实验，不能被宣传为比 `Box<T>` 更好的生产抽象。

- [ ] **Step 5：运行行为测试**

```bash
cargo test -p phase0-labs --test owned_ptr
```

Expected: `3 passed; 0 failed`。

- [ ] **Step 6：审计 `unsafe`**

Run:

```bash
rg -n "unsafe|SAFETY" labs/src/owned_ptr.rs
cargo clippy -p phase0-labs --all-targets -- -D warnings
```

Expected:

- 每一个 `unsafe` 使用点附近都有对应 `SAFETY` 说明。
- Clippy 退出 0，无 warning。

- [ ] **Step 7：解释安全契约**

学习者必须回答：如果为该类型错误实现 `Clone`，为何可能 double free；如果 `get_mut` 接收 `&self`，为何可能产生别名可变引用；如果忘记 `Drop`，为何会泄漏。

- [ ] **Step 8：提交 unsafe 实验**

```bash
git add labs/src/lib.rs labs/src/owned_ptr.rs labs/tests/owned_ptr.rs
git commit -m "feat: practice an unsafe ownership boundary"
```

## Task 7：观察 ABI、ELF 与 `no_std`

**Files:**
- Create: `labs/src/bin/abi_probe.rs`
- Modify: `freestanding/src/lib.rs`
- Create: `docs/lessons/06-abi-elf-no-std.md`

- [ ] **Step 1：创建 C ABI 布局观察程序**

Create `labs/src/bin/abi_probe.rs`:

```rust
use std::mem::{align_of, offset_of, size_of};

#[repr(C)]
struct TrapFrameProbe {
    rip: u64,
    cs: u64,
    rflags: u64,
    rsp: u64,
    ss: u64,
}

fn main() {
    println!("size={}", size_of::<TrapFrameProbe>());
    println!("align={}", align_of::<TrapFrameProbe>());
    println!("rip_offset={}", offset_of!(TrapFrameProbe, rip));
    println!("rsp_offset={}", offset_of!(TrapFrameProbe, rsp));
    println!("ss_offset={}", offset_of!(TrapFrameProbe, ss));
}
```

- [ ] **Step 2：运行并记录布局**

Run:

```bash
cargo run -p phase0-labs --bin abi_probe
```

Expected:

```text
size=40
align=8
rip_offset=0
rsp_offset=24
ss_offset=32
```

- [ ] **Step 3：导出一个稳定符号**

Replace `freestanding/src/lib.rs` with:

```rust
#![no_std]

#[unsafe(no_mangle)]
pub extern "C" fn phase0_add(left: u64, right: u64) -> u64 {
    left.wrapping_add(right)
}
```

- [ ] **Step 4：验证裸机 crate 不依赖 `std`**

Run:

```bash
cargo check -p freestanding --target x86_64-unknown-none
cargo tree -p freestanding --target x86_64-unknown-none
```

Expected:

- `cargo check` 退出 0。
- 依赖树只显示 `freestanding v0.1.0`，没有 `std` 或第三方依赖。

- [ ] **Step 5：生成确定路径的 ELF object**

Run:

```bash
rustc --edition 2024 --crate-name freestanding --crate-type lib --target x86_64-unknown-none --emit=obj=target/phase0-freestanding.o freestanding/src/lib.rs
xcrun llvm-objdump --file-headers target/phase0-freestanding.o
```

Expected: 输出包含 `file format elf64-x86-64`。

- [ ] **Step 6：检查 section 与符号**

Run:

```bash
xcrun llvm-objdump --section-headers target/phase0-freestanding.o
xcrun llvm-objdump --syms target/phase0-freestanding.o
```

Expected:

- section 列表至少包含代码 section。
- symbol table 中存在未改名的 `phase0_add`。

- [ ] **Step 7：执行一次受控失败实验**

临时在 `freestanding/src/lib.rs` 的函数中加入 `let _items = Vec::<u8>::new();`，运行：

```bash
cargo check -p freestanding --target x86_64-unknown-none
```

Expected: 编译失败，指出 `Vec` 不在当前作用域；这证明 `no_std` 环境不会自动提供标准库集合。观察错误后立即删除该行，再次运行同一命令并确认退出 0。失败代码不得提交。

- [ ] **Step 8：创建观察记录**

Create `docs/lessons/06-abi-elf-no-std.md`:

```markdown
# ABI、ELF 与 no_std 观察

## 已运行命令

- `cargo run -p phase0-labs --bin abi_probe`
- `cargo check -p freestanding --target x86_64-unknown-none`
- `xcrun llvm-objdump --file-headers target/phase0-freestanding.o`
- `xcrun llvm-objdump --section-headers target/phase0-freestanding.o`
- `xcrun llvm-objdump --syms target/phase0-freestanding.o`

## 必答问题

1. `#[repr(C)]` 保证的是什么，未保证的又是什么？
2. ABI 为什么不仅是“函数名约定”？
3. `#[unsafe(no_mangle)]` 改变了哪个可观察结果？
4. ELF object 与可启动内核镜像有什么区别？
5. section 和 symbol 分别描述什么？
6. `no_std` 移除了什么？`core` 为什么仍然可用？
7. 本实验为什么还不需要 panic handler？

## 观察结论

结论必须引用实际输出中的字段、section 或 symbol，不凭记忆填写。
```

- [ ] **Step 9：填写记录并完成口头检查**

学习者根据实际输出回答七个问题，并能指着 `llvm-objdump` 输出区分 ELF header、section 和 symbol。

- [ ] **Step 10：提交 ABI/ELF 实验**

```bash
git add labs/src/bin/abi_probe.rs freestanding/src/lib.rs docs/lessons/06-abi-elf-no-std.md
git commit -m "feat: inspect ABI and freestanding ELF output"
```

## Task 8：执行阶段 0 门禁

**Files:**
- Create: `docs/lessons/phase-0-review.md`
- Modify: `PROJECT_PROGRESS.md`
- Modify: `PROJECT_TODO.md`

- [ ] **Step 1：创建阶段复盘题**

Create `docs/lessons/phase-0-review.md`:

```markdown
# 阶段 0 复盘

## 口试题

1. workspace、package、crate、target 和 profile 各是什么？
2. `Cargo.lock` 为什么是可复现构建的一部分？
3. 地址向上对齐在哪一步可能溢出？
4. 页分配器如何保证一个 frame 不会被同时分配两次？
5. round-robin 的 Ready 队列和 current 如何变化？
6. `OwnedPtr` 的每个 `unsafe` 块依赖什么不变量？
7. `#[repr(C)]`、`extern "C"` 和 `#[unsafe(no_mangle)]` 各影响什么？
8. ELF header、section 和 symbol table 分别回答什么问题？
9. `no_std`、`core` 与 `alloc` 的边界是什么？
10. 宿主调度模拟器与真实内核上下文切换之间缺少什么？

## 实现复盘

对 address、frame、scheduler 和 owned_ptr 四个模块，分别记录：公开 API、核心不变量、一个失败路径、一次调试证据。

## 阶段结论

只有在全部自动检查通过且口试完成后，才能写入阶段结论。
```

- [ ] **Step 2：运行完整格式检查**

```bash
cargo fmt --check
```

Expected: 退出码为 0，无 diff。

- [ ] **Step 3：运行完整 lint**

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Expected: 退出码为 0，无 warning。

- [ ] **Step 4：运行所有宿主测试**

```bash
cargo test --workspace
```

Expected:

- `address_alignment` 至少 7 项通过。
- `frame_allocator` 9 项通过。
- `scheduler` 6 项通过。
- `owned_ptr` 3 项通过。
- 所有 test target 均显示 `ok`，总失败数为 0。

- [ ] **Step 5：重新验证裸机 object**

```bash
cargo check -p freestanding --target x86_64-unknown-none
rustc --edition 2024 --crate-name freestanding --crate-type lib --target x86_64-unknown-none --emit=obj=target/phase0-freestanding.o freestanding/src/lib.rs
xcrun llvm-objdump --file-headers --syms target/phase0-freestanding.o
```

Expected: 所有命令退出 0，输出同时包含 `elf64-x86-64` 和 `phase0_add`。

- [ ] **Step 6：完成十题口试与实现复盘**

学习者在 `docs/lessons/phase-0-review.md` 中写下自己的答案。导师逐题追问；不能从代码、测试或命令输出举证的答案不通过。

- [ ] **Step 7：检查提交历史与工作树**

```bash
git log --oneline --decorate -10
git status --short
```

Expected:

- 历史中能看到本计划要求的细粒度提交。
- 更新项目记忆前工作树只包含本步骤的复盘文档改动。

- [ ] **Step 8：更新项目记忆**

更新 `PROJECT_PROGRESS.md`：

- 当前阶段改为“阶段 0 已完成，准备阶段 1 设计”。
- 已完成中记录 Cargo、地址对齐、页分配、调度、unsafe、ABI/ELF/no_std 的实际成果。
- 最近进展记录完整测试结果、工具链和完成日期。

更新 `PROJECT_TODO.md`：

- 删除已经完成的设计审阅和阶段 0 条目。
- 下一步只保留“为阶段 1 最小可启动内核编写设计与实施计划”。
- QEMU 安装、兼容 nightly 选择、bootloader 0.11.15 冒烟验证作为阶段 1 待办。

- [ ] **Step 9：提交阶段门禁记录**

```bash
git add docs/lessons/phase-0-review.md PROJECT_PROGRESS.md PROJECT_TODO.md
git commit -m "docs: complete phase zero review"
```

- [ ] **Step 10：执行最终清洁验证**

```bash
git status --short --branch
cargo test --workspace
cargo check -p freestanding --target x86_64-unknown-none
```

Expected:

- Git 只显示当前分支，不显示未提交文件。
- 所有宿主测试通过。
- `freestanding` 裸机检查通过。

## 阶段 0 完成定义

只有同时满足以下条件，才进入阶段 1：

- 所有任务的红灯和绿灯都由学习者亲自运行并报告。
- 地址、frame、scheduler 与 `OwnedPtr` 核心实现由学习者完成。
- 格式、Clippy、宿主测试和裸机 target 检查全部通过。
- `llvm-objdump` 已证明产物是 x86_64 ELF object，且能找到 `phase0_add`。
- 十道口试题能结合代码或输出作答。
- 项目记忆已更新，Git 工作树干净。
