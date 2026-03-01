import sys
import winreg
import os
import shutil
import msvcrt
import subprocess
from fontTools.ttLib import TTFont
import tkinter as tk
from tkinter import filedialog
import zipfile
import locale

language = 'auto'  # Language configuration: auto, en, zh

def get_system_language():
    """Detect system language, return 'zh' for Simplified Chinese, 'en' for others"""
    try:
        windows_locale = locale.getdefaultlocale()[0]
        if windows_locale and (windows_locale.startswith('zh_CN') or windows_locale.startswith('zh_SG')):
            return 'zh'
        return 'en'
    except:
        return 'en'

LANG = 'zh' if language == 'zh' else ('en' if language == 'en' else get_system_language())

MESSAGES = {
    'zh': {
        'log_clear_desc': '清空日志缓冲区',
        'copy_to_clipboard_desc': '将文本复制到剪贴板',
        'logger_clear': '清空日志缓冲区',
        'clipboard_success': '日志已成功复制到剪贴板',
        'clipboard_action': '运行任务已结束，按 [\033[92m回车键\033[0m] 复制执行日志至剪贴板，或直接关闭',
        'error_exception': '异常',
        'error_diagnostic': '诊断详情',
        'missing_msg': '字体目录缺失',
        'created_msg': '已自动创建字体目录',
        'core_missing_msg': '核心配置目录缺失',
        'core_created_msg': '已自动创建核心配置目录',
        'ui_cleaning': '正在清理旧版本的 UI 字体索引文件',
        'ui_clean_failed': '清理旧版本 UI 字体索引失败',
        'ttf_removing': '正在移除冲突的旧字体文件：{file}',
        'ttf_remove_failed': '移除旧字体文件失败：{file}',
        'fonts_conf_success': '\n正在生成局部字体配置文件：fonts.conf',
        'fonts_conf_error': '生成 fonts.conf 失败',
        'repl_conf_success': '正在生成全局字体映射文件：42-repl-global.conf',
        'repl_conf_error': '生成 42-repl-global.conf 失败',
        'verify_check': '正在对安装结果进行一致性校验',
        'verify_failed_font': '校验失败：未能定位到目标字体文件 {file}',
        'verify_failed_conf': '校验失败：未能定位到必要配置文件 {conf_file}',
        'verify_failed_size': '校验失败：目标字体文件大小异常（0 字节）',
        'verify_success': '校验完成，所有必要文件均已就绪',
        'backuping': '正在备份',
        'backup_success': '备份成功，\033[90m{backup_path}\033[0m',
        'backup_failed': '备份失败',
        'restore_not_found': '未发现可用的备份文件：{backup_path}',
        'restore_ing': '正在从复原文件 {backup_name} 还原游戏初始字体',
        'restore_cleaned': '已清理自定义配置',
        'restore_success': '还原完成',
        'restore_failed': '还原失败',
        'game_running_warn': '\033[93m注意：检测到 Counter-Strike 2 正在运行，可能会影响字体替换效果，建议先关闭游戏再进行操作\033[0m',
        'menu_title': 'CS2 字体更改器 v3.0 | 作者：Cairl',
        'menu_select_font': '选择导入字体',
        'menu_reimport_font': '重新导入字体',
        'menu_select_path': '选择游戏路径',
        'menu_adjust_ui': '调整 UI 缩放',
        'menu_restore': '按 [\033[92m0\033[0m] 恢复默认字体，注意：程序首次运行时会在游戏根目录自动创建恢复文件，恢复操作需要该文件存在',
        'menu_start': '\n\033[96m•\033[0m 按 [\033[92m回车键\033[0m] 开始替换字体',
        'menu_current_font': '当前',
        'menu_current_path': '当前',
        'menu_current_scale': '当前',
        'menu_not_selected': '未选择',
        'menu_not_identified': '未识别',
        'menu_enter_key': '回车键',
        'dialog_font_title': '选择字体或恢复文件',
        'dialog_game_title': '选择 Counter-Strike Global Offensive 文件夹',
        'dialog_restore_title': '选择 Counter-Strike Global Offensive 文件夹以进行还原',
        'dialog_font_type': '支持的文件',
        'dialog_font_ttf': '字体文件',
        'dialog_restore_file': '恢复文件',
        'error_wrong_zip': '所选 ZIP 文件名称不正确，应为 "backup_original_fonts.zip"',
        'error_invalid_font': '所选文件 "{file}" 并非有效的字体格式',
        'error_unsupported_font': '"{file}" 是不受支持的字体集',
        'press_enter': '请按回车键',
        'press_enter_exit': '请按回车键退出',
        'press_enter_return': '请按回车键返回主菜单',
        'press_enter_continue': '还原失败，按回车键返回主菜单尝试手动处理',
        'press_enter_invalid_path': '指定的路径无效，请确保选择了正确的游戏根目录',
        'ui_scale_prompt': '\033[93m•\033[0m 输入 UI 缩放倍率（推荐区间 0.9 至 1.1）：',
        'ui_scale_invalid': '请输入合法的数值',
        'restore_need_path': '执行还原程序前，请先指定游戏的安装路径',
        'restore_cancelled': '操作已取消，按回车键返回',
        'restore_failed_select': '还原失败，按回车键返回主菜单尝试手动处理',
        'restore_invalid_path': '指定的路径无效，请确保选择了正确的游戏根目录',
        'error_no_font': '尚未加载字体源文件，无法执行配置，请先按 1 导入字体',
        'error_no_path': '游戏路径配置不正确或尚未设定，请先确认路径信息',
        'error_unrecognized': '未能识别的指令，请按回车键刷新菜单',
        'deploy_success': '已将新字体文件部署至目标路径：{font_name}.ttf',
        'deploy_failed': '部署字体文件失败',
        'final_success': '\n所有配置任务已成功执行，请启动 Counter-Strike 2 以查看新的字体效果',
        'final_failed': '\n配置执行过程中可能存在不完整的操作，请检查上述错误信息',
        'task_complete': '运行任务已结束，按 [\033[92m回车键\033[0m] 复制执行日志至剪贴板，或直接关闭',
        'logs_copied': '日志已成功复制到剪贴板',
        'exception': '异常',
        'diagnostic': '诊断详情',
        'validation_check': '正在对安装结果进行一致性校验',
        'validation_failed_font': '校验失败：未能定位到目标字体文件 {file}',
        'validation_failed_conf': '校验失败：未能定位到必要配置文件 {conf_file}',
        'validation_failed_size': '校验失败：目标字体文件大小异常（0 字节）',
        'validation_success': '校验完成，所有必要文件均已就绪',
        'backing_up': '正在备份',
        'backup_created': '备份成功，\033[90m{backup_path}\033[0m',
        'backup_error': '备份失败',
        'restore_file_missing': '未发现可用的备份文件：{backup_path}',
        'restoring_fonts': '正在从复原文件 {backup_name} 还原游戏初始字体',
        'config_cleaned': '已清理自定义配置',
        'restore_complete': '还原完成',
        'restore_error': '还原失败',
        'cs2_running': '\033[93m注意：检测到 Counter-Strike 2 正在运行，可能会影响字体替换效果，建议先关闭游戏再进行操作\033[0m',
        'select_font': '选择导入字体',
        'reimport_font': '重新导入字体',
        'select_game_path': '选择游戏路径',
        'adjust_ui_scale': '调整 UI 缩放',
        'restore_default': '按 [\033[92m0\033[0m] 恢复默认字体，注意：程序首次运行时会在游戏根目录自动创建恢复文件，恢复操作需要该文件存在',
        'start_replace': '\n\033[96m•\033[0m 按 [\033[92m回车键\033[0m] 开始替换字体',
        'current': '当前',
        'not_selected': '未选择',
        'not_identified': '未识别',
        'enter_key': '回车键',
        'select_font_or_restore': '选择字体或恢复文件',
        'supported_files': '支持的文件',
        'font_files': '字体文件',
        'restore_file': '恢复文件',
        'wrong_zip_name': '所选 ZIP 文件名称不正确，应为 "backup_original_fonts.zip"',
        'invalid_font_format': '所选文件 "{file}" 并非有效的字体格式',
        'unsupported_font': '"{file}" 是不受支持的字体集',
        'press_enter_to_exit': '请按回车键退出',
        'press_enter_menu': '请按回车键返回主菜单',
        'restore_failed_menu': '还原失败，按回车键返回主菜单尝试手动处理',
        'invalid_path_specified': '指定的路径无效，请确保选择了正确的游戏根目录',
        'ui_scale_input': '\033[93m•\033[0m 输入 UI 缩放倍率（推荐区间 0.9 至 1.1）：',
        'invalid_number': '请输入合法的数值',
        'specify_path_first': '执行还原程序前，请先指定游戏的安装路径',
        'operation_cancelled': '操作已取消，按回车键返回',
        'no_font_loaded': '尚未加载字体源文件，无法执行配置，请先按 1 导入字体',
        'path_not_configured': '游戏路径配置不正确或尚未设定，请先确认路径信息',
        'unrecognized_command': '未能识别的指令，请按回车键刷新菜单',
        'font_deployed': '已将新字体文件部署至目标路径：{font_name}.ttf',
        'deploy_font_failed': '部署字体文件失败',
        'config_success': '\n所有配置任务已成功执行，请启动 Counter-Strike 2 以查看新的字体效果',
        'config_failed': '\n配置执行过程中可能存在不完整的操作，请检查上述错误信息',
        'auto_restore_detect': '检测到备份压缩包，准备执行自动还原',
        'auto_detect_failed': '未能自动检测到游戏安装路径，请手动进行选择',
        'restore_success_exit': '还原程序执行失败，请按回车键退出',
        'invalid_path_exit': '指定的路径无效或未选择任何目录，请按回车键退出',
        'send_to_dev': '请将以上错误详情发送给开发者排查，按回车键退出程序',
        'font_dir_missing': '字体目录缺失：{csgo_fonts}',
        'font_dir_created': '已自动创建字体目录',
        'core_dir_missing': '核心配置目录缺失：{core_fonts}',
        'core_dir_created': '已自动创建核心配置目录',
        'cleaning_ui_font': '正在清理旧版本的 UI 字体索引文件',
        'cleaning_ui_font_failed': '清理旧版本 UI 字体索引失败',
        'removing_ttf': '正在移除冲突的旧字体文件：{file}',
        'removing_ttf_failed': '移除旧字体文件失败：{file}',
        'generating_fonts_conf': '\n正在生成局部字体配置文件：fonts.conf',
        'generating_fonts_conf_failed': '生成 fonts.conf 失败',
        'generating_repl_conf': '正在生成全局字体映射文件：42-repl-global.conf',
        'generating_repl_conf_failed': '生成 42-repl-global.conf 失败',
    },
    'en': {
        'log_clear_desc': 'Clear the log buffer',
        'copy_to_clipboard_desc': 'Copies text to the clipboard',
        'logger_clear': 'Clear the log buffer',
        'clipboard_success': 'Logs have been successfully copied to the clipboard',
        'clipboard_action': 'Task completed, Press [\033[92mEnter\033[0m] to copy execution logs to clipboard, or close directly',
        'error_exception': 'Exception',
        'error_diagnostic': 'Diagnostic Details',
        'missing_msg': 'Font directory missing',
        'created_msg': 'Created font directory automatically',
        'core_missing_msg': 'Core configuration directory missing',
        'core_created_msg': 'Created core configuration directory automatically',
        'ui_cleaning': 'Cleaning up legacy UI font index files',
        'ui_clean_failed': 'Failed to clean up legacy UI font index',
        'ttf_removing': 'Removing conflicting legacy font: {file}',
        'ttf_remove_failed': 'Failed to remove legacy font: {file}',
        'fonts_conf_success': '\nGenerating local font configuration: fonts.conf',
        'fonts_conf_error': 'Failed to generate fonts.conf',
        'repl_conf_success': 'Generating global font mapping: 42-repl-global.conf',
        'repl_conf_error': 'Failed to generate 42-repl-global.conf',
        'verify_check': 'Performing consistency check on installation results',
        'verify_failed_font': 'Validation failed: Could not locate target font file {file}',
        'verify_failed_conf': 'Validation failed: Could not locate required configuration file {conf_file}',
        'verify_failed_size': 'Validation failed: Target font file size is abnormal (0 bytes)',
        'verify_success': 'Validation complete, all necessary files are ready',
        'backuping': 'Backing up',
        'backup_success': 'Backup success, \033[90m{backup_path}\033[0m',
        'backup_failed': 'Backup failed',
        'restore_not_found': 'No restoration file found at: {backup_path}',
        'restore_ing': 'Restoring initial game fonts from {backup_name}',
        'restore_cleaned': 'Cleaned custom config',
        'restore_success': 'Restore complete',
        'restore_failed': 'Restore failed',
        'game_running_warn': '\033[93mNote: Counter-Strike 2 is currently running, which may affect the font replacement, It is recommended to close the game before proceeding\033[0m',
        'menu_title': 'CS2 Font Changer v3.0 | Author: Cairl',
        'menu_select_font': 'select font to import',
        'menu_reimport_font': 're-import font',
        'menu_select_path': 'select game path',
        'menu_adjust_ui': 'adjust UI scale',
        'menu_restore': 'Press [\033[92m0\033[0m] to restore default fonts, Note: A restoration file is automatically created in the game root during the first run, and restoration requires this file to exist',
        'menu_start': '\n\033[96m•\033[0m Press [\033[92mEnter\033[0m] to start font replacement',
        'menu_current_font': 'current',
        'menu_current_path': 'current',
        'menu_current_scale': 'current',
        'menu_not_selected': 'Not Selected',
        'menu_not_identified': 'Not Identified',
        'menu_enter_key': 'Enter',
        'dialog_font_title': 'Select Font or Restoration File',
        'dialog_game_title': 'Select Counter-Strike Global Offensive folder',
        'dialog_restore_title': 'Select Counter-Strike Global Offensive folder for restore',
        'dialog_font_type': 'Supported Files',
        'dialog_font_ttf': 'Font Files',
        'dialog_restore_file': 'Restoration File',
        'error_wrong_zip': 'Selected ZIP file name is incorrect, Should be "backup_original_fonts.zip"',
        'error_invalid_font': 'Selected file "{file}" is not a valid font format',
        'error_unsupported_font': '"{file}" is an unsupported font',
        'press_enter': 'Press Enter',
        'press_enter_exit': 'Press Enter to exit',
        'press_enter_return': 'Press Enter to return to main menu',
        'press_enter_continue': 'Restoration failed, Press Enter to return to main menu and try manually',
        'press_enter_invalid_path': 'Invalid path specified, Please ensure you select the correct game root directory',
        'ui_scale_prompt': '\033[93m•\033[0m Enter UI scale (suggested 0.9 to 1.1): ',
        'ui_scale_invalid': 'Please enter a valid number',
        'restore_need_path': 'Please specify the game installation path before running restoration',
        'restore_cancelled': 'Operation cancelled, Press Enter to return',
        'restore_failed_select': 'Restoration failed, Press Enter to return to main menu and try manually',
        'restore_invalid_path': 'Invalid path specified, Please ensure you select the correct game root directory',
        'error_no_font': 'No font source file loaded, Cannot perform configuration, Press 1 to import font first',
        'error_no_path': 'Game path configuration incorrect or not yet set, Please confirm path info first',
        'error_unrecognized': 'Unrecognized command, Press Enter to refresh menu',
        'deploy_success': 'Deployed new font file to target: {font_name}.ttf',
        'deploy_failed': 'Failed to deploy font file',
        'final_success': '\nAll configuration tasks have been successfully executed, Please launch Counter-Strike 2 to see the new font',
        'final_failed': '\nSome operations might have failed during execution, Please review the error messages above',
        'task_complete': 'Task completed, Press [\033[92mEnter\033[0m] to copy execution logs to clipboard, or close directly',
        'logs_copied': 'Logs have been successfully copied to the clipboard',
        'exception': 'Exception',
        'diagnostic': 'Diagnostic Details',
        'validation_check': 'Performing consistency check on installation results',
        'validation_failed_font': 'Validation failed: Could not locate target font file {file}',
        'validation_failed_conf': 'Validation failed: Could not locate required configuration file {conf_file}',
        'validation_failed_size': 'Validation failed: Target font file size is abnormal (0 bytes)',
        'validation_success': 'Validation complete, all necessary files are ready',
        'backing_up': 'Backing up',
        'backup_created': 'Backup success, \033[90m{backup_path}\033[0m',
        'backup_error': 'Backup failed',
        'restore_file_missing': 'No restoration file found at: {backup_path}',
        'restoring_fonts': 'Restoring initial game fonts from {backup_name}',
        'config_cleaned': 'Cleaned custom config',
        'restore_complete': 'Restore complete',
        'restore_error': 'Restore failed',
        'cs2_running': '\033[93mNote: Counter-Strike 2 is currently running, which may affect the font replacement, It is recommended to close the game before proceeding\033[0m',
        'select_font': 'select font to import',
        'reimport_font': 're-import font',
        'select_game_path': 'select game path',
        'adjust_ui_scale': 'adjust UI scale',
        'restore_default': 'Press [\033[92m0\033[0m] to restore default fonts, Note: A restoration file is automatically created in the game root during the first run, and restoration requires this file to exist',
        'start_replace': '\n\033[96m•\033[0m Press [\033[92mEnter\033[0m] to start font replacement',
        'current': 'current',
        'not_selected': 'Not Selected',
        'not_identified': 'Not Identified',
        'enter_key': 'Enter',
        'select_font_or_restore': 'Select Font or Restoration File',
        'supported_files': 'Supported Files',
        'font_files': 'Font Files',
        'restore_file': 'Restoration File',
        'wrong_zip_name': 'Selected ZIP file name is incorrect, Should be "backup_original_fonts.zip"',
        'invalid_font_format': 'Selected file "{file}" is not a valid font format',
        'unsupported_font': '"{file}" is an unsupported font',
        'press_enter_to_exit': 'Press Enter to exit',
        'press_enter_menu': 'Press Enter to return to main menu',
        'restore_failed_menu': 'Restoration failed, Press Enter to return to main menu and try manually',
        'invalid_path_specified': 'Invalid path specified, Please ensure you select the correct game root directory',
        'ui_scale_input': '\033[93m•\033[0m Enter UI scale (suggested 0.9 to 1.1): ',
        'invalid_number': 'Please enter a valid number',
        'specify_path_first': 'Please specify the game installation path before running restoration',
        'operation_cancelled': 'Operation cancelled, Press Enter to return',
        'no_font_loaded': 'No font source file loaded, Cannot perform configuration, Press 1 to import font first',
        'path_not_configured': 'Game path configuration incorrect or not yet set, Please confirm path info first',
        'unrecognized_command': 'Unrecognized command, Press Enter to refresh menu',
        'font_deployed': 'Deployed new font file to target: {font_name}.ttf',
        'deploy_font_failed': 'Failed to deploy font file',
        'config_success': '\nAll configuration tasks have been successfully executed, Please launch Counter-Strike 2 to see the new font',
        'config_failed': '\nSome operations might have failed during execution, Please review the error messages above',
        'auto_restore_detect': 'Backup zip detected, preparing for auto-restore',
        'auto_detect_failed': 'Auto-detection of game path failed, Please specify the path manually',
        'restore_success_exit': 'Restoration failed, Press Enter to exit',
        'invalid_path_exit': 'Invalid path or no path selected, Press Enter to exit',
        'send_to_dev': 'Please send the error details above to the developer, Press Enter to exit',
        'font_dir_missing': 'Font directory missing: {csgo_fonts}',
        'font_dir_created': 'Created font directory automatically',
        'core_dir_missing': 'Core configuration directory missing: {core_fonts}',
        'core_dir_created': 'Created core configuration directory automatically',
        'cleaning_ui_font': 'Cleaning up legacy UI font index files',
        'cleaning_ui_font_failed': 'Failed to clean up legacy UI font index',
        'removing_ttf': 'Removing conflicting legacy font: {file}',
        'removing_ttf_failed': 'Failed to remove legacy font: {file}',
        'generating_fonts_conf': '\nGenerating local font configuration: fonts.conf',
        'generating_fonts_conf_failed': 'Failed to generate fonts.conf',
        'generating_repl_conf': 'Generating global font mapping: 42-repl-global.conf',
        'generating_repl_conf_failed': 'Failed to generate 42-repl-global.conf',
    }
}

def t(key, **kwargs):
    """Translate function, get message from MESSAGES based on current LANG"""
    msg = MESSAGES[LANG].get(key, MESSAGES['en'].get(key, key))
    if kwargs:
        try:
            return msg.format(**kwargs)
        except:
            return msg
    return msg

class Logger(object):
    def __init__(self):
        self.terminal = sys.stdout
        self.log = []

    def write(self, message):
        self.terminal.write(message)
        self.log.append(message)

    def flush(self):
        self.terminal.flush()

    def get_logs(self):
        import re
        ansi_escape = re.compile(r'\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])')
        return ansi_escape.sub('', "".join(self.log))

    def clear(self):
        self.log = []

sys_logger = Logger()
sys.stdout = sys_logger

def copy_to_clipboard(text):
    try:
        process = subprocess.Popen(['clip'], stdin=subprocess.PIPE, text=False)
        process.communicate(input=text.encode('gbk', errors='ignore'))
    except Exception:
        pass

def normalize_path(path):
    return path.replace('/', '\\') if path else None

def wait_for_enter(prompt):
    print(prompt, end='', flush=True)
    while True:
        if msvcrt.getch() in [b'\r', b'\n']:
            sys.stdout.write('\n')
            break
    print()

def select_file_dialog(title, filetypes):
    root = tk.Tk()
    root.withdraw()
    root.attributes('-topmost', True)
    res = filedialog.askopenfilename(title=title, filetypes=filetypes)
    root.destroy()
    return normalize_path(res)

def select_dir_dialog(title):
    root = tk.Tk()
    root.withdraw()
    root.attributes('-topmost', True)
    res = filedialog.askdirectory(title=title)
    root.destroy()
    return normalize_path(res)

def is_valid_install_location(path):
    return bool(path) and os.path.exists(path) and path.endswith('Counter-Strike Global Offensive')

def get_fonts_paths(install_location):
    csgo_fonts = os.path.join(install_location, 'game', 'csgo', 'panorama', 'fonts')
    core_fonts = os.path.join(install_location, 'game', 'core', 'panorama', 'fonts', 'conf.d')
    ui_font = os.path.join(csgo_fonts, 'stratum2.uifont')
    return csgo_fonts, core_fonts, ui_font

def ensure_directory(path, missing_msg, created_msg):
    if not os.path.exists(path):
        print(missing_msg)
        os.makedirs(path, exist_ok=True)
        print(created_msg)

def remove_existing_fonts(csgo_fonts, ui_font, ui_msg, ui_err_msg, ttf_msg_format, ttf_err_msg_format):
    if os.path.exists(ui_font):
        try:
            os.remove(ui_font)
            print(ui_msg)
        except Exception as e:
            print_error(ui_err_msg, e)
    for file in os.listdir(csgo_fonts):
        if file.endswith('.ttf'):
            try:
                os.remove(os.path.join(csgo_fonts, file))
                print(ttf_msg_format.format(file=file))
            except Exception as e:
                print_error(ttf_err_msg_format.format(file=file), e)

def write_fonts_conf(csgo_fonts, safe_font_name, ui_scale, success_msg, error_msg):
    try:
        with open(os.path.join(csgo_fonts, 'fonts.conf'), 'w', encoding='utf-8') as f:
            f.write(f"""<?xml version='1.0'?>
<!DOCTYPE fontconfig SYSTEM 'fonts.dtd'>
<fontconfig>

	<dir prefix="default">../../csgo/panorama/fonts</dir>
	<dir>WINDOWSFONTDIR</dir>
	<dir>~/.fonts</dir>
	<dir>/usr/share/fonts</dir>
	<dir>/usr/local/share/fonts</dir>
	<dir prefix="xdg">fonts</dir>

	<fontpattern>Arial</fontpattern>
	<fontpattern>.uifont</fontpattern>
	<fontpattern>notosans</fontpattern>
	<fontpattern>notoserif</fontpattern>
	<fontpattern>notomono-regular</fontpattern>
	<fontpattern>{safe_font_name}</fontpattern>
	<fontpattern>.ttf</fontpattern>
	<fontpattern>FONTFILENAME</fontpattern>
	
	<cachedir>WINDOWSTEMPDIR_FONTCONFIG_CACHE</cachedir>
	<cachedir>~/.fontconfig</cachedir>

	<!-- Vietnamese language support -->
	<match>
		<test name="lang">
			<string>vi-vn</string>
		</test>
		<test name="family" compare="contains">
			<string>Stratum2</string>
		</test>
		<test qual="all" name="family" compare="not_contains">
			<string>TF</string>
		</test>
		<test qual="all" name="family" compare="not_contains">
			<string>Mono</string>
		</test>
		<test qual="all" name="family" compare="not_contains">
			<string>ForceStratum2</string>
		</test>
		<edit name="weight" mode="assign">
			<if>
				<contains>
					<name>family</name>
					<string>Stratum2 Black</string>
				</contains>
				<int>210</int>
				<name>weight</name>
			</if>
		</edit>
		<edit name="slant" mode="assign">
			<if>
				<contains>
					<name>family</name>
					<string>Italic</string>
				</contains>
				<int>100</int>
				<name>slant</name>
			</if>
		</edit>
		<edit name="pixelsize" mode="assign">
			<if>
				<or>
					<contains>
						<name>family</name>
						<string>Condensed</string>
					</contains>
					<less_eq>
						<name>width</name>
						<int>75</int>
					</less_eq>
				</or>
				<times>
					<name>pixelsize</name>
					<double>0.7</double>
				</times>
				<times>
					<name>pixelsize</name>
					<double>0.9</double>
				</times>
			</if>
		</edit>
		<edit name="family" mode="assign" binding="same">
			<string>notosans</string>
		</edit>
	</match>

	<selectfont> 
		<rejectfont> 
			<pattern> 
				<patelt name="fontformat" > 
					<string>Type 1</string> 
				</patelt> 
			</pattern> 
		</rejectfont> 
	</selectfont> 

	<match target="font" >
		<edit name="embeddedbitmap" mode="assign">
			<bool>false</bool>
		</edit>
	</match>

	<match target="pattern" >
		<edit name="prefer_outline" mode="assign">
			<bool>true</bool>
		</edit>
	</match>

	<match target="pattern" >
		<edit name="do_substitutions" mode="assign">
			<bool>true</bool>
		</edit>
	</match>

	<match target="font">
		<edit name="force_autohint" mode="assign">
			<bool>false</bool>
		</edit>
	</match>

	<include>../../../core/panorama/fonts/conf.d</include>

	<!-- Adjust HUD font size (Money, Health, Ammo) -->
	<match target="font">
		<test name="family" compare="contains">
			<string>Stratum2</string>
		</test>
		<test name="family" compare="contains">
			<string>{safe_font_name}</string>
		</test>
		<edit name="pixelsize" mode="assign">
			<times>
				<name>pixelsize</name>
				<double>{ui_scale}</double>
			</times>
		</edit>
	</match>
	
	<!-- Custom fonts -->
	<match>
		<test name="family">
			<string>Stratum2</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>Stratum2 Bold</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>Arial</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>Times New Roman</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>Courier New</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>

	<match>
		<test name="family">
			<string>notosans</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>notoserif</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>
	
	<match>
		<test name="family">
			<string>notomono-regular</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>

	<match>
		<test name="family">
			<string>noto</string>
		</test>
		<edit name="family" mode="append" binding="strong">
			<string>{safe_font_name}</string>
		</edit>
	</match>

</fontconfig>""")
        print(success_msg)
    except Exception as e:
        print_error(error_msg, e)

def write_repl_conf(core_fonts, safe_font_name, ui_scale, success_msg, error_msg):
    try:
        with open(os.path.join(core_fonts, '42-repl-global.conf'), 'w', encoding='utf-8') as f:
            f.write(f"""<?xml version='1.0'?>
<!DOCTYPE fontconfig SYSTEM 'fonts.dtd'>
<fontconfig>

	<match target="font">
		<test name="family" compare="contains"><string>Stratum2</string></test>
		<edit name="pixelsize" mode="assign">
			<times><name>pixelsize</name><double>{ui_scale}</double></times>
		</edit>
	</match>
	<match target="font">
		<test name="family" compare="contains"><string>{safe_font_name}</string></test>
		<edit name="pixelsize" mode="assign">
			<times><name>pixelsize</name><double>{ui_scale}</double></times>
		</edit>
	</match>
""")
            fonts_to_replace = ['Stratum2', 'Stratum2 Bold', 'Arial', 'Times New Roman', 'Courier New', 'notosans', 'notoserif', 'notomono-regular', 'noto']
            for font_to_repl in fonts_to_replace:
                f.write(f"""
	<match target="font">
		<test name="family"><string>{font_to_repl}</string></test>
		<edit name="family" mode="assign"><string>{safe_font_name}</string></edit>
	</match>
	<match target="pattern">
		<test name="family"><string>{font_to_repl}</string></test>
		<edit name="family" mode="prepend" binding="strong"><string>{safe_font_name}</string></edit>
	</match>""")
            f.write("\n</fontconfig>")
        print(success_msg)
    except Exception as e:
        print_error(error_msg, e)

def read_menu_key(valid_keys, enter_label):
    try:
        while True:
            char = msvcrt.getch()
            if char in valid_keys:
                if char in [b'\r', b'\n']:
                    sys.stdout.write(f'\033[92m[{enter_label}]\033[0m\n\n')
                    return ""
                decoded_char = char.decode('utf-8')
                sys.stdout.write(f'\033[92m{decoded_char}\033[0m\n\n')
                return decoded_char.strip()
    except:
        return ""

def get_auto_install_location():
    try:
        key = winreg.OpenKey(winreg.HKEY_LOCAL_MACHINE, r'SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Uninstall\\Steam App 730')
        return winreg.QueryValueEx(key, 'InstallLocation')[0]
    except FileNotFoundError:
        return None

def finish_execution(exit_code=0):
    try:
        print(t('task_complete'), end='', flush=True)
        while True:
            char = msvcrt.getch()
            if char in [b'\r', b'\n']:
                sys.stdout.write('\n')
                copy_to_clipboard(sys_logger.get_logs())
                print(t('logs_copied'))
                break
    except (EOFError, KeyboardInterrupt):
        pass
    sys.exit(exit_code)

def print_error(message, exception=None):
    print(f'{t("exception")}: {message}')
    if exception:
        print(f'{t("diagnostic")}: {str(exception)}')

def verify_files(csgo_fonts, font_name):
    font_file = os.path.join(csgo_fonts, f"{font_name}.ttf")
    conf_file = os.path.join(csgo_fonts, 'fonts.conf')
    
    print(t('validation_check'))
    if not os.path.exists(font_file):
        print(t('validation_failed_font', file=font_file))
        return False
    if not os.path.exists(conf_file):
        print(t('validation_failed_conf', conf_file=conf_file))
        return False
    if os.path.getsize(font_file) == 0:
        print(t('validation_failed_size'))
        return False
    
    print(t('validation_success'))
    return True

def create_backup(install_location):
    backup_path = os.path.join(install_location, 'backup_original_fonts.zip')
    
    if os.path.exists(backup_path):
        return

    print(t('backing_up'))
    csgo_fonts, core_fonts, _ = get_fonts_paths(install_location)
    
    try:
        with zipfile.ZipFile(backup_path, 'w', zipfile.ZIP_DEFLATED) as zf:
            if os.path.exists(csgo_fonts):
                for root, dirs, files in os.walk(csgo_fonts):
                    for file in files:
                        file_path = os.path.join(root, file)
                        arcname = os.path.join('csgo_fonts', os.path.relpath(file_path, csgo_fonts))
                        zf.write(file_path, arcname)
            
            if os.path.exists(core_fonts):
                for root, dirs, files in os.walk(core_fonts):
                    for file in files:
                        file_path = os.path.join(root, file)
                        arcname = os.path.join('core_fonts', os.path.relpath(file_path, core_fonts))
                        zf.write(file_path, arcname)
            
        print(t('backup_created', backup_path=backup_path))
    except Exception as e:
        print_error(t('backup_error'), e)

def restore_backup(install_location, backup_path_override=None):
    backup_path = backup_path_override if backup_path_override else os.path.join(install_location, 'backup_original_fonts.zip')
    
    if not os.path.exists(backup_path):
        print(t('restore_file_missing', backup_path=backup_path))
        return False

    print(t('restoring_fonts', backup_name=os.path.basename(backup_path)))
    csgo_fonts, core_fonts, _ = get_fonts_paths(install_location)

    try:
        if os.path.exists(csgo_fonts):
            for item in os.listdir(csgo_fonts):
                item_path = os.path.join(csgo_fonts, item)
                if os.path.isdir(item_path):
                    shutil.rmtree(item_path)
                else:
                    os.remove(item_path)
        
        with zipfile.ZipFile(backup_path, 'r') as zf:
            for item in zf.infolist():
                if item.filename.startswith('csgo_fonts/'):
                    rel_path = item.filename[len('csgo_fonts/'):]
                    if not rel_path: continue
                    target_path = os.path.join(csgo_fonts, rel_path)
                    os.makedirs(os.path.dirname(target_path), exist_ok=True)
                    with zf.open(item) as source, open(target_path, 'wb') as target:
                        shutil.copyfileobj(source, target)
                elif item.filename.startswith('core_fonts/'):
                    rel_path = item.filename[len('core_fonts/'):]
                    if not rel_path: continue
                    target_path = os.path.join(core_fonts, rel_path)
                    os.makedirs(os.path.dirname(target_path), exist_ok=True)
                    with zf.open(item) as source, open(target_path, 'wb') as target:
                        shutil.copyfileobj(source, target)
        
        repl_conf = os.path.join(core_fonts, '42-repl-global.conf')
        with zipfile.ZipFile(backup_path, 'r') as zf:
            if 'core_fonts/42-repl-global.conf' not in zf.namelist():
                if os.path.exists(repl_conf):
                    os.remove(repl_conf)
                    print(t('config_cleaned'))

        print(t('restore_complete'))
        return True
    except Exception as e:
        print_error(t('restore_error'), e)
        return False

os.system('')

def is_game_running():
    try:
        output = subprocess.check_output('tasklist /FI "IMAGENAME eq cs2.exe" /NH', shell=True, stderr=subprocess.DEVNULL).decode('gbk', errors='ignore')
        return 'cs2.exe' in output.lower()
    except:
        return False

def get_font_name(file_path):
    try:
        font = TTFont(file_path)
        font_name = next((record.toUnicode().strip() 
            for record in font['name'].names 
            if record.nameID == 1 and record.platformID == 3), None)
        return font_name
    except Exception:
        return None

input_file = sys.argv[1] if len(sys.argv) == 2 else None
font_name = None

if input_file and os.path.isfile(input_file):
    if os.path.basename(input_file).lower() == 'backup_original_fonts.zip':
        print(t('auto_restore_detect'))
        install_location = get_auto_install_location()
        
        if not install_location:
            print(t('auto_detect_failed'))
            install_location = select_dir_dialog(t('dialog_restore_title'))
        
        if is_valid_install_location(install_location):
            if restore_backup(install_location, input_file):
                finish_execution(0)
            else:
                input(t('restore_success_exit'))
                finish_execution(1)
        else:
            input(t('invalid_path_exit'))
            finish_execution(1)

    if input_file.endswith(('.ttf', '.otf')):
        font_name = get_font_name(input_file)
        if not font_name:
            print_error(t('unsupported_font', file=os.path.basename(input_file)))
            input(t('send_to_dev'))
            finish_execution(1)

auto_install_location = get_auto_install_location()

install_location = auto_install_location

ui_scale = 1.0

while True:
    os.system('cls' if os.name == 'nt' else 'clear')
    sys_logger.clear()
    
    print(t('menu_title'))
    print()

    if is_game_running():
        print(t('cs2_running'))

    has_backup = False
    if install_location:
        backup_path = os.path.join(install_location, 'backup_original_fonts.zip')
        has_backup = os.path.exists(backup_path)

    can_start = bool(input_file) and is_valid_install_location(install_location)
    print(f"\033[96m[1]\033[0m {t('menu_select_font') if not input_file else t('menu_reimport_font')}, {t('current')}: \033[90m{font_name or t('not_selected')}\033[0m")
    print(f"\033[96m[2]\033[0m {t('menu_select_path')}, {t('current')}: \033[90m{install_location or t('not_identified')}\033[0m")
    print(f"\033[96m[3]\033[0m {t('menu_adjust_ui')}, {t('current')}: \033[90m{ui_scale}\033[0m")
    if has_backup:
        print(f"\033[96m[0]\033[0m {t('restore_default')}")
    if can_start:
        print(t('menu_start'))
    
    sys.stdout.write('\n> ')
    sys.stdout.flush()
    
    valid_keys = [b'1', b'2', b'3']
    if has_backup:
        valid_keys.append(b'0')
    if can_start:
        valid_keys.extend([b'\r', b'\n'])

    user_input = read_menu_key(valid_keys, t('enter_key'))
    
    sys.stdout.flush()

    if user_input == '1':
        selected_file = select_file_dialog(
            t('select_font_or_restore'),
            [(t('supported_files'), "*.ttf;*.otf;*.zip"), (t('font_files'), "*.ttf;*.otf"), (t('restore_file'), "backup_original_fonts.zip")]
        )
        if selected_file:
            input_file = selected_file
            if input_file.lower().endswith('.zip'):
                if os.path.basename(input_file) == 'backup_original_fonts.zip':
                    font_name = t('restore_file')
                else:
                    print_error(t('wrong_zip_name'))
                    input_file = None
                    wait_for_enter(t('press_enter_menu'))
            else:
                font_name = get_font_name(input_file)
                if not font_name:
                    print_error(t('invalid_font_format', file=os.path.basename(input_file)))
                    input_file = None
                    wait_for_enter(t('press_enter_menu'))
        continue

    elif user_input == '2':
        selected_path = select_dir_dialog(t('dialog_game_title'))
        if selected_path:
            install_location = selected_path
        else:
            continue

    elif user_input == '3':
        while True:
            try:
                val = input(t('ui_scale_input'))
                if not val: break
                ui_scale = float(val)
                print()
                break
            except ValueError:
                print(t('invalid_number'))
        continue

    elif user_input == '0' and has_backup:
        if not install_location:
            print(t('specify_path_first'))
            selected_path = select_dir_dialog(t('dialog_restore_title'))
            if selected_path:
                install_location = selected_path
            else:
                input(t('operation_cancelled'))
                continue
        
        if is_valid_install_location(install_location):
            if restore_backup(install_location):
                finish_execution(0)
            else:
                input(t('restore_failed_menu'))
                continue
        else:
            input(t('invalid_path_specified'))
            continue

    elif user_input == "":
        if not input_file:
            input(t('no_font_loaded'))
            continue
        if not is_valid_install_location(install_location):
            input(t('path_not_configured'))
            continue
        
        create_backup(install_location)
        break

    else:
        input(t('unrecognized_command'))
        continue

csgo_fonts, core_fonts, ui_font = get_fonts_paths(install_location)

ensure_directory(csgo_fonts, t('font_dir_missing', csgo_fonts=csgo_fonts), t('font_dir_created'))
ensure_directory(core_fonts, t('core_dir_missing', core_fonts=core_fonts), t('core_dir_created'))

remove_existing_fonts(
    csgo_fonts,
    ui_font,
    t('cleaning_ui_font'),
    t('cleaning_ui_font_failed'),
    t('removing_ttf'),
    t('removing_ttf_failed')
)

try:
    safe_font_name = font_name
    shutil.copy(input_file, os.path.join(csgo_fonts, f"{font_name}.ttf"))
    print(t('font_deployed', font_name=font_name))
except Exception as e:
    print_error(t('deploy_font_failed'), e)
    finish_execution(1)

write_fonts_conf(csgo_fonts, safe_font_name, ui_scale, t('generating_fonts_conf'), t('generating_fonts_conf_failed'))
write_repl_conf(core_fonts, safe_font_name, ui_scale, t('generating_repl_conf'), t('generating_repl_conf_failed'))

if verify_files(csgo_fonts, font_name):
    print(t('config_success'))
    finish_execution(0)
else:
    print(t('config_failed'))
    finish_execution(1)
