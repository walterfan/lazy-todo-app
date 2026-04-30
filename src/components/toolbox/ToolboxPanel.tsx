import { useRef, useState, KeyboardEvent } from "react";
import { ConversionTools } from "./ConversionTools";
import { ChecksumTools } from "./ChecksumTools";
import { GenerationTools } from "./GenerationTools";
import { EncryptionTools } from "./EncryptionTools";
import { DatabaseTools } from "./DatabaseTools";
import { ToolsHelp } from "./ToolsHelp";
import { useTranslation } from "react-i18next";

type InnerTab = "conversion" | "checksum" | "generation" | "encryption" | "database" | "help";

const TABS: { key: InnerTab; icon: string; labelKey: string }[] = [
  { key: "conversion", icon: "🔄", labelKey: "conversion" },
  { key: "checksum", icon: "🔏", labelKey: "checksum" },
  { key: "generation", icon: "✨", labelKey: "generation" },
  { key: "encryption", icon: "🔐", labelKey: "encryption" },
  { key: "database", icon: "🗄️", labelKey: "database" },
  { key: "help", icon: "❔", labelKey: "help" },
];

export function ToolboxPanel() {
  const [active, setActive] = useState<InnerTab>("conversion");
  const btnRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const { t } = useTranslation();

  const onKey = (e: KeyboardEvent<HTMLDivElement>) => {
    const currentIdx = TABS.findIndex((t) => t.key === active);
    if (currentIdx < 0) return;
    let nextIdx = currentIdx;
    if (e.key === "ArrowRight") nextIdx = (currentIdx + 1) % TABS.length;
    else if (e.key === "ArrowLeft") nextIdx = (currentIdx - 1 + TABS.length) % TABS.length;
    else if (e.key === "Home") nextIdx = 0;
    else if (e.key === "End") nextIdx = TABS.length - 1;
    else return;
    e.preventDefault();
    setActive(TABS[nextIdx].key);
    btnRefs.current[nextIdx]?.focus();
  };

  return (
    <div className="toolbox-panel">
      <div
        className="toolbox-tabs"
        role="tablist"
        aria-label={t("toolboxCategories")}
        onKeyDown={onKey}
      >
        {TABS.map((tab, i) => (
          <button
            key={tab.key}
            ref={(el) => {
              btnRefs.current[i] = el;
            }}
            role="tab"
            aria-selected={active === tab.key}
            tabIndex={active === tab.key ? 0 : -1}
            className={`toolbox-tab ${active === tab.key ? "active" : ""}`}
            onClick={() => setActive(tab.key)}
            type="button"
          >
            <span className="toolbox-tab-icon">{tab.icon}</span>
            <span className="toolbox-tab-label">{t(tab.labelKey)}</span>
          </button>
        ))}
      </div>

      <div className="toolbox-content">
        <div style={{ display: active === "conversion" ? "block" : "none" }}>
          <ConversionTools />
        </div>
        <div style={{ display: active === "checksum" ? "block" : "none" }}>
          <ChecksumTools />
        </div>
        <div style={{ display: active === "generation" ? "block" : "none" }}>
          <GenerationTools />
        </div>
        <div style={{ display: active === "encryption" ? "block" : "none" }}>
          <EncryptionTools />
        </div>
        <div style={{ display: active === "database" ? "block" : "none" }}>
          <DatabaseTools />
        </div>
        <div style={{ display: active === "help" ? "block" : "none" }}>
          <ToolsHelp />
        </div>
      </div>
    </div>
  );
}
