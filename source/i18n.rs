//! Internationalization: bilingual (zh/en) message table mirroring the Python MESSAGES dict.

use std::sync::OnceLock;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Zh,
    En,
}

static LANG: OnceLock<Lang> = OnceLock::new();

pub fn current_lang() -> Lang {
    *LANG.get_or_init(detect_system_language)
}

fn detect_system_language() -> Lang {
    // GetUserDefaultUILanguage: 0x0804 zh-CN, 0x1004 zh-SG, 0x0C04 zh-HK, 0x1404 zh-MO, 0x0404 zh-TW
    use windows::Win32::Globalization::GetUserDefaultUILanguage;
    let lang_id = unsafe { GetUserDefaultUILanguage() };
    match lang_id {
        0x0804 | 0x1004 | 0x0C04 | 0x1404 | 0x0404 => Lang::Zh,
        _ => Lang::En,
    }
}

/// Look up `key` in the current language's table, falling back to English, then to the key itself.
/// Supports `{name}`-style placeholder substitution via the provided pairs.
pub fn t(key: &str) -> String {
    lookup(current_lang(), key).to_string()
}

pub fn t_with(key: &str, args: &[(&str, &str)]) -> String {
    let mut msg = lookup(current_lang(), key).to_string();
    for (k, v) in args {
        let placeholder = format!("{{{}}}", k);
        msg = msg.replace(&placeholder, v);
    }
    msg
}

fn lookup(lang: Lang, key: &str) -> &'static str {
    let table = match lang {
        Lang::Zh => ZH_MESSAGES,
        Lang::En => EN_MESSAGES,
    };
    for (k, v) in table {
        if *k == key {
            return v;
        }
    }
    // Fallback to English
    for (k, v) in EN_MESSAGES {
        if *k == key {
            return v;
        }
    }
    // Last resort: return a static empty (callers pass key by value; we can't return it as 'static)
    // The Python version returns the key itself; we mimic by leaking only when truly missing.
    // In practice all keys below are defined; missing keys would be a bug.
    Box::leak(key.to_string().into_boxed_str())
}

const ZH_MESSAGES: &[(&str, &str)] = &[
    ("auto_detect_failed", "未能自动检测到游戏安装路径，请手动进行选择"),
    ("auto_restore_detect", "检测到备份压缩包，准备执行自动还原"),
    ("backing_up", "正在备份"),
    ("backup_created", "备份成功，\x1b[90m{backup_path}\x1b[0m"),
    ("backup_error", "备份失败"),
    ("cleaning_ui_font", "正在清理旧版本的 UI 字体索引文件"),
    ("cleaning_ui_font_failed", "清理旧版本 UI 字体索引失败"),
    ("config_cleaned", "已清理自定义配置"),
    ("config_failed", "\n配置执行过程中可能存在不完整的操作，请检查上述错误信息"),
    ("config_success", "\n所有配置任务已成功执行，请启动 Counter-Strike 2 以查看新的字体效果"),
    ("core_dir_created", "已自动创建核心配置目录"),
    ("core_dir_missing", "核心配置目录缺失：{core_fonts}"),
    ("current", "当前"),
    ("deploy_font_failed", "部署字体文件失败"),
    ("dialog_game_title", "选择 Counter-Strike Global Offensive 文件夹"),
    ("dialog_restore_title", "选择 Counter-Strike Global Offensive 文件夹以进行还原"),
    ("enter_key", "回车键"),
    ("font_deployed", "已将新字体文件部署至目标路径：{font_name}.ttf"),
    ("font_dir_created", "已自动创建字体目录"),
    ("font_dir_missing", "字体目录缺失：{csgo_fonts}"),
    ("font_files", "字体文件"),
    ("generating_fonts_conf", "\n正在生成局部字体配置文件：fonts.conf"),
    ("generating_fonts_conf_failed", "生成 fonts.conf 失败"),
    ("generating_repl_conf", "正在生成全局字体映射文件：42-repl-global.conf"),
    ("generating_repl_conf_failed", "生成 42-repl-global.conf 失败"),
    ("invalid_font_format", "所选文件 \"{file}\" 并非有效的字体格式"),
    ("invalid_number", "请输入合法的数值"),
    ("invalid_path_exit", "指定的路径无效或未选择任何目录，请按回车键退出"),
    ("invalid_path_specified", "指定的路径无效，请确保选择了正确的游戏根目录"),
    ("logs_copied", "日志已成功复制到剪贴板"),
    ("menu_adjust_ui", "调整 UI 缩放"),
    ("menu_reimport_font", "重新导入字体"),
    ("menu_select_font", "选择导入字体"),
    ("menu_select_path", "选择游戏路径"),
    ("menu_start", "\n\x1b[96m•\x1b[0m 按 [\x1b[92m回车键\x1b[0m] 开始替换字体"),
    ("menu_title", "CS2 字体更改器 v3.10 | 作者：Cairl"),
    ("no_font_loaded", "尚未加载字体源文件，无法执行配置，请先按 1 导入字体"),
    ("not_identified", "未识别"),
    ("not_selected", "未选择"),
    ("operation_cancelled", "操作已取消，按回车键返回"),
    ("path_not_configured", "游戏路径配置不正确或尚未设定，请先确认路径信息"),
    ("press_enter_menu", "请按回车键返回主菜单"),
    ("removing_ttf", "正在移除冲突的旧字体文件：{file}"),
    ("removing_ttf_failed", "移除旧字体文件失败：{file}"),
    ("restore_complete", "还原完成"),
    ("restore_default", "恢复默认字体，注意: \x1b[90m程序首次运行时会在游戏根目录自动创建恢复文件，恢复操作需要该文件存在\x1b[0m"),
    ("restore_error", "还原失败"),
    ("restore_failed_menu", "还原失败，按回车键返回主菜单尝试手动处理"),
    ("restore_file", "恢复文件"),
    ("restore_file_missing", "未发现可用的备份文件：{backup_path}"),
    ("restore_success_exit", "还原程序执行失败，请按回车键退出"),
    ("restoring_fonts", "正在从复原文件 {backup_name} 还原游戏初始字体"),
    ("select_font_or_restore", "选择字体或恢复文件"),
    ("send_to_dev", "请将以上错误详情发送给开发者排查，按回车键退出程序"),
    ("specify_path_first", "执行还原程序前，请先指定游戏的安装路径"),
    ("supported_files", "支持的文件"),
    ("task_complete", "运行任务已结束，按 [\x1b[92m回车键\x1b[0m] 复制执行日志至剪贴板，或直接关闭"),
    ("ui_scale_input", "\x1b[93m•\x1b[0m 输入 UI 缩放倍率（推荐区间 0.9 至 1.1）："),
    ("unrecognized_command", "未能识别的指令，请按回车键刷新菜单"),
    ("unsupported_font", "\"{file}\" 是不受支持的字体集"),
    ("validation_check", "正在对安装结果进行一致性校验"),
    ("validation_failed_conf", "校验失败：未能定位到必要配置文件 {conf_file}"),
    ("validation_failed_font", "校验失败：未能定位到目标字体文件 {file}"),
    ("validation_failed_size", "校验失败：目标字体文件大小异常（0 字节）"),
    ("validation_success", "校验完成，所有必要文件均已就绪"),
    ("wrong_zip_name", "所选 ZIP 文件名称不正确，应为 \"backup_original_fonts.zip\""),
    // Used by print_error / menu but not defined in the original Python table (fell back to key).
    ("exception", "异常"),
    ("diagnostic", "诊断信息"),
    ("cs2_running", "\x1b[93m检测到 CS2 正在运行，建议关闭游戏后再进行替换\x1b[0m"),
];

const EN_MESSAGES: &[(&str, &str)] = &[
    ("auto_detect_failed", "Auto-detection of game path failed, Please specify the path manually"),
    ("auto_restore_detect", "Backup zip detected, preparing for auto-restore"),
    ("backing_up", "Backing up"),
    ("backup_created", "Backup success, \x1b[90m{backup_path}\x1b[0m"),
    ("backup_error", "Backup failed"),
    ("cleaning_ui_font", "Cleaning up legacy UI font index files"),
    ("cleaning_ui_font_failed", "Failed to clean up legacy UI font index"),
    ("config_cleaned", "Cleaned custom config"),
    ("config_failed", "\nSome operations might have failed during execution, Please review the error messages above"),
    ("config_success", "\nAll configuration tasks have been successfully executed, Please launch Counter-Strike 2 to see the new font"),
    ("core_dir_created", "Created core configuration directory automatically"),
    ("core_dir_missing", "Core configuration directory missing: {core_fonts}"),
    ("current", "current"),
    ("deploy_font_failed", "Failed to deploy font file"),
    ("dialog_game_title", "Select Counter-Strike Global Offensive folder"),
    ("dialog_restore_title", "Select Counter-Strike Global Offensive folder for restore"),
    ("enter_key", "Enter"),
    ("font_deployed", "Deployed new font file to target: {font_name}.ttf"),
    ("font_dir_created", "Created font directory automatically"),
    ("font_dir_missing", "Font directory missing: {csgo_fonts}"),
    ("font_files", "Font Files"),
    ("generating_fonts_conf", "\nGenerating local font configuration: fonts.conf"),
    ("generating_fonts_conf_failed", "Failed to generate fonts.conf"),
    ("generating_repl_conf", "Generating global font mapping: 42-repl-global.conf"),
    ("generating_repl_conf_failed", "Failed to generate 42-repl-global.conf"),
    ("invalid_font_format", "Selected file \"{file}\" is not a valid font format"),
    ("invalid_number", "Please enter a valid number"),
    ("invalid_path_exit", "Invalid path or no path selected, Press Enter to exit"),
    ("invalid_path_specified", "Invalid path specified, Please ensure you select the correct game root directory"),
    ("logs_copied", "Logs have been successfully copied to the clipboard"),
    ("menu_adjust_ui", "adjust UI scale"),
    ("menu_reimport_font", "re-import font"),
    ("menu_select_font", "select font to import"),
    ("menu_select_path", "select game path"),
    ("menu_start", "\n\x1b[96m•\x1b[0m Press [\x1b[92mEnter\x1b[0m] to start font replacement"),
    ("menu_title", "CS2 Font Changer v3.10 | Author: Cairl"),
    ("no_font_loaded", "No font source file loaded, Cannot perform configuration, Press 1 to import font first"),
    ("not_identified", "Not Identified"),
    ("not_selected", "Not Selected"),
    ("operation_cancelled", "Operation cancelled, Press Enter to return"),
    ("path_not_configured", "Game path configuration incorrect or not yet set, Please confirm path info first"),
    ("press_enter_menu", "Press Enter to return to main menu"),
    ("removing_ttf", "Removing conflicting legacy font: {file}"),
    ("removing_ttf_failed", "Failed to remove legacy font: {file}"),
    ("restore_complete", "Restore complete"),
    ("restore_default", "Restore default fonts, Note: \x1b[90mA restoration file is automatically created in the game root during the first run, and restoration requires this file to exist\x1b[0m"),
    ("restore_error", "Restore failed"),
    ("restore_failed_menu", "Restoration failed, Press Enter to return to main menu and try manually"),
    ("restore_file", "Restoration File"),
    ("restore_file_missing", "No restoration file found at: {backup_path}"),
    ("restore_success_exit", "Restoration failed, Press Enter to exit"),
    ("restoring_fonts", "Restoring initial game fonts from {backup_name}"),
    ("select_font_or_restore", "Select Font or Restoration File"),
    ("send_to_dev", "Please send the error details above to the developer, Press Enter to exit"),
    ("specify_path_first", "Please specify the game installation path before running restoration"),
    ("supported_files", "Supported Files"),
    ("task_complete", "Task completed, Press [\x1b[92mEnter\x1b[0m] to copy execution logs to clipboard, or close directly"),
    ("ui_scale_input", "\x1b[93m•\x1b[0m Enter UI scale (suggested 0.9 to 1.1): "),
    ("unrecognized_command", "Unrecognized command, Press Enter to refresh menu"),
    ("unsupported_font", "\"{file}\" is an unsupported font"),
    ("validation_check", "Performing consistency check on installation results"),
    ("validation_failed_conf", "Validation failed: Could not locate required configuration file {conf_file}"),
    ("validation_failed_font", "Validation failed: Could not locate target font file {file}"),
    ("validation_failed_size", "Validation failed: Target font file size is abnormal (0 bytes)"),
    ("validation_success", "Validation complete, all necessary files are ready"),
    ("wrong_zip_name", "Selected ZIP file name is incorrect, Should be \"backup_original_fonts.zip\""),
    ("exception", "Exception"),
    ("diagnostic", "Diagnostic"),
    ("cs2_running", "\x1b[93mCS2 is currently running, it is recommended to close the game before replacing fonts\x1b[0m"),
];
