# 快速开始指南

## 一分钟运行

```bash
# 进入项目目录
cd /home/samera2022/Projects/Recitor/recitor

# 直接运行（已编译）
./target/release/recitor

# 或重新编译后运行
cargo run --release
```

## 使用说明

1. **启动应用**
   - 窗口标题：英语单词测验 - Recitor
   - 初始界面显示"点击开始测验"

2. **开始测验**
   - 点击"开始测验"按钮
   - 看到第1题的提示（中文或部分拼写）

3. **答题**
   - 在输入框输入英文单词
   - 点击"提交答案"或按 `Enter` 键
   - 大小写不敏感（scream = SCREAM = Scream）

4. **查看结果**
   - ✅ **答对**：绿色反馈，点击"下一题"继续
   - ❌ **答错**：红色反馈 + 正确答案，点击"重新开始"

5. **计分**
   - 右上角显示连续答对次数
   - 答错后归零

## 示例测验题

- **scream** → 提示：高声喊
- **platform** → 提示：月台
- **absorb** → 提示：a? a lot of information
- **grateful** → 提示：感激

## 键盘快捷键

- `Enter` - 提交当前答案
- （待扩展：`Esc` 退出应用）

## 词库位置

当前词库硬编码在 `src/word_bank.rs`，包含 58 个单词。

未来可通过 `data/words.toml` 扩展（需实现加载功能）。

## 故障排除

### 无法启动
- 确保已安装图形环境（X11 或 Wayland）
- 检查 `DISPLAY` 环境变量

### 中文显示异常
- 确保系统已安装中文字体（Noto Sans CJK 或类似）

### 重新编译
```bash
cargo clean
cargo build --release
```

## 更多信息

详见项目 `README.md`
