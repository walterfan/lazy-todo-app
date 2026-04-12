import { useState, useEffect } from "react";

export interface CountdownResult {
  label: string;
  urgent: boolean; // less than 1 hour
  overdue: boolean;
}

export function useCountdown(deadline: string | null): CountdownResult | null {
  const [now, setNow] = useState(() => Date.now());

  useEffect(() => {
    if (!deadline) return;
    const timer = setInterval(() => setNow(Date.now()), 1000);
    return () => clearInterval(timer);
  }, [deadline]);

  if (!deadline) return null;

  const target = new Date(deadline).getTime();
  const diff = target - now;

  if (diff <= 0) {
    const elapsed = -diff;
    const days = Math.floor(elapsed / (1000 * 60 * 60 * 24));
    const hours = Math.floor((elapsed % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
    const mins = Math.floor((elapsed % (1000 * 60 * 60)) / (1000 * 60));
    let overdueLabel: string;
    if (days > 0) {
      overdueLabel = `Overdue ${days}d ${hours}h`;
    } else if (hours > 0) {
      overdueLabel = `Overdue ${hours}h ${mins}m`;
    } else {
      overdueLabel = `Overdue ${mins}m`;
    }
    return { label: overdueLabel, urgent: true, overdue: true };
  }

  const days = Math.floor(diff / (1000 * 60 * 60 * 24));
  const hours = Math.floor((diff % (1000 * 60 * 60 * 24)) / (1000 * 60 * 60));
  const minutes = Math.floor((diff % (1000 * 60 * 60)) / (1000 * 60));
  const seconds = Math.floor((diff % (1000 * 60)) / 1000);

  let label: string;
  if (days > 0) {
    label = `${days}d ${hours}h ${minutes}m left`;
  } else if (hours > 0) {
    label = `${hours}h ${minutes}m ${seconds}s left`;
  } else {
    label = `${minutes}m ${seconds}s left`;
  }

  const urgent = diff < 1000 * 60 * 60; // less than 1 hour

  return { label, urgent, overdue: false };
}
