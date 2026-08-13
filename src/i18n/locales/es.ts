import type { LocaleStrings } from "..";

export const es: LocaleStrings = {
  "app.title": "Universal Save Editor",
  "app.pickGame": "Elige un juego para empezar",
  "app.chooseSave": "Elige qué partida editar",
  "app.backups": "Copias de seguridad",
  "app.backupsSub": "Devuelve cualquier partida a como estaba",
  "app.settings": "Ajustes",
  "app.back": "Atrás",

  "common.loading": "Cargando…",
  "common.cancel": "Cancelar",

  "games.search": "Buscar juegos…",
  "games.noneInstalled": "No hay complementos de juegos instalados.",
  "games.addPlugin": "Añade uno a tu carpeta de complementos — mira los Ajustes.",
  "games.noMatch": "Ningún juego coincide con «{query}».",
  "games.moreSoon":
    "Pronto habrá más juegos: cada uno es un complemento y cualquiera puede escribir uno.",

  "saves.looking": "Buscando partidas guardadas…",
  "saves.none": "No se encontraron partidas de {game} en este ordenador.",
  "saves.playOnce":
    "Juega una vez para que se cree una partida y vuelve luego.",
  "saves.found_one":
    "Se encontró {count} partida. No cambia nada hasta que elijas una y pulses Guardar.",
  "saves.found_other":
    "Se encontraron {count} partidas. No cambia nada hasta que elijas una y pulses Guardar.",
  "saves.lastPlayed": "Jugada por última vez el {when}",

  "editor.opening": "Abriendo la partida…",
  "editor.save": "Guardar cambios",
  "editor.saving": "Guardando…",
  "editor.noChanges": "Aún no hay cambios",
  "editor.ready_one": "{count} cambio listo",
  "editor.ready_other": "{count} cambios listos",
  "editor.needFixing_one": "Hay {count} valor que corregir",
  "editor.needFixing_other": "Hay {count} valores que corregir",
  "editor.savedNothing": "No hacía falta cambiar nada: tu partida sigue intacta.",
  "editor.saved_one":
    "Guardado. Se actualizó {count} valor, y antes se creó una copia de seguridad del original.",
  "editor.saved_other":
    "Guardado. Se actualizaron {count} valores, y antes se creó una copia de seguridad del original.",
  "editor.backupNote":
    "Se crea una copia de seguridad antes de modificar tu partida. Puedes restaurarla cuando quieras desde Copias de seguridad.",
  "editor.notPresent": "No está presente en este archivo de guardado.",
  "editor.nothingHere": "Aquí no hay nada en esta partida.",
  "editor.notInList": "{value} (no está en la lista)",

  "backups.none": "Todavía no hay copias de seguridad.",
  "backups.autoNote": "Se crea una automáticamente cada vez que guardas un cambio.",
  "backups.restore": "Restaurar",
  "backups.delete": "Eliminar",
  "backups.confirm": "¿Reemplazar la partida actual?",
  "backups.yesRestore": "Sí, restaurar",
  "backups.restored":
    "Restaurada. Tu partida vuelve a estar como estaba, y también se guardó una copia de la versión reemplazada.",

  "settings.language": "Idioma",
  "settings.languageDesc":
    "También se usa para las etiquetas que aporta el complemento de un juego.",
  "settings.backupsDesc":
    "Aquí están todas las copias de seguridad que ha hecho esta aplicación.",
  "settings.openFolder": "Abrir carpeta",
  "settings.plugins": "Complementos",
  "settings.pluginsDesc":
    "Una carpeta por juego. Pon un complemento en cualquiera de ellas y pulsa Recargar.",
  "settings.reload": "Recargar complementos",
  "settings.reloaded_one": "{count} juego disponible.",
  "settings.reloaded_other": "{count} juegos disponibles.",
  "settings.failedPlugins": "Complementos que no se pudieron cargar",
  "settings.cantOpenFolder": "No se pudo abrir esa carpeta.",
  "settings.about": "Acerca de",
  "settings.aboutText":
    "Universal Save Editor {version}: edita tus partidas sin conexión sin tocar los archivos en crudo. Todo se ejecuta en este ordenador; no se envía nada a ninguna parte.",

  "field.enterValue": "Introduce un valor.",
  "field.enterNumber": "Introduce un número.",
  "field.wholeNumber": "Introduce un número entero, sin decimales.",
  "field.tooSmall": "No puede ser menor que {limit}.",
  "field.tooLarge": "No puede ser mayor que {limit}.",
  "field.tooLong": "Debe tener como máximo {limit} caracteres.",

  "error.pluginNotFound": "No se encontró el complemento «{id}».",
  "error.pluginLoad": "No se pudo leer la carpeta del complemento: {detail}",
  "error.saveMissing": "Este archivo de guardado ya no existe en el disco.",
  "error.saveRead": "No se pudo leer este archivo de guardado: {detail}",
  "error.saveFormat": "Esto no parece un archivo de guardado de {game}.",
  "error.validation": "«{field}» {reason}",
  "error.fieldRule": "«{field}» {reason}",
  "error.unknownField": "«{field}» no es un campo editable en esta partida.",
  "error.pathNotAllowed":
    "Ese archivo está fuera de las carpetas que gestiona este complemento.",
  "error.backupFailed":
    "No se pudo crear la copia de seguridad, así que tu partida quedó intacta: {detail}",
  "error.backupNotFound": "No se encontró esa copia de seguridad.",
  "error.writeFailed":
    "No se pudo escribir la partida: {detail} Tu partida original está sin cambios.",
  "error.io": "{detail}",
  "error.unknown": "Algo salió mal.",

  "rule.notWholeNumber": "debe ser un número entero.",
  "rule.hasDecimalPoint": "debe ser un número entero, sin decimales.",
  "rule.notANumber": "debe ser un número.",
  "rule.notText": "debe ser texto.",
  "rule.notABoolean": "debe estar activado o desactivado.",
  "rule.notAnOption": "no es una de las opciones disponibles.",
  "rule.tooSmall": "no puede ser menor que {limit}.",
  "rule.tooLarge": "no puede ser mayor que {limit}.",
  "rule.tooLong": "debe tener como máximo {limit} caracteres.",
  "rule.tooLargeForGame": "es demasiado grande para este juego.",
  "rule.notPresent": "no está presente en este archivo de guardado.",

  // --- Added with the confirmation flow ---
  "confirm.title": "¿Seguro?",
  "confirm.intro": "{count} valores están fuera del rango que este complemento considera seguro.",
  "confirm.intro_one": "{count} valor está fuera del rango que este complemento considera seguro.",
  "confirm.intro_other": "{count} valores están fuera del rango que este complemento considera seguro.",
  "confirm.risk": "El juego puede comportarse de forma extraña, rechazar la partida o cerrarse. Aun así se hace una copia de seguridad antes, así que puedes deshacerlo.",
  "confirm.suggestedMax": "máximo sugerido {limit}",
  "confirm.suggestedMin": "mínimo sugerido {limit}",
  "confirm.go": "Guardar de todos modos",
  "changes.title": "Qué va a cambiar",
  "banner.gameRunning": "{game} está abierto ahora mismo. Puede sobrescribir tus cambios al cerrarse.",
  "banner.cloud": "Steam Cloud sincroniza esta carpeta. Si la copia de la nube es más reciente, puede deshacer tu edición.",
  "banner.staleReload": "Esta partida cambió en el disco. Recarga para verla.",
  "banner.reload": "Recargar",
  "editor.discardConfirm": "Tienes cambios sin guardar. ¿Salir sin guardarlos?",
  "error.saveChangedOnDisk": "El juego cambió esta partida después de que la abrieras. Recarga antes de guardar, o desharías lo que el juego acaba de escribir.",
  "error.constraint": "{message}",
  "error.needsConfirmation": "Algunos valores están fuera del rango seguro.",

  // --- Added with quick actions, list editing and themes ---
  "presets.title": "Acciones rápidas",
  "presets.help": "Cada una es un conjunto normal de cambios: se hace copia de seguridad y se comprueba igual.",
  "editor.allSections": "Todo",
  "editor.searchFields": "Buscar campos…",
  "editor.revert": "Deshacer este cambio",
  "combo.search": "Escribe para filtrar…",
  "combo.noMatch": "Nada coincide con «{query}».",
  "combo.more": "{count} más: sigue escribiendo.",
  "list.add": "Añadir",
  "list.remove": "Quitar",
  "list.saveFirst": "Guarda primero tus cambios.",
  "list.added": "Añadido. Antes se hizo una copia de seguridad.",
  "list.removed": "Quitado. Antes se hizo una copia de seguridad.",
  "recovery.title": "Las copias del propio juego",
  "recovery.help": "Copias que este juego hizo para sí mismo, junto a tu partida. Solo se leen, nunca se escriben. Al restaurar una se guarda lo que reemplaza.",
  "recovery.compare": "Comparar",
  "recovery.use": "Restaurar esta",
  "recovery.identical": "Idéntica a tu partida actual.",
  "recovery.andMore": "…y {count} más.",
  "settings.appearance": "Apariencia",
  "settings.theme": "Tema",
  "settings.themeSystem": "Igual que el sistema",
  "settings.themeLight": "Claro",
  "settings.themeDark": "Oscuro",
  "error.listNotEditable": "En «{list}» no se pueden añadir ni quitar filas.",
  "error.listFull": "«{list}» no puede tener más de {max} entradas.",
  "error.listAtMinimum": "«{list}» debe conservar al menos {min} entradas.",
};
