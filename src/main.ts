import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";
import { Editor } from "@tiptap/core";
import StarterKit from "@tiptap/starter-kit";
import Placeholder from "@tiptap/extension-placeholder";
import { TaskList } from "@tiptap/extension-task-list";
import { TaskItem } from "@tiptap/extension-task-item";
import { Underline } from "@tiptap/extension-underline";
import { Table } from "@tiptap/extension-table";
import { TableRow } from "@tiptap/extension-table-row";
import { TableHeader } from "@tiptap/extension-table-header";
import { TableCell } from "@tiptap/extension-table-cell";
import { Markdown } from "tiptap-markdown";
import { openSettings, initSettings, applyAppearance, type Settings } from "./settings";
import { t, applyLanguage } from "./i18n";
import { shouldDeleteTable, tableAction, type TableAction } from "./table-actions";
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
    void invoke("append_debug_log", { message: "frontend_init" });
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
        Table.configure({ resizable: false }),
        TableRow,
        TableHeader,
        TableCell,
        Markdown.configure({ breaks: true }),
      ],
      content: "",
      autofocus: true,
      onUpdate: () => {
        scheduleSave();
        updateActiveButtons();
        updateTableHotzones();
      },
      onSelectionUpdate: () => updateTableHotzones(),
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
      void invoke("append_debug_log", { message: `frontend_pin_change locked=${locked}` });
      const btn = e.currentTarget as HTMLElement;
      btn.classList.toggle("is-locked", locked);
      const icon = btn.querySelector("i");
      if (icon) icon.className = locked ? "ph-fill ph-push-pin" : "ph ph-push-pin";
      void invoke("set_window_pinned", { pinned: locked });
      e.preventDefault();
    });
    await getCurrentWindow().onFocusChanged((event) => {
      void invoke("append_debug_log", { message: `frontend_focus_changed focused=${event.payload} locked=${locked}` });
      if (!event.payload && !locked) {
        void invoke("handle_window_focus_lost");
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
        case "table":
          editor.chain().focus().insertTable({ rows: 3, cols: 3, withHeaderRow: true }).run();
          break;
      }
    });
  });
  bindTableInteractions();
  updateActiveButtons();
  updateTableHotzones();
}

function currentTableSize(): { rows: number; columns: number } | null {
  const { $from } = editor.state.selection;
  for (let depth = $from.depth; depth > 0; depth -= 1) {
    const node = $from.node(depth);
    if (node.type.name === "table") {
      return { rows: node.childCount, columns: node.firstChild?.childCount ?? 0 };
    }
  }
  return null;
}

function runTableCommand(action: TableAction) {
  focusHoveredTable();
  if (action === "deleteTable") {
    editor.chain().focus().deleteTable().run();
    hoveredTable = null;
    updateTableHotzones();
    return;
  }
  const size = currentTableSize();
  if (!size) return;
  if (action === "deleteRow" && shouldDeleteTable("row", size.rows, size.columns)) {
    editor.chain().focus().deleteTable().run();
    hoveredTable = null;
    updateTableHotzones();
    return;
  }
  if (action === "deleteColumn" && shouldDeleteTable("column", size.rows, size.columns)) {
    editor.chain().focus().deleteTable().run();
    hoveredTable = null;
    updateTableHotzones();
    return;
  }
  editor.chain().focus()[tableAction(action)]().run();
  window.setTimeout(updateTableHotzones, 0);
}

function focusHoveredTable() {
  if (!hoveredTable) return;
  const cell = hoveredTable.querySelector("th, td");
  if (!(cell instanceof HTMLElement)) return;
  const rect = cell.getBoundingClientRect();
  const position = editor.view.posAtCoords({ left: rect.left + 8, top: rect.top + 8 });
  if (position) editor.commands.setTextSelection(position.pos);
}

const TABLE_GAP_HIT_HEIGHT = 14;

function tablePosition(table: HTMLTableElement): number | null {
  try {
    const domPosition = editor.view.posAtDOM(table, 0);
    const resolved = editor.state.doc.resolve(domPosition);
    for (let depth = resolved.depth; depth > 0; depth -= 1) {
      if (resolved.node(depth).type.name === "table") return resolved.before(depth);
    }
  } catch {
    // The table may have been removed between the hit test and this click.
  }
  return null;
}

function tableGapAt(clientX: number, clientY: number): { table: HTMLTableElement; before: boolean } | null {
  const editorElement = document.getElementById("editor");
  if (!editorElement) return null;
  const tables = editorElement.querySelectorAll<HTMLTableElement>(".tableWrapper > table");
  for (const table of tables) {
    const rect = table.getBoundingClientRect();
    const inHorizontalRange = clientX >= rect.left - 10 && clientX <= rect.right + 10;
    if (!inHorizontalRange) continue;
    if (clientY >= rect.top - TABLE_GAP_HIT_HEIGHT && clientY < rect.top) {
      return { table, before: true };
    }
    if (clientY > rect.bottom && clientY <= rect.bottom + TABLE_GAP_HIT_HEIGHT) {
      return { table, before: false };
    }
  }
  return null;
}

function insertParagraphAtTableGap(table: HTMLTableElement, before: boolean) {
  const position = tablePosition(table);
  if (position === null) return;
  const tableNode = editor.state.doc.nodeAt(position);
  if (!tableNode || tableNode.type.name !== "table") return;
  const insertAt = before ? position : position + tableNode.nodeSize;
  const paragraph = editor.schema.nodes.paragraph.create();
  editor.view.dispatch(editor.state.tr.insert(insertAt, paragraph));
  editor.commands.focus(insertAt + 1);
  editor.commands.setTextSelection(insertAt + 1);
}

let hoveredTable: HTMLTableElement | null = null;

function tableFromTarget(target: EventTarget | null): HTMLTableElement | null {
  const element = target instanceof Element ? target : null;
  const table = element?.closest("table");
  return table instanceof HTMLTableElement && document.getElementById("editor")?.contains(table)
    ? table
    : null;
}

function updateTableHotzones() {
  const editorElement = document.getElementById("editor");
  const hotzones = document.getElementById("table-hotzones");
  if (!editorElement || !hotzones || !hoveredTable || !editorElement.contains(hoveredTable)) {
    if (hotzones) hotzones.hidden = true;
    return;
  }
  const tableRect = hoveredTable.getBoundingClientRect();
  hotzones.hidden = false;
  hotzones.style.left = `${tableRect.left - 11}px`;
  hotzones.style.top = `${tableRect.top - 11}px`;
  hotzones.style.width = `${tableRect.width + 22}px`;
  hotzones.style.height = `${tableRect.height + 22}px`;
  const rowButton = hotzones.querySelector<HTMLElement>(".table-hotzone-row");
  const columnButton = hotzones.querySelector<HTMLElement>(".table-hotzone-column");
  if (rowButton) {
    rowButton.style.left = "0px";
    rowButton.style.top = `${tableRect.height}px`;
  }
  if (columnButton) {
    columnButton.style.left = `${tableRect.width}px`;
    columnButton.style.top = "0px";
  }
}

function bindTableInteractions() {
  const editorElement = document.getElementById("editor");
  const hotzones = document.getElementById("table-hotzones");
  const menu = document.getElementById("table-context-menu");
  if (!editorElement || !hotzones || !menu) return;

  editorElement.addEventListener("mousedown", (event) => {
    if (!(event instanceof MouseEvent) || event.button !== 0) return;
    if (tableFromTarget(event.target)) return;
    const gap = tableGapAt(event.clientX, event.clientY);
    if (!gap) return;
    event.preventDefault();
    insertParagraphAtTableGap(gap.table, gap.before);
  });

  editorElement.addEventListener("mouseover", (event) => {
    const table = tableFromTarget(event.target);
    if (table) {
      hoveredTable = table;
      updateTableHotzones();
    }
  });
  editorElement.addEventListener("mouseout", (event) => {
    const table = tableFromTarget(event.target);
    const related = event.relatedTarget instanceof Node ? event.relatedTarget : null;
    if (table && !table.contains(related) && !hotzones.contains(related)) {
      window.setTimeout(() => {
        if (!hotzones.matches(":hover")) {
          hoveredTable = null;
          updateTableHotzones();
        }
      }, 80);
    }
  });

  editorElement.addEventListener("contextmenu", (event) => {
    const table = tableFromTarget(event.target);
    if (!table) return;
    event.preventDefault();
    hoveredTable = table;
    const position = editor.view.posAtCoords({ left: event.clientX, top: event.clientY });
    if (position) editor.commands.setTextSelection(position.pos);
    menu.hidden = false;
    menu.style.left = `${Math.min(event.clientX, window.innerWidth - menu.offsetWidth - 8)}px`;
    menu.style.top = `${Math.min(event.clientY, window.innerHeight - menu.offsetHeight - 8)}px`;
    updateTableHotzones();
  });

  hotzones.querySelectorAll<HTMLButtonElement>("[data-table-cmd]").forEach((button) => {
    button.addEventListener("mousedown", (event) => event.stopPropagation());
    button.addEventListener("click", () => {
      runTableCommand(button.dataset.tableCmd as TableAction);
      updateTableHotzones();
    });
  });
  menu.querySelectorAll<HTMLButtonElement>("[data-table-cmd]").forEach((button) => {
    button.addEventListener("click", () => {
      runTableCommand(button.dataset.tableCmd as TableAction);
      menu.hidden = true;
      updateTableHotzones();
    });
  });
  document.addEventListener("mousedown", (event) => {
    if (!menu.contains(event.target as Node)) menu.hidden = true;
  });
  window.addEventListener("resize", updateTableHotzones);
  editorElement.addEventListener("scroll", updateTableHotzones);
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
  document.getElementById("tl-close")?.addEventListener("click", () => {
    void invoke("append_debug_log", { message: "frontend_close_button" });
    locked = false;
    document.getElementById("btn-lock")?.classList.remove("is-locked");
    const icon = document.querySelector("#btn-lock i");
    if (icon) icon.className = "ph ph-push-pin";
    void invoke("dismiss_window_from_ui");
  });
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
