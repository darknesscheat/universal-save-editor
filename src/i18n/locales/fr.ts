import type { LocaleStrings } from "..";

export const fr: LocaleStrings = {
  "app.title": "Universal Save Editor",
  "app.pickGame": "Choisis un jeu pour commencer",
  "app.chooseSave": "Choisis la sauvegarde à modifier",
  "app.backups": "Sauvegardes",
  "app.backupsSub": "Remets n'importe quelle partie comme elle était",
  "app.settings": "Réglages",
  "app.back": "Retour",

  "common.loading": "Chargement…",
  "common.cancel": "Annuler",

  "games.search": "Rechercher des jeux…",
  "games.noneInstalled": "Aucun module de jeu n'est installé.",
  "games.addPlugin": "Ajoutes-en un dans ton dossier de modules — voir les Réglages.",
  "games.noMatch": "Aucun jeu ne correspond à « {query} ».",
  "games.moreSoon":
    "D'autres jeux arrivent — chacun est un module, et n'importe qui peut en écrire un.",

  "saves.looking": "Recherche des fichiers de sauvegarde…",
  "saves.none": "Aucune sauvegarde de {game} trouvée sur cet ordinateur.",
  "saves.playOnce":
    "Joue une fois pour qu'une sauvegarde soit créée, puis reviens.",
  "saves.found_one":
    "{count} sauvegarde trouvée. Rien n'est modifié tant que tu n'en choisis pas une et n'appuies pas sur Enregistrer.",
  "saves.found_other":
    "{count} sauvegardes trouvées. Rien n'est modifié tant que tu n'en choisis pas une et n'appuies pas sur Enregistrer.",
  "saves.lastPlayed": "Dernière partie le {when}",

  "editor.opening": "Ouverture de la sauvegarde…",
  "editor.save": "Enregistrer les modifications",
  "editor.saving": "Enregistrement…",
  "editor.noChanges": "Aucune modification pour l'instant",
  "editor.ready_one": "{count} modification prête",
  "editor.ready_other": "{count} modifications prêtes",
  "editor.needFixing_one": "{count} valeur à corriger",
  "editor.needFixing_other": "{count} valeurs à corriger",
  "editor.savedNothing":
    "Il n'y avait rien à changer — ta sauvegarde n'a pas été touchée.",
  "editor.saved_one":
    "Enregistré. {count} valeur mise à jour, après création d'une sauvegarde de l'original.",
  "editor.saved_other":
    "Enregistré. {count} valeurs mises à jour, après création d'une sauvegarde de l'original.",
  "editor.backupNote":
    "Une sauvegarde est créée avant toute modification. Tu peux la restaurer à tout moment depuis Sauvegardes.",
  "editor.notPresent": "Absent de ce fichier de sauvegarde.",
  "editor.nothingHere": "Rien ici dans cette sauvegarde.",
  "editor.notInList": "{value} (absent de la liste)",

  "backups.none": "Pas encore de sauvegardes.",
  "backups.autoNote":
    "Une sauvegarde est créée automatiquement à chaque modification enregistrée.",
  "backups.restore": "Restaurer",
  "backups.delete": "Supprimer",
  "backups.confirm": "Remplacer la sauvegarde actuelle ?",
  "backups.yesRestore": "Oui, restaurer",
  "backups.restored":
    "Restaurée. Ta sauvegarde est revenue à son état précédent, et la version remplacée a également été sauvegardée.",

  "settings.language": "Langue",
  "settings.languageDesc":
    "Utilisée aussi pour les libellés fournis par un module de jeu.",
  "settings.backupsDesc":
    "Toutes les sauvegardes créées par cette application se trouvent ici.",
  "settings.openFolder": "Ouvrir le dossier",
  "settings.plugins": "Modules",
  "settings.pluginsDesc":
    "Un dossier par jeu. Dépose un module dans l'un d'eux et appuie sur Recharger.",
  "settings.reload": "Recharger les modules",
  "settings.reloaded_one": "{count} jeu disponible.",
  "settings.reloaded_other": "{count} jeux disponibles.",
  "settings.failedPlugins": "Modules qui n'ont pas pu être chargés",
  "settings.cantOpenFolder": "Ce dossier n'a pas pu être ouvert.",
  "settings.about": "À propos",
  "settings.aboutText":
    "Universal Save Editor {version} — modifie tes sauvegardes hors ligne sans toucher aux fichiers bruts. Tout fonctionne sur cet ordinateur ; rien n'est envoyé ailleurs.",

  "field.enterValue": "Saisis une valeur.",
  "field.enterNumber": "Saisis un nombre.",
  "field.wholeNumber": "Saisis un nombre entier, sans virgule.",
  "field.tooSmall": "Ne peut pas être inférieur à {limit}.",
  "field.tooLarge": "Ne peut pas être supérieur à {limit}.",
  "field.tooLong": "Doit faire au plus {limit} caractères.",

  "error.pluginNotFound": "Le module « {id} » est introuvable.",
  "error.pluginLoad": "Le dossier du module n'a pas pu être lu : {detail}",
  "error.saveMissing": "Ce fichier de sauvegarde n'existe plus sur le disque.",
  "error.saveRead": "Ce fichier de sauvegarde n'a pas pu être lu : {detail}",
  "error.saveFormat": "Cela ne ressemble pas à un fichier de sauvegarde {game}.",
  "error.validation": "« {field} » {reason}",
  "error.fieldRule": "« {field} » {reason}",
  "error.unknownField":
    "« {field} » n'est pas un champ modifiable dans cette sauvegarde.",
  "error.pathNotAllowed":
    "Ce fichier se trouve hors des dossiers gérés par ce module.",
  "error.backupFailed":
    "La sauvegarde de secours n'a pas pu être créée, ta partie n'a donc pas été touchée : {detail}",
  "error.backupNotFound": "Cette sauvegarde est introuvable.",
  "error.writeFailed":
    "La sauvegarde n'a pas pu être écrite : {detail} Ton fichier d'origine est inchangé.",
  "error.io": "{detail}",
  "error.unknown": "Quelque chose s'est mal passé.",

  "rule.notWholeNumber": "doit être un nombre entier.",
  "rule.hasDecimalPoint": "doit être un nombre entier, sans virgule.",
  "rule.notANumber": "doit être un nombre.",
  "rule.notText": "doit être du texte.",
  "rule.notABoolean": "doit être activé ou désactivé.",
  "rule.notAnOption": "ne fait pas partie des options disponibles.",
  "rule.tooSmall": "ne peut pas être inférieur à {limit}.",
  "rule.tooLarge": "ne peut pas être supérieur à {limit}.",
  "rule.tooLong": "doit faire au plus {limit} caractères.",
  "rule.tooLargeForGame": "est trop grand pour ce jeu.",
  "rule.notPresent": "est absent de ce fichier de sauvegarde.",

  // --- Added with the confirmation flow ---
  "confirm.title": "Tu es sûr ?",
  "confirm.intro": "{count} valeurs sortent de la plage que ce module considère comme sûre.",
  "confirm.intro_one": "{count} valeur sort de la plage que ce module considère comme sûre.",
  "confirm.intro_other": "{count} valeurs sortent de la plage que ce module considère comme sûre.",
  "confirm.risk": "Le jeu peut se comporter bizarrement, refuser la sauvegarde ou planter. Une sauvegarde de secours est tout de même créée avant, tu peux revenir en arrière.",
  "confirm.suggestedMax": "maximum conseillé {limit}",
  "confirm.suggestedMin": "minimum conseillé {limit}",
  "confirm.go": "Enregistrer quand même",
  "changes.title": "Ce qui va changer",
  "banner.gameRunning": "{game} est ouvert en ce moment. Il peut écraser tes modifications en se fermant.",
  "banner.cloud": "Steam Cloud synchronise ce dossier. Si la copie du cloud est plus récente, elle peut annuler ta modification.",
  "banner.staleReload": "Cette sauvegarde a changé sur le disque. Recharge pour la voir.",
  "banner.reload": "Recharger",
  "editor.discardConfirm": "Tu as des modifications non enregistrées. Quitter sans les enregistrer ?",
  "error.saveChangedOnDisk": "Le jeu a modifié cette sauvegarde après que tu l'as ouverte. Recharge avant d'enregistrer, sinon tu annulerais ce que le jeu vient d'écrire.",
  "error.constraint": "{message}",
  "error.needsConfirmation": "Certaines valeurs sortent de la plage sûre.",

  // --- Added with quick actions, list editing and themes ---
  "presets.title": "Actions rapides",
  "presets.help": "Chacune est un ensemble de modifications ordinaire — sauvegardé et vérifié de la même façon.",
  "editor.allSections": "Tout",
  "editor.searchFields": "Rechercher des champs…",
  "editor.revert": "Annuler cette modification",
  "combo.search": "Tape pour filtrer…",
  "combo.noMatch": "Rien ne correspond à « {query} ».",
  "combo.more": "{count} de plus — continue à taper.",
  "list.add": "Ajouter",
  "list.remove": "Retirer",
  "list.saveFirst": "Enregistre d'abord tes modifications.",
  "list.added": "Ajouté. Une sauvegarde a été faite avant.",
  "list.removed": "Retiré. Une sauvegarde a été faite avant.",
  "recovery.title": "Les sauvegardes du jeu lui-même",
  "recovery.help": "Copies que ce jeu a faites pour lui-même, à côté de ta sauvegarde. Uniquement lues, jamais écrites. Restaurer l'une d'elles sauvegarde ce qu'elle remplace.",
  "recovery.compare": "Comparer",
  "recovery.use": "Restaurer celle-ci",
  "recovery.identical": "Identique à ta sauvegarde actuelle.",
  "recovery.andMore": "…et {count} de plus.",
  "settings.appearance": "Apparence",
  "settings.theme": "Thème",
  "settings.themeSystem": "Comme le système",
  "settings.themeLight": "Clair",
  "settings.themeDark": "Sombre",
  "error.listNotEditable": "« {list} » n'accepte ni ajout ni suppression de lignes.",
  "error.listFull": "« {list} » ne peut pas contenir plus de {max} entrées.",
  "error.listAtMinimum": "« {list} » doit garder au moins {min} entrées.",
};
