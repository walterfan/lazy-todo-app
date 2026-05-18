import { useState, useRef, useEffect } from "react";
import type { CreateNote, NoteColor, NoteTemplate } from "../types/note";
import type { Translator } from "../i18n";

const COLORS: NoteColor[] = ["yellow", "green", "blue", "pink", "purple", "orange"];

interface NoteEditorProps {
  onAdd: (input: CreateNote) => Promise<void>;
  autoFocus?: boolean;
  templates?: NoteTemplate[];
  onTemplatesRefresh?: () => Promise<void> | void;
  t: Translator;
}

export function NoteEditor({
  onAdd,
  autoFocus,
  templates = [],
  onTemplatesRefresh,
  t,
}: NoteEditorProps) {
  const [title, setTitle] = useState("");
  const [content, setContent] = useState("");
  const [color, setColor] = useState<NoteColor>("yellow");
  const [expanded, setExpanded] = useState(false);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>(
    templates[0]?.id ?? "",
  );
  const titleRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (autoFocus) {
      setExpanded(true);
      titleRef.current?.focus();
    }
  }, [autoFocus]);

  useEffect(() => {
    if (!selectedTemplateId && templates.length > 0) {
      setSelectedTemplateId(templates[0].id);
    }
  }, [templates, selectedTemplateId]);

  const applyTemplate = (template: NoteTemplate | undefined) => {
    if (!template) return;
    if (!title.trim()) {
      setTitle(template.title);
    }
    if (!content.trim()) {
      setContent(template.body);
    }
  };

  const handleExpand = () => {
    setExpanded(true);
    const template = templates.find((tpl) => tpl.id === selectedTemplateId) ?? templates[0];
    applyTemplate(template);
  };

  const handleTemplateChange = (nextId: string) => {
    setSelectedTemplateId(nextId);
    const template = templates.find((tpl) => tpl.id === nextId);
    applyTemplate(template);
  };

  const handleSubmit = async () => {
    if (!content.trim() && !title.trim()) return;
    await onAdd({ title: title || undefined, content: content || undefined, color });
    setTitle("");
    setContent("");
    setColor("yellow");
    setExpanded(false);
  };

  if (!expanded) {
    return (
      <div className="note-editor-collapsed" onClick={handleExpand}>
        <span className="note-editor-placeholder">{t("addNotePlaceholder")}</span>
      </div>
    );
  }

  return (
    <div className={`note-editor note-color-${color}`}>
      {templates.length > 0 && (
        <div className="note-editor-template-row">
          <label htmlFor="note-template-select">{t("noteTemplate")}</label>
          <select
            id="note-template-select"
            className="note-editor-template-select"
            value={selectedTemplateId}
            onChange={(e) => handleTemplateChange(e.target.value)}
            onFocus={() => onTemplatesRefresh?.()}
          >
            {templates.map((tpl) => (
              <option key={tpl.id} value={tpl.id}>
                {tpl.name}
              </option>
            ))}
          </select>
        </div>
      )}
      <input
        ref={titleRef}
        className="note-editor-title"
        placeholder={t("noteTitlePlaceholder")}
        value={title}
        onChange={(e) => setTitle(e.target.value)}
      />
      <textarea
        className="note-editor-content"
        placeholder={t("noteContentPlaceholder")}
        value={content}
        onChange={(e) => setContent(e.target.value)}
        rows={6}
      />
      <div className="note-editor-footer">
        <div className="color-picker">
          {COLORS.map((c) => (
            <button
              key={c}
              className={`color-dot color-dot-${c} ${c === color ? "active" : ""}`}
              onClick={() => setColor(c)}
              title={c}
            />
          ))}
        </div>
        <div className="note-editor-actions">
          <button className="btn-cancel" onClick={() => { setExpanded(false); setTitle(""); setContent(""); }}>
            {t("cancel")}
          </button>
          <button
            className="btn-add"
            onClick={handleSubmit}
            disabled={!content.trim() && !title.trim()}
          >
            {t("addNote")}
          </button>
        </div>
      </div>
    </div>
  );
}
