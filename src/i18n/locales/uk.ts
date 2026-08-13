import type { LocaleStrings } from "..";

/** Ukrainian has four plural categories: one / few / many / other. */
export const uk: LocaleStrings = {
  "app.title": "Universal Save Editor",
  "app.pickGame": "Виберіть гру, щоб почати",
  "app.chooseSave": "Виберіть збереження для редагування",
  "app.backups": "Резервні копії",
  "app.backupsSub": "Повернути будь-яке збереження до попереднього стану",
  "app.settings": "Налаштування",
  "app.back": "Назад",

  "common.loading": "Завантаження…",
  "common.cancel": "Скасувати",

  "games.search": "Пошук ігор…",
  "games.noneInstalled": "Не встановлено жодного плагіна гри.",
  "games.addPlugin": "Додайте плагін у теку плагінів — дивіться Налаштування.",
  "games.noMatch": "Немає ігор, що відповідають «{query}».",
  "games.moreSoon":
    "Скоро будуть інші ігри — кожна з них плагін, і написати його може будь-хто.",

  "saves.looking": "Пошук файлів збереження…",
  "saves.none": "На цьому комп'ютері не знайдено збережень {game}.",
  "saves.playOnce": "Зіграйте один раз, щоб гра створила збереження, і поверніться.",
  "saves.found_one":
    "Знайдено {count} збереження. Нічого не зміниться, доки ви не виберете його й не натиснете «Зберегти».",
  "saves.found_few":
    "Знайдено {count} збереження. Нічого не зміниться, доки ви не виберете одне й не натиснете «Зберегти».",
  "saves.found_many":
    "Знайдено {count} збережень. Нічого не зміниться, доки ви не виберете одне й не натиснете «Зберегти».",
  "saves.found_other":
    "Знайдено {count} збережень. Нічого не зміниться, доки ви не виберете одне й не натиснете «Зберегти».",
  "saves.lastPlayed": "Востаннє грали: {when}",

  "editor.opening": "Відкриття збереження…",
  "editor.save": "Зберегти зміни",
  "editor.saving": "Збереження…",
  "editor.noChanges": "Змін поки немає",
  "editor.ready_one": "{count} зміна готова",
  "editor.ready_few": "{count} зміни готові",
  "editor.ready_many": "{count} змін готові",
  "editor.ready_other": "{count} змін готові",
  "editor.needFixing_one": "{count} значення потребує виправлення",
  "editor.needFixing_few": "{count} значення потребують виправлення",
  "editor.needFixing_many": "{count} значень потребують виправлення",
  "editor.needFixing_other": "{count} значень потребують виправлення",
  "editor.savedNothing": "Не було чого змінювати — збереження лишилося недоторканим.",
  "editor.saved_one":
    "Збережено. Оновлено {count} значення, а перед цим створено резервну копію оригіналу.",
  "editor.saved_few":
    "Збережено. Оновлено {count} значення, а перед цим створено резервну копію оригіналу.",
  "editor.saved_many":
    "Збережено. Оновлено {count} значень, а перед цим створено резервну копію оригіналу.",
  "editor.saved_other":
    "Збережено. Оновлено {count} значень, а перед цим створено резервну копію оригіналу.",
  "editor.backupNote":
    "Перед зміною збереження створюється резервна копія. Ви можете повернути її будь-коли в розділі «Резервні копії».",
  "editor.notPresent": "Відсутнє в цьому файлі збереження.",
  "editor.nothingHere": "У цьому збереженні тут нічого немає.",
  "editor.notInList": "{value} (немає у списку)",

  "backups.none": "Резервних копій поки немає.",
  "backups.autoNote":
    "Копія створюється автоматично щоразу, коли ви зберігаєте зміну.",
  "backups.restore": "Відновити",
  "backups.delete": "Видалити",
  "backups.confirm": "Замінити поточне збереження?",
  "backups.yesRestore": "Так, відновити",
  "backups.restored":
    "Відновлено. Збереження повернулося до попереднього стану, а замінену версію також збережено.",

  "settings.language": "Мова",
  "settings.languageDesc":
    "Також використовується для назв, які надає плагін гри.",
  "settings.backupsDesc":
    "Тут зберігаються всі резервні копії, створені цим застосунком.",
  "settings.openFolder": "Відкрити теку",
  "settings.plugins": "Плагіни",
  "settings.pluginsDesc":
    "По одній теці на гру. Покладіть плагін у будь-яку з них і натисніть «Перезавантажити».",
  "settings.reload": "Перезавантажити плагіни",
  "settings.reloaded_one": "Доступна {count} гра.",
  "settings.reloaded_few": "Доступно {count} гри.",
  "settings.reloaded_many": "Доступно {count} ігор.",
  "settings.reloaded_other": "Доступно {count} ігор.",
  "settings.failedPlugins": "Плагіни, які не вдалося завантажити",
  "settings.cantOpenFolder": "Не вдалося відкрити цю теку.",
  "settings.about": "Про програму",
  "settings.aboutText":
    "Universal Save Editor {version} — редагуйте збереження офлайн-ігор, не чіпаючи сирі файли. Усе працює на цьому комп'ютері; нікуди нічого не надсилається.",

  "field.enterValue": "Введіть значення.",
  "field.enterNumber": "Введіть число.",
  "field.wholeNumber": "Введіть ціле число, без десяткової частини.",
  "field.tooSmall": "Не може бути менше ніж {limit}.",
  "field.tooLarge": "Не може бути більше ніж {limit}.",
  "field.tooLong": "Не більше ніж {limit} символів.",

  "error.pluginNotFound": "Плагін «{id}» не знайдено.",
  "error.pluginLoad": "Не вдалося прочитати теку плагіна: {detail}",
  "error.saveMissing": "Цього файлу збереження більше немає на диску.",
  "error.saveRead": "Не вдалося прочитати цей файл збереження: {detail}",
  "error.saveFormat": "Це не схоже на файл збереження {game}.",
  "error.validation": "«{field}» {reason}",
  "error.fieldRule": "«{field}» {reason}",
  "error.unknownField": "«{field}» — не редаговане поле в цьому збереженні.",
  "error.pathNotAllowed":
    "Цей файл перебуває поза теками, якими керує цей плагін.",
  "error.backupFailed":
    "Не вдалося створити резервну копію, тому збереження лишилося недоторканим: {detail}",
  "error.backupNotFound": "Цю резервну копію не знайдено.",
  "error.writeFailed":
    "Не вдалося записати збереження: {detail} Ваш початковий файл не змінено.",
  "error.io": "{detail}",
  "error.unknown": "Щось пішло не так.",

  "rule.notWholeNumber": "має бути цілим числом.",
  "rule.hasDecimalPoint": "має бути цілим числом, без десяткової частини.",
  "rule.notANumber": "має бути числом.",
  "rule.notText": "має бути текстом.",
  "rule.notABoolean": "має бути увімкнено або вимкнено.",
  "rule.notAnOption": "не входить до доступних варіантів.",
  "rule.tooSmall": "не може бути менше ніж {limit}.",
  "rule.tooLarge": "не може бути більше ніж {limit}.",
  "rule.tooLong": "має бути не довше ніж {limit} символів.",
  "rule.tooLargeForGame": "завелике для цієї гри.",
  "rule.notPresent": "відсутнє в цьому файлі збереження.",

  // --- Added with the confirmation flow ---
  "confirm.title": "Ви впевнені?",
  "confirm.intro": "{count} значень поза діапазоном, який плагін вважає безпечним.",
  "confirm.intro_one": "{count} значення поза діапазоном, який плагін вважає безпечним.",
  "confirm.intro_few": "{count} значення поза діапазоном, який плагін вважає безпечним.",
  "confirm.intro_many": "{count} значень поза діапазоном, який плагін вважає безпечним.",
  "confirm.intro_other": "{count} значень поза діапазоном, який плагін вважає безпечним.",
  "confirm.risk": "Гра може повестися дивно, відмовитися від збереження або вилетіти. Резервну копію все одно створено заздалегідь, тож це можна скасувати.",
  "confirm.suggestedMax": "рекомендований максимум {limit}",
  "confirm.suggestedMin": "рекомендований мінімум {limit}",
  "confirm.go": "Усе одно зберегти",
  "changes.title": "Що зміниться",
  "banner.gameRunning": "{game} зараз запущена. Під час закриття вона може перезаписати ваші зміни.",
  "banner.cloud": "Steam Cloud синхронізує цю теку. Якщо копія у хмарі новіша, вона може скасувати вашу правку.",
  "banner.staleReload": "Це збереження змінилося на диску. Перезавантажте, щоб побачити.",
  "banner.reload": "Перезавантажити",
  "editor.discardConfirm": "Є незбережені зміни. Вийти без збереження?",
  "error.saveChangedOnDisk": "Гра змінила це збереження після того, як ви його відкрили. Перезавантажте перед збереженням, інакше ви скасуєте те, що гра щойно записала.",
  "error.constraint": "{message}",
  "error.needsConfirmation": "Деякі значення поза безпечним діапазоном.",

  // --- Added with quick actions, list editing and themes ---
  "presets.title": "Швидкі дії",
  "presets.help": "Кожна — звичайний набір змін: із резервною копією і такою самою перевіркою.",
  "editor.allSections": "Усе",
  "editor.searchFields": "Пошук по полях…",
  "editor.revert": "Скасувати цю зміну",
  "combo.search": "Введіть для фільтра…",
  "combo.noMatch": "Нічого не збігається з «{query}».",
  "combo.more": "Ще {count} — продовжуйте вводити.",
  "list.add": "Додати",
  "list.remove": "Видалити",
  "list.saveFirst": "Спершу збережіть зміни.",
  "list.added": "Додано. Перед цим створено резервну копію.",
  "list.removed": "Видалено. Перед цим створено резервну копію.",
  "recovery.title": "Власні копії гри",
  "recovery.help": "Копії, які гра зробила для себе, поруч із вашим збереженням. Лише читаються, ніколи не перезаписуються. Під час відновлення те, що замінюється, теж зберігається.",
  "recovery.compare": "Порівняти",
  "recovery.use": "Відновити цю",
  "recovery.identical": "Збігається з поточним збереженням.",
  "recovery.andMore": "…і ще {count}.",
  "settings.appearance": "Оформлення",
  "settings.theme": "Тема",
  "settings.themeSystem": "Як у системі",
  "settings.themeLight": "Світла",
  "settings.themeDark": "Темна",
  "error.listNotEditable": "У «{list}» не можна додавати чи видаляти рядки.",
  "error.listFull": "«{list}» не може містити більше ніж {max} записів.",
  "error.listAtMinimum": "«{list}» має зберігати щонайменше {min} записів.",
};
