import type { LocaleStrings } from "..";

export const ptBR: LocaleStrings = {
  "app.title": "Universal Save Editor",
  "app.pickGame": "Escolha um jogo para começar",
  "app.chooseSave": "Escolha qual save editar",
  "app.backups": "Backups",
  "app.backupsSub": "Devolva qualquer save ao que era",
  "app.settings": "Configurações",
  "app.back": "Voltar",

  "common.loading": "Carregando…",
  "common.cancel": "Cancelar",

  "games.search": "Buscar jogos…",
  "games.noneInstalled": "Nenhum plugin de jogo instalado.",
  "games.addPlugin": "Coloque um na sua pasta de plugins — veja as Configurações.",
  "games.noMatch": "Nenhum jogo corresponde a “{query}”.",
  "games.moreSoon":
    "Mais jogos em breve — cada um é um plugin, e qualquer pessoa pode escrever um.",

  "saves.looking": "Procurando arquivos de save…",
  "saves.none": "Nenhum save de {game} foi encontrado neste computador.",
  "saves.playOnce": "Jogue uma vez para criar um save e depois volte aqui.",
  "saves.found_one":
    "{count} save encontrado. Nada muda até você escolher um e clicar em Salvar.",
  "saves.found_other":
    "{count} saves encontrados. Nada muda até você escolher um e clicar em Salvar.",
  "saves.lastPlayed": "Jogado pela última vez em {when}",

  "editor.opening": "Abrindo o save…",
  "editor.save": "Salvar alterações",
  "editor.saving": "Salvando…",
  "editor.noChanges": "Nenhuma alteração ainda",
  "editor.ready_one": "{count} alteração pronta",
  "editor.ready_other": "{count} alterações prontas",
  "editor.needFixing_one": "{count} valor precisa de correção",
  "editor.needFixing_other": "{count} valores precisam de correção",
  "editor.savedNothing": "Não havia nada para mudar — seu save ficou intacto.",
  "editor.saved_one":
    "Salvo. {count} valor atualizado, e antes foi criado um backup do original.",
  "editor.saved_other":
    "Salvo. {count} valores atualizados, e antes foi criado um backup do original.",
  "editor.backupNote":
    "Um backup é criado antes de o save ser modificado. Você pode restaurá-lo a qualquer momento em Backups.",
  "editor.notPresent": "Não existe neste arquivo de save.",
  "editor.nothingHere": "Nada aqui neste save.",
  "editor.notInList": "{value} (fora da lista)",

  "backups.none": "Ainda não há backups.",
  "backups.autoNote":
    "Um é criado automaticamente toda vez que você salva uma alteração.",
  "backups.restore": "Restaurar",
  "backups.delete": "Excluir",
  "backups.confirm": "Substituir o save atual?",
  "backups.yesRestore": "Sim, restaurar",
  "backups.restored":
    "Restaurado. Seu save voltou ao que era, e a versão substituída também foi guardada.",

  "settings.language": "Idioma",
  "settings.languageDesc":
    "Também usado para os rótulos que o plugin de um jogo fornece.",
  "settings.backupsDesc":
    "Todos os backups que este aplicativo já fez ficam aqui.",
  "settings.openFolder": "Abrir pasta",
  "settings.plugins": "Plugins",
  "settings.pluginsDesc":
    "Uma pasta por jogo. Coloque um plugin em qualquer uma delas e clique em Recarregar.",
  "settings.reload": "Recarregar plugins",
  "settings.reloaded_one": "{count} jogo disponível.",
  "settings.reloaded_other": "{count} jogos disponíveis.",
  "settings.failedPlugins": "Plugins que não puderam ser carregados",
  "settings.cantOpenFolder": "Não foi possível abrir essa pasta.",
  "settings.about": "Sobre",
  "settings.aboutText":
    "Universal Save Editor {version} — edite seus saves offline sem mexer nos arquivos brutos. Tudo roda neste computador; nada é enviado para lugar nenhum.",

  "field.enterValue": "Digite um valor.",
  "field.enterNumber": "Digite um número.",
  "field.wholeNumber": "Digite um número inteiro, sem vírgula.",
  "field.tooSmall": "Não pode ser menor que {limit}.",
  "field.tooLarge": "Não pode ser maior que {limit}.",
  "field.tooLong": "Deve ter no máximo {limit} caracteres.",

  "error.pluginNotFound": "O plugin “{id}” não foi encontrado.",
  "error.pluginLoad": "Não foi possível ler a pasta do plugin: {detail}",
  "error.saveMissing": "Este arquivo de save não existe mais no disco.",
  "error.saveRead": "Não foi possível ler este arquivo de save: {detail}",
  "error.saveFormat": "Isto não parece um arquivo de save de {game}.",
  "error.validation": "“{field}” {reason}",
  "error.fieldRule": "“{field}” {reason}",
  "error.unknownField": "“{field}” não é um campo editável neste save.",
  "error.pathNotAllowed":
    "Esse arquivo está fora das pastas que este plugin gerencia.",
  "error.backupFailed":
    "Não foi possível criar o backup, então seu save ficou intacto: {detail}",
  "error.backupNotFound": "Esse backup não foi encontrado.",
  "error.writeFailed":
    "Não foi possível gravar o save: {detail} Seu save original está inalterado.",
  "error.io": "{detail}",
  "error.unknown": "Algo deu errado.",

  "rule.notWholeNumber": "deve ser um número inteiro.",
  "rule.hasDecimalPoint": "deve ser um número inteiro, sem vírgula.",
  "rule.notANumber": "deve ser um número.",
  "rule.notText": "deve ser texto.",
  "rule.notABoolean": "deve estar ligado ou desligado.",
  "rule.notAnOption": "não é uma das opções disponíveis.",
  "rule.tooSmall": "não pode ser menor que {limit}.",
  "rule.tooLarge": "não pode ser maior que {limit}.",
  "rule.tooLong": "deve ter no máximo {limit} caracteres.",
  "rule.tooLargeForGame": "é grande demais para este jogo.",
  "rule.notPresent": "não existe neste arquivo de save.",

  // --- Added with the confirmation flow ---
  "confirm.title": "Tem certeza?",
  "confirm.intro": "{count} valores estão fora da faixa que este plugin considera segura.",
  "confirm.intro_one": "{count} valor está fora da faixa que este plugin considera segura.",
  "confirm.intro_other": "{count} valores estão fora da faixa que este plugin considera segura.",
  "confirm.risk": "O jogo pode se comportar de forma estranha, recusar o save ou travar. Mesmo assim um backup é feito antes, então dá para desfazer.",
  "confirm.suggestedMax": "máximo sugerido {limit}",
  "confirm.suggestedMin": "mínimo sugerido {limit}",
  "confirm.go": "Salvar assim mesmo",
  "changes.title": "O que vai mudar",
  "banner.gameRunning": "{game} está aberto agora. Ele pode sobrescrever suas alterações ao fechar.",
  "banner.cloud": "O Steam Cloud sincroniza esta pasta. Se a cópia na nuvem for mais recente, ela pode desfazer sua edição.",
  "banner.staleReload": "Este save mudou no disco. Recarregue para vê-lo.",
  "banner.reload": "Recarregar",
  "editor.discardConfirm": "Você tem alterações não salvas. Sair sem salvar?",
  "error.saveChangedOnDisk": "O jogo alterou este save depois que você o abriu. Recarregue antes de salvar, ou você desfaria o que o jogo acabou de gravar.",
  "error.constraint": "{message}",
  "error.needsConfirmation": "Alguns valores estão fora da faixa segura.",

  // --- Added with quick actions, list editing and themes ---
  "presets.title": "Ações rápidas",
  "presets.help": "Cada uma é um conjunto comum de alterações — com backup e verificada do mesmo jeito.",
  "editor.allSections": "Tudo",
  "editor.searchFields": "Buscar campos…",
  "editor.revert": "Desfazer esta alteração",
  "combo.search": "Digite para filtrar…",
  "combo.noMatch": "Nada corresponde a “{query}”.",
  "combo.more": "Mais {count} — continue digitando.",
  "list.add": "Adicionar",
  "list.remove": "Remover",
  "list.saveFirst": "Salve suas alterações primeiro.",
  "list.added": "Adicionado. Um backup foi feito antes.",
  "list.removed": "Removido. Um backup foi feito antes.",
  "recovery.title": "Os backups do próprio jogo",
  "recovery.help": "Cópias que este jogo fez para si mesmo, ao lado do seu save. Apenas lidas, nunca escritas. Restaurar uma faz backup do que ela substitui.",
  "recovery.compare": "Comparar",
  "recovery.use": "Restaurar esta",
  "recovery.identical": "Idêntica ao seu save atual.",
  "recovery.andMore": "…e mais {count}.",
  "settings.appearance": "Aparência",
  "settings.theme": "Tema",
  "settings.themeSystem": "Igual ao sistema",
  "settings.themeLight": "Claro",
  "settings.themeDark": "Escuro",
  "error.listNotEditable": "“{list}” não permite adicionar nem remover linhas.",
  "error.listFull": "“{list}” não pode ter mais de {max} entradas.",
  "error.listAtMinimum": "“{list}” precisa manter pelo menos {min} entradas.",
};
