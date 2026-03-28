export interface Todo {
  id: number;
  title: string;
  description: string;
  priority: number; // 1=High, 2=Medium, 3=Low
  completed: boolean;
  deadline: string | null; // ISO 8601
  created_at: string;
}

export interface CreateTodo {
  title: string;
  description?: string;
  priority?: number;
  deadline?: string;
}

export interface UpdateTodo {
  id: number;
  title?: string;
  description?: string;
  priority?: number;
  deadline?: string;
}
