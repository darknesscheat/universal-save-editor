import type { LocaleStrings } from "..";

/**
 * Russian uses four plural categories, so counted strings supply `_one`,
 * `_few`, `_many` and `_other` — `Intl.PluralRules` picks the right one.
 */
export const ru: LocaleStrings = {
  "app.title": "Universal Save Editor",
  "app.pickGame": "Выберите игру, чтобы начать",
  "app.chooseSave": "Выберите сохранение для редактирования",
  "app.backups": "Резервные копии",
  "app.backupsSub": "Вернуть любое сохранение в прежний вид",
  "app.settings": "Настройки",
  "app.back": "Назад",

  "common.loading": "Загрузка…",
  "common.cancel": "Отмена",

  "games.search": "Поиск игр…",
  "games.noneInstalled": "Не установлено ни одного плагина игры.",
  "games.addPlugin": "Добавьте плагин в папку плагинов — смотрите Настройки.",
  "games.noMatch": "Нет игр, подходящих под «{query}».",
  "games.moreSoon":
    "Скоро будут другие игры — каждая из них плагин, и написать его может кто угодно.",

  "saves.looking": "Поиск файлов сохранения…",
  "saves.none": "На этом компьютере не найдено сохранений {game}.",
  "saves.playOnce": "Сыграйте один раз, чтобы игра создала сохранение, и вернитесь.",
  "saves.found_one":
    "Найдено {count} сохранение. Ничего не изменится, пока вы не выберете его и не нажмёте «Сохранить».",
  "saves.found_few":
    "Найдено {count} сохранения. Ничего не изменится, пока вы не выберете одно и не нажмёте «Сохранить».",
  "saves.found_many":
    "Найдено {count} сохранений. Ничего не изменится, пока вы не выберете одно и не нажмёте «Сохранить».",
  "saves.found_other":
    "Найдено {count} сохранений. Ничего не изменится, пока вы не выберете одно и не нажмёте «Сохранить».",
  "saves.lastPlayed": "Последняя игра: {when}",

  "editor.opening": "Открытие сохранения…",
  "editor.save": "Сохранить изменения",
  "editor.saving": "Сохранение…",
  "editor.noChanges": "Изменений пока нет",
  "editor.ready_one": "{count} изменение готово",
  "editor.ready_few": "{count} изменения готовы",
  "editor.ready_many": "{count} изменений готовы",
  "editor.ready_other": "{count} изменений готовы",
  "editor.needFixing_one": "{count} значение нужно исправить",
  "editor.needFixing_few": "{count} значения нужно исправить",
  "editor.needFixing_many": "{count} значений нужно исправить",
  "editor.needFixing_other": "{count} значений нужно исправить",
  "editor.savedNothing": "Менять было нечего — сохранение осталось нетронутым.",
  "editor.saved_one":
    "Сохранено. Обновлено {count} значение, а перед этим создана резервная копия оригинала.",
  "editor.saved_few":
    "Сохранено. Обновлено {count} значения, а перед этим создана резервная копия оригинала.",
  "editor.saved_many":
    "Сохранено. Обновлено {count} значений, а перед этим создана резервная копия оригинала.",
  "editor.saved_other":
    "Сохранено. Обновлено {count} значений, а перед этим создана резервная копия оригинала.",
  "editor.backupNote":
    "Перед изменением сохранения создаётся резервная копия. Вернуть её можно в любой момент в разделе «Резервные копии».",
  "editor.notPresent": "Отсутствует в этом файле сохранения.",
  "editor.nothingHere": "В этом сохранении здесь ничего нет.",
  "editor.notInList": "{value} (нет в списке)",

  "backups.none": "Резервных копий пока нет.",
  "backups.autoNote":
    "Копия создаётся автоматически каждый раз, когда вы сохраняете изменение.",
  "backups.restore": "Восстановить",
  "backups.delete": "Удалить",
  "backups.confirm": "Заменить текущее сохранение?",
  "backups.yesRestore": "Да, восстановить",
  "backups.restored":
    "Восстановлено. Сохранение вернулось к прежнему виду, а заменённая версия тоже помещена в резервные копии.",

  "settings.language": "Язык",
  "settings.languageDesc":
    "Также используется для названий, которые предоставляет плагин игры.",
  "settings.backupsDesc":
    "Здесь хранятся все резервные копии, созданные этим приложением.",
  "settings.openFolder": "Открыть папку",
  "settings.plugins": "Плагины",
  "settings.pluginsDesc":
    "По одной папке на игру. Поместите плагин в любую из них и нажмите «Перезагрузить».",
  "settings.reload": "Перезагрузить плагины",
  "settings.reloaded_one": "Доступна {count} игра.",
  "settings.reloaded_few": "Доступно {count} игры.",
  "settings.reloaded_many": "Доступно {count} игр.",
  "settings.reloaded_other": "Доступно {count} игр.",
  "settings.failedPlugins": "Плагины, которые не удалось загрузить",
  "settings.cantOpenFolder": "Не удалось открыть эту папку.",
  "settings.about": "О программе",
  "settings.aboutText":
    "Universal Save Editor {version} — редактируйте сохранения офлайн-игр, не трогая сырые файлы. Всё работает на этом компьютере; никуда ничего не отправляется.",

  "field.enterValue": "Введите значение.",
  "field.enterNumber": "Введите число.",
  "field.wholeNumber": "Введите целое число, без десятичной части.",
  "field.tooSmall": "Не может быть меньше {limit}.",
  "field.tooLarge": "Не может быть больше {limit}.",
  "field.tooLong": "Не более {limit} символов.",

  "error.pluginNotFound": "Плагин «{id}» не найден.",
  "error.pluginLoad": "Не удалось прочитать папку плагина: {detail}",
  "error.saveMissing": "Этого файла сохранения больше нет на диске.",
  "error.saveRead": "Не удалось прочитать этот файл сохранения: {detail}",
  "error.saveFormat": "Это не похоже на файл сохранения {game}.",
  "error.validation": "«{field}» {reason}",
  "error.fieldRule": "«{field}» {reason}",
  "error.unknownField": "«{field}» — не редактируемое поле в этом сохранении.",
  "error.pathNotAllowed":
    "Этот файл находится вне папок, которыми управляет плагин.",
  "error.backupFailed":
    "Не удалось создать резервную копию, поэтому сохранение осталось нетронутым: {detail}",
  "error.backupNotFound": "Эта резервная копия не найдена.",
  "error.writeFailed":
    "Не удалось записать сохранение: {detail} Исходный файл не изменён.",
  "error.io": "{detail}",
  "error.unknown": "Что-то пошло не так.",

  "rule.notWholeNumber": "должно быть целым числом.",
  "rule.hasDecimalPoint": "должно быть целым числом, без десятичной части.",
  "rule.notANumber": "должно быть числом.",
  "rule.notText": "должно быть текстом.",
  "rule.notABoolean": "должно быть включено или выключено.",
  "rule.notAnOption": "не входит в число доступных вариантов.",
  "rule.tooSmall": "не может быть меньше {limit}.",
  "rule.tooLarge": "не может быть больше {limit}.",
  "rule.tooLong": "должно быть не длиннее {limit} символов.",
  "rule.tooLargeForGame": "слишком велико для этой игры.",
  "rule.notPresent": "отсутствует в этом файле сохранения.",

  // --- Added with the confirmation flow ---
  "confirm.title": "Вы уверены?",
  "confirm.intro": "{count} значений вне диапазона, который плагин считает безопасным.",
  "confirm.intro_one": "{count} значение вне диапазона, который плагин считает безопасным.",
  "confirm.intro_few": "{count} значения вне диапазона, который плагин считает безопасным.",
  "confirm.intro_many": "{count} значений вне диапазона, который плагин считает безопасным.",
  "confirm.intro_other": "{count} значений вне диапазона, который плагин считает безопасным.",
  "confirm.risk": "Игра может повести себя странно, отказаться от сохранения или вылететь. Резервная копия всё равно создаётся заранее, так что это можно отменить.",
  "confirm.suggestedMax": "рекомендуемый максимум {limit}",
  "confirm.suggestedMin": "рекомендуемый минимум {limit}",
  "confirm.go": "Всё равно сохранить",
  "changes.title": "Что изменится",
  "banner.gameRunning": "{game} сейчас запущена. При закрытии она может перезаписать ваши изменения.",
  "banner.cloud": "Steam Cloud синхронизирует эту папку. Если копия в облаке новее, она может отменить вашу правку.",
  "banner.staleReload": "Это сохранение изменилось на диске. Перезагрузите, чтобы увидеть.",
  "banner.reload": "Перезагрузить",
  "editor.discardConfirm": "Есть несохранённые изменения. Выйти без сохранения?",
  "error.saveChangedOnDisk": "Игра изменила это сохранение после того, как вы его открыли. Перезагрузите перед сохранением, иначе вы отмените то, что игра только что записала.",
  "error.constraint": "{message}",
  "error.needsConfirmation": "Некоторые значения вне безопасного диапазона.",

  // --- Added with quick actions, list editing and themes ---
  "presets.title": "Быстрые действия",
  "presets.help": "Каждое — обычный набор изменений: с резервной копией и такой же проверкой.",
  "editor.allSections": "Все",
  "editor.searchFields": "Поиск по полям…",
  "editor.revert": "Отменить это изменение",
  "combo.search": "Введите для фильтра…",
  "combo.noMatch": "Ничего не совпадает с «{query}».",
  "combo.more": "Ещё {count} — продолжайте вводить.",
  "list.add": "Добавить",
  "list.remove": "Удалить",
  "list.saveFirst": "Сначала сохраните изменения.",
  "list.added": "Добавлено. Перед этим создана резервная копия.",
  "list.removed": "Удалено. Перед этим создана резервная копия.",
  "recovery.title": "Собственные копии игры",
  "recovery.help": "Копии, которые игра сделала для себя, рядом с вашим сохранением. Только читаются, никогда не перезаписываются. При восстановлении то, что заменяется, тоже сохраняется.",
  "recovery.compare": "Сравнить",
  "recovery.use": "Восстановить эту",
  "recovery.identical": "Совпадает с текущим сохранением.",
  "recovery.andMore": "…и ещё {count}.",
  "settings.appearance": "Оформление",
  "settings.theme": "Тема",
  "settings.themeSystem": "Как в системе",
  "settings.themeLight": "Светлая",
  "settings.themeDark": "Тёмная",
  "error.listNotEditable": "В «{list}» нельзя добавлять и удалять строки.",
  "error.listFull": "«{list}» не может содержать больше {max} записей.",
  "error.listAtMinimum": "«{list}» должен сохранять хотя бы {min} записей.",
};
