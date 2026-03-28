# Lazy Todo App

一个带优先级和倒计时的桌面 Todo 应用，用 **Tauri v2 + Rust + React + TypeScript** 构建。

这个项目是 [Harness Engineering 博客文章](https://www.fanyamin.com/tech/harness-engineering.html) 的配套实战案例——一个不会 Rust 的老程序员，靠 Harness Engineering 的思路，让 AI Agent 按规则生成了整个项目的代码。

## 功能

- **Todo CRUD**：添加、编辑、完成、删除任务
- **三级优先级**：🔴 高 / 🟡 中 / 🟢 低，左边框颜色区分
- **截止时间 + 实时倒计时**：每秒刷新，< 1 小时变橙色，过期变红色
- **SQLite 持久化**：数据存在本地，重启不丢失
- **智能排序**：按优先级 → 截止时间排序，已完成的沉底
- **暗色主题 UI**

## 技术栈

| 层 | 技术 | 职责 |
|----|------|------|
| 前端 | React 18 + TypeScript | UI 组件、倒计时 Hook |
| 后端 | Rust + Tauri v2 | 命令处理、状态管理 |
| 存储 | SQLite (rusqlite) | 本地持久化 |
| 构建 | Vite + Cargo | 前后端打包 |

## 项目结构

```
lazy-todo-app/
├── CLAUDE.md                    # Harness: AI Agent 的架构规则
├── package.json
├── src/                         # React 前端
│   ├── App.tsx                  # 主组件
│   ├── App.css                  # 暗色主题样式
│   ├── types/todo.ts            # TypeScript 类型定义
│   ├── hooks/
│   │   ├── useTodos.ts          # Tauri invoke 封装
│   │   └── useCountdown.ts      # 实时倒计时 Hook
│   └── components/
│       ├── AddTodo.tsx           # 添加表单（优先级 + 截止时间）
│       ├── TodoItem.tsx          # 单条任务（倒计时 + 内联编辑）
│       └── TodoList.tsx          # 列表（待完成/已完成分组）
└── src-tauri/                   # Rust 后端
    ├── Cargo.toml
    ├── tauri.conf.json
    └── src/
        ├── main.rs
        ├── lib.rs               # Tauri 启动 + 命令注册
        ├── db.rs                # SQLite CRUD
        ├── models/todo.rs       # Todo 数据模型
        └── commands/todo.rs     # 5 个 Tauri 命令
```

## 快速开始

### 前置条件

- [Rust](https://www.rust-lang.org/tools/install) (1.70+)
- [Node.js](https://nodejs.org/) (18+)
- macOS / Linux / Windows

### 安装 & 运行

```bash
# 克隆项目
git clone https://github.com/walterfan/lazy-todo-app.git
cd lazy-todo-app

# 安装前端依赖
npm install

# 开发模式（同时启动 Vite + Tauri）
npm run tauri dev

# 构建生产版本
npm run tauri build
```

## Harness Engineering 实践

这个项目是 Harness Engineering 的实战演示。核心思路：不是自己写 Rust 代码，而是给 AI Agent 搭一个"跑不偏"的环境。

### 缰绳：CLAUDE.md

`CLAUDE.md` 定义了项目的架构规则、文件结构、编码规范和常见坑点。Agent 按照这份"操作手册"生成代码，不需要自由发挥。

### 围栏：Pre-commit Hook

```bash
# 安装 pre-commit
pip install pre-commit
pre-commit install
```

每次提交前自动跑 `cargo clippy`、`cargo test`、`tsc --noEmit` 三道检查。

### 马场：Tauri 的天然沙箱

Tauri 本身就是一个很好的 Harness：前端只能通过 `invoke()` 调用后端命令，不能直接访问文件系统；后端用 Rust 的类型系统和所有权机制约束内存安全。

## Tauri 命令一览

| 命令 | 功能 |
|------|------|
| `list_todos` | 查询所有 Todo，按优先级和截止时间排序 |
| `add_todo` | 添加新 Todo（支持标题、描述、优先级、截止时间） |
| `toggle_todo` | 切换完成状态 |
| `update_todo` | 编辑 Todo（标题、描述、优先级、截止时间） |
| `delete_todo` | 删除 Todo |

## 相关文章

- [从 Prompt Engineering 到 Harness Engineering：AI 编程的四次进化](https://www.fanyamin.com/tech/harness-engineering.html)
- [Harness Engineering: Leveraging Codex in an Agent-First World](https://openai.com/index/harness-engineering/) - OpenAI 原文
- [Tauri v2 Documentation](https://v2.tauri.app/)

## License

Apache-2.0
