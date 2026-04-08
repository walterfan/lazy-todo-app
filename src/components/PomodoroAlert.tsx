import { useEffect, useRef } from "react";
import type { TimerPhase } from "../types/pomodoro";

interface PomodoroAlertProps {
  completedPhase: TimerPhase;
  onDismiss: () => void;
}

function playChime() {
  try {
    const ctx = new AudioContext();
    const gain = ctx.createGain();
    gain.connect(ctx.destination);
    gain.gain.setValueAtTime(0.3, ctx.currentTime);

    const notes = [523.25, 659.25, 783.99, 1046.50]; // C5, E5, G5, C6
    notes.forEach((freq, i) => {
      const osc = ctx.createOscillator();
      const noteGain = ctx.createGain();
      osc.type = "sine";
      osc.frequency.value = freq;
      osc.connect(noteGain);
      noteGain.connect(gain);

      const start = ctx.currentTime + i * 0.2;
      noteGain.gain.setValueAtTime(0, start);
      noteGain.gain.linearRampToValueAtTime(0.4, start + 0.05);
      noteGain.gain.exponentialRampToValueAtTime(0.01, start + 0.5);

      osc.start(start);
      osc.stop(start + 0.5);
    });

    // Second round (repeat after a pause)
    const offset = 1.2;
    notes.forEach((freq, i) => {
      const osc = ctx.createOscillator();
      const noteGain = ctx.createGain();
      osc.type = "sine";
      osc.frequency.value = freq;
      osc.connect(noteGain);
      noteGain.connect(gain);

      const start = ctx.currentTime + offset + i * 0.2;
      noteGain.gain.setValueAtTime(0, start);
      noteGain.gain.linearRampToValueAtTime(0.4, start + 0.05);
      noteGain.gain.exponentialRampToValueAtTime(0.01, start + 0.5);

      osc.start(start);
      osc.stop(start + 0.5);
    });

    setTimeout(() => ctx.close(), 3000);
  } catch {
    // Web Audio API unavailable
  }
}

const MESSAGES: Record<TimerPhase, { emoji: string; title: string; body: string }> = {
  work: {
    emoji: "🍅",
    title: "Pomodoro Complete!",
    body: "Great work! Time to take a break and recharge.",
  },
  short_break: {
    emoji: "☕",
    title: "Break Over!",
    body: "Feeling refreshed? Let's get back to work!",
  },
  long_break: {
    emoji: "🎉",
    title: "Long Break Over!",
    body: "Well rested? A new cycle awaits!",
  },
};

export function PomodoroAlert({ completedPhase, onDismiss }: PomodoroAlertProps) {
  const played = useRef(false);

  useEffect(() => {
    if (!played.current) {
      played.current = true;
      playChime();
    }
  }, []);

  const msg = MESSAGES[completedPhase];

  return (
    <div className="pomo-alert-overlay" onClick={onDismiss}>
      <div className="pomo-alert" onClick={(e) => e.stopPropagation()}>
        <div className="pomo-alert-emoji">{msg.emoji}</div>
        <h2 className="pomo-alert-title">{msg.title}</h2>
        <p className="pomo-alert-body">{msg.body}</p>
        <button className="pomo-btn pomo-btn-start" onClick={onDismiss}>
          OK
        </button>
      </div>
    </div>
  );
}
