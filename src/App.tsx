import { useEffect, useMemo, useState } from "react";
import {
  I18nContext,
  createTranslator,
  detectLanguage,
  saveLanguage,
} from "./i18n";
import { Backups } from "./pages/Backups";
import { Editor } from "./pages/Editor";
import { GameSelect } from "./pages/GameSelect";
import { SaveSelect } from "./pages/SaveSelect";
import { Settings } from "./pages/Settings";
import { applyTheme, loadTheme, type Theme } from "./theme";
import type { GameSummary, SaveSummary } from "./types";
import "./styles.css";

/**
 * Four screens, one line of travel: game -> save -> editor.
 * Backups and Settings hang off the header rather than interrupting that path.
 */
type Screen = "games" | "saves" | "editor" | "backups" | "settings";

export default function App() {
  const [screen, setScreen] = useState<Screen>("games");
  const [game, setGame] = useState<GameSummary | null>(null);
  const [save, setSave] = useState<SaveSummary | null>(null);
  const [reloadKey, setReloadKey] = useState(0);
  const [tag, setTag] = useState(detectLanguage);
  /** True while the editor holds changes that have not been written. */
  const [dirty, setDirty] = useState(false);
  const [theme, setTheme] = useState<Theme>(loadTheme);

  useEffect(() => applyTheme(theme), [theme]);

  // Escape goes back. Ctrl+S is handled inside the editor, which is the only
  // screen that can save.
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      const typing =
        e.target instanceof HTMLInputElement || e.target instanceof HTMLSelectElement;
      if (e.key === "Escape" && !typing && screen !== "games") {
        e.preventDefault();
        back();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const i18n = useMemo(
    () => ({
      tag,
      t: createTranslator(tag),
      setLanguage: (next: string) => {
        saveLanguage(next);
        setTag(next);
        // The document language matters for hyphenation and for screen readers.
        document.documentElement.lang = next;
      },
    }),
    [tag],
  );

  const { t } = i18n;

  /**
   * Leaving the editor with unsaved work used to discard it without a word.
   * Ask once, and only when there is actually something to lose.
   */
  const leaveEditor = (go: () => void) => {
    if (!dirty || window.confirm(t("editor.discardConfirm"))) {
      setDirty(false);
      go();
    }
  };

  const back = () => {
    if (screen === "editor") leaveEditor(() => setScreen("saves"));
    else if (screen === "saves") {
      setGame(null);
      setScreen("games");
    } else setScreen(game && save ? "editor" : game ? "saves" : "games");
  };

  const heading = () => {
    switch (screen) {
      case "games":
        return { title: t("app.title"), sub: t("app.pickGame") };
      case "saves":
        return { title: game?.name ?? "", sub: t("app.chooseSave") };
      case "editor":
        return { title: game?.name ?? "", sub: save?.title ?? "" };
      case "backups":
        return { title: t("app.backups"), sub: t("app.backupsSub") };
      case "settings":
        return { title: t("app.settings"), sub: "" };
    }
  };

  const { title, sub } = heading();

  return (
    <I18nContext.Provider value={i18n}>
      <div className="app">
        <header className="header">
          {screen !== "games" && (
            <button className="ghost" onClick={back} title={t("app.back")}>
              ←
            </button>
          )}
          <div>
            <h1>{title}</h1>
            {sub && <p className="sub">{sub}</p>}
          </div>
          <div className="spacer" />
          {screen !== "backups" && (
            <button
              className="ghost"
              onClick={() => leaveEditor(() => setScreen("backups"))}
            >
              {t("app.backups")}
            </button>
          )}
          {screen !== "settings" && (
            <button
              className="ghost"
              onClick={() => leaveEditor(() => setScreen("settings"))}
            >
              {t("app.settings")}
            </button>
          )}
        </header>

        {screen === "games" && (
          <GameSelect
            key={reloadKey}
            onPick={(g) => {
              setGame(g);
              setSave(null);
              setScreen("saves");
            }}
          />
        )}

        {screen === "saves" && game && (
          <SaveSelect
            game={game}
            onPick={(s) => {
              setSave(s);
              setScreen("editor");
            }}
          />
        )}

        {screen === "editor" && game && save && (
          <Editor
            key={save.path}
            game={game}
            save={save}
            onDirtyChange={setDirty}
          />
        )}

        {screen === "backups" && <Backups gameId={game?.id} />}

        {screen === "settings" && (
          <Settings
            theme={theme}
            onThemeChange={setTheme}
            onPluginsReloaded={() => {
              setReloadKey((k) => k + 1);
              setGame(null);
              setSave(null);
            }}
          />
        )}
      </div>
    </I18nContext.Provider>
  );
}
