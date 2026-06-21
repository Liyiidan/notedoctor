# NoteDoctor

> 一个面向本地 Markdown / Obsidian 知识库的健康诊断工具。  

---

## 项目简介 

**NoteDoctor** 是一个使用 Rust 编写的本地 Markdown 知识库健康诊断工具。它可以递归扫描指定文件夹中的 Markdown 文件和图片资源，分析笔记之间的链接关系，并找出常见的知识库维护问题，例如死链、孤儿笔记和未使用的图片资源。


这个项目主要面向个人知识库、课程笔记、Obsidian vault 等场景。它不依赖云端服务，所有扫描都在本地完成。

---

## 核心特性 

### 1. 死链诊断 

扫描 Markdown 文件中的 Obsidian 双向链接：

```markdown
[[目标笔记]]
[[目标笔记|显示别名]]
```

如果链接指向的笔记不存在，NoteDoctor 会将其标记为 `BrokenLink`，并在终端中用红色高亮显示。

### 2. 孤儿笔记检测 

NoteDoctor 会统计每篇笔记的：

- 入链数量：有多少笔记链接到它
- 出链数量：它链接到了多少其他笔记

如果一篇笔记既没有被其他笔记引用，也没有引用任何其他笔记，就会被识别为孤儿笔记 `OrphanNote`。

这类笔记通常不一定是错误，但可能说明它还没有真正融入知识网络。

### 3. 冗余资源检测 

工具会扫描本地 `.png`、`.jpg`、`.jpeg` 图片资源，并解析 Markdown 图片引用：

```markdown
![alt](assets/image.png)
```

如果某张图片存在于文件夹中，但没有被任何 Markdown 文件引用，就会被标记为 `DeadAsset`。这有助于清理长期堆积的无用截图、插图或附件。

### 4. 终端交互界面 

项目使用 `inquire` 实现命令行路径输入，使用 `colored` 输出彩色诊断报告：

- 绿色：正常统计信息
- 红色：死链
- 黄色：孤儿笔记
- 蓝色：冗余资源

### 5. 单元测试覆盖 

项目目前包含 19 个单元测试，覆盖：

- 目录扫描
- Markdown 链接解析
- 图片引用解析
- 死链识别
- 孤儿笔记识别
- 冗余资源识别
- 综合集成场景

---

## 技术栈 

| 类别 | 技术 |
|------|------|
| 编程语言 | Rust |
| 目录遍历 | `walkdir` |
| 正则解析 | `regex` |
| 终端颜色 | `colored` |
| 终端交互 | `inquire` |
| 测试 | Rust built-in test framework |

---

## 项目结构

```text
notedoctor/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs        # CLI 入口和终端输出
│   ├── models.rs      # 诊断结果数据结构
│   ├── scanner.rs     # 文件和资源扫描
│   ├── parser.rs      # Markdown 链接解析
│   └── diagnostic.rs  # 核心诊断逻辑
└── test_vault/        # 手动测试用 Markdown 知识库
```

---

## 安装与运行 

### 1. 克隆项目

```bash
git clone https://github.com/Liyiidan/notedoctor.git
cd notedoctor
```

### 2. 编译项目

```bash
cargo build
```

### 3. 运行程序

```bash
cargo run
```

程序启动后会提示输入要扫描的本地知识库路径，例如：

```text
/Users/yourname/Documents/ObsidianVault
```

如果想直接测试项目自带的测试库，可以输入：

```text
/Users/liyidan/notedoctor/test_vault
```

---

## 测试 

运行全部测试：

```bash
cargo test
```

当前测试结果：

```text
running 19 tests

test result: ok. 19 passed; 0 failed
```

测试中会在系统临时目录动态创建 Markdown 文件和图片文件，测试结束后自动清理，不会污染项目目录。

---

## 核心设计思路 

### 1. 扫描阶段

`scanner.rs` 使用 `walkdir` 递归遍历目标目录，分别收集：

- Markdown 文件：`.md`
- 图片资源：`.png`、`.jpg`、`.jpeg`

### 2. 解析阶段

`parser.rs` 使用正则表达式提取两类引用：

- Obsidian 双向链接：`[[note]]`
- Markdown 图片链接：`![alt](path)`

目前解析策略比较轻量，适合课程项目和中小型本地知识库。如果要支持更复杂的 Markdown 语法，后续可以考虑引入 AST 解析库。

### 3. 诊断阶段

`diagnostic.rs` 负责把扫描和解析结果合并起来：

- 建立笔记名索引
- 统计每篇笔记的入度和出度
- 判断死链
- 判断孤儿笔记
- 判断未引用图片资源

最终结果统一写入 `DiagnosticReport`，再由 `main.rs` 负责格式化输出。

---

## 示例输出 

```text
╔══════════════════════════════════════╗
║       NoteDoctor  🩺  知识库体检       ║
╚══════════════════════════════════════╝

─── 诊断报告 ───────────────────────────
✔  扫描笔记总数：3
✔  扫描图片总数：0
✘  发现问题总数：2

✘  死链数量：1
─── 死链详情 ───────────────────────────
  ✘ 在 index.md 中发现死链 [[不存在的笔记]]

⚠  孤儿笔记数量：1
─── 孤儿笔记详情 ────────────────────────
  ⚠ 孤儿笔记.md （无入链也无出链）
```

---

## 当前限制 

目前版本主要完成课程项目要求中的核心功能，还有一些可以继续改进的地方：

- 暂未完整支持 Obsidian 的所有高级链接语法
- 同名笔记冲突目前按文件名简单处理
- 图片匹配主要按文件名判断，复杂相对路径还可以继续优化
- 暂未实现自动修复，只负责诊断和报告

这些限制不影响当前核心功能，但如果将来作为真正的日常工具使用，还可以继续增强。

---

## 后续改进方向 

- 支持更多资源类型，例如 `.gif`、`.webp`、`.pdf`
- 支持导出 JSON / Markdown 格式诊断报告
- 支持忽略规则，例如跳过 `.obsidian/`、`.git/`、`node_modules/`
- 支持命令行参数模式，减少交互输入
- 支持更准确的 Markdown AST 解析

---

## 课程项目总结 

通过这个项目，我实践了 Rust 中比较核心的工程能力：

- 使用第三方 crate 构建实际命令行工具
- 处理文件系统遍历和路径判断
- 使用 `Result` 做基本错误处理
- 使用 `enum` 和 `struct` 组织领域模型
- 编写模块化代码
- 编写单元测试和集成场景测试

整体来说，NoteDoctor 不是一个很复杂的工具，但它覆盖了一个完整 Rust 小项目应该具备的基本结构：输入、扫描、解析、诊断、输出和测试。

---

## License

This project is created for course work and personal learning purposes.
