use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

#[cfg(target_os = "macos")]
use std::sync::OnceLock;

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
    pinned: Mutex<bool>,
    mode: Mutex<WindowMode>,
    fullscreen_space_ids: Mutex<Vec<i64>>,
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

fn debug_log_path() -> Option<PathBuf> {
    config_dir().map(|dir| dir.join("window-debug.log"))
}

fn write_debug_log(message: &str) {
    let Some(path) = debug_log_path() else { return };
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if fs::metadata(&path).map(|meta| meta.len() > 512 * 1024).unwrap_or(false) {
        let _ = fs::write(&path, "");
    }
    if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{} | {}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"), message);
    }
}

#[tauri::command]
fn append_debug_log(message: String) {
    write_debug_log(&message);
}

#[tauri::command]
fn get_debug_log_path() -> String {
    debug_log_path().map(|path| path.display().to_string()).unwrap_or_default()
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
                    activate_window(app, ActivationSource::Shortcut);
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
fn set_window_pinned(app: tauri::AppHandle, pinned: bool) {
    let fullscreen = current_space_is_fullscreen();
    write_debug_log(&format!("pin_change requested={} detected_fullscreen={}", pinned, fullscreen));
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut current) = state.pinned.lock() {
            *current = pinned;
        }
    }
    if let Some(window) = app.get_webview_window("main") {
        apply_window_mode(&app, &window, window_mode(pinned, fullscreen));
    }
}

fn dismiss_window(app: &tauri::AppHandle, reason: &str) {
    write_debug_log(&format!("dismiss_window reason={}", reason));
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut pinned) = state.pinned.lock() {
            *pinned = false;
        }
    }
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.hide();
        // A hidden window must not retain an auxiliary/full-screen association.
        apply_window_mode(app, &window, WindowMode::Desktop);
    }
}

#[tauri::command]
fn dismiss_window_from_ui(app: tauri::AppHandle) {
    dismiss_window(&app, "ui_close");
}

#[tauri::command]
fn handle_window_focus_lost(app: tauri::AppHandle) {
    let should_hide = app
        .try_state::<AppState>()
        .map(|state| {
            let pinned = state.pinned.lock().map(|value| *value).unwrap_or(false);
            let mode = state.mode.lock().map(|value| value.clone()).unwrap_or(WindowMode::Desktop);
            should_hide_on_focus_loss(&mode, pinned)
        })
        .unwrap_or(false);
    write_debug_log(&format!("focus_lost should_hide={}", should_hide));
    if should_hide {
        dismiss_window(&app, "fullscreen_focus_lost");
    }
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

fn set_macos_window_behavior(window: &tauri::WebviewWindow, mode: WindowMode) {
    write_debug_log(&format!("native_apply_window_behavior mode={:?}", mode));
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
            // Desktop uses ordinary window stacking. Full-screen and pinned modes float.
            panel.setFloatingPanel(mode != WindowMode::Desktop);
            ns_window.setHidesOnDeactivate(false);
            ns_window.setIgnoresMouseEvents(false);

            // Desktop: behave like an ordinary app in the current desktop.
            // Fullscreen: temporarily join only the active full-screen space.
            // Pinned: the only mode allowed to cross spaces.
            let behavior = match mode {
                WindowMode::Desktop => NSWindowCollectionBehavior::MoveToActiveSpace
                    | NSWindowCollectionBehavior::FullScreenNone,
                WindowMode::Fullscreen => NSWindowCollectionBehavior::MoveToActiveSpace
                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
                WindowMode::Pinned => NSWindowCollectionBehavior::CanJoinAllSpaces
                    | NSWindowCollectionBehavior::FullScreenAuxiliary,
            };
            ns_window.setCollectionBehavior(behavior);
            ns_window.setLevel(if mode == WindowMode::Desktop { 0 } else { 1000 });
            println!("[floating-notes] swizzled to NSPanel, level={:?}", ns_window.level());
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window;
    }
}

fn apply_window_mode(app: &tauri::AppHandle, window: &tauri::WebviewWindow, mode: WindowMode) {
    write_debug_log(&format!("apply_window_mode mode={:?}", mode));
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut current) = state.mode.lock() {
            *current = mode.clone();
        }
        if let Ok(mut space_ids) = state.fullscreen_space_ids.lock() {
            *space_ids = if mode == WindowMode::Fullscreen {
                current_space_context().ids
            } else {
                Vec::new()
            };
        }
    }
    set_macos_window_behavior(window, mode);
}

#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
unsafe extern "C" {
    fn CGSMainConnectionID() -> u32;
    fn CGSCopyManagedDisplaySpaces(connection: u32) -> core_foundation::array::CFArrayRef;
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SpaceContext {
    fullscreen: bool,
    ids: Vec<i64>,
}

// Window bounds cannot tell a maximized desktop window from a true macOS
// full-screen space. Ask Mission Control for the current space type instead.
fn current_space_context() -> SpaceContext {
    #[cfg(target_os = "macos")]
    {
        use core_foundation::base::TCFType;
        use core_foundation::array::CFArray;
        use core_foundation::dictionary::CFDictionary;
        use core_foundation::number::CFNumber;
        use core_foundation::string::CFString;
        use core_foundation::base::CFType;

        let raw_spaces = unsafe { CGSCopyManagedDisplaySpaces(CGSMainConnectionID()) };
        if raw_spaces.is_null() {
            write_debug_log("space_context result=desktop reason=no_spaces");
            return SpaceContext { fullscreen: false, ids: Vec::new() };
        }
        let displays: CFArray<CFDictionary<CFString, CFType>> = unsafe {
            TCFType::wrap_under_create_rule(raw_spaces)
        };
        let current_space_key = CFString::new("Current Space");
        let type_key = CFString::new("type");
        let id_key = CFString::new("ManagedSpaceID");
        let mut types = Vec::new();
        let mut ids = Vec::new();
        for display in displays.iter() {
            let Some(raw_space) = display
                .find(&current_space_key)
                .and_then(|value| value.downcast::<CFDictionary>())
            else {
                continue;
            };
            let space: CFDictionary<CFString, CFType> = unsafe {
                TCFType::wrap_under_get_rule(raw_space.as_concrete_TypeRef())
            };
            let Some(space_type) = space
                .find(&type_key)
                .and_then(|value| value.downcast::<CFNumber>())
                .and_then(|value| value.to_i32())
            else {
                continue;
            };
            types.push(space_type);
            if let Some(space_id) = space
                .find(&id_key)
                .and_then(|value| value.downcast::<CFNumber>())
                .and_then(|value| value.to_i64())
            {
                ids.push(space_id);
            }
        }
        let fullscreen = types.contains(&4);
        write_debug_log(&format!(
            "space_context result={} types={:?} ids={:?}",
            if fullscreen { "fullscreen" } else { "desktop" },
            types,
            ids
        ));
        SpaceContext { fullscreen, ids }
    }
    #[cfg(not(target_os = "macos"))]
    { SpaceContext { fullscreen: false, ids: Vec::new() } }
}

fn current_space_is_fullscreen() -> bool {
    current_space_context().fullscreen
}

#[tauri::command]
fn is_fullscreen_context() -> bool {
    current_space_is_fullscreen()
}

#[cfg(target_os = "macos")]
fn install_active_space_observer(app: &tauri::AppHandle) {
    use block2::RcBlock;
    use objc2_app_kit::{NSWorkspace, NSWorkspaceActiveSpaceDidChangeNotification};
    use objc2_foundation::NSNotification;
    use std::ptr::NonNull;

    let app_handle = app.clone();
    let block = RcBlock::new(move |_notification: NonNull<NSNotification>| {
        let (pinned, mode, expected_space_ids) = app_handle
            .try_state::<AppState>()
            .map(|state| {
                let pinned = state.pinned.lock().map(|value| *value).unwrap_or(false);
                let mode = state.mode.lock().map(|value| value.clone()).unwrap_or(WindowMode::Desktop);
                let ids = state.fullscreen_space_ids.lock().map(|value| value.clone()).unwrap_or_default();
                (pinned, mode, ids)
            })
            .unwrap_or((false, WindowMode::Desktop, Vec::new()));
        let current = current_space_context();
        let should_hide = !pinned
            && mode == WindowMode::Fullscreen
            && current.ids != expected_space_ids;
        write_debug_log(&format!(
            "active_space_changed should_hide={} expected_ids={:?} current_ids={:?}",
            should_hide,
            expected_space_ids,
            current.ids
        ));
        if should_hide {
            dismiss_window(&app_handle, "fullscreen_space_changed");
        }
    });
    let center = NSWorkspace::sharedWorkspace().notificationCenter();
    let observer = unsafe {
        center.addObserverForName_object_queue_usingBlock(
            Some(NSWorkspaceActiveSpaceDidChangeNotification),
            None,
            None,
            &block,
        )
    };
    std::mem::forget(observer);
}

// Tao forwards a Dock click to the Tauri Reopen event, but returns `false`
// when FloatNote is hidden. macOS then performs its own default reopen action
// and can switch back to the app's old desktop after our handler has already
// shown the note in a full-screen space. Keep Tao's event dispatch, but tell
// macOS that the reopen was handled by FloatNote.
#[cfg(target_os = "macos")]
type ReopenHandler = unsafe extern "C-unwind" fn(
    *mut objc2::runtime::AnyObject,
    objc2::runtime::Sel,
    *mut objc2::runtime::AnyObject,
    objc2::runtime::Bool,
) -> objc2::runtime::Bool;

#[cfg(target_os = "macos")]
static ORIGINAL_DOCK_REOPEN_HANDLER: OnceLock<ReopenHandler> = OnceLock::new();

#[cfg(target_os = "macos")]
unsafe extern "C-unwind" fn floatnote_dock_reopen_handler(
    delegate: *mut objc2::runtime::AnyObject,
    selector: objc2::runtime::Sel,
    application: *mut objc2::runtime::AnyObject,
    has_visible_windows: objc2::runtime::Bool,
) -> objc2::runtime::Bool {
    if let Some(original) = ORIGINAL_DOCK_REOPEN_HANDLER.get() {
        // Preserve Tao's Reopen event so the normal FloatNote activation path runs.
        unsafe { original(delegate, selector, application, has_visible_windows) };
    }
    write_debug_log("dock_reopen handled_by_floatnote=true");
    objc2::runtime::Bool::YES
}

#[cfg(target_os = "macos")]
fn install_dock_reopen_handler() {
    use objc2::runtime::{AnyObject, Imp};
    use objc2::{sel, MainThreadMarker};
    use objc2_app_kit::NSApplication;

    let Some(mtm) = MainThreadMarker::new() else {
        write_debug_log("dock_reopen_override skipped=not_main_thread");
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    let Some(delegate) = app.delegate() else {
        write_debug_log("dock_reopen_override skipped=no_delegate");
        return;
    };
    let delegate_object: &AnyObject = delegate.as_ref();
    let selector = sel!(applicationShouldHandleReopen:hasVisibleWindows:);
    let Some(method) = delegate_object.class().instance_method(selector) else {
        write_debug_log("dock_reopen_override skipped=no_method");
        return;
    };
    let replacement: Imp = unsafe {
        std::mem::transmute(floatnote_dock_reopen_handler as ReopenHandler)
    };
    let original = unsafe { method.set_implementation(replacement) };
    let original: ReopenHandler = unsafe { std::mem::transmute(original) };
    let _ = ORIGINAL_DOCK_REOPEN_HANDLER.set(original);
    write_debug_log("dock_reopen_override installed");
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum ActivationSource {
    Dock,
    TrayIcon,
    Shortcut,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum ActivationAction {
    ShowAndFocus,
    Toggle,
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum WindowMode {
    Desktop,
    Fullscreen,
    Pinned,
}

fn window_mode(pinned: bool, fullscreen: bool) -> WindowMode {
    if pinned { WindowMode::Pinned } else if fullscreen { WindowMode::Fullscreen } else { WindowMode::Desktop }
}

fn should_hide_on_focus_loss(mode: &WindowMode, pinned: bool) -> bool {
    !pinned && *mode == WindowMode::Fullscreen
}

fn activation_action(_source: ActivationSource) -> ActivationAction {
    match _source {
        // Dock now matches the shortcut. The menu-bar icon keeps its established
        // "show and focus" behavior and never hides an already visible note.
        ActivationSource::Dock | ActivationSource::Shortcut => ActivationAction::Toggle,
        ActivationSource::TrayIcon => ActivationAction::ShowAndFocus,
    }
}

fn activate_window(app: &tauri::AppHandle, source: ActivationSource) {
    let action = activation_action(source.clone());
    write_debug_log(&format!("activation source={:?} action={:?}", source, action));
    match action {
        ActivationAction::ShowAndFocus => show_and_focus_window(app),
        ActivationAction::Toggle => toggle_window(app),
    }
}

fn show_and_focus_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        write_debug_log(&format!("show_and_focus visible_before={:?}", window.is_visible().ok()));
        let pinned = app
            .state::<AppState>()
            .pinned
            .lock()
            .map(|value| *value)
            .unwrap_or(false);
        let _ = window.unminimize();
        apply_window_mode(app, &window, window_mode(pinned, current_space_is_fullscreen()));
        let _ = window.show();
        bring_window_to_front(&window);
        write_debug_log(&format!(
            "show_and_focus after_raise visible={:?} focused={:?}",
            window.is_visible().ok(),
            window.is_focused().ok()
        ));
    }
}

fn bring_window_to_front(window: &tauri::WebviewWindow) {
    #[cfg(target_os = "macos")]
    {
        use objc2_app_kit::NSWindow;
        if let Ok(ptr) = window.ns_window() {
            unsafe {
                let ns_window: &NSWindow = (ptr as *mut NSWindow).as_ref().expect("NSWindow");
                ns_window.orderFrontRegardless();
                ns_window.makeKeyWindow();
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = window.set_focus();
    }
}

fn toggle_window(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let visible = window.is_visible().unwrap_or(false);
        let focused = window.is_focused().unwrap_or(false);
        write_debug_log(&format!("toggle visible_before={} focused_before={}", visible, focused));
        if visible && focused {
            let _ = window.hide();
            write_debug_log("toggle hid_window");
        } else {
            let pinned = app
                .state::<AppState>()
                .pinned
                .lock()
                .map(|value| *value)
                .unwrap_or(false);
            apply_window_mode(app, &window, window_mode(pinned, current_space_is_fullscreen()));
            let _ = window.show();
            bring_window_to_front(&window);
            write_debug_log(&format!(
                "toggle showed_or_raised_window visible_after={} focused_after={}",
                window.is_visible().unwrap_or(false),
                window.is_focused().unwrap_or(false)
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{activation_action, should_hide_on_focus_loss, window_mode, ActivationAction, ActivationSource, WindowMode};

    #[test]
    fn dock_matches_shortcut_while_menu_bar_keeps_show_behavior() {
        assert_eq!(activation_action(ActivationSource::Dock), ActivationAction::Toggle);
        assert_eq!(activation_action(ActivationSource::Shortcut), ActivationAction::Toggle);
        assert_eq!(activation_action(ActivationSource::TrayIcon), ActivationAction::ShowAndFocus);
    }

    #[test]
    fn window_mode_only_crosses_spaces_when_pinned() {
        assert_eq!(window_mode(false, false), WindowMode::Desktop);
        assert_eq!(window_mode(false, true), WindowMode::Fullscreen);
        assert_eq!(window_mode(true, false), WindowMode::Pinned);
        assert_eq!(window_mode(true, true), WindowMode::Pinned);
    }

    #[test]
    fn only_fullscreen_transient_mode_hides_on_focus_loss() {
        assert!(!should_hide_on_focus_loss(&WindowMode::Desktop, false));
        assert!(should_hide_on_focus_loss(&WindowMode::Fullscreen, false));
        assert!(!should_hide_on_focus_loss(&WindowMode::Pinned, true));
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
                write_debug_log("native_close_requested");
                dismiss_window(&window.app_handle(), "native_close_requested");
                api.prevent_close();
            }
        })
        .setup({
            move |app| {
                app.manage(AppState {
                    current_file: Mutex::new(None),
                    pinned: Mutex::new(false),
                    mode: Mutex::new(WindowMode::Desktop),
                    fullscreen_space_ids: Mutex::new(Vec::new()),
                });
                write_debug_log("app_setup");

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
                                show_and_focus_window(app);
                                let _ = window.emit("open-settings", ());
                            }
                        }
                        "quit" => app.exit(0),
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click { button: MouseButton::Left, .. } = event {
                            activate_window(tray.app_handle(), ActivationSource::TrayIcon);
                        }
                    })
                    .build(app)?;

                let settings = load_settings();
                let sc_str = settings.shortcut.clone().unwrap_or_else(|| "Cmd+Shift+N".to_string());
                if let Some(sc) = parse_shortcut(&sc_str) {
                    app.global_shortcut()
                        .on_shortcut(sc, |app, _shortcut, event| {
                            if event.state() == ShortcutState::Pressed {
                                activate_window(app, ActivationSource::Shortcut);
                            }
                        })?;
                }

                if let Some(window) = app.get_webview_window("main") {
                    if let Some(rect) = settings.window_rect {
                        let _ = window.set_size(tauri::PhysicalSize::new(rect.w, rect.h));
                        let _ = window.set_position(tauri::PhysicalPosition::new(rect.x, rect.y));
                    }
                    apply_window_mode(&app.handle(), &window, WindowMode::Desktop);
                    let _ = apply_vibrancy(&window, NSVisualEffectMaterial::Sidebar, None, Some(12.0));
                    let _ = window.set_focus();
                }

                #[cfg(target_os = "macos")]
                {
                    install_active_space_observer(&app.handle());
                    install_dock_reopen_handler();
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
            set_window_pinned,
            dismiss_window_from_ui,
            handle_window_focus_lost,
            is_fullscreen_context,
            append_debug_log,
            get_debug_log_path,
        ])
        .build(tauri::generate_context!())
        .expect("error while building Tauri application")
        .run(|app, event| {
            #[cfg(target_os = "macos")]
            if let tauri::RunEvent::Reopen { .. } = event {
                activate_window(app, ActivationSource::Dock);
            }
        })
}
