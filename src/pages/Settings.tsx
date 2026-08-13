import { useEffect, useState } from "react";
import { revealItemInDir } from "@tauri-apps/plugin-opener";
import { LANGUAGES, useI18n } from "../i18n";
import * as api from "../services/api";
import type { Theme } from "../theme";
import type { AppInfo } from "../types";

interface Props {
  theme: Theme;
  onThemeChange: (theme: Theme) => void;
  onPluginsReloaded: () => void;
}

export function Settings({ theme, onThemeChange, onPluginsReloaded }: Props) {
  const { t, tag, setLanguage } = useI18n();
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [status, setStatus] = useState<string | null>(null);

  const load = () => api.appInfo().then(setInfo);
  useEffect(() => {
    load();
  }, []);

  const reload = async () => {
    const games = await api.reloadPlugins();
    await load();
    onPluginsReloaded();
    setStatus(t("settings.reloaded", { count: games.length }));
  };

  const reveal = async (path: string) => {
    try {
      await revealItemInDir(path);
    } catch {
      setStatus(t("settings.cantOpenFolder"));
    }
  };

  if (!info) return <div className="empty">{t("common.loading")}</div>;

  return (
    <>
      {status && <div className="notice info">{status}</div>}

      <section className="group">
        <h2>{t("settings.language")}</h2>
        <p className="desc">{t("settings.languageDesc")}</p>
        <div className="field">
          <label htmlFor="language">{t("settings.language")}</label>
          <select
            id="language"
            value={tag}
            onChange={(e) => setLanguage(e.target.value)}
          >
            {/* Each language is listed in its own words, so someone who cannot
                read the current one can still find theirs. */}
            {LANGUAGES.map((l) => (
              <option key={l.tag} value={l.tag}>
                {l.name}
              </option>
            ))}
          </select>
        </div>
      </section>

      <section className="group">
        <h2>{t("settings.appearance")}</h2>
        <div className="field">
          <label htmlFor="theme">{t("settings.theme")}</label>
          <select
            id="theme"
            value={theme}
            onChange={(e) => onThemeChange(e.target.value as Theme)}
          >
            <option value="system">{t("settings.themeSystem")}</option>
            <option value="light">{t("settings.themeLight")}</option>
            <option value="dark">{t("settings.themeDark")}</option>
          </select>
        </div>
      </section>

      <section className="group">
        <h2>{t("app.backups")}</h2>
        <p className="desc">{t("settings.backupsDesc")}</p>
        <div className="card">
          <div className="row">
            <div className="path" style={{ flex: 1 }}>
              {info.backupFolder}
            </div>
            <button className="small" onClick={() => reveal(info.backupFolder)}>
              {t("settings.openFolder")}
            </button>
          </div>
        </div>
      </section>

      <section className="group">
        <h2>{t("settings.plugins")}</h2>
        <p className="desc">{t("settings.pluginsDesc")}</p>
        {info.pluginFolders.map((f) => (
          <div className="card" key={f}>
            <div className="row">
              <div className="path" style={{ flex: 1 }}>
                {f}
              </div>
              <button className="small" onClick={() => reveal(f)}>
                {t("settings.openFolder")}
              </button>
            </div>
          </div>
        ))}
        <button className="small" style={{ marginTop: 4 }} onClick={reload}>
          {t("settings.reload")}
        </button>
      </section>

      {info.pluginProblems.length > 0 && (
        <section className="group">
          <h2>{t("settings.failedPlugins")}</h2>
          {info.pluginProblems.map((p) => (
            <div className="notice error" key={p.path}>
              <div className="path" style={{ color: "inherit" }}>
                {p.path}
              </div>
              {/* Plugin load failures come from the manifest parser and stay in
                  English, they are aimed at whoever wrote the plugin. */}
              <div style={{ marginTop: 4 }}>{p.reason}</div>
            </div>
          ))}
        </section>
      )}

      <section className="group">
        <h2>{t("settings.about")}</h2>
        <p className="desc">{t("settings.aboutText", { version: info.version })}</p>
      </section>
    </>
  );
}
