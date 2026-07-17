# FloatNote v0.5.0

<p align="center"><a href="#english">English</a> · <a href="#中文版">中文</a></p>

<a id="english"></a>

## FloatNote for macOS

FloatNote is a floating notes app for macOS. Summon it anywhere, write while viewing another app, and automatically save notes as Markdown files.

### Download

**[Download FloatNote v0.5.0 for Apple Silicon](https://github.com/roverlee23/floatnote/releases/download/v0.5.0/FloatNote_0.5.0_aarch64.dmg)**

### Highlights

- **Works across apps and full-screen spaces**: summon FloatNote with the default `Cmd+Shift+N` shortcut while using another app, including a full-screen app.
- **Normal window behavior when unpinned**: in a full-screen space, clicking outside FloatNote's functional area sends it to the background; on the desktop, it follows normal app window layering.
- **Pin to stay on top**: once pinned, FloatNote remains above the current app and does not hide when it loses focus. When you switch or swipe between spaces, it follows the current space.
- **Bring it forward anytime**: clicking the Dock icon, menu bar icon, or using the shortcut brings FloatNote to the foreground and makes it ready for editing.
- **Markdown + Obsidian**: notes are saved locally as Markdown files, with an optional storage folder for an Obsidian vault.
- **Rich text editing**: bold, italic, underline, strikethrough, headings, bullet and numbered lists, tasks, and code blocks.
- **10 languages**: Chinese, English, Spanish, French, German, Italian, Japanese, Arabic (RTL), Portuguese, and Russian.

### Install

1. Download and open the DMG.
2. Drag FloatNote to Applications.
3. On first launch, macOS may block the unsigned app. Right-click FloatNote, choose **Open**, then confirm **Open**.

Alternatively, run:

```bash
xattr -dr com.apple.quarantine /Applications/FloatNote.app
```

### System requirements

- macOS 13 or later
- Apple Silicon (M1, M2, M3, or M4)

### Privacy

- Runs locally with no network requests
- Notes are stored only in local files
- No data collection, telemetry, or account required

### Known limitations

- Unsigned and unnotarized; first launch requires manual approval
- Apple Silicon only in this release

### License

MIT License. See [LICENSE](https://github.com/roverlee23/floatnote/blob/main/LICENSE).

<a id="中文版"></a>

## FloatNote macOS 悬浮便签

FloatNote 是一款 macOS 悬浮便签工具。你可以在任何界面快速调起它，一边查看其他 App 一边记录，并自动把笔记保存为 Markdown 文件。

### 下载

**[下载 FloatNote v0.5.0（Apple Silicon）](https://github.com/roverlee23/floatnote/releases/download/v0.5.0/FloatNote_0.5.0_aarch64.dmg)**

### 核心特色

- **支持跨 App 和全屏空间使用**：在其他 App 中，包括全屏 App 内，都可以通过默认快捷键 `Cmd+Shift+N` 调起 FloatNote。
- **未图钉时遵循普通窗口逻辑**：在全屏界面，点击 FloatNote 功能区域以外的位置，FloatNote 退到后台；在桌面上则遵循普通 App 的窗口层级关系。
- **图钉后保持最上层**：激活图钉后，FloatNote 会一直浮在当前 App 上方，不因失去焦点而隐藏；用户切换或滑动不同桌面/全屏空间时，FloatNote 会跟随当前空间。
- **随时拉回前台**：在任何界面点击 Dock 图标、菜单栏图标，或使用快捷键，都会将 FloatNote 拉到前台并准备编辑。
- **Markdown + Obsidian**：笔记以 Markdown 文件保存在本地，可选择 Obsidian 库作为存储位置。
- **富文本编辑**：支持加粗、斜体、下划线、删除线、标题、无序/有序列表、待办和代码块。
- **10 种语言**：中文、英文、西班牙语、法语、德语、意大利语、日语、阿拉伯语（RTL）、葡萄牙语和俄语。

### 安装

1. 下载并打开 DMG。
2. 将 FloatNote 拖入“应用程序”。
3. 首次打开时，macOS 可能会拦截未签名应用。请右键 FloatNote，选择“打开”，再确认“打开”。

也可以在终端执行：

```bash
xattr -dr com.apple.quarantine /Applications/FloatNote.app
```

### 系统要求

- macOS 13 或更高版本
- Apple Silicon（M1、M2、M3 或 M4）

### 隐私说明

- 完全本地运行，不发起网络请求
- 笔记只保存到本地文件
- 无数据收集、无遥测、无需账号

### 已知限制

- 应用未签名、未公证，首次打开需要手动确认
- 本版本仅支持 Apple Silicon

### 开源协议

MIT License，详见 [LICENSE](https://github.com/roverlee23/floatnote/blob/main/LICENSE)。
