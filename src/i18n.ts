const dict: Record<string, Record<string, string>> = {
  zh: {
    title: "便签", new_note: "新建便签", lock: "锁定置顶", settings: "设置", close: "关闭",
    bold: "加粗", italic: "斜体", underline: "下划线", strike: "删除线",
    h2: "标题", h3: "副标题", bullet: "无序列表", ordered: "有序列表", task: "待办", code: "代码块", link: "链接",
    ready: "已就绪", saving: "保存中…", saved: "已保存", new_short: "新便签",
    load_fail: "加载失败", save_fail: "保存失败", init_fail: "初始化失败", error_prefix: "错误",
    editor_placeholder: "在这里记点什么…", link_prompt: "链接地址：",
    storage: "存储位置", storage_hint: "建议放到 Obsidian 的 daily 文件夹，方便和日记一起回顾",
    opacity: "透明度", open_when: "打开时", open_last: "继续上次的便签", open_new: "总是新建便签",
    theme: "主题", theme_system: "跟随系统", theme_light: "浅色", theme_dark: "深色",
    language: "语言", autostart: "开机自启动", shortcut: "唤起快捷键",
    pick: "选择…", clear: "清除", shortcut_hint: "点击录制", recording: "按下组合键…（Esc 取消）", custom: "自定义…", disabled: "禁用",
  },
  en: {
    title: "Notes", new_note: "New note", lock: "Lock on top", settings: "Settings", close: "Close",
    bold: "Bold", italic: "Italic", underline: "Underline", strike: "Strikethrough",
    h2: "Heading 2", h3: "Heading 3", bullet: "Bullet list", ordered: "Numbered list", task: "Task", code: "Code block", link: "Link",
    ready: "Ready", saving: "Saving…", saved: "Saved", new_short: "New note",
    load_fail: "Load failed", save_fail: "Save failed", init_fail: "Init failed", error_prefix: "Error",
    editor_placeholder: "Jot something down…", link_prompt: "Link URL:",
    storage: "Storage", storage_hint: "Tip: put it in your Obsidian daily folder to review with your journal",
    opacity: "Opacity", open_when: "On open", open_last: "Continue last note", open_new: "Always new note",
    theme: "Theme", theme_system: "System", theme_light: "Light", theme_dark: "Dark",
    language: "Language", autostart: "Launch at login", shortcut: "Show shortcut",
    pick: "Choose…", clear: "Clear", shortcut_hint: "Click to record", recording: "Press a key combo… (Esc to cancel)", custom: "Custom…", disabled: "Disabled",
  },
  es: {
    title: "Notas", new_note: "Nueva nota", lock: "Fijar arriba", settings: "Ajustes", close: "Cerrar",
    bold: "Negrita", italic: "Cursiva", underline: "Subrayado", strike: "Tachado",
    h2: "Título 2", h3: "Título 3", bullet: "Lista", ordered: "Lista numerada", task: "Tarea", code: "Código", link: "Enlace",
    ready: "Listo", saving: "Guardando…", saved: "Guardado", new_short: "Nueva nota",
    load_fail: "Error al cargar", save_fail: "Error al guardar", init_fail: "Error de inicio", error_prefix: "Error",
    editor_placeholder: "Escribe algo…", link_prompt: "URL del enlace:",
    storage: "Almacenamiento", storage_hint: "Sugerencia: ponlo en la carpeta daily de Obsidian para revisarlo con tu diario",
    opacity: "Opacidad", open_when: "Al abrir", open_last: "Continuar última nota", open_new: "Siempre nueva nota",
    theme: "Tema", theme_system: "Sistema", theme_light: "Claro", theme_dark: "Oscuro",
    language: "Idioma", autostart: "Abrir al iniciar", shortcut: "Atajo",
    pick: "Elegir…", clear: "Limpiar", shortcut_hint: "Click para grabar", recording: "Pulsa una combinación… (Esc para cancelar)", custom: "Personalizado…", disabled: "Desactivado",
  },
  fr: {
    title: "Notes", new_note: "Nouvelle note", lock: "Épingler", settings: "Réglages", close: "Fermer",
    bold: "Gras", italic: "Italique", underline: "Souligné", strike: "Barré",
    h2: "Titre 2", h3: "Titre 3", bullet: "Liste", ordered: "Liste numérotée", task: "Tâche", code: "Code", link: "Lien",
    ready: "Prêt", saving: "Enregistrement…", saved: "Enregistré", new_short: "Nouvelle note",
    load_fail: "Échec du chargement", save_fail: "Échec de l'enregistrement", init_fail: "Échec de l'init", error_prefix: "Erreur",
    editor_placeholder: "Notez quelque chose…", link_prompt: "URL du lien :",
    storage: "Stockage", storage_hint: "Astuce : mettez-le dans le dossier daily d'Obsidian pour le revoir avec votre journal",
    opacity: "Opacité", open_when: "À l'ouverture", open_last: "Continuer la dernière note", open_new: "Toujours nouvelle note",
    theme: "Thème", theme_system: "Système", theme_light: "Clair", theme_dark: "Sombre",
    language: "Langue", autostart: "Ouvrir à la connexion", shortcut: "Raccourci",
    pick: "Choisir…", clear: "Effacer", shortcut_hint: "Cliquez pour enregistrer", recording: "Appuyez sur une combinaison… (Esc pour annuler)", custom: "Personnalisé…", disabled: "Désactivé",
  },
  de: {
    title: "Notizen", new_note: "Neue Notiz", lock: "Oben halten", settings: "Einstellungen", close: "Schließen",
    bold: "Fett", italic: "Kursiv", underline: "Unterstrichen", strike: "Durchgestrichen",
    h2: "Überschrift 2", h3: "Überschrift 3", bullet: "Liste", ordered: "Nummerierte Liste", task: "Aufgabe", code: "Code", link: "Link",
    ready: "Bereit", saving: "Speichern…", saved: "Gespeichert", new_short: "Neue Notiz",
    load_fail: "Laden fehlgeschlagen", save_fail: "Speichern fehlgeschlagen", init_fail: "Init fehlgeschlagen", error_prefix: "Fehler",
    editor_placeholder: "Notiere etwas…", link_prompt: "Link-URL:",
    storage: "Speicherort", storage_hint: "Tipp: Lege ihn in den Obsidian Daily-Ordner, um ihn mit deinem Journal zu überprüfen",
    opacity: "Deckkraft", open_when: "Beim Öffnen", open_last: "Letzte Notiz fortsetzen", open_new: "Immer neue Notiz",
    theme: "Design", theme_system: "System", theme_light: "Hell", theme_dark: "Dunkel",
    language: "Sprache", autostart: "Bei Anmeldung öffnen", shortcut: "Tastenkürzel",
    pick: "Wählen…", clear: "Löschen", shortcut_hint: "Klicken zum Aufnehmen", recording: "Tastenkombination drücken… (Esc zum Abbrechen)", custom: "Benutzerdefiniert…", disabled: "Deaktiviert",
  },
  it: {
    title: "Note", new_note: "Nuova nota", lock: "Fissa in alto", settings: "Impostazioni", close: "Chiudi",
    bold: "Grassetto", italic: "Corsivo", underline: "Sottolineato", strike: "Barrato",
    h2: "Titolo 2", h3: "Titolo 3", bullet: "Elenco", ordered: "Elenco numerato", task: "Attività", code: "Codice", link: "Link",
    ready: "Pronto", saving: "Salvataggio…", saved: "Salvato", new_short: "Nuova nota",
    load_fail: "Caricamento fallito", save_fail: "Salvataggio fallito", init_fail: "Init fallita", error_prefix: "Errore",
    editor_placeholder: "Scrivi qualcosa…", link_prompt: "URL del link:",
    storage: "Posizione", storage_hint: "Suggerimento: mettilo nella cartella daily di Obsidian per rivederlo con il tuo diario",
    opacity: "Opacità", open_when: "All'apertura", open_last: "Continua ultima nota", open_new: "Sempre nuova nota",
    theme: "Tema", theme_system: "Sistema", theme_light: "Chiaro", theme_dark: "Scuro",
    language: "Lingua", autostart: "Apri al login", shortcut: "Scorciatoia",
    pick: "Scegli…", clear: "Cancella", shortcut_hint: "Clicca per registrare", recording: "Premi una combinazione… (Esc per annullare)", custom: "Personalizzato…", disabled: "Disattivato",
  },
  ja: {
    title: "ノート", new_note: "新規ノート", lock: "最前面に固定", settings: "設定", close: "閉じる",
    bold: "太字", italic: "斜体", underline: "下線", strike: "取り消し線",
    h2: "見出し2", h3: "見出し3", bullet: "箇条書き", ordered: "番号付きリスト", task: "タスク", code: "コード", link: "リンク",
    ready: "準備完了", saving: "保存中…", saved: "保存済み", new_short: "新規ノート",
    load_fail: "読み込み失敗", save_fail: "保存失敗", init_fail: "初期化失敗", error_prefix: "エラー",
    editor_placeholder: "何かメモする…", link_prompt: "リンクURL：",
    storage: "保存先", storage_hint: "ヒント：Obsidianのdailyフォルダに入れると日記と一緒に見返せます",
    opacity: "不透明度", open_when: "開く時", open_last: "前回のノートを続ける", open_new: "常に新規ノート",
    theme: "テーマ", theme_system: "システム", theme_light: "ライト", theme_dark: "ダーク",
    language: "言語", autostart: "ログイン時に起動", shortcut: "ショートカット",
    pick: "選択…", clear: "クリア", shortcut_hint: "クリックして録音", recording: "キー組み合わせを押す…（Escでキャンセル）", custom: "カスタム…", disabled: "無効",
  },
  ar: {
    title: "ملاحظات", new_note: "ملاحظة جديدة", lock: "تثبيت بالأعلى", settings: "الإعدادات", close: "إغلاق",
    bold: "عريض", italic: "مائل", underline: "تحته خط", strike: "يتوسطه خط",
    h2: "عنوان 2", h3: "عنوان 3", bullet: "قائمة نقطية", ordered: "قائمة مرقمة", task: "مهمة", code: "كود", link: "رابط",
    ready: "جاهز", saving: "جارٍ الحفظ…", saved: "تم الحفظ", new_short: "ملاحظة جديدة",
    load_fail: "فشل التحميل", save_fail: "فشل الحفظ", init_fail: "فشل التهيئة", error_prefix: "خطأ",
    editor_placeholder: "اكتب شيئاً…", link_prompt: "رابط URL:",
    storage: "موقع التخزين", storage_hint: "نصيحة: ضعه في مجلد daily في Obsidian لمراجعته مع يومياتك",
    opacity: "الشفافية", open_when: "عند الفتح", open_last: "متابعة آخر ملاحظة", open_new: "دائماً ملاحظة جديدة",
    theme: "السمة", theme_system: "النظام", theme_light: "فاتح", theme_dark: "داكن",
    language: "اللغة", autostart: "فتح عند تسجيل الدخول", shortcut: "اختصار",
    pick: "اختيار…", clear: "مسح", shortcut_hint: "انقر للتسجيل", recording: "اضغط مجموعة مفاتيح… (Esc للإلغاء)", custom: "مخصص…", disabled: "معطل",
  },
  pt: {
    title: "Notas", new_note: "Nova nota", lock: "Fixar no topo", settings: "Configurações", close: "Fechar",
    bold: "Negrito", italic: "Itálico", underline: "Sublinhado", strike: "Tachado",
    h2: "Título 2", h3: "Título 3", bullet: "Lista", ordered: "Lista numerada", task: "Tarefa", code: "Código", link: "Link",
    ready: "Pronto", saving: "Salvando…", saved: "Salvo", new_short: "Nova nota",
    load_fail: "Falha ao carregar", save_fail: "Falha ao salvar", init_fail: "Falha na init", error_prefix: "Erro",
    editor_placeholder: "Anote algo…", link_prompt: "URL do link:",
    storage: "Armazenamento", storage_hint: "Dica: coloque na pasta daily do Obsidian para revisar com seu diário",
    opacity: "Opacidade", open_when: "Ao abrir", open_last: "Continuar última nota", open_new: "Sempre nova nota",
    theme: "Tema", theme_system: "Sistema", theme_light: "Claro", theme_dark: "Escuro",
    language: "Idioma", autostart: "Abrir ao iniciar", shortcut: "Atalho",
    pick: "Escolher…", clear: "Limpar", shortcut_hint: "Clique para gravar", recording: "Pressione uma combinação… (Esc para cancelar)", custom: "Personalizado…", disabled: "Desativado",
  },
  ru: {
    title: "Заметки", new_note: "Новая заметка", lock: "Закрепить", settings: "Настройки", close: "Закрыть",
    bold: "Жирный", italic: "Курсив", underline: "Подчёркнутый", strike: "Зачёркнутый",
    h2: "Заголовок 2", h3: "Заголовок 3", bullet: "Список", ordered: "Нумерованный список", task: "Задача", code: "Код", link: "Ссылка",
    ready: "Готово", saving: "Сохранение…", saved: "Сохранено", new_short: "Новая заметка",
    load_fail: "Ошибка загрузки", save_fail: "Ошибка сохранения", init_fail: "Ошибка инициализации", error_prefix: "Ошибка",
    editor_placeholder: "Запишите что-нибудь…", link_prompt: "URL ссылки:",
    storage: "Хранилище", storage_hint: "Совет: поместите в папку daily в Obsidian, чтобы пересматривать вместе с дневником",
    opacity: "Непрозрачность", open_when: "При открытии", open_last: "Продолжить последнюю", open_new: "Всегда новая",
    theme: "Тема", theme_system: "Система", theme_light: "Светлая", theme_dark: "Тёмная",
    language: "Язык", autostart: "Запуск при входе", shortcut: "Горячая клавиша",
    pick: "Выбрать…", clear: "Очистить", shortcut_hint: "Нажмите для записи", recording: "Нажмите сочетание… (Esc для отмены)", custom: "Своё…", disabled: "Отключено",
  },
};

let lang = "zh";

export function t(key: string): string {
  return dict[lang]?.[key] ?? dict.zh[key] ?? key;
}

export function applyLanguage(l: string) {
  lang = l || "zh";
  document.documentElement.lang = lang;
  // 阿拉伯语从右到左
  document.documentElement.dir = lang === "ar" ? "rtl" : "ltr";
  document.querySelectorAll<HTMLElement>("[data-i18n]").forEach((el) => {
    el.textContent = t(el.dataset.i18n!);
  });
  document.querySelectorAll<HTMLElement>("[data-i18n-title]").forEach((el) => {
    el.title = t(el.dataset.i18nTitle!);
  });
  document.querySelectorAll<HTMLElement>("[data-i18n-placeholder]").forEach((el) => {
    el.setAttribute("placeholder", t(el.dataset.i18nPlaceholder!));
  });
}
