import type { LocaleStrings } from "..";

export const de: LocaleStrings = {
  "app.title": "Universal Save Editor",
  "app.pickGame": "Wähle ein Spiel, um zu beginnen",
  "app.chooseSave": "Wähle den Spielstand zum Bearbeiten",
  "app.backups": "Sicherungen",
  "app.backupsSub": "Jeden Spielstand wiederherstellen",
  "app.settings": "Einstellungen",
  "app.back": "Zurück",

  "common.loading": "Wird geladen…",
  "common.cancel": "Abbrechen",

  "games.search": "Spiele suchen…",
  "games.noneInstalled": "Es sind keine Spiel-Plugins installiert.",
  "games.addPlugin": "Lege eines in deinen Plugin-Ordner — siehe Einstellungen.",
  "games.noMatch": "Kein Spiel passt zu „{query}“.",
  "games.moreSoon":
    "Weitere Spiele folgen — jedes ist ein Plugin, und jeder kann eines schreiben.",

  "saves.looking": "Suche nach Spielständen…",
  "saves.none": "Auf diesem Computer wurden keine {game}-Spielstände gefunden.",
  "saves.playOnce":
    "Spiele das Spiel einmal, damit ein Spielstand entsteht, und komm dann zurück.",
  "saves.found_one":
    "{count} Spielstand gefunden. Es ändert sich nichts, bis du einen auswählst und speicherst.",
  "saves.found_other":
    "{count} Spielstände gefunden. Es ändert sich nichts, bis du einen auswählst und speicherst.",
  "saves.lastPlayed": "Zuletzt gespielt {when}",

  "editor.opening": "Spielstand wird geöffnet…",
  "editor.save": "Änderungen speichern",
  "editor.saving": "Wird gespeichert…",
  "editor.noChanges": "Noch keine Änderungen",
  "editor.ready_one": "{count} Änderung bereit",
  "editor.ready_other": "{count} Änderungen bereit",
  "editor.needFixing_one": "{count} Wert muss korrigiert werden",
  "editor.needFixing_other": "{count} Werte müssen korrigiert werden",
  "editor.savedNothing":
    "Es gab nichts zu ändern — dein Spielstand blieb unberührt.",
  "editor.saved_one":
    "Gespeichert. {count} Wert aktualisiert; zuvor wurde eine Sicherung des Originals angelegt.",
  "editor.saved_other":
    "Gespeichert. {count} Werte aktualisiert; zuvor wurde eine Sicherung des Originals angelegt.",
  "editor.backupNote":
    "Vor jeder Änderung wird eine Sicherung angelegt. Du kannst sie jederzeit unter Sicherungen zurückholen.",
  "editor.notPresent": "In dieser Speicherdatei nicht vorhanden.",
  "editor.nothingHere": "In diesem Spielstand ist hier nichts.",
  "editor.notInList": "{value} (nicht in der Liste)",

  "backups.none": "Noch keine Sicherungen.",
  "backups.autoNote":
    "Bei jedem Speichern einer Änderung wird automatisch eine angelegt.",
  "backups.restore": "Wiederherstellen",
  "backups.delete": "Löschen",
  "backups.confirm": "Den aktuellen Spielstand ersetzen?",
  "backups.yesRestore": "Ja, wiederherstellen",
  "backups.restored":
    "Wiederhergestellt. Dein Spielstand ist wieder wie zuvor, und die ersetzte Fassung wurde ebenfalls gesichert.",

  "settings.language": "Sprache",
  "settings.languageDesc":
    "Gilt auch für die Bezeichnungen, die ein Spiel-Plugin liefert.",
  "settings.backupsDesc":
    "Hier liegen alle Sicherungen, die diese App je angelegt hat.",
  "settings.openFolder": "Ordner öffnen",
  "settings.plugins": "Plugins",
  "settings.pluginsDesc":
    "Ein Ordner pro Spiel. Lege ein Plugin in einen davon und drücke Neu laden.",
  "settings.reload": "Plugins neu laden",
  "settings.reloaded_one": "{count} Spiel verfügbar.",
  "settings.reloaded_other": "{count} Spiele verfügbar.",
  "settings.failedPlugins": "Plugins, die nicht geladen werden konnten",
  "settings.cantOpenFolder": "Dieser Ordner konnte nicht geöffnet werden.",
  "settings.about": "Über",
  "settings.aboutText":
    "Universal Save Editor {version} — bearbeite deine Offline-Spielstände, ohne die Rohdateien anzufassen. Alles läuft auf diesem Computer; nichts wird irgendwohin gesendet.",

  "field.enterValue": "Gib einen Wert ein.",
  "field.enterNumber": "Gib eine Zahl ein.",
  "field.wholeNumber": "Gib eine ganze Zahl ohne Komma ein.",
  "field.tooSmall": "Darf nicht kleiner als {limit} sein.",
  "field.tooLarge": "Darf nicht größer als {limit} sein.",
  "field.tooLong": "Darf höchstens {limit} Zeichen lang sein.",

  "error.pluginNotFound": "Das Plugin „{id}“ wurde nicht gefunden.",
  "error.pluginLoad": "Der Plugin-Ordner konnte nicht gelesen werden: {detail}",
  "error.saveMissing": "Diese Speicherdatei existiert nicht mehr.",
  "error.saveRead": "Diese Speicherdatei konnte nicht gelesen werden: {detail}",
  "error.saveFormat": "Das sieht nicht nach einer {game}-Speicherdatei aus.",
  "error.validation": "„{field}“ {reason}",
  "error.fieldRule": "„{field}“ {reason}",
  "error.unknownField":
    "„{field}“ ist in diesem Spielstand kein bearbeitbares Feld.",
  "error.pathNotAllowed":
    "Diese Datei liegt außerhalb der Ordner, die dieses Plugin verwaltet.",
  "error.backupFailed":
    "Die Sicherung konnte nicht angelegt werden, daher blieb dein Spielstand unberührt: {detail}",
  "error.backupNotFound": "Diese Sicherung wurde nicht gefunden.",
  "error.writeFailed":
    "Der Spielstand konnte nicht geschrieben werden: {detail} Dein ursprünglicher Spielstand ist unverändert.",
  "error.io": "{detail}",
  "error.unknown": "Etwas ist schiefgelaufen.",

  "rule.notWholeNumber": "muss eine ganze Zahl sein.",
  "rule.hasDecimalPoint": "muss eine ganze Zahl ohne Komma sein.",
  "rule.notANumber": "muss eine Zahl sein.",
  "rule.notText": "muss Text sein.",
  "rule.notABoolean": "muss an oder aus sein.",
  "rule.notAnOption": "ist keine der verfügbaren Optionen.",
  "rule.tooSmall": "darf nicht kleiner als {limit} sein.",
  "rule.tooLarge": "darf nicht größer als {limit} sein.",
  "rule.tooLong": "darf höchstens {limit} Zeichen lang sein.",
  "rule.tooLargeForGame": "ist für dieses Spiel zu groß.",
  "rule.notPresent": "ist in dieser Speicherdatei nicht vorhanden.",

  // --- Added with the confirmation flow ---
  "confirm.title": "Bist du sicher?",
  "confirm.intro": "{count} Werte liegen außerhalb des Bereichs, den dieses Plugin als sicher ansieht.",
  "confirm.intro_one": "{count} Wert liegt außerhalb des Bereichs, den dieses Plugin als sicher ansieht.",
  "confirm.intro_other": "{count} Werte liegen außerhalb des Bereichs, den dieses Plugin als sicher ansieht.",
  "confirm.risk": "Das Spiel kann sich seltsam verhalten, den Spielstand ablehnen oder abstürzen. Vorher wird trotzdem eine Sicherung angelegt, du kannst das rückgängig machen.",
  "confirm.suggestedMax": "empfohlenes Maximum {limit}",
  "confirm.suggestedMin": "empfohlenes Minimum {limit}",
  "confirm.go": "Trotzdem speichern",
  "changes.title": "Was sich ändert",
  "banner.gameRunning": "{game} läuft gerade. Es kann deine Änderungen beim Beenden überschreiben.",
  "banner.cloud": "Steam Cloud synchronisiert diesen Ordner. Ist die Cloud-Fassung neuer, kann sie deine Änderung rückgängig machen.",
  "banner.staleReload": "Dieser Spielstand wurde auf der Festplatte geändert. Neu laden, um ihn zu sehen.",
  "banner.reload": "Neu laden",
  "editor.discardConfirm": "Du hast ungespeicherte Änderungen. Ohne Speichern verlassen?",
  "error.saveChangedOnDisk": "Das Spiel hat diesen Spielstand geändert, nachdem du ihn geöffnet hast. Lade neu, bevor du speicherst — sonst machst du rückgängig, was das Spiel gerade geschrieben hat.",
  "error.constraint": "{message}",
  "error.needsConfirmation": "Einige Werte liegen außerhalb des sicheren Bereichs.",

  // --- Added with quick actions, list editing and themes ---
  "presets.title": "Schnellaktionen",
  "presets.help": "Jede ist ein gewöhnlicher Satz Änderungen — gesichert und genauso geprüft.",
  "editor.allSections": "Alle",
  "editor.searchFields": "Felder durchsuchen…",
  "editor.revert": "Diese Änderung zurücknehmen",
  "combo.search": "Tippen zum Filtern…",
  "combo.noMatch": "Nichts passt zu „{query}“.",
  "combo.more": "{count} weitere — tippe weiter.",
  "list.add": "Hinzufügen",
  "list.remove": "Entfernen",
  "list.saveFirst": "Speichere zuerst deine Änderungen.",
  "list.added": "Hinzugefügt. Vorher wurde gesichert.",
  "list.removed": "Entfernt. Vorher wurde gesichert.",
  "recovery.title": "Die eigenen Sicherungen des Spiels",
  "recovery.help": "Kopien, die dieses Spiel für sich selbst angelegt hat, direkt neben deinem Spielstand. Werden nur gelesen, nie beschrieben. Beim Wiederherstellen wird das Ersetzte gesichert.",
  "recovery.compare": "Vergleichen",
  "recovery.use": "Diese wiederherstellen",
  "recovery.identical": "Identisch mit deinem aktuellen Spielstand.",
  "recovery.andMore": "…und {count} weitere.",
  "settings.appearance": "Darstellung",
  "settings.theme": "Design",
  "settings.themeSystem": "Wie das System",
  "settings.themeLight": "Hell",
  "settings.themeDark": "Dunkel",
  "error.listNotEditable": "Bei „{list}“ können keine Einträge hinzugefügt oder entfernt werden.",
  "error.listFull": "„{list}“ fasst höchstens {max} Einträge.",
  "error.listAtMinimum": "„{list}“ muss mindestens {min} Einträge behalten.",
};
