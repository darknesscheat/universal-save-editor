import { useEffect, useState } from "react";
import { useI18n } from "../i18n";
import * as api from "../services/api";
import type { GameSummary, SaveSummary } from "../types";

interface Props {
  game: GameSummary;
  onPick: (save: SaveSummary) => void;
}

export function SaveSelect({ game, onPick }: Props) {
  const { t, tag } = useI18n();
  const [saves, setSaves] = useState<SaveSummary[] | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setSaves(null);
    setError(null);
    api
      .listSaves(game.id, tag)
      .then(setSaves)
      .catch((e) => setError(api.errorMessage(e, t)));
  }, [game.id, tag]);

  if (error) return <div className="notice error">{error}</div>;
  if (!saves) return <div className="empty">{t("saves.looking")}</div>;

  if (saves.length === 0) {
    return (
      <div className="empty">
        {t("saves.none", { game: game.name })}
        <br />
        {t("saves.playOnce")}
      </div>
    );
  }

  return (
    <>
      <div className="notice info">{t("saves.found", { count: saves.length })}</div>

      {saves.map((s) => (
        <button key={s.path} className="card" onClick={() => onPick(s)}>
          <div className="title">{s.title}</div>
          <div className="meta">
            {s.subtitle && <>{s.subtitle} · </>}
            {t("saves.lastPlayed", { when: s.modified })}
          </div>
        </button>
      ))}
    </>
  );
}
