/**
 * Mirrors the structs in `src-tauri/src/core/model.rs`.
 * Keep the two in step: the Rust side is the source of truth.
 */

export type FieldType = "integer" | "number" | "text" | "boolean" | "choice";

/** Any JSON value a save field can hold. */
export type FieldValue = string | number | boolean | null;

export interface GameSummary {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  /** A `data:` URI, or null when the app should draw its own tile. */
  icon: string | null;
}

/** Identifies the revision of a file, so a write can notice the game got there first. */
export interface SaveStamp {
  modifiedMs: number;
  sizeBytes: number;
}

/** A value outside the range the plugin calls safe. Not an error. */
export interface Warning {
  pointer: string;
  field: string;
  /** `rule.tooSmall` or `rule.tooLarge`. */
  rule: string;
  limit: string;
  value: string;
}

/** One pending change, for the summary and the confirmation dialog. */
export interface Change {
  pointer: string;
  label: string;
  before: unknown;
  after: unknown;
}

export interface SaveSummary {
  path: string;
  title: string;
  subtitle: string;
  modified: string;
  sizeBytes: number;
}

export interface Choice {
  value: FieldValue;
  label: string;
}

export interface FieldView {
  id: string;
  label: string;
  help: string;
  /** JSON pointer, echoed back when submitting an edit. */
  pointer: string;
  type: FieldType;
  value: FieldValue;
  min: number | null;
  max: number | null;
  maxLength: number | null;
  options: Choice[];
  readOnly: boolean;
  /** The pointer does not resolve in this particular save. */
  missing: boolean;
}

export interface ListItemView {
  label: string;
  /** Position in the underlying array, what a delete has to name. */
  index: number;
  fields: FieldView[];
}

export interface ListView {

  id: string;
  label: string;
  description: string;
  items: ListItemView[];
  /** Games define some lists themselves, so not every one may grow or shrink. */
  allowAdd: boolean;
  allowRemove: boolean;
  bulkActions: BulkActionView[];
}

/** Sets the same field on every row of a list at once. */
export interface BulkActionView {
  id: string;
  label: string;
  /** Which field of each row to write; null for object-backed lists. */
  field: string | null;
  value: FieldValue;
}

export interface GroupView {
  id: string;
  label: string;
  description: string;
  /** Set when the section does not apply, shown instead of hiding it. */
  absentReason: string | null;
  fields: FieldView[];
  lists: ListView[];
}

export interface EditorDocument {
  gameId: string;
  gameName: string;
  savePath: string;
  groups: GroupView[];
  presets: PresetView[];
  stamp: SaveStamp;
  /** This game's processes found running. Empty is the normal case. */
  gameRunning: string[];
  cloudSynced: boolean;
}

/** A one-click set of edits declared by the plugin. */
export interface PresetView {
  id: string;
  label: string;
  description: string;
}

export interface Edit {
  pointer: string;
  value: FieldValue;
}

export interface WriteReport {
  backupId: string;
  changedFields: number;
  savePath: string;
  stamp: SaveStamp;
}

/** A copy of a save that the game itself made. */
export interface RecoveryFile {
  path: string;
  name: string;
  created: string;
  sizeBytes: number;
}

/** One field that differs between two versions of a save. */
export interface FieldChange {
  pointer: string;
  label: string;
  before: unknown;
  after: unknown;
}

export interface BackupEntry {
  id: string;
  gameId: string;
  originalPath: string;
  created: string;
  sizeBytes: number;
}

export interface PluginProblem {
  path: string;
  reason: string;
}

export interface AppInfo {
  version: string;
  backupFolder: string;
  pluginFolders: string[];
  pluginProblems: PluginProblem[];
}
