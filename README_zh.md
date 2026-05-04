# Lazy Todo App

[English Version](README.md) | [Chinese Version](README_zh.md)

一个基于 **Tauri v2 + Rust + React + TypeScript** 的跨平台桌面效率应用，整合了 Todo 管理、桌面即时贴、番茄钟、开发者工具箱、本地 AI Agents 和应用设置。

这个项目也是一个 [Harness Engineering](https://www.fanyamin.com/tech/harness-engineering.html) 的实战案例：通过明确的架构规则和约束，让 AI Agent 在可控边界内参与构建，而不是在没有边界的情况下自由生成代码。

## 功能

### Todo 管理

- **Todo CRUD**：添加、编辑、完成、删除任务。
- **优先级与截止时间**：支持高/中/低优先级和实时倒计时。
- **重复任务**：支持每天、每周、每月、每年重复，完成后自动推进到下一次到期时间，并支持显式选择每周星期几、每月第几天。
- **本地提醒**：可设置提前提醒，在应用运行时显示状态并触发系统通知。
- **搜索与显示模式**：支持任务搜索，以及列表/网格两种展示方式。

### 桌面即时贴

- **Markdown 便签**：支持 GitHub Flavored Markdown 渲染。
- **多色卡片**：支持多种便签颜色和内联编辑。
- **弹出窗口**：单条便签可在独立窗口中打开并持续同步数据。
- **托盘快捷新建**：可通过系统托盘快速进入便签创建流。

### 番茄钟

- **可视化计时**：SVG 圆环进度与阶段倒计时。
- **可配置周期**：支持工作、短休息、长休息和轮次设置。
- **里程碑与统计**：支持里程碑跟踪、今日完成数与 7 日统计。
- **提醒能力**：支持窗口提醒、音效提示和系统通知。

### 工具箱

- **编码转换**：支持 Base64、Hex ↔ ASCII、URL、HTML 转义、进制、时间戳（秒/毫秒、批量输入）和 JWT HS256 编解码。
- **校验摘要**：基于 Web Crypto 支持 MD5、SHA-1、SHA-256、SHA-384、SHA-512。
- **随机生成**：支持 UUID v4、自定义字符集随机字符串和强密码生成。
- **加密工具**：支持 AES-GCM / AES-CBC（128/192/256-bit key）以及 ROT13、Caesar、Atbash。
- **完全本地**：输入和输出不离开应用，不持久化，也不走网络。

### AI Agents

- **Agent 对话**：可在桌面应用中与内置本地 Agent 对话，例如 Personal Secretary 和 Confucius。
- **文件化人格**：Agent 来自静态智能体目录，包含 manifest、prompt、config、avatar、README 和可选 RAG 知识文件。
- **本地身份与记忆**：用户身份、长期记忆、记忆提案和历史会话召回都保存在本地 SQLite。
- **RAG 知识**：每个 Agent 的 `rag_knowledge.md` 独立切块和检索，只会注入当前 Agent 的知识。
- **安全工具动作**：Agent 可以提议 Todo、便签、里程碑、文件、记忆和外部 CLI 动作；写入或敏感操作必须经过应用确认流。
- **智能体管理**：可在 Settings 中刷新、启用、禁用、安装、查看、卸载智能体，并重建 RAG 索引。

### 设置与桌面体验

- **应用设置**：支持页面尺寸、Todo/Note 显示模式、便签模板和便签目录标签。
- **SQLite 持久化**：todos、notes、pomodoro、Agents 数据和 app settings 都存储在本地 SQLite 中。
- **数据库路径可配置**：支持环境变量和本地配置文件覆盖默认数据库目录。
- **系统托盘行为**：关闭主窗口时隐藏到托盘，支持显示/隐藏和退出。

## 技术栈

| 层级 | 技术 | 职责 |
|---|---|---|
| 桌面壳 | Tauri v2 | 原生窗口、系统托盘、插件接入 |
| 前端 | React 18 + TypeScript | 主 UI、搜索、设置、倒计时、弹出便签窗口 |
| 后端 | Rust | Tauri 命令、窗口管理、状态与数据库访问 |
| 存储 | SQLite via `rusqlite` | 本地持久化 todos、sticky notes、Pomodoro、Agents、settings |
| 通知 | `tauri-plugin-notification` | 原生系统提醒 |
| 外链处理 | `@tauri-apps/plugin-shell` | 在系统浏览器中打开 HTTP 链接 |
| Markdown | `react-markdown` + `remark-gfm` | 便签内容渲染 |
| 构建 | Vite + Cargo | 前端打包与桌面应用构建 |

## 快速开始

### 前置条件

- [Rust](https://www.rust-lang.org/tools/install) `1.70+`
- [Node.js](https://nodejs.org/) `18+`
- macOS / Linux / Windows

### 安装与运行

```bash
git clone https://github.com/walterfan/lazy-todo-app.git
cd lazy-todo-app
npm install

# 开发模式
npm run tauri dev

# 生产构建
npm run tauri build
```

### 数据库路径

默认数据库路径：

| 平台 | 路径 |
|---|---|
| macOS | `~/Library/Application Support/com.fanyamin.lazytodoapp/todos.db` |
| Linux | `~/.local/share/com.fanyamin.lazytodoapp/todos.db` |
| Windows | `%APPDATA%\com.fanyamin.lazytodoapp\todos.db` |

可以通过环境变量覆盖：

```bash
LAZY_TODO_DB_DIR=/path/to/your/folder npm run tauri dev
```

也可以通过本地配置文件持久化：

```json
{
  "db_dir": "~/Documents/lazy-todo-db"
}
```

配置文件位置：

```text
~/.config/lazy-todo-app/config.json
```

## 文档

项目知识库位于 `doc/`，使用 **Sphinx + MyST + sphinx-intl** 生成中英文站点。

在线文档地址：[https://walterfan.github.io/lazy-todo-app](https://walterfan.github.io/lazy-todo-app)

### 本地构建文档

```bash
cd doc
poetry install
poetry run make html
```

输出目录：

- 英文：`doc/_build/en/html/`
- 中文：`doc/_build/zh_CN/html/`

### 生成 GitHub Pages 站点

```bash
cd doc
poetry run make pages
```

生成内容：

- 站点根目录：`doc/_build/site/`
- 语言入口页：`doc/_build/site/index.html`
- 英文站点：`doc/_build/site/en/`
- 中文站点：`doc/_build/site/zh_CN/`

### 自动发布文档

仓库包含 `/.github/workflows/docs.yml`，会在以下情况自动发布 GitHub Pages：

- 推送到 `master` 或 `main`，且变更涉及 `doc/**`、`README.md` 或文档工作流本身
- 手动触发 `workflow_dispatch`

首次启用请在 GitHub 中设置：

1. 打开 `Settings -> Pages`
2. 将 `Source` 设置为 `GitHub Actions`

## 发布二进制

仓库包含 `/.github/workflows/release.yml`，在推送 `v*` 标签时自动构建 Tauri 安装包并发布到 GitHub Releases。

推荐使用发布辅助脚本：

```bash
./scripts/release_version.sh v0.1.1
```

或者：

```bash
npm run release:tag -- v0.1.1
```

这个脚本会更新 `package.json`、`src-tauri/tauri.conf.json` 和 `src-tauri/Cargo.toml` 中的版本号，创建发布提交，推送当前分支，然后创建并推送发布标签。

示例：

```bash
git tag v0.1.1
git push origin v0.1.1
```

完整的发布检查清单和双语文档发布步骤请参见 `doc/08-build.md`。

## Harness Engineering 实践

这个项目展示了如何通过规则、约束和自动化流程来“驯化” AI 编程，而不是让模型在没有边界的情况下自由生成代码。

### `AGENTS.md` 作为架构约束

`AGENTS.md` 定义了命令放置位置、前后端调用边界、数据库访问规则和 Tauri 命令返回约定。

### Pre-commit 检查

```bash
pip install pre-commit
pre-commit install
```

常见检查：

- `cargo clippy`
- `cargo test`
- `tsc --noEmit`

### Tauri 作为天然沙箱

前端只能通过 `invoke()` 调用后端命令，不能直接访问数据库或文件系统；后端则依赖 Rust 的类型系统和所有权模型来约束安全边界。

## Tauri 命令

| 领域 | 命令 | 说明 |
|---|---|---|
| Todo | `list_todos` | 查询所有任务 |
| Todo | `add_todo` | 添加任务 |
| Todo | `toggle_todo` | 切换完成状态 |
| Todo | `update_todo` | 更新任务 |
| Todo | `delete_todo` | 删除任务 |
| Todo | `list_due_todo_reminders` | 查询到期和逾期任务提醒 |
| Todo | `mark_todo_reminded` | 标记任务提醒已送达 |
| Notes | `list_notes` | 查询便签 |
| Notes | `add_note` | 新建便签 |
| Notes | `update_note` | 更新便签 |
| Notes | `delete_note` | 删除便签 |
| Pomodoro | `get_pomodoro_settings` | 获取番茄钟设置 |
| Pomodoro | `save_pomodoro_settings` | 保存番茄钟设置 |
| Pomodoro | `record_pomodoro_session` | 记录完成会话 |
| Pomodoro | `get_today_pomodoro_count` | 获取今日统计 |
| Pomodoro | `get_weekly_pomodoro_stats` | 获取 7 日统计 |
| Pomodoro | `update_tray_tooltip` | 更新托盘提示 |
| Agents | `list_agents` | 查询内置和已安装 Agent |
| Agents | `refresh_agents` | 重新扫描 Agent |
| Agents | `start_agent_session` | 创建单 Agent 对话 |
| Agents | `start_agent_group_session` | 创建多 Agent 群聊 |
| Agents | `send_agent_message_stream` | 流式发送单 Agent 回复 |
| Agents | `send_agent_group_message_stream` | 流式发送多 Agent 回复 |
| Agents | `get_agent_detail` | 查看 Agent 元数据、README、诊断和 RAG 状态 |
| Agents | `rebuild_agent_rag_index` | 重建单个 Agent 的 RAG 知识索引 |
| Agents | `list_agent_memories` | 查询本地 Agent 记忆 |
| Agents | `confirm_agent_tool_action` | 确认或拒绝 Agent 提议的写入/工具动作 |
| Agents | `list_agent_external_cli_tools` | 管理已注册外部 CLI 工具 |
| App | `get_db_path` | 获取数据库路径 |
| App | `get_app_settings` | 获取应用设置 |
| App | `save_app_settings` | 保存应用设置 |
| App | `quit_app` | 退出应用 |
| App | `open_note_window` | 打开独立便签窗口 |

## 相关文章

- [从 Prompt Engineering 到 Harness Engineering：AI 编程的四次进化](https://www.fanyamin.com/tech/harness-engineering.html)
- [Harness Engineering: Leveraging Codex in an Agent-First World](https://openai.com/index/harness-engineering/)
- [Tauri v2 Documentation](https://v2.tauri.app/)

## License

Apache-2.0
