import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { sendNotification, isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import type { Todo, CreateTodo, UpdateTodo } from "../types/todo";

export function useTodos() {
  const [todos, setTodos] = useState<Todo[]>([]);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    try {
      const data = await invoke<Todo[]>("list_todos");
      setTodos(data);
    } catch (err) {
      console.error("Failed to load todos:", err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    refresh();
  }, [refresh]);

  const notifyReminder = useCallback(async (todo: Todo) => {
    try {
      let granted = await isPermissionGranted();
      if (!granted) {
        const permission = await requestPermission();
        granted = permission === "granted";
      }
      if (granted) {
        const state = todo.reminder_state === "overdue" ? "Overdue" : "Reminder";
        sendNotification({
          title: `${state}: ${todo.title}`,
          body: todo.deadline ? `Due ${new Date(todo.deadline).toLocaleString()}` : todo.description || "Todo reminder",
        });
      }
    } catch {
      // Notification support can be unavailable; in-app reminder state still updates.
    }
  }, []);

  const checkDueReminders = useCallback(async () => {
    try {
      const dueTodos = await invoke<Todo[]>("list_due_todo_reminders");
      if (dueTodos.length === 0) return;
      for (const todo of dueTodos) {
        await notifyReminder(todo);
        await invoke("mark_todo_reminded", { id: todo.id });
      }
      await refresh();
    } catch (err) {
      console.error("Failed to check todo reminders:", err);
    }
  }, [notifyReminder, refresh]);

  useEffect(() => {
    void checkDueReminders();
    const timer = window.setInterval(() => {
      void checkDueReminders();
    }, 60_000);
    return () => window.clearInterval(timer);
  }, [checkDueReminders]);

  const addTodo = async (input: CreateTodo) => {
    await invoke("add_todo", { input });
    await refresh();
  };

  const toggleTodo = async (id: number) => {
    await invoke("toggle_todo", { id });
    await refresh();
  };

  const updateTodo = async (input: UpdateTodo) => {
    await invoke("update_todo", { input });
    await refresh();
  };

  const deleteTodo = async (id: number) => {
    await invoke("delete_todo", { id });
    await refresh();
  };

  return { todos, loading, addTodo, toggleTodo, updateTodo, deleteTodo };
}
