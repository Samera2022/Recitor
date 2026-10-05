# Recitor - Rust + Slint

基于 Rust 和 Slint 实现的英语单词记忆测验工具，从 Java DesktopHelper 项目移植而来。

## 功能特性

- ✅ 随机单词测验
- ✅ 中文提示或部分拼写提示
- ✅ 即时答案验证
- ✅ 连续答对计数
- ✅ 现代化 GUI 界面
- 📝 错误时显示正确答案
- 🎯 答对后继续下一题，答错后重新开始

## 技术栈

- **语言**: Rust 1.96.1
- **GUI 框架**: Slint 1.9
- **依赖**:
  - `rand` - 随机数生成
  - `serde` + `toml` - 配置文件解析（可选扩展）

## 快速开始

### 1. 前置要求

确保已安装 Rust 工具链：

```bash
rustc --version
cargo --version
```

### 2. 安装 Slint LSP（可选，用于 IDE 支持）

```bash
cargo install slint-lsp
```

### 3. 构建项目

```bash
cd recitor
cargo build --release
```

### 4. 运行应用

```bash
cargo run --release
```

或直接运行编译后的二进制：

```bash
./target/release/recitor
```

## 使用方法

1. 启动应用后，点击 **"开始测验"** 按钮
2. 阅读显示的中文提示或部分拼写提示
3. 在输入框中输入对应的英文单词
4. 点击 **"提交答案"** 或按 Enter 键
5. 查看反馈：
   - ✅ **正确**：点击"下一题"继续
   - ❌ **错误**：查看正确答案，点击"重新开始"

## 项目结构

```
recitor/
├── Cargo.toml           # 项目配置和依赖
├── build.rs             # Slint 编译脚本
├── ui/
│   └── app.slint        # Slint UI 定义
├── src/
│   ├── main.rs          # 应用入口和事件处理
│   ├── quiz.rs          # 测验逻辑
│   └── word_bank.rs     # 词库管理
└── data/
    └── words.toml       # 词库数据（扩展用）
```

## 词库

当前内置约 60 个单词，涵盖：

- 基础词汇：scream, platform, channel, session
- 学术词汇：resistance, sustain, exceptional
- 短语搭配：arise from, be convinced that, spring up
- 混合提示：完整中文 + 首字母提示

### 扩展词库

未来可通过修改 `data/words.toml` 添加新单词（需实现 TOML 加载功能）。

## 开发计划

- [x] 基础测验逻辑
- [x] Slint GUI 界面
- [x] 随机选词
- [x] 答案验证
- [x] 连续答对计数
- [ ] 从 TOML 文件加载词库
- [ ] 错题记录与复习模式
- [ ] 学习进度统计
- [ ] 难度分级
- [ ] 测验历史记录

## 原始项目

移植自 Java DesktopHelper 项目：`/mnt/netshares/192.168.50.1/DesktopHelper`

原始实现使用 Java Swing 的 `JOptionPane` 对话框，本项目使用 Slint 提供更现代化的用户界面。

## 许可证

待定
