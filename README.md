# Lazy Todo App

一个集 Todo 管理、桌面即时贴、番茄钟于一体的桌面效率应用，用 **Tauri v2 + Rust + React + TypeScript** 构建。

这个项目是 [Harness Engineering 博客文章](https://www.fanyamin.com/tech/harness-engineering.html) 的配套实战案例——一个不会 Rust 的老程序员，靠 Harness Engineering 的思路，让 AI Agent 按规则生成了整个项目的代码。

## 功能

### Todo 管理
- **Todo CRUD**：添加、编辑、完成、删除任务
- **三级优先级**：🔴 高 / 🟡 中 / 🟢 低，左边框颜色区分
- **截止时间 + 实时倒计时**：每秒刷新，< 1 小时变橙色，过期变红色
- **智能排序**：按优先级 → 截止时间排序，已完成的沉底

### 桌面即时贴
- **Markdown 备忘录**：支持 GitHub Flavored Markdown 渲染
- **多色便签**：黄、绿、蓝、粉、紫五种颜色
- **内联编辑**：点击即可编辑标题和内容
- **系统托盘集成**：收缩到任务栏，右键菜单快捷新建便签

### 番茄钟
- **可视化计时**：SVG 圆环进度条，实时倒计时
- **可配置时长**：自定义工作、短休息、长休息时间和轮次
- **每日统计**：当日完成番茄数 + 7 日柱状图
- **到时提醒**：窗口弹出 + 音效提示，切换标签页不中断计时
- **系统通知**：利用原生通知提醒工作/休息

### 通用
- **SQLite 持久化**：数据存在本地，重启不丢失
- **可配置数据库路径**：通过环境变量 `LAZY_TODO_DB_DIR` 自定义存储位置
- **暗色主题 UI**
- **系统托盘**：关闭窗口时隐藏到托盘，左键点击显示/隐藏

## 技术栈

| 层 | 技术 | 职责 |
|----|------|------|
| 前端 | React 18 + TypeScript | UI 组件、倒计时/番茄钟 Hook |
| 后端 | Rust + Tauri v2 | 命令处理、状态管理、系统托盘 |
| 存储 | SQLite (rusqlite) | 本地持久化（todos、notes、pomodoro） |
| 通知 | tauri-plugin-notification | 原生系统通知 |
| Markdown | react-markdown + remark-gfm | 便签内容渲染 |
| 构建 | Vite + Cargo | 前后端打包 |

## 项目结构

```
lazy-todo-app/
├── CLAUDE.md                    # Harness: AI Agent 的架构规则
├── package.json
├── src/                         # React 前端
│   ├── App.tsx                  # 主组件（Tab 导航）
│   ├── App.css                  # 暗色主题样式
│   ├── types/
│   │   ├── todo.ts              # Todo 类型定义
│   │   ├── note.ts              # 便签类型定义
│   │   └── pomodoro.ts          # 番茄钟类型定义
│   ├── hooks/
│   │   ├── useTodos.ts          # Todo CRUD Hook
│   │   ├── useCountdown.ts      # 实时倒计时 Hook
│   │   ├── useNotes.ts          # 便签 CRUD Hook
│   │   ├── usePomodoro.ts       # 番茄钟计时逻辑 Hook
│   │   └── usePomodoroStats.ts  # 番茄钟统计 Hook
│   └── components/
│       ├── AddTodo.tsx           # Todo 添加表单
│       ├── TodoItem.tsx          # 单条任务（倒计时 + 内联编辑）
│       ├── TodoList.tsx          # Todo 列表
│       ├── NoteEditor.tsx        # 便签编辑器
│       ├── NoteCard.tsx          # 便签卡片
│       ├── NoteList.tsx          # 便签列表
│       ├── MarkdownPreview.tsx   # Markdown 渲染
│       ├── PomodoroPanel.tsx     # 番茄钟主面板
│       ├── PomodoroRing.tsx      # SVG 圆环进度
│       ├── PomodoroControls.tsx  # 开始/暂停/重置
│       ├── PomodoroSettings.tsx  # 时长配置
│       ├── PomodoroStats.tsx     # 每日/每周统计
│       └── PomodoroAlert.tsx     # 到时弹窗 + 音效
└── src-tauri/                   # Rust 后端
    ├── Cargo.toml
    ├── tauri.conf.json
    ├── icons/                   # 应用图标（各尺寸）
    └── src/
        ├── main.rs
        ├── lib.rs               # Tauri 启动 + 命令注册 + 系统托盘
        ├── db.rs                # SQLite 数据库（4 张表）
        ├── models/
        │   ├── todo.rs          # Todo 数据模型
        │   ├── note.rs          # 便签数据模型
        │   └── pomodoro.rs      # 番茄钟数据模型
        └── commands/
            ├── todo.rs          # Todo 命令（5 个）
            ├── note.rs          # 便签命令（4 个）
            ├── pomodoro.rs      # 番茄钟命令（5 个）
            └── app.rs           # 应用信息命令
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

### 自定义数据库路径

默认情况下，数据库存储在系统应用数据目录：

| 平台 | 默认路径 |
|------|----------|
| macOS | `~/Library/Application Support/com.fanyamin.lazytodoapp/todos.db` |
| Linux | `~/.local/share/com.fanyamin.lazytodoapp/todos.db` |
| Windows | `%APPDATA%\com.fanyamin.lazytodoapp\todos.db` |

通过环境变量指定自定义位置：

```bash
LAZY_TODO_DB_DIR=/path/to/your/folder npm run tauri dev
```

当前数据库路径会显示在应用底部的 footer 中。

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
| **Todo** | |
| `list_todos` | 查询所有 Todo，按优先级和截止时间排序 |
| `add_todo` | 添加新 Todo（支持标题、描述、优先级、截止时间） |
| `toggle_todo` | 切换完成状态 |
| `update_todo` | 编辑 Todo（标题、描述、优先级、截止时间） |
| `delete_todo` | 删除 Todo |
| **便签** | |
| `list_notes` | 查询所有便签，按更新时间倒序 |
| `add_note` | 新建便签（标题、内容、颜色） |
| `update_note` | 编辑便签 |
| `delete_note` | 删除便签 |
| **番茄钟** | |
| `get_pomodoro_settings` | 获取番茄钟配置 |
| `save_pomodoro_settings` | 保存番茄钟配置 |
| `record_pomodoro_session` | 记录一次完成的番茄 |
| `get_today_pomodoro_count` | 获取今日完成数 |
| `get_weekly_pomodoro_stats` | 获取最近 7 天统计 |
| `update_tray_tooltip` | 更新托盘提示文字 |
| **应用** | |
| `get_db_path` | 获取当前数据库文件路径 |

## 相关文章

- [从 Prompt Engineering 到 Harness Engineering：AI 编程的四次进化](https://www.fanyamin.com/tech/harness-engineering.html)
- [Harness Engineering: Leveraging Codex in an Agent-First World](https://openai.com/index/harness-engineering/) - OpenAI 原文
- [Tauri v2 Documentation](https://v2.tauri.app/)

## License

Apache-2.0
