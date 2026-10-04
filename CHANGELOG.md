# Changelog

## v3.10（2026-10-05）

### 中文

- 新增 Rust 原生实现，发布约 350 KB 单文件可执行程序（cs2_font_changer.exe），不再依赖 Python 运行时环境与 fontTools 库
- 新增 TTF/OTF 字体族名解析器，支持 TrueType Collection（TTC）字体集合，替代 fontTools 完成字体名读取
- 新增构建脚本 build.py，支持编译后自动部署到项目根目录并清理旧构建产物，提供 --build-only 与 --launch 参数
- 新增拖入字体时自动解除 Windows 的 Internet 安全标记（Zone.Identifier），避免每次启动都弹出「这些文件可能对你的计算机有害」提示
- 优化 CS2 进程检测方式，改用 ToolHelp API 枚举进程，检测期间不再闪现命令行黑窗
- 优化执行日志复制到剪贴板的兼容性，改用 Win32 剪贴板 API 直写 UTF-16 文本，避免 GBK 转码静默丢弃部分字符
- 修复检测到 CS2 正在运行时主菜单直接显示键名「cs2_running」而非提示文案的问题
- 修复错误详情输出中「exception」「diagnostic」文案缺失、直接显示英文键名的问题
- 移除 Python 源码、PyInstaller 打包配置与 requirements.txt 依赖清单

### English

- Added a native Rust implementation, shipping a single ~350 KB executable (cs2_font_changer.exe) with no Python runtime or fontTools dependency
- Added a built-in TTF/OTF font family-name parser with TrueType Collection (TTC) support, replacing fontTools
- Added the build.py build script, which deploys to the repo root and cleans stale build artifacts automatically, with --build-only and --launch flags
- Added automatic removal of the Windows "Mark of the Web" (Zone.Identifier) from dragged-in fonts, so Windows no longer shows the "These files may be harmful to your computer" prompt on every launch
- Improved CS2 process detection by enumerating processes via the ToolHelp API, so detection no longer flashes a console window
- Improved log-to-clipboard compatibility by writing UTF-16 text directly through the Win32 clipboard API, avoiding silent character loss from GBK transcoding
- Fixed the main menu showing the raw key "cs2_running" instead of a proper message when CS2 is detected as running
- Fixed missing "exception" / "diagnostic" labels in error output that fell back to raw key names
- Removed the Python source, PyInstaller packaging config, and requirements.txt
