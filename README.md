# FloatNote

FloatNote 是一款 macOS 悬浮便签：随时调起、浮在其他窗口上方，并自动把笔记保存为 Markdown 文件。

## 下载

**[下载 FloatNote v0.5.0（Apple Silicon / DMG）](https://github.com/roverlee23/floatnote/releases/download/v0.5.0/FloatNote_0.5.0_aarch64.dmg)**

系统要求：macOS 13 或更高版本，Apple Silicon（M1/M2/M3/M4）。

## 安装

1. 下载并打开 DMG。
2. 将 FloatNote 拖入“应用程序”。
3. 首次打开时，如果 macOS 提示应用未签名，请右键 FloatNote，选择“打开”，再确认“仍要打开”。

也可以在终端执行：

```bash
xattr -dr com.apple.quarantine /Applications/FloatNote.app
```

## 功能

- 悬浮在其他窗口上方，随时记录
- 默认快捷键 `Cmd+Shift+N`
- 自动保存为 Markdown，可选择 Obsidian 文件夹
- 支持加粗、斜体、下划线、删除线、标题、列表、待办和代码块
- 支持中文、英文、西班牙语、法语、德语、意大利语、日语、阿拉伯语、葡萄牙语和俄语
- 支持锁定置顶、主题、透明度和开机自启动

## 隐私

FloatNote 在本地运行，不联网、不收集数据、不需要账号。笔记只保存在你选择的本地文件夹中。

## 开发

```bash
npm install
npm run tauri dev
```

构建发布包：

```bash
npm run tauri build
```

## 许可证

[MIT License](LICENSE)
