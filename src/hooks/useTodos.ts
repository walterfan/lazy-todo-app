import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
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
