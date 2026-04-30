import { useTranslation } from "react-i18next";

export function ToolsHelp() {
  const { t } = useTranslation();

  return (
    <div className="tool-group tool-help">
      <h4>🔄 {t("conversion")}</h4>
      <p>{t("toolHelpConversionIntro")}</p>
      <ul>
        <li><b>Base64</b> - {t("toolHelpBase64")}</li>
        <li><b>Hex ↔ ASCII</b> - {t("toolHelpHexAscii")}</li>
        <li><b>URL / HTML</b> - {t("toolHelpUrlHtml")}</li>
        <li><b>{t("fromBase")}</b> - {t("toolHelpNumberBase")}</li>
        <li><b>Timestamp</b> - {t("toolHelpTimestamp")}</li>
        <li><b>JWT</b> - {t("toolHelpJwt")}</li>
      </ul>

      <h4>🔏 {t("checksum")}</h4>
      <p>{t("toolHelpChecksumIntro")}</p>
      <ul>
        <li>{t("toolHelpSha")}</li>
        <li>{t("toolHelpMd5")}</li>
      </ul>

      <h4>✨ {t("generation")}</h4>
      <p>{t("toolHelpGenerationIntro")}</p>
      <ul>
        <li><b>UUID v4</b> - {t("toolHelpUuid")}</li>
        <li><b>{t("randomString")}</b> - {t("toolHelpRandomString")}</li>
        <li><b>{t("password")}</b> - {t("toolHelpPassword")}</li>
      </ul>

      <h4>🔐 {t("encryption")}</h4>
      <p>{t("toolHelpEncryptionIntro")}</p>
      <ul>
        <li><b>AES-GCM / AES-CBC</b> - {t("toolHelpAes")}</li>
        <li><b>ROT13 / Atbash / Caesar</b> - {t("toolHelpClassic")}</li>
      </ul>

      <h4>🗄️ {t("database")}</h4>
      <p>{t("toolHelpDatabaseIntro")}</p>
      <ul>
        <li>{t("toolHelpDatabaseReadonly")}</li>
        <li>{t("toolHelpDatabaseLimit")}</li>
      </ul>

      <h4 style={{ color: "var(--warning)" }}>{t("privacy")}</h4>
      <p>{t("toolHelpPrivacyBody")}</p>
    </div>
  );
}
