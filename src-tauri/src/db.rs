use rusqlite::{Connection, Result, params};
use std::sync::Mutex;

use crate::models::todo::Todo;

pub struct Database {
    conn: Mutex<Connection>,
}

impl Database {
    pub fn new(app_dir: &std::path::Path) -> Result<Self> {
        std::fs::create_dir_all(app_dir).ok();
        let db_path = app_dir.join("todos.db");
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS todos (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                title       TEXT NOT NULL,
                description TEXT NOT NULL DEFAULT '',
                priority    INTEGER NOT NULL DEFAULT 2,
                completed   INTEGER NOT NULL DEFAULT 0,
                deadline    TEXT,
                created_at  TEXT NOT NULL DEFAULT (datetime('now'))
            );"
        )?;

        Ok(Self { conn: Mutex::new(conn) })
    }

    pub fn list_todos(&self) -> Result<Vec<Todo>> {
        let conn = self.conn.lock().expect("db lock poisoned");
        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, completed, deadline, created_at
             FROM todos ORDER BY completed ASC, priority ASC, deadline ASC NULLS LAST"
        )?;

        let todos = stmt.query_map([], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                deadline: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?.collect::<Result<Vec<_>>>()?;

        Ok(todos)
    }

    pub fn add_todo(&self, title: &str, description: &str, priority: i32, deadline: Option<&str>) -> Result<Todo> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "INSERT INTO todos (title, description, priority, deadline) VALUES (?1, ?2, ?3, ?4)",
            params![title, description, priority, deadline],
        )?;
        let id = conn.last_insert_rowid();

        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, completed, deadline, created_at FROM todos WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                deadline: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
    }

    pub fn toggle_todo(&self, id: i64) -> Result<Todo> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute(
            "UPDATE todos SET completed = NOT completed WHERE id = ?1",
            params![id],
        )?;

        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, completed, deadline, created_at FROM todos WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                deadline: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
    }

    pub fn update_todo(&self, id: i64, title: Option<&str>, description: Option<&str>, priority: Option<i32>, deadline: Option<&str>) -> Result<Todo> {
        let conn = self.conn.lock().expect("db lock poisoned");

        if let Some(t) = title {
            conn.execute("UPDATE todos SET title = ?1 WHERE id = ?2", params![t, id])?;
        }
        if let Some(d) = description {
            conn.execute("UPDATE todos SET description = ?1 WHERE id = ?2", params![d, id])?;
        }
        if let Some(p) = priority {
            conn.execute("UPDATE todos SET priority = ?1 WHERE id = ?2", params![p, id])?;
        }
        if let Some(dl) = deadline {
            conn.execute("UPDATE todos SET deadline = ?1 WHERE id = ?2", params![dl, id])?;
        }

        let mut stmt = conn.prepare(
            "SELECT id, title, description, priority, completed, deadline, created_at FROM todos WHERE id = ?1"
        )?;
        stmt.query_row(params![id], |row| {
            Ok(Todo {
                id: row.get(0)?,
                title: row.get(1)?,
                description: row.get(2)?,
                priority: row.get(3)?,
                completed: row.get::<_, i32>(4)? != 0,
                deadline: row.get(5)?,
                created_at: row.get(6)?,
            })
        })
    }

    pub fn delete_todo(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().expect("db lock poisoned");
        conn.execute("DELETE FROM todos WHERE id = ?1", params![id])?;
        Ok(())
    }
}
