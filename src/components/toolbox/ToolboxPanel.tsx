import { useRef, useState, KeyboardEvent } from "react";
import { ConversionTools } from "./ConversionTools";
import { ChecksumTools } from "./ChecksumTools";
import { GenerationTools } from "./GenerationTools";
import { EncryptionTools } from "./EncryptionTools";
import { ToolsHelp } from "./ToolsHelp";

type InnerTab = "conversion" | "checksum" | "generation" | "encryption" | "help";

const TABS: { key: InnerTab; icon: string; label: string }[] = [
  { key: "conversion", icon: "🔄", label: "Conversion" },
  { key: "checksum", icon: "🔏", label: "Checksum" },
  { key: "generation", icon: "✨", label: "Generation" },
  { key: "encryption", icon: "🔐", label: "Encryption" },
  { key: "help", icon: "❔", label: "Help" },
];

export function ToolboxPanel() {
  const [active, setActive] = useState<InnerTab>("conversion");
  const btnRefs = useRef<Array<HTMLButtonElement | null>>([]);

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
        aria-label="Toolbox categories"
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
            <span className="toolbox-tab-label">{tab.label}</span>
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
        <div style={{ display: active === "help" ? "block" : "none" }}>
          <ToolsHelp />
        </div>
      </div>
    </div>
  );
}
