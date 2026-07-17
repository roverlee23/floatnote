import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { TaskList } from "@tiptap/extension-task-list";
import { TaskItem } from "@tiptap/extension-task-item";
import { Underline } from "@tiptap/extension-underline";
import { Markdown } from "tiptap-markdown";
import { openSettings, initSettings, applyAppearance, type Settings } from "./settings";
import { t, applyLanguage } from "./i18n";
import "@phosphor-icons/web/regular";
import "@phosphor-icons/web/fill";

let editor: Editor;
let saveTimer: number | null = null;
let locked = false;

async function initWindowPersistence() {
  const win = getCurrentWindow();
  let timer: number | null = null;
  const save = () => {
    if (timer) window.clearTimeout(timer);
    timer = window.setTimeout(async () => {
      try {
        const size = await win.outerSize();
        const pos = await win.outerPosition();
        await invoke("save_window_rect", {
          rect: { x: pos.x, y: pos.y, w: size.width, h: size.height },
        });
      } catch (e) {
        console.error(e);
      }
    }, 500);
  };
  await listen("tauri://resize", save);
  await listen("tauri://move", save);
}

async function init() {
  try {
    const s = await invoke<Settings>("get_settings");
    applyAppearance(s);
    applyLanguage(s.language ?? "zh");
    editor = new Editor({
      element: document.getElementById("editor")!,
      extensions: [
        StarterKit.configure({ heading: { levels: [2, 3] } }),
        Placeholder.configure({ placeholder: t("editor_placeholder") }),
        TaskList,
        TaskItem.configure({ nested: true }),
        Underline,
        Markdown.configure({ breaks: true }),
      ],
      content: "",
      autofocus: true,
      onUpdate: () => {
        scheduleSave();
        updateActiveButtons();
      },
    });
    bindToolbar();
    bindTitlebar();
    initSettings();
    document.addEventListener("mousedown", () => {
      void invoke("make_key_window");
    });
    await loadCurrentNote();
    await listen("open-settings", () => void openSettings());
    await initWindowPersistence();
    document.getElementById("btn-lock")?.addEventListener("mousedown", (e) => {
      locked = !locked;
      const btn = e.currentTarget as HTMLElement;
      btn.classList.toggle("is-locked", locked);
      const icon = btn.querySelector("i");
      if (icon) icon.className = locked ? "ph-fill ph-push-pin" : "ph ph-push-pin";
      e.preventDefault();
    });
    await getCurrentWindow().onFocusChanged((event) => {
      if (!event.payload && !locked) {
        void getCurrentWindow().hide();
      }
    });
  } catch (e) {
    console.error("初始化失败", e);
    setStatus(t("init_fail") + ": " + String(e), "unsaved");
  }
}

async function loadCurrentNote() {
  try {
    const note = await invoke<{ content: string; filename: string; createdAt: string } | null>(
      "load_current_note"
    );
    if (note && note.content) {
      editor.commands.setContent(note.content, false);
      updateMeta(note.filename);
      setStatus(t("ready"), "");
    } else {
      updateMeta("");
      setStatus(t("new_short"), "");
    }
  } catch (e) {
    setStatus(t("load_fail"), "unsaved");
    console.error(e);
  }
}

function scheduleSave() {
  setStatus(t("saving"), "saving");
  if (saveTimer) window.clearTimeout(saveTimer);
  saveTimer = window.setTimeout(() => {
    void saveNote();
  }, 500);
}

async function saveNote() {
  if (editor.isEmpty) {
    setStatus(t("new_short"), "");
    return;
  }
  const md = editor.storage.markdown.getMarkdown();
  try {
    const res = await invoke<{ filename: string }>("save_note", { content: md });
    updateMeta(res.filename);
    setStatus(t("saved"), "");
  } catch (e) {
    setStatus(t("save_fail"), "unsaved");
    console.error(e);
  }
}

async function newNote() {
  if (saveTimer) {
    window.clearTimeout(saveTimer);
    saveTimer = null;
    await saveNote();
  }
  try {
    await invoke("start_new_note");
  } catch (e) {
    console.error(e);
  }
  editor.commands.clearContent();
  updateMeta("");
  setStatus(t("new_short"), "");
  editor.commands.focus();
}

function bindToolbar() {
  document.querySelectorAll<HTMLButtonElement>(".tool-btn[data-cmd]").forEach((btn) => {
    btn.addEventListener("click", () => {
      const cmd = btn.dataset.cmd!;
      switch (cmd) {
        case "bold":
          editor.chain().focus().toggleBold().run();
          break;
        case "italic":
          editor.chain().focus().toggleItalic().run();
          break;
        case "underline":
          editor.chain().focus().toggleUnderline().run();
          break;
        case "strike":
          editor.chain().focus().toggleStrike().run();
          break;
        case "h2":
          editor.chain().focus().toggleHeading({ level: 2 }).run();
          break;
        case "h3":
          editor.chain().focus().toggleHeading({ level: 3 }).run();
          break;
        case "bulletList":
          editor.chain().focus().toggleBulletList().run();
          break;
        case "orderedList":
          editor.chain().focus().toggleOrderedList().run();
          break;
        case "taskList":
          editor.chain().focus().toggleTaskList().run();
          break;
        case "codeBlock":
          editor.chain().focus().toggleCodeBlock().run();
          break;
      }
    });
  });
  updateActiveButtons();
}

function updateActiveButtons() {
  document.querySelectorAll<HTMLButtonElement>(".tool-btn[data-cmd]").forEach((btn) => {
    const cmd = btn.dataset.cmd!;
    let active = false;
    switch (cmd) {
      case "bold":
        active = editor.isActive("bold");
        break;
      case "italic":
        active = editor.isActive("italic");
        break;
      case "underline":
        active = editor.isActive("underline");
        break;
      case "strike":
        active = editor.isActive("strike");
        break;
      case "h2":
        active = editor.isActive("heading", { level: 2 });
        break;
      case "h3":
        active = editor.isActive("heading", { level: 3 });
        break;
      case "bulletList":
        active = editor.isActive("bulletList");
        break;
      case "orderedList":
        active = editor.isActive("orderedList");
        break;
      case "taskList":
        active = editor.isActive("taskList");
        break;
      case "codeBlock":
        active = editor.isActive("codeBlock");
        break;
    }
    btn.classList.toggle("is-active", active);
  });
}

function bindTitlebar() {
  document.getElementById("btn-new")?.addEventListener("click", () => void newNote());
  document.getElementById("btn-settings")?.addEventListener("click", openSettings);
  document.getElementById("tl-close")?.addEventListener("click", () => void getCurrentWindow().hide());
  document.getElementById("tl-min")?.addEventListener("click", () => void getCurrentWindow().minimize());
  document.getElementById("tl-max")?.addEventListener("click", () => void getCurrentWindow().toggleMaximize());
}

function setStatus(text: string, cls: string) {
  const el = document.getElementById("status");
  if (el) {
    el.textContent = text;
    el.className = "status" + (cls ? " " + cls : "");
  }
}

function updateMeta(filename: string) {
  const el = document.getElementById("meta");
  if (el) el.textContent = filename;
}

window.addEventListener("unhandledrejection", (e) => {
  console.error("未捕获错误", e.reason);
  setStatus(t("error_prefix") + ": " + String(e.reason), "unsaved");
});
window.addEventListener("error", (e) => {
  setStatus(t("error_prefix") + ": " + e.message, "unsaved");
});

init();
