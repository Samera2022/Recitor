# v0.8.0 文件清单

**版本**：v0.8.0  
**日期**：2026-10-02

---

## 核心源代码

### 新增文件

| 文件 | 行数 | 说明 |
|------|------|------|
| `src/porting.rs` | 448 | 序列化/解析核心模块，支持 JSON/TSV/TXT |

### 修改文件

| 文件 | 变更 | 说明 |
|------|------|------|
| `src/main.rs` | +209 行 | 新增导入/导出 UI 回调，rfd 文件对话框 |
| `src/word_bank.rs` | +53 行 | 新增 export_books() / import_books() |
| `src/quiz.rs` | +13 行 | 新增导入/导出代理方法 |
| `ui/app.slint` | +65 行 | 新增导出/导入按钮，状态提示框 |
| `Cargo.toml` | +2 依赖 | rfd 0.16 + tokio 1 |

---

## 示例文件

| 文件 | 格式 | 说明 |
|------|------|------|
| `data/sample-import.txt` | TXT | 包含 2 个单词本，7 个单词 |
| `data/sample-import.tsv` | TSV | TSV 格式示例，可用 Excel 打开 |

---

## 文档

### 技术文档

| 文件 | 行数 | 说明 |
|------|------|------|
| `pros_docs/update-8-import-export.md` | 465 | 完整技术文档，含格式说明/实现细节 |
| `V0.8_DELIVERABLE.md` | 317 | 交付文档，功能验证/测试覆盖 |

### 用户文档

| 文件 | 行数 | 说明 |
|------|------|------|
| `IMPORT_EXPORT_GUIDE.md` | 307 | 用户指南，快速开始/格式详解/常见问题 |
| `CHANGELOG.md` | 更新 | 新增 v0.8.0 条目 |
| `README.md` | 更新 | 功能列表新增导入/导出 |

---

## 代码统计

```
语言             文件    空行    注释    代码
───────────────────────────────────────
Rust               6     182      87    1532
Slint              1      98       0    1352
Markdown          13     422       0    2285
TOML               1       3       0      19
───────────────────────────────────────
总计              21     705      87    5188
```

**新增代码**：
- Rust：约 788 行（porting.rs + 集成）
- Slint：约 65 行（UI 扩展）
- 文档：约 1500 行

---

## 依赖变更

### 新增依赖

| 依赖 | 版本 | 说明 |
|------|------|------|
| `rfd` | 0.16.0 | 跨平台文件对话框 |
| `tokio` | 1.53.1 | 异步 runtime |

### 传递依赖（新增）

- `ashpd` 0.11.1：XDG Desktop Portal 绑定
- `pollster` 0.4.0：同步阻塞异步调用
- `zbus` 5.19.0：D-Bus 通信
- `urlencoding` 2.1.3：URL 编码
- 约 50 个其他传递依赖

---

## 构建产物

| 文件 | 大小 | 说明 |
|------|------|------|
| `target/release/recitor` | ~8-10 MB | 发布版二进制 |
| `target/debug/recitor` | ~50-70 MB | 调试版二进制 |

---

## 测试文件

| 测试 | 位置 | 说明 |
|------|------|------|
| `porting::tests::json_roundtrip` | `src/porting.rs:449` | JSON 往返测试 |
| `porting::tests::tsv_roundtrip` | `src/porting.rs:455` | TSV 往返测试 |
| `porting::tests::txt_roundtrip` | `src/porting.rs:463` | TXT 往返测试 |
| `porting::tests::tsv_without_header` | `src/porting.rs:470` | TSV 无标题行测试 |

---

## 数据文件（运行时）

| 文件 | 说明 |
|------|------|
| `~/.recitor-books.json` | 单词本持久化（v0.7 引入） |
| `~/.recitor-progress.json` | 进度持久化（v0.4 引入） |
| `~/.recitor-statistics.json` | 统计持久化（v0.6 引入） |

**注意**：导入/导出功能会读写 `~/.recitor-books.json`。

---

## 关键文件路径速查

### 源代码

```
src/
├── main.rs           # 主程序，UI 回调（734 行）
├── word_bank.rs      # 单词本管理（546 行）
├── quiz.rs           # 测验逻辑（375 行）
├── porting.rs        # 导入/导出（448 行，新增）
├── progress.rs       # 进度管理（101 行）
└── statistics.rs     # 统计管理（203 行）
```

### UI

```
ui/
└── app.slint         # Slint UI（1352 行）
```

### 配置

```
./
├── Cargo.toml        # 依赖配置
├── Cargo.lock        # 依赖锁定
└── build.rs          # 构建脚本
```

### 文档

```
./
├── README.md         # 项目说明
├── CHANGELOG.md      # 更新日志
├── QUICKSTART.md     # 快速开始
├── QUICK_REFERENCE.md      # 快速参考
├── START_HERE.md           # 入门指南
├── STATISTICS_GUIDE.md     # 统计指南
├── IMPORT_EXPORT_GUIDE.md  # 导入/导出指南（新增）
├── V0.8_DELIVERABLE.md     # 交付文档（新增）
└── FILE_MANIFEST_V0.8.md   # 本文件

pros_docs/
├── update-2-multi-meaning.md
├── update-3-word-book-selector.md
├── update-4-progress-colors.md
├── update-5-word-management.md
├── update-6-statistics.md
├── update-7-book-manager.md
└── update-8-import-export.md  # 新增
```

---

## Git 状态

**注意**：本项目当前无 Git 历史（fatal: your current branch 'master' does not have any commits yet）。

如果需要版本控制，建议：

```bash
git add .
git commit -m "feat: batch import/export & book manager & statistics"
```

---

## 文件完整性校验

所有文件已创建并验证：

- [x] 核心源代码（6 个 .rs 文件）
- [x] UI 文件（1 个 .slint 文件）
- [x] 配置文件（Cargo.toml / build.rs）
- [x] 示例文件（2 个导入示例）
- [x] 技术文档（2 个详细文档）
- [x] 用户文档（1 个使用指南）
- [x] 更新日志（CHANGELOG.md）
- [x] README 更新

---

**文件清单版本**：1.0  
**对应功能版本**：v0.8.0  
**更新日期**：2026-10-02
