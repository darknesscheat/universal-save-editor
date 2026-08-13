import { invoke } from "@tauri-apps/api/core";
import type { Translator } from "../i18n";
import type {
  AppInfo,
  BackupEntry,
  Edit,
  EditorDocument,
  FieldChange,
  GameSummary,
  RecoveryFile,
  SaveStamp,
  SaveSummary,
  Warning,
  WriteReport,
} from "../types";

/**
 * The only place the frontend talks to Rust.
 *
 * `locale` is passed to the commands that render plugin-supplied text, so a
 * field labelled "Money" in the manifest can come back as "Para".
 */

export const appInfo = () => invoke<AppInfo>("app_info");

export const listGames = () => invoke<GameSummary[]>("list_games");

export const reloadPlugins = () => invoke<GameSummary[]>("reload_plugins");

export const listSaves = (gameId: string, locale: string) =>
  invoke<SaveSummary[]>("list_saves", { gameId, locale });

export const openSave = (gameId: string, path: string, locale: string) =>
  invoke<EditorDocument>("open_save", { gameId, path, locale });

/**
 * `expected` is the revision the editor read; the backend refuses the write if
 * the game has rewritten the file since. `confirm` is false on the first
 * attempt, see {@link confirmationNeeded}.
 */
export const writeSave = (
  gameId: string,
  path: string,
  edits: Edit[],
  expected: SaveStamp,
  confirm: boolean,
) => invoke<WriteReport>("write_save", { gameId, path, edits, expected, confirm });

/**
 * Run a preset.
 *
 * It becomes ordinary edits on the backend, so it can come back asking for
 * confirmation exactly like a hand-typed value.
 */
export const applyPreset = (
  gameId: string,
  path: string,
  presetId: string,
  expected: SaveStamp,
  confirm: boolean,
) => invoke<WriteReport>("apply_preset", { gameId, path, presetId, expected, confirm });

/**
 * Add or delete one row of a list.
 *
 * Written immediately rather than batched with field edits: inserting or
 * deleting renumbers everything after it, and a pointer like
 * `/player/loadout/2/rarity` would then address the wrong row.
 */
export const changeListRow = (
  gameId: string,
  path: string,
  listId: string,
  change: "add" | "remove",
  index: number,
  expected: SaveStamp,
) =>
  invoke<WriteReport>("change_list_row", {
    gameId,
    path,
    listId,
    change,
    index,
    expected,
  });

/**
 * Pictures for the items in this game's dropdowns, read from the player's own
 * installed copy. Empty when the game is not installed, the editor simply
 * shows names, as it did before.
 */
export const itemIcons = (gameId: string) =>
  invoke<Record<string, string>>("item_icons", { gameId });

/** Copies of this save that the game made: its own `.bak` and quarantine files. */
export const listRecoveryFiles = (gameId: string, path: string) =>
  invoke<RecoveryFile[]>("list_recovery_files", { gameId, path });

/** What restoring `sourcePath` over `savePath` would change, field by field. */
export const previewRestore = (
  gameId: string,
  savePath: string,
  sourcePath: string,
  locale: string,
) => invoke<FieldChange[]>("preview_restore", { gameId, savePath, sourcePath, locale });

export const restoreRecoveryFile = (gameId: string, savePath: string, sourcePath: string) =>
  invoke<WriteReport>("restore_recovery_file", { gameId, savePath, sourcePath });

export const listBackups = (gameId?: string) =>
  invoke<BackupEntry[]>("list_backups", { gameId: gameId ?? null });

export const restoreBackup = (backupId: string) =>
  invoke<string>("restore_backup", { backupId });

export const deleteBackup = (backupId: string) =>
  invoke<void>("delete_backup", { backupId });

/** What Rust sends when a command fails. */
interface AppError {
  code: string;
  message: string;
  params: Record<string, unknown>;
}

function isAppError(e: unknown): e is AppError {
  return (
    typeof e === "object" &&
    e !== null &&
    typeof (e as AppError).code === "string" &&
    typeof (e as AppError).message === "string"
  );
}

/**
 * Did the backend stop to ask rather than fail?
 *
 * `error.needsConfirmation` is not a failure: the edit is legal but some values
 * are outside the range the plugin calls safe. Returns the offending values so
 * the GUI can show them and offer to go ahead.
 */
export function confirmationNeeded(err: unknown): Warning[] | null {
  if (isAppError(err) && err.code === "error.needsConfirmation") {
    const warnings = err.params?.warnings;
    return Array.isArray(warnings) ? (warnings as Warning[]) : [];
  }
  return null;
}

/**
 * Turn whatever `invoke` rejected with into a sentence worth showing.
 *
 * Backend errors arrive as a stable code plus parameters, so they are
 * translated here. The English message travels with them and is used whenever
 * the current language has no entry for that code, a missing translation can
 * never leave the user staring at a blank box or a raw identifier.
 */
export function errorMessage(err: unknown, t: Translator): string {
  if (isAppError(err)) {
    const params = { ...err.params };

    // A field rule carries its own nested code, e.g. `rule.tooSmall`, which
    // becomes the `{reason}` inside the outer message.
    if (typeof params.rule === "string") {
      params.reason = t(params.rule as never, params);
    }

    const translated = t(err.code as never, params);
    // `t` returns the key itself when nothing matched.
    return translated === err.code ? err.message : translated;
  }

  if (typeof err === "string") return err;
  if (err instanceof Error) return err.message;
  return t("error.unknown");
}
