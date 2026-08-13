import type { LocaleStrings } from "..";

export const it: LocaleStrings = {
  "app.title": "Universal Save Editor",
  "app.pickGame": "Scegli un gioco per iniziare",
  "app.chooseSave": "Scegli quale salvataggio modificare",
  "app.backups": "Backup",
  "app.backupsSub": "Riporta qualsiasi salvataggio com'era",
  "app.settings": "Impostazioni",
  "app.back": "Indietro",

  "common.loading": "Caricamento…",
  "common.cancel": "Annulla",

  "games.search": "Cerca giochi…",
  "games.noneInstalled": "Non è installato nessun plugin di gioco.",
  "games.addPlugin": "Aggiungine uno nella cartella dei plugin — vedi Impostazioni.",
  "games.noMatch": "Nessun gioco corrisponde a «{query}».",
  "games.moreSoon":
    "Presto altri giochi: ognuno è un plugin e chiunque può scriverne uno.",

  "saves.looking": "Ricerca dei file di salvataggio…",
  "saves.none": "Nessun salvataggio di {game} trovato su questo computer.",
  "saves.playOnce": "Gioca una volta per creare un salvataggio, poi torna qui.",
  "saves.found_one":
    "Trovato {count} salvataggio. Non cambia nulla finché non ne scegli uno e premi Salva.",
  "saves.found_other":
    "Trovati {count} salvataggi. Non cambia nulla finché non ne scegli uno e premi Salva.",
  "saves.lastPlayed": "Ultima partita il {when}",

  "editor.opening": "Apertura del salvataggio…",
  "editor.save": "Salva le modifiche",
  "editor.saving": "Salvataggio…",
  "editor.noChanges": "Ancora nessuna modifica",
  "editor.ready_one": "{count} modifica pronta",
  "editor.ready_other": "{count} modifiche pronte",
  "editor.needFixing_one": "{count} valore da correggere",
  "editor.needFixing_other": "{count} valori da correggere",
  "editor.savedNothing":
    "Non c'era nulla da cambiare: il tuo salvataggio è rimasto intatto.",
  "editor.saved_one":
    "Salvato. {count} valore aggiornato, e prima è stato creato un backup dell'originale.",
  "editor.saved_other":
    "Salvato. {count} valori aggiornati, e prima è stato creato un backup dell'originale.",
  "editor.backupNote":
    "Viene creato un backup prima di modificare il salvataggio. Puoi ripristinarlo quando vuoi da Backup.",
  "editor.notPresent": "Non presente in questo file di salvataggio.",
  "editor.nothingHere": "Qui non c'è nulla in questo salvataggio.",
  "editor.notInList": "{value} (non in elenco)",

  "backups.none": "Ancora nessun backup.",
  "backups.autoNote": "Ne viene creato uno automaticamente ogni volta che salvi.",
  "backups.restore": "Ripristina",
  "backups.delete": "Elimina",
  "backups.confirm": "Sostituire il salvataggio attuale?",
  "backups.yesRestore": "Sì, ripristina",
  "backups.restored":
    "Ripristinato. Il salvataggio è tornato com'era, e anche la versione sostituita è stata salvata.",

  "settings.language": "Lingua",
  "settings.languageDesc":
    "Usata anche per le etichette fornite dal plugin di un gioco.",
  "settings.backupsDesc":
    "Qui si trovano tutti i backup mai creati da questa applicazione.",
  "settings.openFolder": "Apri cartella",
  "settings.plugins": "Plugin",
  "settings.pluginsDesc":
    "Una cartella per gioco. Metti un plugin in una di queste e premi Ricarica.",
  "settings.reload": "Ricarica i plugin",
  "settings.reloaded_one": "{count} gioco disponibile.",
  "settings.reloaded_other": "{count} giochi disponibili.",
  "settings.failedPlugins": "Plugin che non è stato possibile caricare",
  "settings.cantOpenFolder": "Non è stato possibile aprire quella cartella.",
  "settings.about": "Informazioni",
  "settings.aboutText":
    "Universal Save Editor {version} — modifica i tuoi salvataggi offline senza toccare i file grezzi. Tutto gira su questo computer; niente viene inviato altrove.",

  "field.enterValue": "Inserisci un valore.",
  "field.enterNumber": "Inserisci un numero.",
  "field.wholeNumber": "Inserisci un numero intero, senza virgola.",
  "field.tooSmall": "Non può essere inferiore a {limit}.",
  "field.tooLarge": "Non può essere superiore a {limit}.",
  "field.tooLong": "Deve avere al massimo {limit} caratteri.",

  "error.pluginNotFound": "Il plugin «{id}» non è stato trovato.",
  "error.pluginLoad": "Non è stato possibile leggere la cartella del plugin: {detail}",
  "error.saveMissing": "Questo file di salvataggio non esiste più sul disco.",
  "error.saveRead": "Non è stato possibile leggere questo salvataggio: {detail}",
  "error.saveFormat": "Non sembra un file di salvataggio di {game}.",
  "error.validation": "«{field}» {reason}",
  "error.fieldRule": "«{field}» {reason}",
  "error.unknownField": "«{field}» non è un campo modificabile in questo salvataggio.",
  "error.pathNotAllowed":
    "Quel file è fuori dalle cartelle gestite da questo plugin.",
  "error.backupFailed":
    "Non è stato possibile creare il backup, quindi il salvataggio è rimasto intatto: {detail}",
  "error.backupNotFound": "Quel backup non è stato trovato.",
  "error.writeFailed":
    "Non è stato possibile scrivere il salvataggio: {detail} Il tuo salvataggio originale è invariato.",
  "error.io": "{detail}",
  "error.unknown": "Qualcosa è andato storto.",

  "rule.notWholeNumber": "deve essere un numero intero.",
  "rule.hasDecimalPoint": "deve essere un numero intero, senza virgola.",
  "rule.notANumber": "deve essere un numero.",
  "rule.notText": "deve essere testo.",
  "rule.notABoolean": "deve essere attivo o disattivo.",
  "rule.notAnOption": "non è una delle opzioni disponibili.",
  "rule.tooSmall": "non può essere inferiore a {limit}.",
  "rule.tooLarge": "non può essere superiore a {limit}.",
  "rule.tooLong": "deve avere al massimo {limit} caratteri.",
  "rule.tooLargeForGame": "è troppo grande per questo gioco.",
  "rule.notPresent": "non è presente in questo file di salvataggio.",

  // --- Added with the confirmation flow ---
  "confirm.title": "Sei sicuro?",
  "confirm.intro": "{count} valori sono fuori dall'intervallo che questo plugin considera sicuro.",
  "confirm.intro_one": "{count} valore è fuori dall'intervallo che questo plugin considera sicuro.",
  "confirm.intro_other": "{count} valori sono fuori dall'intervallo che questo plugin considera sicuro.",
  "confirm.risk": "Il gioco potrebbe comportarsi in modo strano, rifiutare il salvataggio o bloccarsi. Un backup viene comunque creato prima, quindi puoi tornare indietro.",
  "confirm.suggestedMax": "massimo consigliato {limit}",
  "confirm.suggestedMin": "minimo consigliato {limit}",
  "confirm.go": "Salva comunque",
  "changes.title": "Cosa cambierà",
  "banner.gameRunning": "{game} è aperto in questo momento. Potrebbe sovrascrivere le tue modifiche alla chiusura.",
  "banner.cloud": "Steam Cloud sincronizza questa cartella. Se la copia nel cloud è più recente, può annullare la tua modifica.",
  "banner.staleReload": "Questo salvataggio è cambiato sul disco. Ricarica per vederlo.",
  "banner.reload": "Ricarica",
  "editor.discardConfirm": "Hai modifiche non salvate. Uscire senza salvarle?",
  "error.saveChangedOnDisk": "Il gioco ha modificato questo salvataggio dopo che l'hai aperto. Ricarica prima di salvare, altrimenti annulleresti ciò che il gioco ha appena scritto.",
  "error.constraint": "{message}",
  "error.needsConfirmation": "Alcuni valori sono fuori dall'intervallo sicuro.",

  // --- Added with quick actions, list editing and themes ---
  "presets.title": "Azioni rapide",
  "presets.help": "Ognuna è un normale insieme di modifiche: viene fatto un backup e controllata allo stesso modo.",
  "editor.allSections": "Tutto",
  "editor.searchFields": "Cerca fra i campi…",
  "editor.revert": "Annulla questa modifica",
  "combo.search": "Scrivi per filtrare…",
  "combo.noMatch": "Nessuna corrispondenza per «{query}».",
  "combo.more": "Altri {count} — continua a scrivere.",
  "list.add": "Aggiungi",
  "list.remove": "Rimuovi",
  "list.saveFirst": "Salva prima le tue modifiche.",
  "list.added": "Aggiunto. Prima è stato fatto un backup.",
  "list.removed": "Rimosso. Prima è stato fatto un backup.",
  "recovery.title": "I backup del gioco stesso",
  "recovery.help": "Copie che questo gioco ha creato per sé, accanto al tuo salvataggio. Solo lette, mai scritte. Ripristinandone una viene salvato ciò che sostituisce.",
  "recovery.compare": "Confronta",
  "recovery.use": "Ripristina questo",
  "recovery.identical": "Identico al tuo salvataggio attuale.",
  "recovery.andMore": "…e altri {count}.",
  "settings.appearance": "Aspetto",
  "settings.theme": "Tema",
  "settings.themeSystem": "Come il sistema",
  "settings.themeLight": "Chiaro",
  "settings.themeDark": "Scuro",
  "error.listNotEditable": "In «{list}» non si possono aggiungere o rimuovere righe.",
  "error.listFull": "«{list}» non può contenere più di {max} voci.",
  "error.listAtMinimum": "«{list}» deve mantenere almeno {min} voci.",
};
