import { useState } from "react";
import type { GameSummary } from "../types";

/**
 * Deterministic colour for a game that has no artwork.
 *
 * The hue comes from the plugin id, so a given game always gets the same
 * colour: the tile becomes something you recognise at a glance rather than a
 * shape that moves around between launches.
 */
function hue(id: string): number {
  let h = 0;
  for (let i = 0; i < id.length; i++) {
    h = (h * 31 + id.charCodeAt(i)) >>> 0;
  }
  return h % 360;
}

/**
 * One or two letters to stand in for the game.
 *
 * Multi-word names give their initials ("Oaken Tower" → OT); a single word
 * gives its first two letters ("Pathogenic" → PA). `Intl.Segmenter` is not
 * needed here, but taking characters via the spread operator keeps names in
 * scripts outside the BMP from being cut in half.
 */
function monogram(name: string): string {
  const words = name.trim().split(/\s+/).filter(Boolean);
  if (words.length === 0) return "?";
  if (words.length === 1) return [...words[0]].slice(0, 2).join("").toUpperCase();
  return words
    .slice(0, 2)
    .map((w) => [...w][0])
    .join("")
    .toUpperCase();
}

interface Props {
  game: GameSummary;
  /**
   * `cover` is the portrait 2:3 tile the game grid is built from; `row` is the
   * small landscape thumbnail for a list. The shape is a caller's decision
   * because it depends on the layout, not on the game.
   */
  shape?: "cover" | "row";
}

export function GameIcon({ game, shape = "row" }: Props) {
  // A data URI that the browser cannot decode should fall back to the tile
  // rather than leaving a broken-image glyph in the list.
  const [failed, setFailed] = useState(false);
  const base = shape === "cover" ? "game-cover" : "game-icon";

  if (game.icon && !failed) {
    return (
      <img
        className={base}
        src={game.icon}
        alt=""
        onError={() => setFailed(true)}
      />
    );
  }

  const h = hue(game.id);
  return (
    <div
      className={`${base} fallback`}
      style={{
        background: `linear-gradient(140deg,
          hsl(${h} 45% 32%),
          hsl(${(h + 40) % 360} 40% 22%))`,
        color: `hsl(${h} 70% 82%)`,
      }}
      aria-hidden="true"
    >
      {monogram(game.name)}
    </div>
  );
}
