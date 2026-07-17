import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { enable, disable } from "@tauri-apps/plugin-autostart";
import { applyLanguage, t } from "./i18n";

export interface Settings {
  storageDir?: string;
  shortcut?: string;
  opacity?: number;
  autostart?: boolean;
  theme?: string;
  openBehavior?: string;
  language?: string;
}

let settings: Settings = {};
let recording = false;

const PRESET_SHORTCUTS = ["Cmd+Shift+N", "Cmd+Shift+E"];

function $(id: string): HTMLInputElement | HTMLSelectElement | HTMLButtonElement | null {
  return document.getElementById(id) as HTMLInputElement | HTMLSelectElement | HTMLButtonElement | null;
}

function applyOpacity(v: number) {
  document.documentElement.style.setProperty("--window-opacity", String(v));
}

function applyTheme(theme: string) {
  const root = document.documentElement;
  root.removeAttribute("data-theme");
  if (theme === "light" || theme === "dark") {
    root.setAttribute("data-theme", theme);
  }
}

export function applyAppearance(s: Settings) {
  applyOpacity(s.opacity ?? 0.85);
  applyTheme(s.theme ?? "system");
}

function fillForm() {
  ($("set-storage") as HTMLInputElement)!.value = settings.storageDir ?? "";
  ($("set-opacity") as HTMLInputElement)!.value = String(settings.opacity ?? 0.85);
  ($("set-open-behavior") as HTMLSelectElement)!.value = settings.openBehavior ?? "last";
  ($("set-theme") as HTMLSelectElement)!.value = settings.theme ?? "system";
  ($("set-language") as HTMLSelectElement)!.value = settings.language ?? "zh";
  ($("set-autostart") as HTMLInputElement)!.checked = settings.autostart ?? false;
  const sel = $("set-shortcut") as HTMLSelectElement;
  const customInput = $("set-shortcut-custom") as HTMLInputElement;
  const sc = settings.shortcut ?? "Cmd+Shift+N";
  if (sc === "" || PRESET_SHORTCUTS.includes(sc)) {
    sel.value = sc;
    customInput.hidden = true;
  } else {
    sel.value = "__custom";
    customInput.hidden = false;
    customInput.value = sc;
  }
}

async function persist() {
  try {
    await invoke("set_settings", { settings });
  } catch (e) {
    console.error("保存设置失败", e);
  }
}

export async function openSettings() {
  settings = await invoke<Settings>("get_settings");
  applyLanguage(settings.language ?? "zh");
  fillForm();
  applyAppearance(settings);
  const panel = document.getElementById("settings-panel");
  if (panel) panel.hidden = false;
}

export function initSettings() {
  document.getElementById("btn-close-settings")?.addEventListener("click", () => {
    const panel = document.getElementById("settings-panel");
    if (panel) panel.hidden = true;
  });

  $("btn-pick-storage")?.addEventListener("click", async () => {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === "string") {
      settings.storageDir = dir;
      ($("set-storage") as HTMLInputElement)!.value = dir;
      await persist();
    }
  });

  ($("set-opacity") as HTMLInputElement)?.addEventListener("input", async (e) => {
    const v = parseFloat((e.target as HTMLInputElement).value);
    settings.opacity = v;
    applyOpacity(v);
    await persist();
  });

  ($("set-open-behavior") as HTMLSelectElement)?.addEventListener("change", async (e) => {
    settings.openBehavior = (e.target as HTMLSelectElement).value;
    await persist();
  });

  ($("set-theme") as HTMLSelectElement)?.addEventListener("change", async (e) => {
    const v = (e.target as HTMLSelectElement).value;
    settings.theme = v;
    applyTheme(v);
    await persist();
  });

  ($("set-language") as HTMLSelectElement)?.addEventListener("change", async (e) => {
    const v = (e.target as HTMLSelectElement).value;
    settings.language = v;
    applyLanguage(v);
    await persist();
  });

  ($("set-autostart") as HTMLInputElement)?.addEventListener("change", async (e) => {
    const on = (e.target as HTMLInputElement).checked;
    settings.autostart = on;
    try {
      if (on) await enable();
      else await disable();
    } catch (err) {
      console.error("自启设置失败", err);
    }
    await persist();
  });

  initShortcutRecorder();
}

function initShortcutRecorder() {
  const sel = $("set-shortcut") as HTMLSelectElement;
  const customInput = $("set-shortcut-custom") as HTMLInputElement;
  if (!sel || !customInput) return;

  sel.addEventListener("change", async () => {
    const v = sel.value;
    if (v === "__custom") {
      customInput.hidden = false;
      customInput.value = "";
      customInput.placeholder = t("recording");
      customInput.classList.add("recording");
      recording = true;
      customInput.focus();
    } else {
      customInput.hidden = true;
      settings.shortcut = v;
      await persist();
    }
  });

  customInput.addEventListener("click", () => {
    recording = true;
    customInput.value = "";
    customInput.placeholder = t("recording");
    customInput.classList.add("recording");
  });

  customInput.addEventListener("keydown", (e) => {
    if (!recording) return;
    e.preventDefault();
    if (e.key === "Escape") {
      recording = false;
      customInput.classList.remove("recording");
      customInput.value = settings.shortcut ?? "";
      customInput.placeholder = t("shortcut_hint");
      return;
    }
    if (["Meta", "Control", "Alt", "Shift"].includes(e.key)) return;
    const parts: string[] = [];
    if (e.metaKey) parts.push("Cmd");
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    let key = e.key;
    if (key === " ") key = "Space";
    else if (key.length === 1) key = key.toUpperCase();
    parts.push(key);
    const combo = parts.join("+");
    recording = false;
    customInput.classList.remove("recording");
    customInput.value = combo;
    customInput.placeholder = t("shortcut_hint");
    settings.shortcut = combo;
    void persist();
  });

  customInput.addEventListener("blur", () => {
    if (recording) {
      recording = false;
      customInput.classList.remove("recording");
      customInput.value = settings.shortcut ?? "";
      customInput.placeholder = t("shortcut_hint");
    }
  });
}
