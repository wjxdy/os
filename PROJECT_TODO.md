# 项目待办

## 下一步
- [ ] 把 `kernel_main` 开头的串口消息恢复为 `Rust OS: kernel entered`，把 framebuffer 完成消息改为 `framebuffer: text RUST OS drawn`。
- [ ] 确认 QEMU 图形窗口中的多行文字、格式化参数、自动折行、底部绿色文字和横线均符合预期，再检查并提交实验 05～06 的源码。
- [ ] 生成实验 07 教程：实现局部 `DualWriter<'a>`，把一次 `writeln!` 同时转发到串口和 framebuffer。
- [ ] 实验 07 中复述生命周期、两个 `&mut` 借用、trait 组合和部分输出失败策略。

## 进行中
- 无。

## 待确认
- 无。

## 阻塞
- 无。

## 后续可做
- [ ] 把基础认知快速路线和两个运行故事穿插到对应内核实验，而不是作为动手前的统一门禁。
- [ ] 完成像素 framebuffer 后实现 CPU 异常与 IDT：先捕获 `int3` 和页错误，再进入外部中断与键盘。
- [ ] 第一版完成后评估 AArch64 移植。
- [ ] 基础 OS 稳定后设计用户态 AI runtime 实验。
