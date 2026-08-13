import { useEffect, useMemo, useState } from "react";
import { GameIcon } from "../components/GameIcon";
import { useI18n } from "../i18n";
import * as api from "../services/api";
import type { GameSummary } from "../types";

interface Props {
  onPick: (game: GameSummary) => void;
}

export function GameSelect({ onPick }: Props) {
  const { t } = useI18n();
  const [games, setGames] = useState<GameSummary[] | null>(null);
  const [query, setQuery] = useState("");
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    api.listGames().then(setGames).catch((e) => setError(api.errorMessage(e, t)));
  }, []);

  const shown = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q || !games) return games ?? [];
    return games.filter((g) => g.name.toLowerCase().includes(q));
  }, [games, query]);

  if (error) return <div className="notice error">{error}</div>;
  if (!games) return <div className="empty">{t("common.loading")}</div>;

  return (
    <>
      <input
        className="search"
        type="text"
        placeholder={t("games.search")}
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        autoFocus
      />

      {shown.length > 0 && (
        <div className="game-grid">
          {shown.map((g) => (
            // The description is a tooltip rather than a caption: at cover size
            // a sentence under every tile turns the grid into a wall of text,
            // and the artwork is already doing the work of telling them apart.
            <button
              key={g.id}
              className="game-tile"
              onClick={() => onPick(g)}
              title={g.description || undefined}
            >
              <GameIcon game={g} shape="cover" />
              <span className="game-tile-name">{g.name}</span>
            </button>
          ))}
        </div>
      )}

      {shown.length === 0 && (
        <div className="empty">
          {games.length === 0 ? (
            <>
              {t("games.noneInstalled")}
              <br />
              {t("games.addPlugin")}
            </>
          ) : (
            t("games.noMatch", { query })
          )}
        </div>
      )}

      {games.length > 0 && shown.length > 0 && (
        <div className="empty" style={{ paddingTop: 24 }}>
          {t("games.moreSoon")}
        </div>
      )}
    </>
  );
}
