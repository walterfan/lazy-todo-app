export interface Todo {
  id: number;
  title: string;
  description: string;
  priority: number; // 1=High, 2=Medium, 3=Low
  completed: boolean;
  deadline: string | null; // ISO 8601
  recurrence: Recurrence;
  recurrence_anchor: string | null;
  reminder_minutes_before: number | null;
  reminder_due_at: string | null;
  reminder_state: ReminderState;
  last_reminded_at: string | null;
  last_reminded_deadline: string | null;
  created_at: string;
}

export type Recurrence = "none" | "daily" | "weekly" | "monthly" | "yearly";
export type ReminderState = "none" | "upcoming" | "due" | "reminded" | "missed" | "overdue";

export interface CreateTodo {
  title: string;
  description?: string;
  priority?: number;
  deadline?: string;
  recurrence?: Recurrence;
  reminder_minutes_before?: number;
}

export interface UpdateTodo {
  id: number;
  title?: string;
  description?: string;
  priority?: number;
  deadline?: string;
  clear_deadline?: boolean;
  recurrence?: Recurrence;
  reminder_minutes_before?: number;
}
