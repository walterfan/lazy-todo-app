import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { sendNotification, isPermissionGranted, requestPermission } from "@tauri-apps/plugin-notification";
import type {
  MaskedSecretarySettings,
  ProposedNoteEdit,
  SaveSecretaryMemory,
  SaveSecretaryPersona,
  SaveSecretaryProfile,
  SaveSecretaryReminder,
  SecretaryAppContext,
  SecretaryConversation,
  SecretaryMemory,
  SecretaryPersona,
  SecretaryProfile,
  SecretaryReminder,
  SecretaryStreamEvent,
  SecretarySkill,
  SelectedAppContext,
  SendSecretaryMessageResult,
} from "../types/secretary";
import { DEFAULT_SELECTED_CONTEXT } from "../types/secretary";

export function useSecretary() {
  const [settings, setSettings] = useState<MaskedSecretarySettings | null>(null);
  const [personas, setPersonas] = useState<SecretaryPersona[]>([]);
  const [profiles, setProfiles] = useState<SecretaryProfile[]>([]);
  const [skills, setSkills] = useState<SecretarySkill[]>([]);
  const [memories, setMemories] = useState<SecretaryMemory[]>([]);
  const [reminders, setReminders] = useState<SecretaryReminder[]>([]);
  const [appContext, setAppContext] = useState<SecretaryAppContext>({ todos: [], milestones: [], notes: [] });
  const [conversations, setConversations] = useState<SecretaryConversation[]>([]);
  const [conversation, setConversation] = useState<SecretaryConversation | null>(null);
  const [selectedContext, setSelectedContext] = useState<SelectedAppContext>(DEFAULT_SELECTED_CONTEXT);
  const [proposedNoteEdit, setProposedNoteEdit] = useState<ProposedNoteEdit | null>(null);
  const [lastResult, setLastResult] = useState<SendSecretaryMessageResult | null>(null);
  const [skillSkipped, setSkillSkipped] = useState<string[]>([]);
  const [loading, setLoading] = useState(true);
  const [sending, setSending] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [pendingUserMessage, setPendingUserMessage] = useState<string | null>(null);
  const [streamingMessage, setStreamingMessage] = useState("");
  const notifiedReminderIds = useRef<Set<number>>(new Set());
  const activeStreamId = useRef<string | null>(null);

  const activeProfile = useMemo(
    () => profiles.find((p) => p.id === settings?.saved.active_profile_id) ?? profiles[0] ?? null,
    [profiles, settings]
  );
  const activePersona = useMemo(
    () => personas.find((p) => p.id === activeProfile?.persona_id) ?? personas[0] ?? null,
    [personas, activeProfile]
  );

  const refresh = useCallback(async () => {
    setError(null);
    try {
      const [
        nextSettings,
        nextPersonas,
        nextProfiles,
        nextSkills,
        nextMemories,
        nextReminders,
        nextContext,
        nextConversations,
      ] = await Promise.all([
        invoke<MaskedSecretarySettings>("get_secretary_settings"),
        invoke<SecretaryPersona[]>("list_secretary_personas"),
        invoke<SecretaryProfile[]>("list_secretary_profiles"),
        invoke<SecretarySkill[]>("list_secretary_skills"),
        invoke<SecretaryMemory[]>("list_secretary_memories"),
        invoke<SecretaryReminder[]>("list_secretary_reminders"),
        invoke<SecretaryAppContext>("get_secretary_app_context"),
        invoke<SecretaryConversation[]>("list_secretary_conversations"),
      ]);
      setSettings(nextSettings);
      setPersonas(nextPersonas);
      setProfiles(nextProfiles);
      setSkills(nextSkills);
      setMemories(nextMemories);
      setReminders(nextReminders);
      setAppContext(nextContext);
      setConversations(nextConversations);
      if (!conversation && nextConversations.length > 0) {
        setConversation(nextConversations[0]);
      }
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, [conversation]);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  useEffect(() => {
    const unlistenChunk = listen<SecretaryStreamEvent>("secretary-stream-chunk", (event) => {
      if (event.payload.stream_id !== activeStreamId.current) return;
      setStreamingMessage((current) => current + (event.payload.content ?? ""));
    });
    const unlistenError = listen<SecretaryStreamEvent>("secretary-stream-error", (event) => {
      if (event.payload.stream_id !== activeStreamId.current) return;
      setError(event.payload.error ?? "Secretary stream failed.");
      setPendingUserMessage(null);
      setStreamingMessage("");
      setSending(false);
      activeStreamId.current = null;
    });
    const unlistenFinish = listen<SecretaryStreamEvent>("secretary-stream-finish", (event) => {
      if (event.payload.stream_id !== activeStreamId.current) return;
      if (event.payload.result) {
        setConversation(event.payload.result.conversation);
        setLastResult(event.payload.result);
        setProposedNoteEdit(event.payload.result.proposed_note_edit);
      }
      setPendingUserMessage(null);
      setStreamingMessage("");
      setSending(false);
      activeStreamId.current = null;
    });

    return () => {
      void unlistenChunk.then((fn) => fn());
      void unlistenError.then((fn) => fn());
      void unlistenFinish.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    const due = reminders.filter((reminder) => {
      if (reminder.status !== "active" || notifiedReminderIds.current.has(reminder.id)) return false;
      const dueMs = Date.parse(reminder.due_at.replace(" ", "T"));
      return Number.isFinite(dueMs) && dueMs <= Date.now();
    });
    if (due.length === 0) return;

    due.forEach((reminder) => notifiedReminderIds.current.add(reminder.id));
    void (async () => {
      try {
        let granted = await isPermissionGranted();
        if (!granted) {
          const permission = await requestPermission();
          granted = permission === "granted";
        }
        if (granted) {
          due.forEach((reminder) => {
            sendNotification({
              title: "Secretary reminder",
              body: reminder.title,
            });
          });
        }
      } catch {
        // notification support may be unavailable
      }
    })();
  }, [reminders]);

  const saveSettings = useCallback(async (input: Record<string, unknown>) => {
    const next = await invoke<MaskedSecretarySettings>("save_secretary_settings", { input });
    setSettings(next);
  }, []);

  const savePersona = useCallback(async (input: SaveSecretaryPersona) => {
    await invoke("save_secretary_persona", { input });
    await refresh();
  }, [refresh]);

  const saveProfile = useCallback(async (input: SaveSecretaryProfile) => {
    const profile = await invoke<SecretaryProfile>("save_secretary_profile", { input });
    await invoke("save_secretary_settings", {
      input: {
        base_url: settings?.saved.base_url ?? "",
        model: settings?.saved.model ?? "",
        skill_folder: settings?.saved.skill_folder ?? "",
        conversation_folder: settings?.saved.conversation_folder ?? "",
        active_persona_id: profile.persona_id,
        active_profile_id: profile.id,
      },
    });
    await refresh();
  }, [refresh, settings]);

  const refreshSkills = useCallback(async () => {
    const result = await invoke<{ skills: SecretarySkill[]; skipped: string[] }>("refresh_secretary_skills");
    setSkills(result.skills);
    setSkillSkipped(result.skipped);
  }, []);

  const saveMemory = useCallback(async (input: SaveSecretaryMemory) => {
    await invoke("save_secretary_memory", { input });
    await refresh();
  }, [refresh]);

  const deleteMemory = useCallback(async (id: number) => {
    await invoke("delete_secretary_memory", { id });
    await refresh();
  }, [refresh]);

  const saveReminder = useCallback(async (input: SaveSecretaryReminder) => {
    await invoke("save_secretary_reminder", { input });
    await refresh();
  }, [refresh]);

  const deleteReminder = useCallback(async (id: number) => {
    await invoke("delete_secretary_reminder", { id });
    await refresh();
  }, [refresh]);

  const startConversation = useCallback(async () => {
    const next = await invoke<SecretaryConversation>("start_secretary_conversation", {
      profileId: activeProfile?.id ?? null,
    });
    setConversation(next);
    await refresh();
  }, [activeProfile, refresh]);

  const loadConversation = useCallback(async (id: number) => {
    const next = await invoke<SecretaryConversation>("load_secretary_conversation", { id });
    setConversation(next);
  }, []);

  const sendMessage = useCallback(async (message: string) => {
    if (!message.trim()) return;
    const streamId = `secretary-${Date.now()}-${Math.random().toString(36).slice(2)}`;
    activeStreamId.current = streamId;
    setPendingUserMessage(message.trim());
    setStreamingMessage("");
    setSending(true);
    setError(null);
    try {
      const result = await invoke<SendSecretaryMessageResult>("send_secretary_message_stream", {
        input: {
          conversation_id: conversation?.id ?? null,
          profile_id: activeProfile?.id ?? null,
          message,
          selected_context: selectedContext,
          stream_id: streamId,
        },
      });
      setConversation(result.conversation);
      setLastResult(result);
      setProposedNoteEdit(result.proposed_note_edit);
      setPendingUserMessage(null);
      setStreamingMessage("");
      activeStreamId.current = null;
      await refresh();
    } catch (err) {
      setError(String(err));
      activeStreamId.current = null;
    } finally {
      setSending(false);
    }
  }, [activeProfile, conversation, refresh, selectedContext]);

  const confirmNoteEdit = useCallback(async (edit: ProposedNoteEdit, accepted: boolean) => {
    await invoke("confirm_secretary_note_edit", {
      input: {
        conversation_id: conversation?.id ?? null,
        edit,
        accepted,
      },
    });
    setProposedNoteEdit(null);
    await refresh();
  }, [conversation, refresh]);

  const saveTranscript = useCallback(async () => {
    if (!conversation) return;
    const next = await invoke<SecretaryConversation>("save_secretary_transcript", { id: conversation.id });
    setConversation(next);
    await refresh();
  }, [conversation, refresh]);

  return {
    settings,
    personas,
    profiles,
    skills,
    memories,
    reminders,
    appContext,
    conversations,
    conversation,
    selectedContext,
    proposedNoteEdit,
    lastResult,
    skillSkipped,
    loading,
    sending,
    error,
    pendingUserMessage,
    streamingMessage,
    activeProfile,
    activePersona,
    setSelectedContext,
    setProposedNoteEdit,
    refresh,
    saveSettings,
    savePersona,
    saveProfile,
    refreshSkills,
    saveMemory,
    deleteMemory,
    saveReminder,
    deleteReminder,
    startConversation,
    loadConversation,
    sendMessage,
    confirmNoteEdit,
    saveTranscript,
  };
}

export type SecretaryController = ReturnType<typeof useSecretary>;
