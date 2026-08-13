/**
 * The reference language. Every key lives here; other locales override what
 * they have translated and fall through to these strings for the rest.
 *
 * Keys ending `_one` / `_other` are plural forms, chosen via `Intl.PluralRules`.
 * A language with more categories (Russian, Polish, Ukrainian) may add
 * `_few` and `_many`.
 */
export const en = {
  // Header and navigation
  "app.title": "Universal Save Editor",
  "app.pickGame": "Pick a game to get started",
  "app.chooseSave": "Choose which save to edit",
  "app.backups": "Backups",
  "app.backupsSub": "Put any save back the way it was",
  "app.settings": "Settings",
  "app.back": "Back",

  "common.loading": "Loading…",
  "common.cancel": "Cancel",

  // Game selection
  "games.search": "Search games…",
  "games.noneInstalled": "No game plugins are installed.",
  "games.addPlugin": "Add one to your plugins folder — see Settings.",
  "games.noMatch": "No game matches “{query}”.",
  "games.moreSoon":
    "More games coming soon — each one is a plugin, and anyone can write one.",

  // Save selection
  "saves.looking": "Looking for save files…",
  "saves.none": "No {game} saves were found on this computer.",
  "saves.playOnce": "Play the game once so it creates a save, then come back.",
  // Bare forms are the last-resort fallback: `t` tries `key_<category>` first,
  // so these only appear if a language declares no plural form at all.
  "saves.found": "Found {count} save files.",
  "saves.found_one":
    "Found {count} save file. Nothing is changed until you pick one and press Save.",
  "saves.found_other":
    "Found {count} save files. Nothing is changed until you pick one and press Save.",
  "saves.lastPlayed": "Last played {when}",

  // Editor
  "editor.opening": "Opening save…",
  "editor.save": "Save changes",
  "editor.saving": "Saving…",
  "editor.noChanges": "No changes yet",
  "editor.ready": "{count} changes ready",
  "editor.ready_one": "{count} change ready",
  "editor.ready_other": "{count} changes ready",
  "editor.needFixing": "{count} values need fixing",
  "editor.needFixing_one": "{count} value needs fixing",
  "editor.needFixing_other": "{count} values need fixing",
  "editor.savedNothing": "Nothing needed changing — your save is untouched.",
  "editor.saved": "Saved. {count} values updated, and a backup of the original was created first.",
  "editor.saved_one":
    "Saved. {count} value updated, and a backup of the original was created first.",
  "editor.saved_other":
    "Saved. {count} values updated, and a backup of the original was created first.",
  "editor.discardConfirm":
    "You have unsaved changes. Leave without saving them?",
  "editor.backupNote":
    "A backup is created before your save is modified. You can put it back at any time from Backups.",
  "editor.notPresent": "Not present in this save file.",
  "editor.nothingHere": "Nothing here in this save.",
  "editor.notInList": "{value} (not in list)",

  // Presets and editor navigation
  "presets.title": "Quick actions",
  "presets.help": "Each one is an ordinary set of changes — backed up, and checked the same way.",
  "editor.allSections": "All",
  "editor.searchFields": "Search fields…",
  "editor.revert": "Undo this change",

  // Searchable dropdown
  "combo.search": "Type to filter…",
  "combo.noMatch": "Nothing matches “{query}”.",
  "combo.more": "{count} more — keep typing to narrow it down.",

  // Adding and removing list rows
  "list.add": "Add",
  "list.remove": "Remove",
  "list.saveFirst": "Save your changes first.",
  "list.added": "Added. A backup was taken first.",
  "list.removed": "Removed. A backup was taken first.",

  // Backups
  "backups.none": "No backups yet.",
  "backups.autoNote": "One is created automatically every time you save a change.",
  "backups.restore": "Restore",
  "backups.delete": "Delete",
  "backups.confirm": "Replace the current save?",
  "backups.yesRestore": "Yes, restore",
  "backups.restored":
    "Restored. Your save is back to how it was, and the version it replaced was backed up too.",

  // The game's own backup copies
  "recovery.title": "The game's own backups",
  "recovery.help":
    "Copies this game made for itself, sitting beside your save. Only read, never written. Restoring one backs up what it replaces.",
  "recovery.compare": "Compare",
  "recovery.use": "Restore this",
  "recovery.identical": "Identical to your current save.",
  "recovery.andMore": "…and {count} more.",

  // Settings
  "settings.appearance": "Appearance",
  "settings.theme": "Theme",
  "settings.themeSystem": "Match the system",
  "settings.themeLight": "Light",
  "settings.themeDark": "Dark",
  "settings.language": "Language",
  "settings.languageDesc": "Also used for the labels a game plugin provides.",
  "settings.backupsDesc": "Every backup this app has ever taken lives here.",
  "settings.openFolder": "Open folder",
  "settings.plugins": "Plugins",
  "settings.pluginsDesc":
    "One folder per game. Drop a plugin into any of these and press Reload.",
  "settings.reload": "Reload plugins",
  "settings.reloaded": "{count} games available.",
  "settings.reloaded_one": "{count} game available.",
  "settings.reloaded_other": "{count} games available.",
  "settings.failedPlugins": "Plugins that failed to load",
  "settings.cantOpenFolder": "That folder could not be opened.",
  "settings.about": "About",
  "settings.aboutText":
    "Universal Save Editor {version} — edit your offline game saves without touching raw save files. Everything runs on this computer; nothing is sent anywhere.",

  // Confirmation before writing values outside the safe range
  "confirm.title": "Are you sure?",
  "confirm.intro": "{count} values are outside the range this plugin considers safe.",
  "confirm.intro_one":
    "{count} value is outside the range this plugin considers safe.",
  "confirm.intro_other":
    "{count} values are outside the range this plugin considers safe.",
  "confirm.risk":
    "The game may behave oddly, refuse the save, or crash. A backup is still taken first, so you can undo this.",
  "confirm.suggestedMax": "suggested maximum {limit}",
  "confirm.suggestedMin": "suggested minimum {limit}",
  "confirm.go": "Save anyway",

  // Change summary
  "changes.title": "What will change",
  "changes.arrow": "→",

  // Warnings shown above the editor
  "banner.gameRunning":
    "{game} is running right now. It may write over your changes when it closes.",
  "banner.cloud":
    "Steam Cloud syncs this folder. If the cloud copy is newer, it can undo your edit.",
  "banner.staleReload": "This save was changed on disk. Reload to see it.",
  "banner.reload": "Reload",

  // Field-level validation, shown while typing
  "field.enterValue": "Enter a value.",
  "field.enterNumber": "Enter a number.",
  "field.wholeNumber": "Enter a whole number, without a decimal point.",
  "field.tooSmall": "Below the safe range (suggested minimum {limit}).",
  "field.tooLarge": "Above the safe range (suggested maximum {limit}).",
  "field.tooLong": "Must be at most {limit} characters.",

  // Errors reported by the backend, looked up by code
  "error.pluginNotFound": "The plugin “{id}” was not found.",
  "error.pluginLoad": "The plugin folder could not be read: {detail}",
  "error.saveMissing": "This save file no longer exists on disk.",
  "error.saveRead": "This save file could not be read: {detail}",
  "error.saveFormat": "This does not look like a {game} save file.",
  "error.validation": "“{field}” {reason}",
  "error.fieldRule": "“{field}” {reason}",
  "error.unknownField": "“{field}” is not an editable field in this save.",
  "error.pathNotAllowed": "That file is outside the folders this plugin manages.",
  "error.backupFailed":
    "The backup could not be created, so your save was left untouched: {detail}",
  "error.backupNotFound": "That backup could not be found.",
  "error.writeFailed":
    "The save could not be written: {detail} Your original save is unchanged.",
  "error.saveChangedOnDisk":
    "The game changed this save after you opened it. Reload before saving, or your edit would undo what the game just wrote.",
  "error.constraint": "{message}",
  "error.needsConfirmation": "Some values are outside the safe range.",
  "error.listNotEditable": "“{list}” cannot have rows added or removed.",
  "error.listFull": "“{list}” cannot hold more than {max} entries.",
  "error.listAtMinimum": "“{list}” must keep at least {min} entries.",
  "error.io": "{detail}",
  "error.unknown": "Something went wrong.",

  // Reasons a value was rejected, substituted into error.fieldRule
  "rule.notWholeNumber": "must be a whole number.",
  "rule.hasDecimalPoint": "must be a whole number, without a decimal point.",
  "rule.notANumber": "must be a number.",
  "rule.notText": "must be text.",
  "rule.notABoolean": "must be on or off.",
  "rule.notAnOption": "is not one of the available options.",
  "rule.tooSmall": "cannot be lower than {limit}.",
  "rule.tooLarge": "cannot be higher than {limit}.",
  "rule.tooLong": "must be at most {limit} characters.",
  "rule.tooLargeForGame": "is too large for this game.",
  "rule.notPresent": "is not present in this save file.",
};
