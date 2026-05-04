import { useEffect, useState } from "react";
import type { AgentsController } from "../hooks/useAgents";
import type { SecretaryController } from "../hooks/useSecretary";
import type { AgentDefinitionDetail } from "../types/agents";
import type { AppSettings, DisplayStyle } from "../types/settings";
import type { Translator } from "../i18n";

interface SettingsPanelProps {
  settings: AppSettings;
  dbPath: string;
  agents: AgentsController;
  secretary: SecretaryController;
  onUpdate: (patch: Partial<AppSettings>) => Promise<void>;
  t: Translator;
}

const DISPLAY_OPTIONS: { value: DisplayStyle; icon: string; labelKey: "list" | "grid" }[] = [
  { value: "list", icon: "📄", labelKey: "list" },
  { value: "grid", icon: "📊", labelKey: "grid" },
];

export function SettingsPanel({ settings, dbPath, agents, secretary, onUpdate, t }: SettingsPanelProps) {
  const [secretaryDraft, setSecretaryDraft] = useState({
    base_url: "",
    model: "",
    api_key: "",
  });
  const [agentDirectory, setAgentDirectory] = useState("");
  const [agentZipPath, setAgentZipPath] = useState("");
  const [agentDetail, setAgentDetail] = useState<AgentDefinitionDetail | null>(null);
  const [safeFileRootsText, setSafeFileRootsText] = useState("");
  const [cliDraft, setCliDraft] = useState({
    tool_id: "",
    display_name: "",
    executable: "",
    allowed_subcommands: "",
    argument_schema: '{\n  "type": "object",\n  "properties": {}\n}',
    working_directory: "",
    environment_allowlist: "",
    timeout_ms: 30000,
    output_limit: 12000,
    safety_class: "read",
    enabled: true,
  });
  const [agentSettingsStatus, setAgentSettingsStatus] = useState("");

  useEffect(() => {
    if (!secretary.settings) return;
    setSecretaryDraft({
      base_url: secretary.settings.saved.base_url,
      model: secretary.settings.saved.model,
      api_key: "",
    });
  }, [secretary.settings]);

  useEffect(() => {
    setAgentDirectory(agents.agentDirectorySettings?.agent_directory ?? "");
  }, [agents.agentDirectorySettings]);

  useEffect(() => {
    setSafeFileRootsText((agents.safeFileRootSettings?.safe_file_roots ?? []).join("\n"));
  }, [agents.safeFileRootSettings]);

  const saveSecretarySettings = async () => {
    await secretary.saveSettings({
      base_url: secretaryDraft.base_url,
      model: secretaryDraft.model,
      api_key: secretaryDraft.api_key || undefined,
      skill_folder: secretary.settings?.saved.skill_folder ?? "",
      conversation_folder: secretary.settings?.saved.conversation_folder ?? "",
      active_persona_id: secretary.activePersona?.id ?? null,
      active_profile_id: secretary.activeProfile?.id ?? null,
    });
  };

  const saveAgentDirectory = async () => {
    setAgentSettingsStatus("");
    try {
      await agents.saveAgentDirectorySettings({ agent_directory: agentDirectory });
      setAgentSettingsStatus(t("agentFolderSaved"));
    } catch (err) {
      setAgentSettingsStatus(String(err));
    }
  };

  const installAgentZip = async () => {
    setAgentSettingsStatus("");
    try {
      const agent = await agents.installAgentZip(agentZipPath);
      setAgentSettingsStatus(t("installedAgent", { name: agent.agent_name }));
      setAgentZipPath("");
    } catch (err) {
      setAgentSettingsStatus(String(err));
    }
  };

  const loadAgentDetail = async (agentId: string) => {
    setAgentSettingsStatus("");
    try {
      setAgentDetail(await agents.loadAgentDetail(agentId));
    } catch (err) {
      setAgentSettingsStatus(String(err));
    }
  };

  const saveSafeFileRoots = async () => {
    setAgentSettingsStatus("");
    const safe_file_roots = safeFileRootsText
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean);
    try {
      await agents.saveSafeFileRootSettings({ safe_file_roots });
      setAgentSettingsStatus(t("safeFileRootsSaved"));
    } catch (err) {
      setAgentSettingsStatus(String(err));
    }
  };

  const runSecretaryMigration = async () => {
    setAgentSettingsStatus("");
    try {
      const status = await agents.runSecretaryMigration();
      setAgentSettingsStatus(t("secretaryMigrationStatus", { status: status.status }));
    } catch (err) {
      setAgentSettingsStatus(String(err));
    }
  };

  const installExternalCliPresets = async () => {
    setAgentSettingsStatus("");
    try {
      const tools = await agents.installExternalCliPresets();
      setAgentSettingsStatus(t("externalCliPresetsChecked", { count: tools.length }));
    } catch (err) {
      setAgentSettingsStatus(String(err));
    }
  };

  const cliDraftInput = () => ({
    tool_id: cliDraft.tool_id || null,
    display_name: cliDraft.display_name,
    executable: cliDraft.executable,
    allowed_subcommands: cliDraft.allowed_subcommands.split(",").map((item) => item.trim()).filter(Boolean),
    argument_schema: JSON.parse(cliDraft.argument_schema),
    working_directory: cliDraft.working_directory,
    environment_allowlist: cliDraft.environment_allowlist.split(",").map((item) => item.trim()).filter(Boolean),
    timeout_ms: cliDraft.timeout_ms,
    output_limit: cliDraft.output_limit,
    safety_class: cliDraft.safety_class,
    enabled: cliDraft.enabled,
  });

  const saveExternalCliTool = async () => {
    setAgentSettingsStatus("");
    try {
      const tool = await agents.saveExternalCliTool(cliDraftInput());
      setAgentSettingsStatus(t("savedExternalCliTool", { name: tool.display_name }));
    } catch (err) {
      setAgentSettingsStatus(String(err));
    }
  };

  const testExternalCliTool = async () => {
    setAgentSettingsStatus("");
    try {
      const result = await agents.testExternalCliTool(cliDraftInput());
      setAgentSettingsStatus(result.message);
    } catch (err) {
      setAgentSettingsStatus(String(err));
    }
  };

  return (
    <div className="settings-panel">
      <section className="settings-section">
        <h3 className="settings-section-title">{t("display")}</h3>

        <div className="settings-row">
          <label>{t("language")}</label>
          <div className="settings-toggle-group">
            <button
              className={`settings-toggle ${settings.language === "en" ? "active" : ""}`}
              onClick={() => onUpdate({ language: "en" })}
            >
              {t("english")}
            </button>
            <button
              className={`settings-toggle ${settings.language === "zh" ? "active" : ""}`}
              onClick={() => onUpdate({ language: "zh" })}
            >
              {t("chinese")}
            </button>
          </div>
        </div>

        <div className="settings-row">
          <label>{t("pageSize")}</label>
          <input
            type="number"
            min={10}
            max={200}
            step={10}
            value={settings.page_size}
            onChange={(e) => onUpdate({ page_size: Number(e.target.value) || 10 })}
          />
        </div>

        <div className="settings-row">
          <label>{t("notesPerPage")}</label>
          <input
            type="number"
            min={1}
            max={200}
            step={1}
            value={settings.note_page_size}
            onChange={(e) => onUpdate({ note_page_size: Number(e.target.value) || 10 })}
          />
        </div>

        <div className="settings-row">
          <label>{t("todoDisplay")}</label>
          <div className="settings-toggle-group">
            {DISPLAY_OPTIONS.map((opt) => (
              <button
                key={opt.value}
                className={`settings-toggle ${settings.todo_display === opt.value ? "active" : ""}`}
                onClick={() => onUpdate({ todo_display: opt.value })}
              >
                {opt.icon} {t(opt.labelKey)}
              </button>
            ))}
          </div>
        </div>

        <div className="settings-row">
          <label>{t("noteDisplay")}</label>
          <div className="settings-toggle-group">
            {DISPLAY_OPTIONS.map((opt) => (
              <button
                key={opt.value}
                className={`settings-toggle ${settings.note_display === opt.value ? "active" : ""}`}
                onClick={() => onUpdate({ note_display: opt.value })}
              >
                {opt.icon} {t(opt.labelKey)}
              </button>
            ))}
          </div>
        </div>
      </section>

      <section className="settings-section">
        <h3 className="settings-section-title">{t("notes")}</h3>

        <div className="settings-row">
          <label>{t("noteFolder")}</label>
          <input
            type="text"
            value={settings.note_folder}
            onChange={(e) => onUpdate({ note_folder: e.target.value })}
            placeholder={t("defaultValue")}
          />
        </div>

        <div className="settings-row settings-row-vertical">
          <label>{t("noteTemplate")}</label>
          <textarea
            value={settings.note_template}
            onChange={(e) => onUpdate({ note_template: e.target.value })}
            placeholder={t("noteTemplatePlaceholder")}
            rows={4}
          />
        </div>
      </section>

      <section className="settings-section">
        <h3 className="settings-section-title">{t("agentsLlm")}</h3>

        {secretary.loading ? (
          <div className="settings-value">{t("loadingAgentLlm")}</div>
        ) : (
          <>
            <div className="settings-row">
              <label>{t("baseUrl")}</label>
              <input
                type="text"
                value={secretaryDraft.base_url}
                onChange={(e) => setSecretaryDraft({ ...secretaryDraft, base_url: e.target.value })}
                placeholder="https://api.openai.com/v1"
              />
            </div>

            <div className="settings-row">
              <label>{t("model")}</label>
              <input
                type="text"
                value={secretaryDraft.model}
                onChange={(e) => setSecretaryDraft({ ...secretaryDraft, model: e.target.value })}
                placeholder="gpt-4.1-mini"
              />
            </div>

            <div className="settings-row">
              <label>{t("apiKey")}</label>
              <input
                type="password"
                value={secretaryDraft.api_key}
                onChange={(e) => setSecretaryDraft({ ...secretaryDraft, api_key: e.target.value })}
                placeholder={secretary.settings?.has_api_key ? "API key saved or from env" : "API key"}
              />
            </div>

            <div className="settings-row settings-row-vertical">
              <label>{t("environmentOverrides")}</label>
              <div className="settings-chip-row">
                {secretary.settings?.base_url_from_env && <span>{t("baseUrlFromEnv")}</span>}
                {secretary.settings?.model_from_env && <span>{t("modelFromEnv")}</span>}
                {secretary.settings?.api_key_from_env && <span>{t("keyFromEnv")}</span>}
                {!secretary.settings?.base_url_from_env && !secretary.settings?.model_from_env && !secretary.settings?.api_key_from_env && <span>{t("savedConfigActive")}</span>}
              </div>
            </div>

            <div className="settings-actions">
              <button className="settings-toggle active" onClick={() => void saveSecretarySettings()}>
                {t("saveLlm")}
              </button>
            </div>
          </>
        )}
      </section>

      <section className="settings-section">
        <h3 className="settings-section-title">{t("agentsContext")}</h3>

        {secretary.loading ? (
          <div className="settings-value">{t("loadingContext")}</div>
        ) : (
          <>
            <div className="settings-checkbox-list">
              <label><input type="checkbox" checked={secretary.selectedContext.include_todos} onChange={(e) => secretary.setSelectedContext({ ...secretary.selectedContext, include_todos: e.target.checked })} /> {t("todos")}</label>
              <label><input type="checkbox" checked={secretary.selectedContext.include_milestones} onChange={(e) => secretary.setSelectedContext({ ...secretary.selectedContext, include_milestones: e.target.checked })} /> {t("milestones")}</label>
              <label><input type="checkbox" checked={secretary.selectedContext.include_notes} onChange={(e) => secretary.setSelectedContext({ ...secretary.selectedContext, include_notes: e.target.checked })} /> {t("notes")}</label>
            </div>

            <div className="settings-row settings-row-vertical">
              <label>{t("stickyNotes")}</label>
              <div className="settings-checkbox-list">
                {secretary.appContext.notes.slice(0, 12).map((note) => (
                  <label key={note.id}>
                    <input
                      type="checkbox"
                      checked={secretary.selectedContext.note_ids.includes(note.id)}
                      onChange={(e) => {
                        const next = e.target.checked
                          ? [...secretary.selectedContext.note_ids, note.id]
                          : secretary.selectedContext.note_ids.filter((id) => id !== note.id);
                        secretary.setSelectedContext({ ...secretary.selectedContext, include_notes: next.length > 0 || secretary.selectedContext.include_notes, note_ids: next });
                      }}
                    />
                    {note.title || `Note #${note.id}`}
                  </label>
                ))}
                {secretary.appContext.notes.length === 0 && <span className="settings-value">{t("noNotesAvailable")}</span>}
              </div>
            </div>
          </>
        )}
      </section>

      <section className="settings-section">
        <h3 className="settings-section-title">{t("agentsFiles")}</h3>

        {agents.loading ? (
          <div className="settings-value">{t("loadingAgentSettings")}</div>
        ) : (
          <>
            <div className="settings-row settings-row-vertical">
              <label>{t("agentFolder")}</label>
              <input
                className="settings-path-input"
                type="text"
                value={agentDirectory}
                onChange={(e) => setAgentDirectory(e.target.value)}
                placeholder="/Users/walterfan/agents"
              />
            </div>

            <div className="settings-actions">
              <button className="settings-toggle active" onClick={() => void saveAgentDirectory()}>
                {t("saveAgentFolder")}
              </button>
              <button className="settings-toggle" onClick={() => void agents.refreshAgents()}>
                {t("refreshAgents")}
              </button>
            </div>

            <div className="settings-row settings-row-vertical">
              <label>{t("installAgentZip")}</label>
              <input
                className="settings-path-input"
                type="text"
                value={agentZipPath}
                onChange={(e) => setAgentZipPath(e.target.value)}
                placeholder="/Users/walterfan/Downloads/confucius_agent_v1.0.0.zip"
              />
            </div>

            <div className="settings-actions">
              <button className="settings-toggle" onClick={() => void installAgentZip()} disabled={!agentZipPath.trim()}>
                {t("installZip")}
              </button>
            </div>

            <div className="settings-row settings-row-vertical">
              <label>{t("installedAgents")}</label>
              <div className="settings-agent-list">
                {agents.agents.map((agent) => (
                  <div className="settings-agent-row" key={agent.agent_id}>
                    <div>
                      <strong>{agent.agent_name}</strong>
                      <span>
                        {agent.agent_id} · {agent.lifecycle_state} · {agent.enabled ? t("statusEnabled") : t("statusDisabled")} ·{" "}
                        {agent.bundled ? t("builtIn") : t("local")}
                      </span>
                      {agent.validation_diagnostics.length > 0 && (
                        <span>{agent.validation_diagnostics.map((item) => item.message).join("; ")}</span>
                      )}
                    </div>
                    <div className="settings-agent-actions">
                      <button className="settings-toggle" onClick={() => void loadAgentDetail(agent.agent_id)}>
                        {t("details")}
                      </button>
                      <button
                        className="settings-toggle"
                        onClick={() => void agents.setAgentEnabled(agent.agent_id, !agent.enabled)}
                        disabled={agent.lifecycle_state === "invalid"}
                      >
                        {agent.enabled ? t("disable") : t("enable")}
                      </button>
                      <button
                        className="settings-toggle"
                        onClick={() => void agents.uninstallAgent(agent.agent_id)}
                        disabled={agent.bundled}
                      >
                        {t("uninstall")}
                      </button>
                    </div>
                  </div>
                ))}
              </div>
            </div>

            {agentDetail && (
              <div className="settings-row settings-row-vertical">
                <label>{agentDetail.agent.agent_name} details</label>
                <div className="settings-agent-detail">
                  <span>{agentDetail.agent.path}</span>
                  <span>
                    {t("rag")} {agentDetail.agent.rag_enabled ? t("statusEnabled") : t("statusDisabled")} ·{" "}
                    {agentDetail.agent.has_rag_knowledge ? t("knowledgeFilePresent") : t("noKnowledgeFile")}
                  </span>
                  <pre>{agentDetail.readme || t("noReadmeContent")}</pre>
                </div>
              </div>
            )}

            <div className="settings-row settings-row-vertical">
              <label>{t("safeFileRoots")}</label>
              <textarea
                value={safeFileRootsText}
                onChange={(e) => setSafeFileRootsText(e.target.value)}
                placeholder={"/Users/walterfan/notes\n/Users/walterfan/projects/scratch"}
                rows={4}
              />
            </div>

            <div className="settings-actions">
              <button className="settings-toggle active" onClick={() => void saveSafeFileRoots()}>
                {t("saveSafeRoots")}
              </button>
              {agentSettingsStatus && <span className="settings-value settings-status">{agentSettingsStatus}</span>}
            </div>

            <div className="settings-row settings-row-vertical">
              <label>{t("secretaryMigration")}</label>
              <div className="settings-chip-row">
                <span>{agents.migrationStatus?.status ?? t("notStarted")}</span>
                {agents.migrationStatus?.updated_at && <span>{agents.migrationStatus.updated_at}</span>}
              </div>
              {agents.migrationStatus?.details && (
                <pre className="settings-json-preview">{agents.migrationStatus.details}</pre>
              )}
            </div>

            <div className="settings-actions">
              <button className="settings-toggle" onClick={() => void runSecretaryMigration()}>
                {t("runMigration")}
              </button>
            </div>

            <div className="settings-row settings-row-vertical">
              <label>{t("externalCliTools")}</label>
              <div className="settings-checkbox-list">
                {agents.externalCliTools.map((tool) => (
                  <div className="settings-list-row" key={tool.tool_id}>
                    <strong>{tool.display_name}</strong>
                    <span>
                      {tool.executable} · {tool.safety_class} · {tool.enabled ? t("statusEnabled") : t("statusDisabled")} ·{" "}
                      {tool.available ? t("available") : tool.availability_error || t("unavailable")}
                    </span>
                    <div className="settings-actions">
                      <button
                        className="settings-toggle"
                        onClick={() => void agents.setExternalCliToolEnabled(tool.tool_id, !tool.enabled)}
                      >
                        {tool.enabled ? t("disable") : t("enable")}
                      </button>
                      <button
                        className="settings-toggle"
                        onClick={() => {
                          setCliDraft({
                            tool_id: tool.tool_id,
                            display_name: tool.display_name,
                            executable: tool.executable,
                            allowed_subcommands: tool.allowed_subcommands.join(", "),
                            argument_schema: JSON.stringify(tool.argument_schema, null, 2),
                            working_directory: tool.working_directory,
                            environment_allowlist: tool.environment_allowlist.join(", "),
                            timeout_ms: tool.timeout_ms,
                            output_limit: tool.output_limit,
                            safety_class: tool.safety_class,
                            enabled: tool.enabled,
                          });
                        }}
                      >
                        {t("edit")}
                      </button>
                      <button className="settings-toggle" onClick={() => void agents.deleteExternalCliTool(tool.tool_id)}>
                        {t("delete")}
                      </button>
                    </div>
                  </div>
                ))}
                {agents.externalCliTools.length === 0 && <span className="settings-value">{t("noExternalCliTools")}</span>}
              </div>
            </div>

            <div className="settings-actions">
              <button className="settings-toggle" onClick={() => void installExternalCliPresets()}>
                {t("addCliPresets")}
              </button>
            </div>

            <div className="settings-row settings-row-vertical">
              <label>{t("registerExternalCli")}</label>
              <div className="settings-cli-grid">
                <input value={cliDraft.tool_id} onChange={(e) => setCliDraft({ ...cliDraft, tool_id: e.target.value })} placeholder={t("toolIdOptional")} />
                <input value={cliDraft.display_name} onChange={(e) => setCliDraft({ ...cliDraft, display_name: e.target.value })} placeholder={t("displayName")} />
                <input value={cliDraft.executable} onChange={(e) => setCliDraft({ ...cliDraft, executable: e.target.value })} placeholder={t("executableOrPath")} />
                <input value={cliDraft.allowed_subcommands} onChange={(e) => setCliDraft({ ...cliDraft, allowed_subcommands: e.target.value })} placeholder={t("allowedSubcommands")} />
                <input value={cliDraft.working_directory} onChange={(e) => setCliDraft({ ...cliDraft, working_directory: e.target.value })} placeholder={t("workingDirectoryOptional")} />
                <input value={cliDraft.environment_allowlist} onChange={(e) => setCliDraft({ ...cliDraft, environment_allowlist: e.target.value })} placeholder={t("envAllowlist")} />
                <input type="number" value={cliDraft.timeout_ms} onChange={(e) => setCliDraft({ ...cliDraft, timeout_ms: Number(e.target.value) })} />
                <input type="number" value={cliDraft.output_limit} onChange={(e) => setCliDraft({ ...cliDraft, output_limit: Number(e.target.value) })} />
                <select value={cliDraft.safety_class} onChange={(e) => setCliDraft({ ...cliDraft, safety_class: e.target.value })}>
                  <option value="read">{t("read")}</option>
                  <option value="write">{t("write")}</option>
                  <option value="networked">{t("networked")}</option>
                  <option value="sensitive">{t("sensitive")}</option>
                  <option value="destructive">{t("destructive")}</option>
                </select>
                <label className="settings-inline-check">
                  <input type="checkbox" checked={cliDraft.enabled} onChange={(e) => setCliDraft({ ...cliDraft, enabled: e.target.checked })} />
                  {t("enabled")}
                </label>
              </div>
              <textarea
                value={cliDraft.argument_schema}
                onChange={(e) => setCliDraft({ ...cliDraft, argument_schema: e.target.value })}
                rows={5}
              />
            </div>

            <div className="settings-actions">
              <button className="settings-toggle" onClick={() => void testExternalCliTool()} disabled={!cliDraft.executable.trim()}>
                {t("testCli")}
              </button>
              <button className="settings-toggle active" onClick={() => void saveExternalCliTool()} disabled={!cliDraft.executable.trim()}>
                {t("saveCli")}
              </button>
            </div>
          </>
        )}
      </section>

      <section className="settings-section">
        <h3 className="settings-section-title">{t("storage")}</h3>
        <div className="settings-row">
          <label>{t("databasePath")}</label>
          <span className="settings-value" title={dbPath}>{dbPath || "—"}</span>
        </div>
      </section>
    </div>
  );
}
