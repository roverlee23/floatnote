use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;

use chrono::Local;
use serde::{Deserialize, Serialize};
use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, State, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;
use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};

#[cfg(target_os = "macos")]
mod panel {
    use objc2_app_kit::NSPanel;
    objc2::define_class!(
        #[unsafe(super(NSPanel))]
        pub struct FloatingNotesPanel;

        impl FloatingNotesPanel {
            // 重写 canBecomeKeyWindow -> true：NonactivatingPanel 默认 false，导致不能编辑/输入法
            #[unsafe(method(canBecomeKeyWindow))]
            fn can_become_key(&self) -> bool {
                true
            }
            // 重写 acceptsFirstMouse: -> true：非 key 窗口第一次点击即响应，否则要点第二下
            #[unsafe(method(acceptsFirstMouse:))]
            fn accepts_first_mouse(&self, _event: *mut objc2::runtime::AnyObject) -> bool {
                true
            }
        }
    );
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WindowRect {
    x: i32,
    y: i32,
    w: u32,
    h: u32,
}

/// 用户设置，序列化为 camelCase 供前端直接使用
#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Settings {
    storage_dir: Option<String>,
    shortcut: Option<String>,
    opacity: Option<f64>,
    autostart: Option<bool>,
    theme: Option<String>,
    open_behavior: Option<String>,
    language: Option<String>,
    window_rect: Option<WindowRect>,
}

/// 跨命令共享的运行时状态：当前正在编辑的便签文件路径
struct AppState {
    current_file: Mutex<Option<PathBuf>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NoteData {
    content: String,
    filename: String,
    created_at: String,
}

fn config_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("floating-notes"))
}

fn settings_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join("settings.json"))
}

fn load_settings() -> Settings {
    settings_path()
        .filter(|p| p.exists())
        .and_then(|p| fs::read_to_string(p).ok())
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

fn save_settings(settings: &Settings) -> Result<(), String> {
    let dir = config_dir().ok_or_else(|| "无法确定配置目录".to_string())?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(dir.join("settings.json"), json).map_err(|e| e.to_string())
}

fn default_storage_dir() -> Option<PathBuf> {
    dirs::document_dir().map(|d| d.join("悬浮便签"))
}

fn storage_dir() -> Result<PathBuf, String> {
    let s = load_settings();
    match s.storage_dir {
        Some(d) if !d.is_empty() => Ok(PathBuf::from(d)),
        _ => default_storage_dir().ok_or_else(|| "无法确定默认存储目录".to_string()),
    }
}

/// 取内容首行前 20 个合法字符，作为文件名可读后缀
fn make_filename(content: &str) -> String {
    let ts = Local::now().format("%Y-%m-%d-%H%M");
    let head: String = content
        .lines()
        .next()
        .unwrap_or("")
        .trim()
        .chars()
        .filter(|c| !matches!(c, '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\n' | '\r' | '\t'))
        .take(20)
        .collect::<String>()
        .trim()
        .replace('.', "");
    let head = if head.is_empty() { "便签".to_string() } else { head };
    format!("{}-{}.md", ts, head)
}

#[tauri::command]
fn load_current_note(state: State<AppState>) -> Result<Option<NoteData>, String> {
    let s = load_settings();
    let open_behavior = s.open_behavior.unwrap_or_else(|| "last".to_string());
    let dir = storage_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let mut current = state.current_file.lock().unwrap();
    if open_behavior == "last" {
        let mut entries: Vec<_> = fs::read_dir(&dir)
            .map_err(|e| e.to_string())?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map_or(false, |x| x == "md"))
            .collect();
        // 文件名以时间戳开头，字典序倒序即最新优先
        entries.sort_by(|a, b| b.file_name().cmp(&a.file_name()));
        if let Some(latest) = entries.first() {
            let path = latest.path();
            let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
            let filename = path.file_name().unwrap().to_string_lossy().to_string();
            *current = Some(path);
            return Ok(Some(NoteData {
                content,
                filename: filename.clone(),
                created_at: filename,
            }));
        }
    }
    *current = None;
    Ok(None)
}

#[tauri::command]
fn save_note(content: String, state: State<AppState>) -> Result<NoteData, String> {
    let dir = storage_dir()?;
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let mut current = state.current_file.lock().unwrap();
    let path = match current.clone() {
        Some(p) if p.exists() => p,
        _ => dir.join(make_filename(&content)),
    };
    fs::write(&path, &content).map_err(|e| e.to_string())?;
    let filename = path.file_name().unwrap().to_string_lossy().to_string();
    *current = Some(path);
    Ok(NoteData {
        content,
        filename: filename.clone(),
        created_at: filename,
    })
}

#[tauri::command]
fn start_new_note(state: State<AppState>) {
    *state.current_file.lock().unwrap() = None;
}

#[tauri::command]
fn get_settings() -> Settings {
    load_settings()
}

#[tauri::command]
fn set_settings(app: tauri::AppHandle, settings: Settings) -> Result<(), String> {
    save_settings(&settings)?;
    let sc_str = settings.shortcut.unwrap_or_else(|| "Cmd+Shift+N".to_string());
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    if !sc_str.is_empty() {
        if let Some(sc) = parse_shortcut(&sc_str) {
            let _ = gs.on_shortcut(sc, |app, _shortcut, event| {
                if event.state() == ShortcutState::Pressed {
                    toggle_window(app);
                }
            });
        }
    }
    Ok(())
}

#[tauri::command]
fn save_window_rect(rect: WindowRect) -> Result<(), String> {
    let mut s = load_settings();
    s.window_rect = Some(rect);
    save_settings(&s)
}

#[tauri::command]
fn make_key_window(app: tauri::AppHandle) {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::NSWindow;
        if let Some(window) = app.get_webview_window("main") {
            if let Ok(ptr) = window.ns_window() {
                unsafe {
                    let ns_window: &NSWindow = (ptr as *mut NSWindow).as_ref().expect("NSWindow");
                    ns_window.makeKeyWindow();
                    println!(
                        "[floating-notes] make_key_window: isKey={} canBecomeKey={}",
                        ns_window.isKeyWindow(),
                        ns_window.canBecomeKeyWindow()
                    );
                }
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = app;
    }
}

fn parse_code(p: &str) -> Option<Code> {
    Some(match p {
        "A" => Code::KeyA, "B" => Code::KeyB, "C" => Code::KeyC, "D" => Code::KeyD,
        "E" => Code::KeyE, "F" => Code::KeyF, "G" => Code::KeyG, "H" => Code::KeyH,
        "I" => Code::KeyI, "J" => Code::KeyJ, "K" => Code::KeyK, "L" => Code::KeyL,
        "M" => Code::KeyM, "N" => Code::KeyN, "O" => Code::KeyO, "P" => Code::KeyP,
        "Q" => Code::KeyQ, "R" => Code::KeyR, "S" => Code::KeyS, "T" => Code::KeyT,
        "U" => Code::KeyU, "V" => Code::KeyV, "W" => Code::KeyW, "X" => Code::KeyX,
        "Y" => Code::KeyY, "Z" => Code::KeyZ,
        "0" => Code::Digit0, "1" => Code::Digit1, "2" => Code::Digit2, "3" => Code::Digit3,
        "4" => Code::Digit4, "5" => Code::Digit5, "6" => Code::Digit6, "7" => Code::Digit7,
        "8" => Code::Digit8, "9" => Code::Digit9,
        "Space" => Code::Space, "Enter" => Code::Enter, "Escape" => Code::Escape, "Tab" => Code::Tab,
        _ => return None,
    })
}

fn parse_shortcut(s: &str) -> Option<Shortcut> {
    let mut mods = Modifiers::empty();
    let mut code = None;
    for part in s.split('+') {
        let p = part.trim();
        match p {
            "Cmd" | "Super" | "Command" | "Meta" => mods |= Modifiers::SUPER,
            "Ctrl" | "Control" => mods |= Modifiers::CONTROL,
            "Alt" | "Option" => mods |= Modifiers::ALT,
            "Shift" => mods |= Modifiers::SHIFT,
            _ => {
                if code.is_some() {
                    return None;
                }
                code = parse_code(p);
            }
        }
    }
    code.map(|c| Shortcut::new(Some(mods), c))
}

fn set_macos_window_behavior(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use objc2::ffi::object_setClass;
        use objc2::runtime::AnyClass;
        use objc2::ClassType;
        use objc2_app_kit::{NSPanel, NSWindow, NSWindowCollectionBehavior, NSWindowStyleMask};

        let ptr = match window.ns_window() {
            Ok(p) if !p.is_null() => p,
            _ => return,
        };
        unsafe {
            let panel_cls = panel::FloatingNotesPanel::class();
            object_setClass(ptr as *mut objc2::runtime::AnyObject, panel_cls as *const AnyClass);

            let ns_window: &NSWindow = (ptr as *mut NSWindow)
                .as_ref()
                .expect("NSWindow null");
            let panel: &NSPanel = (ptr as *mut NSPanel)
                .as_ref()
                .expect("NSPanel null");

            // 浮动面板样式：非激活 + 工具窗口
            let mask = ns_window.styleMask()
                | NSWindowStyleMask::NonactivatingPanel
                | NSWindowStyleMask::UtilityWindow;
            ns_window.setStyleMask(mask);
            panel.setBecomesKeyOnlyIfNeeded(false);
            panel.setFloatingPanel(true);
            ns_window.setHidesOnDeactivate(false);
            ns_window.setIgnoresMouseEvents(false);

            // 进所有桌面 + 全屏辅助
            let behavior = NSWindowCollectionBehavior::CanJoinAllSpaces
                | NSWindowCollectionBehavior::FullScreenAuxiliary;
            ns_window.setCollectionBehavior(behavior);
            // 截屏级层级
            ns_window.setLevel(1000);
            // 成为 key window 以接收输入 + 唤起输入法（不 orderFront，不切桌面）
            ns_window.makeKeyWindow();
            println!("[floating-notes] swizzled to NSPanel, level={:?}", ns_window.level());
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
    }
}

fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            let _ = window.show();
            set_macos_window_behavior(&window);
            // 不调 set_focus：makeKeyAndOrderFront 会触发桌面切换
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .plugin(tauri_plugin_dialog::init())
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                let _ = window.hide();
                api.prevent_close();
            }
        })
        .setup({
            move |app| {
                app.manage(AppState {
                    current_file: Mutex::new(None),
                });

                let toggle_item = MenuItem::with_id(app, "toggle", "显示/隐藏", true, None::<&str>)?;
                let settings_item = MenuItem::with_id(app, "settings", "设置…", true, None::<&str>)?;
                let quit_item = MenuItem::with_id(app, "quit", "退出悬浮便签", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&toggle_item, &settings_item, &quit_item])?;

                let _tray = TrayIconBuilder::with_id("main-tray")
                    .icon(app.default_window_icon().unwrap().clone())
                    .tooltip("悬浮便签")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id.as_ref() {
                        "toggle" => toggle_window(app),
                        "settings" => {
                            if let Some(window) = app.get_webview_window("main") {
                                let _ = window.show();
                                let _ = window.set_focus();
                                let _ = window.emit("open-settings", ());
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                            toggle_window(tray.app_handle());
                        }
                    })
                    .build(app)?;

                let settings = load_settings();
                let sc_str = settings.shortcut.clone().unwrap_or_else(|| "Cmd+Shift+N".to_string());
                if let Some(sc) = parse_shortcut(&sc_str) {
                    app.global_shortcut()
                        .on_shortcut(sc, |app, _shortcut, event| {
                            if event.state() == ShortcutState::Pressed {
                                toggle_window(app);
                            }
                        })?;
                }

                if let Some(window) = app.get_webview_window("main") {
                    if let Some(rect) = settings.window_rect {
                        let _ = window.set_size(tauri::PhysicalSize::new(rect.w, rect.h));
                        let _ = window.set_position(tauri::PhysicalPosition::new(rect.x, rect.y));
                    }
                    set_macos_window_behavior(&window);
                    let _ = apply_vibrancy(&window, NSVisualEffectMaterial::Sidebar, None, Some(12.0));
                    let _ = window.set_focus();
                }

                Ok(())
            }
        })
        .invoke_handler(tauri::generate_handler![
            load_current_note,
            save_note,
            start_new_note,
            get_settings,
            set_settings,
            save_window_rect,
            make_key_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
