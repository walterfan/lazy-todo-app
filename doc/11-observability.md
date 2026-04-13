# Observability — Log, Metrics, Alert

<!-- maintained-by: human+ai -->

This document describes the **LMA** (Logging, Metrics, Alerting) strategy for diagnosing issues and monitoring the Lazy Todo App.

> **Relationship to [10-runbook.md](10-runbook.md)**: The runbook covers *operational procedures* (how to respond). This doc covers the *observability infrastructure* (how to see what's happening).

---

## Philosophy

Even a desktop/local-first app benefits from structured observability:

- **Logging** — understand what happened and why
- **Metrics** — understand how the system is performing
- **Alerting** — get notified when something goes wrong

```
┌──────────────────────────────────────────────┐
│                Application                    │
│  ┌──────────┐  ┌──────────┐  ┌────────────┐ │
│  │  Logs    │  │ Metrics  │  │  Traces    │ │
│  └────┬─────┘  └────┬─────┘  └─────┬──────┘ │
└───────┼──────────────┼──────────────┼────────┘
        ▼              ▼              ▼
   ┌─────────┐   ┌──────────┐  ┌──────────┐
   │Log Store│   │Metrics DB│  │Trace Store│
   └────┬────┘   └────┬─────┘  └─────┬────┘
        └──────────────┼──────────────┘
                       ▼
               ┌──────────────┐
               │  Dashboard   │
               │  & Alerting  │
               └──────────────┘
```

---

## 1. Logging

### Frontend (TypeScript / React)

| Aspect | Approach |
|--------|----------|
| **Library** | `console.*` in dev; structured logger (e.g. `loglevel` or custom) in prod |
| **Levels** | `debug`, `info`, `warn`, `error` |
| **Format** | JSON-structured where possible: `{ ts, level, module, msg, data }` |
| **Output** | Browser DevTools console; optionally forward to Rust backend via Tauri commands |

### Backend (Rust / Tauri)

| Aspect | Approach |
|--------|----------|
| **Library** | `tracing` + `tracing-subscriber` (Rust ecosystem standard) |
| **Levels** | `TRACE`, `DEBUG`, `INFO`, `WARN`, `ERROR` |
| **Format** | Structured JSON or human-readable (configurable via `RUST_LOG`) |
| **Output** | `stdout` in dev; log file in production (`app_data_dir/logs/`) |
| **Rotation** | `tracing-appender` with daily rotation, keep 7 days |

### Log Configuration

```bash
# Development — verbose
RUST_LOG=lazy_todo_app=debug,tauri=info

# Production — errors and warnings only
RUST_LOG=lazy_todo_app=info,tauri=warn
```

### Log Location

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/com.lazy-todo-app/logs/` |
| Linux | `~/.local/share/lazy-todo-app/logs/` |
| Windows | `%APPDATA%\lazy-todo-app\logs\` |

---

## 2. Metrics

### Key Metrics to Track

#### Application Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `todo.created_total` | Counter | Total todos created |
| `todo.completed_total` | Counter | Total todos completed |
| `todo.deleted_total` | Counter | Total todos deleted |
| `todo.active_count` | Gauge | Current active (incomplete) todos |
| `app.startup_duration_ms` | Histogram | Time from launch to UI ready |
| `db.query_duration_ms` | Histogram | SQLite query latency |

#### System Metrics (Desktop)

| Metric | Type | Description |
|--------|------|-------------|
| `app.memory_usage_bytes` | Gauge | RSS memory usage |
| `app.cpu_usage_percent` | Gauge | CPU usage of the app process |
| `db.file_size_bytes` | Gauge | SQLite database file size |

### Collection Approach

For a desktop app, heavy metric infrastructure (Prometheus, Grafana) is overkill. Recommended approach:

1. **Development**: Log metrics to console / log file periodically
2. **Optional telemetry**: If user opts in, send anonymous usage metrics to a lightweight backend
3. **Health check endpoint**: If the app exposes a local API, add `/health` returning basic stats

```rust
// Example: periodic metrics logging
fn log_app_metrics(db: &Database) {
    let stats = db.get_stats();
    tracing::info!(
        active_todos = stats.active_count,
        db_size_bytes = stats.db_file_size,
        "app_metrics"
    );
}
```

---

## 3. Alerting

### Desktop App Alerting Strategy

Since this is a local-first desktop app, "alerting" means **in-app notifications** rather than PagerDuty:

| Condition | Alert Type | Action |
|-----------|-----------|--------|
| Database file corrupted | Error dialog | Prompt user to restore from backup |
| Database size > 100 MB | Warning toast | Suggest cleanup / archive |
| Unhandled Rust panic | Error dialog + log | Log stack trace, offer to report |
| Frontend uncaught exception | Error boundary | Show fallback UI, log error |
| App update available | Info toast | Prompt to download new version |

### Error Boundary (React)

```tsx
// Top-level error boundary captures all React rendering errors
<ErrorBoundary
  fallback={<ErrorFallback />}
  onError={(error, info) => {
    logError({ error, componentStack: info.componentStack });
  }}
>
  <App />
</ErrorBoundary>
```

### Rust Panic Hook

```rust
std::panic::set_hook(Box::new(|info| {
    tracing::error!(
        panic = %info,
        "unhandled_panic"
    );
    // Optionally write crash report to disk
}));
```

---

## 4. Diagnostic Tools

### Built-in Diagnostics

Consider adding a **Debug / Diagnostics** page in the app (dev mode only):

| Feature | Description |
|---------|-------------|
| Log viewer | Tail the last N log entries |
| DB stats | Row counts, file size, last vacuum |
| System info | OS, app version, Tauri version, WebView version |
| Export logs | Bundle logs into a zip for bug reports |

### External Tools

| Tool | Use Case |
|------|----------|
| **Tauri DevTools** | Inspect WebView, network, console |
| **SQLite Browser** | Inspect the local database directly |
| **Activity Monitor / htop** | Check resource usage |
| **`RUST_LOG`** | Adjust log verbosity at runtime |

---

## 5. Observability Checklist

- [ ] Structured logging configured for both frontend and backend
- [ ] Log rotation enabled in production builds
- [ ] Error boundaries catch and log React errors
- [ ] Rust panic hook logs crash info
- [ ] Key business metrics (todo CRUD counts) logged
- [ ] Database health checked on startup
- [ ] User-facing error messages are helpful (not raw stack traces)
- [ ] Debug/diagnostics page available in dev mode

---

## References

- [Tauri Logging Plugin](https://v2.tauri.app/plugin/logging/)
- [tracing crate](https://docs.rs/tracing/latest/tracing/)
- [React Error Boundaries](https://react.dev/reference/react/Component#catching-rendering-errors-with-an-error-boundary)
- [10-runbook.md](10-runbook.md) — operational procedures

---
<!-- PKB-metadata
last_updated: 2025-07-14
updated_by: human+ai
-->
