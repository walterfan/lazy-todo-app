import { useEffect, useMemo, useState } from "react";
import type { SecretaryController } from "../hooks/useSecretary";
import type {
  ProposedNoteEdit,
  SaveSecretaryMemory,
  SaveSecretaryReminder,
  SecretaryProfile,
} from "../types/secretary";

const ROLE_OPTIONS = [
  { value: "question_answer", label: "Answer" },
  { value: "question_asker", label: "Ask" },
  { value: "idea_critic", label: "Critic" },
  { value: "idea_raiser", label: "Ideas" },
];

function localDueValue() {
  const date = new Date(Date.now() + 60 * 60 * 1000);
  const pad = (n: number) => String(n).padStart(2, "0");
  return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}`;
}

function sqlDateTime(value: string) {
  return value ? value.replace("T", " ") + ":00" : "";
}

interface SecretaryPanelProps {
  secretary: SecretaryController;
}

export function SecretaryPanel({ secretary }: SecretaryPanelProps) {
  const [message, setMessage] = useState("");
  const [settingsDraft, setSettingsDraft] = useState({
    skill_folder: "",
    conversation_folder: "",
  });
  const [profileDraft, setProfileDraft] = useState({
    name: "",
    role: "question_answer",
    domain: "",
  });
  const [personaDraft, setPersonaDraft] = useState({
    name: "",
    voice: "",
    values: "",
    style: "",
    boundaries: "",
  });
  const [memoryDraft, setMemoryDraft] = useState("");
  const [reminderDraft, setReminderDraft] = useState({ title: "", notes: "", due_at: localDueValue() });
  const [editableNoteEdit, setEditableNoteEdit] = useState<ProposedNoteEdit | null>(null);

  useEffect(() => {
    if (!secretary.settings) return;
    setSettingsDraft({
      skill_folder: secretary.settings.saved.skill_folder,
      conversation_folder: secretary.settings.saved.conversation_folder,
    });
  }, [secretary.settings]);

  useEffect(() => {
    if (secretary.activeProfile) {
      setProfileDraft({
        name: secretary.activeProfile.name,
        role: secretary.activeProfile.role,
        domain: secretary.activeProfile.domain,
      });
    }
  }, [secretary.activeProfile]);

  useEffect(() => {
    if (secretary.activePersona) {
      setPersonaDraft({
        name: secretary.activePersona.name,
        voice: secretary.activePersona.voice,
        values: secretary.activePersona.values,
        style: secretary.activePersona.style,
        boundaries: secretary.activePersona.boundaries,
      });
    }
  }, [secretary.activePersona]);

  useEffect(() => {
    setEditableNoteEdit(secretary.proposedNoteEdit);
  }, [secretary.proposedNoteEdit]);

  const activeSkillIds = secretary.activeProfile?.skill_ids ?? [];
  const selectedNoteTitles = useMemo(() => {
    const selected = secretary.appContext.notes.filter((note) =>
      secretary.selectedContext.note_ids.includes(note.id)
    );
    return selected.map((note) => note.title || `Note #${note.id}`).join(", ");
  }, [secretary.appContext.notes, secretary.selectedContext.note_ids]);

  if (secretary.loading) {
    return <div className="loading">Loading secretary...</div>;
  }

  const saveSettings = async () => {
    await secretary.saveSettings({
      base_url: secretary.settings?.saved.base_url ?? "",
      model: secretary.settings?.saved.model ?? "",
      skill_folder: settingsDraft.skill_folder,
      conversation_folder: settingsDraft.conversation_folder,
      active_persona_id: secretary.activePersona?.id ?? null,
      active_profile_id: secretary.activeProfile?.id ?? null,
    });
  };

  const savePersona = () => {
    void secretary.savePersona({
      id: secretary.activePersona?.id ?? null,
      ...personaDraft,
    });
  };

  const saveProfile = (patch: Partial<SecretaryProfile> = {}) => {
    void secretary.saveProfile({
      id: secretary.activeProfile?.id ?? null,
      name: profileDraft.name || "General Secretary",
      role: patch.role ?? profileDraft.role,
      domain: profileDraft.domain,
      persona_id: secretary.activePersona?.id ?? null,
      skill_ids: patch.skill_ids ?? activeSkillIds,
    });
  };

  const send = (quick?: string) => {
    const text = quick ?? message;
    if (!text.trim()) return;
    setMessage("");
    void secretary.sendMessage(text);
  };

  const addMemory = (content: string) => {
    const input: SaveSecretaryMemory = {
      content,
      scope: "global",
      domain: profileDraft.domain,
      profile_id: secretary.activeProfile?.id ?? null,
      status: "active",
      pinned: false,
      source_conversation_id: secretary.conversation?.id ?? null,
    };
    void secretary.saveMemory(input);
    setMemoryDraft("");
  };

  const addReminder = (reminder?: SaveSecretaryReminder) => {
    const input: SaveSecretaryReminder = reminder ?? {
      title: reminderDraft.title,
      notes: reminderDraft.notes,
      due_at: sqlDateTime(reminderDraft.due_at),
      status: "active",
      source_conversation_id: secretary.conversation?.id ?? null,
    };
    void secretary.saveReminder({ ...input, status: "active" });
    setReminderDraft({ title: "", notes: "", due_at: localDueValue() });
  };

  return (
    <div className="secretary-shell">
      {secretary.error && <div className="secretary-error">{secretary.error}</div>}

      <section className="secretary-main">
        <div className="secretary-identity">
          <div>
            <div className="secretary-name">{secretary.activePersona?.name ?? "Secretary"}</div>
            <div className="secretary-subtitle">
              {secretary.activeProfile?.domain || "Personal productivity"} · {ROLE_OPTIONS.find((r) => r.value === secretary.activeProfile?.role)?.label ?? "Answer"}
            </div>
          </div>
          <div className="secretary-status">
            <span>{secretary.memories.filter((m) => m.status === "active").length} memories</span>
            <span>{secretary.reminders.filter((r) => r.status === "active").length} reminders</span>
            {secretary.lastResult && (
              <span>
                used {secretary.lastResult.used_context.todos.length + secretary.lastResult.used_context.milestones.length + secretary.lastResult.used_context.notes.length} context
              </span>
            )}
          </div>
        </div>

        <div className="secretary-transcript">
          {secretary.conversation?.messages.length ? (
            secretary.conversation.messages.map((item, index) => (
              <div className={`secretary-message secretary-message-${item.role}`} key={`${item.created_at}-${index}`}>
                <div className="secretary-message-meta">{item.role} · {item.created_at}</div>
                <div className="secretary-message-body">{item.content}</div>
              </div>
            ))
          ) : (
            !secretary.pendingUserMessage && !secretary.streamingMessage && <div className="secretary-empty">
              Ask your secretary to review today, question an idea, summarize selected notes, or propose a reminder.
            </div>
          )}
          {secretary.pendingUserMessage && (
            <div className="secretary-message secretary-message-user secretary-message-pending">
              <div className="secretary-message-meta">user · sending</div>
              <div className="secretary-message-body">{secretary.pendingUserMessage}</div>
            </div>
          )}
          {secretary.streamingMessage && (
            <div className="secretary-message secretary-message-assistant secretary-message-streaming">
              <div className="secretary-message-meta">assistant · streaming</div>
              <div className="secretary-message-body">{secretary.streamingMessage}</div>
            </div>
          )}
        </div>

        {secretary.lastResult?.proposed_memory && (
          <div className="secretary-proposal">
            <strong>Proposed memory</strong>
            <p>{secretary.lastResult.proposed_memory}</p>
            <button onClick={() => addMemory(secretary.lastResult!.proposed_memory!)}>Save</button>
          </div>
        )}

        {secretary.lastResult?.proposed_reminder && (
          <div className="secretary-proposal">
            <strong>Proposed reminder</strong>
            <p>{secretary.lastResult.proposed_reminder.title}</p>
            <button onClick={() => addReminder(secretary.lastResult!.proposed_reminder!)}>Save</button>
          </div>
        )}

        {editableNoteEdit && (
          <div className="secretary-note-edit">
            <div className="secretary-note-edit-header">
              <strong>Proposed note edit #{editableNoteEdit.note_id}</strong>
              <span>{editableNoteEdit.before_title}</span>
            </div>
            <div className="secretary-note-diff">
              <textarea value={editableNoteEdit.before_content} readOnly />
              <textarea
                value={editableNoteEdit.content ?? ""}
                onChange={(e) => setEditableNoteEdit({ ...editableNoteEdit, content: e.target.value })}
              />
            </div>
            <div className="secretary-actions">
              <button onClick={() => void secretary.confirmNoteEdit(editableNoteEdit, true)}>Apply</button>
              <button onClick={() => void secretary.confirmNoteEdit(editableNoteEdit, false)}>Reject</button>
            </div>
          </div>
        )}

        <div className="secretary-composer">
          <textarea
            value={message}
            onChange={(e) => setMessage(e.target.value)}
            placeholder={selectedNoteTitles ? `Ask about: ${selectedNoteTitles}` : "Ask your secretary..."}
          />
          <div className="secretary-composer-actions">
            <button onClick={() => send()} disabled={secretary.sending || !message.trim()}>
              {secretary.sending ? "Streaming..." : "Send"}
            </button>
            <button onClick={() => send("What should I focus on next based on my selected context?")}>Suggest</button>
            <button onClick={() => send("Ask me three clarifying questions about my current work.")}>Ask Me</button>
            <button onClick={() => send("Critique my current plan using the selected context.")}>Critique</button>
          </div>
        </div>
      </section>

      <aside className="secretary-side">
        <section className="secretary-panel">
          <h3>Persona</h3>
          <input value={personaDraft.name} onChange={(e) => setPersonaDraft({ ...personaDraft, name: e.target.value })} placeholder="Name" />
          <input value={personaDraft.voice} onChange={(e) => setPersonaDraft({ ...personaDraft, voice: e.target.value })} placeholder="Voice" />
          <textarea value={personaDraft.values} onChange={(e) => setPersonaDraft({ ...personaDraft, values: e.target.value })} placeholder="Values" />
          <textarea value={personaDraft.style} onChange={(e) => setPersonaDraft({ ...personaDraft, style: e.target.value })} placeholder="Style" />
          <textarea value={personaDraft.boundaries} onChange={(e) => setPersonaDraft({ ...personaDraft, boundaries: e.target.value })} placeholder="Boundaries" />
          <button onClick={savePersona}>Save Persona</button>
        </section>

        <section className="secretary-panel">
          <h3>Profile</h3>
          <input value={profileDraft.name} onChange={(e) => setProfileDraft({ ...profileDraft, name: e.target.value })} placeholder="Profile name" />
          <input value={profileDraft.domain} onChange={(e) => setProfileDraft({ ...profileDraft, domain: e.target.value })} placeholder="Domain" />
          <div className="secretary-role-row">
            {ROLE_OPTIONS.map((role) => (
              <button
                key={role.value}
                className={profileDraft.role === role.value ? "active" : ""}
                onClick={() => setProfileDraft({ ...profileDraft, role: role.value })}
              >
                {role.label}
              </button>
            ))}
          </div>
          <button onClick={() => saveProfile()}>Save Profile</button>
        </section>

        <section className="secretary-panel">
          <h3>Memory</h3>
          <textarea value={memoryDraft} onChange={(e) => setMemoryDraft(e.target.value)} placeholder="Add memory..." />
          <button onClick={() => addMemory(memoryDraft)} disabled={!memoryDraft.trim()}>Save Memory</button>
          <div className="secretary-list">
            {secretary.memories.slice(0, 5).map((memory) => (
              <div key={memory.id} className="secretary-list-item">
                <span>{memory.content}</span>
                <button onClick={() => void secretary.deleteMemory(memory.id)}>Forget</button>
              </div>
            ))}
          </div>
        </section>

        <section className="secretary-panel">
          <h3>Reminders</h3>
          <input value={reminderDraft.title} onChange={(e) => setReminderDraft({ ...reminderDraft, title: e.target.value })} placeholder="Reminder title" />
          <input value={reminderDraft.due_at} onChange={(e) => setReminderDraft({ ...reminderDraft, due_at: e.target.value })} type="datetime-local" />
          <button onClick={() => addReminder()} disabled={!reminderDraft.title.trim()}>Add Reminder</button>
          <div className="secretary-list">
            {secretary.reminders.slice(0, 5).map((reminder) => (
              <div key={reminder.id} className="secretary-list-item">
                <span>{reminder.title}</span>
                <button onClick={() => void secretary.saveReminder({ ...reminder, status: "completed" })}>Done</button>
              </div>
            ))}
          </div>
        </section>

        <section className="secretary-panel">
          <h3>Skills</h3>
          <input value={settingsDraft.skill_folder} onChange={(e) => setSettingsDraft({ ...settingsDraft, skill_folder: e.target.value })} placeholder="Skill folder" />
          <button onClick={() => void (async () => { await saveSettings(); await secretary.refreshSkills(); })()}>Refresh Skills</button>
          <div className="secretary-list">
            {secretary.skills.slice(0, 6).map((skill) => (
              <label key={skill.id}>
                <input
                  type="checkbox"
                  checked={activeSkillIds.includes(skill.id)}
                  onChange={(e) => {
                    const next = e.target.checked
                      ? [...activeSkillIds, skill.id]
                      : activeSkillIds.filter((id) => id !== skill.id);
                    saveProfile({ skill_ids: next });
                  }}
                />
                {skill.name}
              </label>
            ))}
            {secretary.skillSkipped.map((item) => <small key={item}>{item}</small>)}
          </div>
        </section>

        <section className="secretary-panel">
          <h3>Conversations</h3>
          <input value={settingsDraft.conversation_folder} onChange={(e) => setSettingsDraft({ ...settingsDraft, conversation_folder: e.target.value })} placeholder="Conversation folder" />
          <div className="secretary-actions">
            <button onClick={() => void secretary.startConversation()}>New</button>
            <button onClick={() => void secretary.saveTranscript()} disabled={!secretary.conversation}>Save</button>
          </div>
          <div className="secretary-list">
            {secretary.conversations.slice(0, 6).map((conversation) => (
              <button key={conversation.id} onClick={() => void secretary.loadConversation(conversation.id)}>
                {conversation.title}
              </button>
            ))}
          </div>
        </section>
      </aside>
    </div>
  );
}
