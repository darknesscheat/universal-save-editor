import type { LocaleStrings } from "..";

/** Polish also has four plural categories: one / few / many / other. */
export const pl: LocaleStrings = {
  "app.title": "Universal Save Editor",
  "app.pickGame": "Wybierz grę, aby zacząć",
  "app.chooseSave": "Wybierz zapis do edycji",
  "app.backups": "Kopie zapasowe",
  "app.backupsSub": "Przywróć dowolny zapis do poprzedniego stanu",
  "app.settings": "Ustawienia",
  "app.back": "Wstecz",

  "common.loading": "Wczytywanie…",
  "common.cancel": "Anuluj",

  "games.search": "Szukaj gier…",
  "games.noneInstalled": "Nie zainstalowano żadnej wtyczki gry.",
  "games.addPlugin": "Dodaj wtyczkę do swojego folderu wtyczek — zobacz Ustawienia.",
  "games.noMatch": "Żadna gra nie pasuje do „{query}”.",
  "games.moreSoon":
    "Wkrótce więcej gier — każda jest wtyczką i każdy może napisać własną.",

  "saves.looking": "Szukanie plików zapisu…",
  "saves.none": "Nie znaleziono zapisów gry {game} na tym komputerze.",
  "saves.playOnce": "Zagraj raz, aby powstał zapis, i wróć tutaj.",
  "saves.found_one":
    "Znaleziono {count} zapis. Nic się nie zmieni, dopóki nie wybierzesz go i nie klikniesz Zapisz.",
  "saves.found_few":
    "Znaleziono {count} zapisy. Nic się nie zmieni, dopóki nie wybierzesz jednego i nie klikniesz Zapisz.",
  "saves.found_many":
    "Znaleziono {count} zapisów. Nic się nie zmieni, dopóki nie wybierzesz jednego i nie klikniesz Zapisz.",
  "saves.found_other":
    "Znaleziono {count} zapisów. Nic się nie zmieni, dopóki nie wybierzesz jednego i nie klikniesz Zapisz.",
  "saves.lastPlayed": "Ostatnia gra: {when}",

  "editor.opening": "Otwieranie zapisu…",
  "editor.save": "Zapisz zmiany",
  "editor.saving": "Zapisywanie…",
  "editor.noChanges": "Na razie brak zmian",
  "editor.ready_one": "{count} zmiana gotowa",
  "editor.ready_few": "{count} zmiany gotowe",
  "editor.ready_many": "{count} zmian gotowych",
  "editor.ready_other": "{count} zmian gotowych",
  "editor.needFixing_one": "{count} wartość wymaga poprawy",
  "editor.needFixing_few": "{count} wartości wymagają poprawy",
  "editor.needFixing_many": "{count} wartości wymaga poprawy",
  "editor.needFixing_other": "{count} wartości wymaga poprawy",
  "editor.savedNothing": "Nie było nic do zmiany — zapis pozostał nietknięty.",
  "editor.saved_one":
    "Zapisano. Zaktualizowano {count} wartość, a wcześniej utworzono kopię zapasową oryginału.",
  "editor.saved_few":
    "Zapisano. Zaktualizowano {count} wartości, a wcześniej utworzono kopię zapasową oryginału.",
  "editor.saved_many":
    "Zapisano. Zaktualizowano {count} wartości, a wcześniej utworzono kopię zapasową oryginału.",
  "editor.saved_other":
    "Zapisano. Zaktualizowano {count} wartości, a wcześniej utworzono kopię zapasową oryginału.",
  "editor.backupNote":
    "Przed modyfikacją zapisu tworzona jest kopia zapasowa. Możesz ją przywrócić w każdej chwili w sekcji Kopie zapasowe.",
  "editor.notPresent": "Brak w tym pliku zapisu.",
  "editor.nothingHere": "W tym zapisie nic tu nie ma.",
  "editor.notInList": "{value} (spoza listy)",

  "backups.none": "Brak kopii zapasowych.",
  "backups.autoNote":
    "Kopia powstaje automatycznie za każdym razem, gdy zapiszesz zmianę.",
  "backups.restore": "Przywróć",
  "backups.delete": "Usuń",
  "backups.confirm": "Zastąpić bieżący zapis?",
  "backups.yesRestore": "Tak, przywróć",
  "backups.restored":
    "Przywrócono. Zapis wrócił do poprzedniego stanu, a zastąpiona wersja również trafiła do kopii zapasowych.",

  "settings.language": "Język",
  "settings.languageDesc":
    "Używany także do etykiet dostarczanych przez wtyczkę gry.",
  "settings.backupsDesc":
    "Tutaj znajdują się wszystkie kopie zapasowe utworzone przez tę aplikację.",
  "settings.openFolder": "Otwórz folder",
  "settings.plugins": "Wtyczki",
  "settings.pluginsDesc":
    "Jeden folder na grę. Wrzuć wtyczkę do dowolnego z nich i kliknij Przeładuj.",
  "settings.reload": "Przeładuj wtyczki",
  "settings.reloaded_one": "Dostępna {count} gra.",
  "settings.reloaded_few": "Dostępne {count} gry.",
  "settings.reloaded_many": "Dostępnych {count} gier.",
  "settings.reloaded_other": "Dostępnych {count} gier.",
  "settings.failedPlugins": "Wtyczki, których nie udało się wczytać",
  "settings.cantOpenFolder": "Nie udało się otworzyć tego folderu.",
  "settings.about": "O programie",
  "settings.aboutText":
    "Universal Save Editor {version} — edytuj zapisy gier offline bez dotykania surowych plików. Wszystko działa na tym komputerze; nic nigdzie nie jest wysyłane.",

  "field.enterValue": "Podaj wartość.",
  "field.enterNumber": "Podaj liczbę.",
  "field.wholeNumber": "Podaj liczbę całkowitą, bez części dziesiętnej.",
  "field.tooSmall": "Nie może być mniejsza niż {limit}.",
  "field.tooLarge": "Nie może być większa niż {limit}.",
  "field.tooLong": "Może mieć najwyżej {limit} znaków.",

  "error.pluginNotFound": "Nie znaleziono wtyczki „{id}”.",
  "error.pluginLoad": "Nie udało się odczytać folderu wtyczki: {detail}",
  "error.saveMissing": "Tego pliku zapisu już nie ma na dysku.",
  "error.saveRead": "Nie udało się odczytać tego pliku zapisu: {detail}",
  "error.saveFormat": "To nie wygląda na plik zapisu gry {game}.",
  "error.validation": "„{field}” {reason}",
  "error.fieldRule": "„{field}” {reason}",
  "error.unknownField": "„{field}” nie jest polem edytowalnym w tym zapisie.",
  "error.pathNotAllowed":
    "Ten plik znajduje się poza folderami, którymi zarządza ta wtyczka.",
  "error.backupFailed":
    "Nie udało się utworzyć kopii zapasowej, więc zapis pozostał nietknięty: {detail}",
  "error.backupNotFound": "Nie znaleziono tej kopii zapasowej.",
  "error.writeFailed":
    "Nie udało się zapisać pliku: {detail} Twój oryginalny zapis pozostał bez zmian.",
  "error.io": "{detail}",
  "error.unknown": "Coś poszło nie tak.",

  "rule.notWholeNumber": "musi być liczbą całkowitą.",
  "rule.hasDecimalPoint": "musi być liczbą całkowitą, bez części dziesiętnej.",
  "rule.notANumber": "musi być liczbą.",
  "rule.notText": "musi być tekstem.",
  "rule.notABoolean": "musi być włączone lub wyłączone.",
  "rule.notAnOption": "nie jest jedną z dostępnych opcji.",
  "rule.tooSmall": "nie może być mniejsza niż {limit}.",
  "rule.tooLarge": "nie może być większa niż {limit}.",
  "rule.tooLong": "może mieć najwyżej {limit} znaków.",
  "rule.tooLargeForGame": "jest zbyt duża dla tej gry.",
  "rule.notPresent": "nie występuje w tym pliku zapisu.",

  // --- Added with the confirmation flow ---
  "confirm.title": "Na pewno?",
  "confirm.intro": "{count} wartości jest poza zakresem, który ta wtyczka uznaje za bezpieczny.",
  "confirm.intro_one": "{count} wartość jest poza zakresem, który ta wtyczka uznaje za bezpieczny.",
  "confirm.intro_few": "{count} wartości są poza zakresem, który ta wtyczka uznaje za bezpieczny.",
  "confirm.intro_many": "{count} wartości jest poza zakresem, który ta wtyczka uznaje za bezpieczny.",
  "confirm.intro_other": "{count} wartości jest poza zakresem, który ta wtyczka uznaje za bezpieczny.",
  "confirm.risk": "Gra może zachowywać się dziwnie, odrzucić zapis albo się zawiesić. Kopia zapasowa i tak powstaje wcześniej, więc da się to cofnąć.",
  "confirm.suggestedMax": "sugerowane maksimum {limit}",
  "confirm.suggestedMin": "sugerowane minimum {limit}",
  "confirm.go": "Zapisz mimo to",
  "changes.title": "Co się zmieni",
  "banner.gameRunning": "{game} jest teraz uruchomiona. Przy zamykaniu może nadpisać twoje zmiany.",
  "banner.cloud": "Steam Cloud synchronizuje ten folder. Jeśli kopia w chmurze jest nowsza, może cofnąć twoją edycję.",
  "banner.staleReload": "Ten zapis zmienił się na dysku. Przeładuj, aby go zobaczyć.",
  "banner.reload": "Przeładuj",
  "editor.discardConfirm": "Masz niezapisane zmiany. Wyjść bez zapisywania?",
  "error.saveChangedOnDisk": "Gra zmieniła ten zapis po tym, jak go otworzyłeś. Przeładuj przed zapisaniem, inaczej cofniesz to, co gra dopiero co zapisała.",
  "error.constraint": "{message}",
  "error.needsConfirmation": "Niektóre wartości są poza bezpiecznym zakresem.",

  // --- Added with quick actions, list editing and themes ---
  "presets.title": "Szybkie akcje",
  "presets.help": "Każda to zwykły zestaw zmian — z kopią zapasową i tak samo sprawdzany.",
  "editor.allSections": "Wszystko",
  "editor.searchFields": "Szukaj w polach…",
  "editor.revert": "Cofnij tę zmianę",
  "combo.search": "Wpisz, aby filtrować…",
  "combo.noMatch": "Nic nie pasuje do „{query}”.",
  "combo.more": "Jeszcze {count} — pisz dalej.",
  "list.add": "Dodaj",
  "list.remove": "Usuń",
  "list.saveFirst": "Najpierw zapisz zmiany.",
  "list.added": "Dodano. Wcześniej powstała kopia zapasowa.",
  "list.removed": "Usunięto. Wcześniej powstała kopia zapasowa.",
  "recovery.title": "Własne kopie gry",
  "recovery.help": "Kopie, które gra zrobiła dla siebie, obok twojego zapisu. Tylko odczytywane, nigdy nadpisywane. Przywrócenie jednej tworzy kopię tego, co zastępuje.",
  "recovery.compare": "Porównaj",
  "recovery.use": "Przywróć tę",
  "recovery.identical": "Identyczna z twoim bieżącym zapisem.",
  "recovery.andMore": "…i jeszcze {count}.",
  "settings.appearance": "Wygląd",
  "settings.theme": "Motyw",
  "settings.themeSystem": "Jak w systemie",
  "settings.themeLight": "Jasny",
  "settings.themeDark": "Ciemny",
  "error.listNotEditable": "W „{list}” nie można dodawać ani usuwać wierszy.",
  "error.listFull": "„{list}” nie może mieć więcej niż {max} wpisów.",
  "error.listAtMinimum": "„{list}” musi zachować co najmniej {min} wpisów.",
};
