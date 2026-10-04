//! CS2 Font Changer — Rust port of cs2_font_changer.py

mod backup;
mod clipboard;
mod config;
mod console;
mod dialog;
mod font;
mod game;
mod i18n;
mod logger;
mod registry;

use std::path::Path;

use crate::backup::{create_backup, restore_backup, BACKUP_FILENAME};
use crate::console::{clear_screen, enter_label, input_discard, read_line, read_menu_key, wait_for_enter};
use crate::font::{get_font_name, xml_escape};
use crate::game::{get_fonts_paths, is_game_running, is_valid_install_location};
use crate::i18n::{t, t_with};

/// Mirror of Python `print_error`: print `exception: message` and optionally `diagnostic: err`.
pub fn print_error(message: &str, exception: Option<&dyn std::error::Error>) {
    lprintln!("{}: {}", t("exception"), message);
    if let Some(e) = exception {
        lprintln!("{}: {}", t("diagnostic"), e);
    }
}

/// Mirror of Python `finish_execution`: print task_complete, wait for Enter, copy logs, exit.
fn finish_execution(exit_code: i32) -> ! {
    use crate::console::getch;
    lprint!("{}", t("task_complete"));
    let _ = std::io::Write::flush(&mut std::io::stdout());
    loop {
        if let Some(c) = getch() {
            if c == '\r' || c == '\n' {
                lprint!("\n");
                let logs = logger::get_logs_plain();
                clipboard::copy_to_clipboard(&logs);
                lprintln!("{}", t("logs_copied"));
                break;
            }
        }
    }
    std::process::exit(exit_code);
}

/// Mirror of Python `ensure_directory`.
fn ensure_directory(path: &Path, missing_msg: String, created_msg: &str) {
    if !path.exists() {
        lprintln!("{}", missing_msg);
        match std::fs::create_dir_all(path) {
            Ok(_) => lprintln!("{}", created_msg),
            Err(e) => {
                print_error(&format!("Failed to create directory: {}", path.display()), Some(&e));
                finish_execution(1);
            }
        }
    }
}

/// Mirror of Python `remove_existing_fonts`.
fn remove_existing_fonts(
    csgo_fonts: &Path,
    ui_font: &Path,
    ui_msg: &str,
    ui_err_msg: &str,
) {
    if !csgo_fonts.is_dir() {
        print_error(
            &format!("Font directory does not exist: {}", csgo_fonts.display()),
            None,
        );
        return;
    }
    if ui_font.exists() {
        match std::fs::remove_file(ui_font) {
            Ok(_) => lprintln!("{}", ui_msg),
            Err(e) => print_error(ui_err_msg, Some(&e)),
        }
    }
    let entries = match std::fs::read_dir(csgo_fonts) {
        Ok(e) => e,
        Err(e) => {
            print_error(
                &format!("Failed to list font directory: {}", csgo_fonts.display()),
                Some(&e),
            );
            return;
        }
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with(".ttf") {
            match std::fs::remove_file(entry.path()) {
                Ok(_) => lprintln!("{}", t_with("removing_ttf", &[("file", &name)])),
                Err(e) => {
                    let msg = t_with("removing_ttf_failed", &[("file", &name)]);
                    print_error(&msg, Some(&e));
                }
            }
        }
    }
}

/// Mirror of Python `verify_files`.
fn verify_files(csgo_fonts: &Path, font_name: &str) -> bool {
    let font_file = csgo_fonts.join(format!("{}.ttf", font_name));
    let conf_file = csgo_fonts.join("fonts.conf");

    lprintln!("{}", t("validation_check"));
    if !font_file.exists() {
        lprintln!(
            "{}",
            t_with("validation_failed_font", &[("file", &font_file.to_string_lossy())])
        );
        return false;
    }
    if !conf_file.exists() {
        lprintln!(
            "{}",
            t_with("validation_failed_conf", &[("conf_file", &conf_file.to_string_lossy())])
        );
        return false;
    }
    if font_file.metadata().map(|m| m.len()).unwrap_or(0) == 0 {
        lprintln!("{}", t("validation_failed_size"));
        return false;
    }
    lprintln!("{}", t("validation_success"));
    true
}

fn main() {
    console::init_console();

    let args: Vec<String> = std::env::args().collect();
    let mut input_file: Option<String> = if args.len() == 2 { Some(args[1].clone()) } else { None };
    let mut font_name: Option<String> = None;

    // Drag-and-drop handling
    if let Some(ref f) = input_file {
        if Path::new(f).is_file() {
            // Strip MOTW so the user doesn't get a Windows security prompt for this font
            // every time they launch the program with it.
            crate::font::unblock_file(f);

            let basename = Path::new(f)
                .file_name()
                .map(|s| s.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            if basename == BACKUP_FILENAME {
                lprintln!("{}", t("auto_restore_detect"));
                let mut install_location = registry::get_auto_install_location();
                if install_location.is_none() {
                    lprintln!("{}", t("auto_detect_failed"));
                    install_location = dialog::select_dir_dialog(&t("dialog_restore_title"));
                }
                match install_location {
                    Some(loc) if is_valid_install_location(&loc) => {
                        if restore_backup(&loc, Some(f)) {
                            finish_execution(0);
                        } else {
                            input_discard(&t("restore_success_exit"));
                            finish_execution(1);
                        }
                    }
                    _ => {
                        input_discard(&t("invalid_path_exit"));
                        finish_execution(1);
                    }
                }
            }

            let lower = f.to_lowercase();
            if lower.ends_with(".ttf") || lower.ends_with(".otf") {
                match get_font_name(f) {
                    Some(name) => font_name = Some(name),
                    None => {
                        let basename = Path::new(f)
                            .file_name()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        print_error(&t_with("unsupported_font", &[("file", &basename)]), None);
                        input_discard(&t("send_to_dev"));
                        finish_execution(1);
                    }
                }
            }
        }
    }

    let mut install_location: Option<String> = registry::get_auto_install_location();
    let mut ui_scale: f64 = 1.0;

    // Main menu loop
    loop {
        clear_screen();
        logger::clear();

        lprintln!("{}", t("menu_title"));
        lprintln!();

        if is_game_running() {
            lprintln!("{}", t("cs2_running"));
        }

        let mut has_backup = false;
        if let Some(ref loc) = install_location {
            let backup_path = Path::new(loc).join(BACKUP_FILENAME);
            has_backup = backup_path.exists();
        }

        let can_start = input_file.is_some()
            && install_location
                .as_ref()
                .map(|p| is_valid_install_location(p))
                .unwrap_or(false);

        let font_label = if input_file.is_some() {
            t("menu_reimport_font")
        } else {
            t("menu_select_font")
        };
        let current_font = font_name.clone().unwrap_or_else(|| t("not_selected"));
        let current_path = install_location.clone().unwrap_or_else(|| t("not_identified"));

        lprintln!(
            "\x1b[96m[1]\x1b[0m {}, {}: \x1b[90m{}\x1b[0m",
            font_label,
            t("current"),
            current_font
        );
        lprintln!(
            "\x1b[96m[2]\x1b[0m {}, {}: \x1b[90m{}\x1b[0m",
            t("menu_select_path"),
            t("current"),
            current_path
        );
        lprintln!(
            "\x1b[96m[3]\x1b[0m {}, {}: \x1b[90m{}\x1b[0m",
            t("menu_adjust_ui"),
            t("current"),
            ui_scale
        );
        if has_backup {
            lprintln!("\x1b[96m[0]\x1b[0m {}", t("restore_default"));
        }
        if can_start {
            lprintln!("{}", t("menu_start"));
        }

        lprint!("\n> ");
        use std::io::Write;
        let _ = std::io::stdout().flush();

        let mut valid_keys: Vec<char> = vec!['1', '2', '3'];
        if has_backup {
            valid_keys.push('0');
        }
        if can_start {
            valid_keys.push('\r');
        }

        let user_input = read_menu_key(&valid_keys, &enter_label());

        match user_input.as_str() {
            "1" => {
                let selected = dialog::select_file_dialog(
                    &t("select_font_or_restore"),
                    &[
                        (&t("supported_files"), "*.ttf;*.otf;*.zip"),
                        (&t("font_files"), "*.ttf;*.otf"),
                        (&t("restore_file"), BACKUP_FILENAME),
                    ],
                );
                if let Some(selected_file) = selected {
                    let lower = selected_file.to_lowercase();
                    if lower.ends_with(".zip") {
                        let basename = Path::new(&selected_file)
                            .file_name()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        if basename == BACKUP_FILENAME {
                            // Restore path chosen — restore instead of setting font
                            if install_location.is_none() {
                                lprintln!("{}", t("specify_path_first"));
                                match dialog::select_dir_dialog(&t("dialog_restore_title")) {
                                    Some(p) => install_location = Some(p),
                                    None => {
                                        input_discard(&t("operation_cancelled"));
                                        continue;
                                    }
                                }
                            }
                            let loc = install_location.clone().unwrap();
                            if is_valid_install_location(&loc) {
                                if restore_backup(&loc, Some(&selected_file)) {
                                    finish_execution(0);
                                } else {
                                    input_discard(&t("restore_failed_menu"));
                                    continue;
                                }
                            } else {
                                input_discard(&t("invalid_path_specified"));
                                continue;
                            }
                        } else {
                            print_error(&t("wrong_zip_name"), None);
                            wait_for_enter(&t("press_enter_menu"));
                        }
                    } else {
                        // ttf/otf
                        match get_font_name(&selected_file) {
                            Some(name) => {
                                input_file = Some(selected_file);
                                font_name = Some(name);
                            }
                            None => {
                                let basename = Path::new(&selected_file)
                                    .file_name()
                                    .map(|s| s.to_string_lossy().into_owned())
                                    .unwrap_or_default();
                                print_error(
                                    &t_with("invalid_font_format", &[("file", &basename)]),
                                    None,
                                );
                                input_file = None;
                                font_name = None;
                                wait_for_enter(&t("press_enter_menu"));
                            }
                        }
                    }
                }
                continue;
            }
            "2" => {
                if let Some(selected_path) = dialog::select_dir_dialog(&t("dialog_game_title")) {
                    install_location = Some(selected_path);
                }
                continue;
            }
            "3" => {
                loop {
                    lprint!("{}", t("ui_scale_input"));
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                    let val = read_line();
                    if val.is_empty() {
                        break;
                    }
                    match val.parse::<f64>() {
                        Ok(parsed) if parsed.is_finite() && (0.5..=2.0).contains(&parsed) => {
                            ui_scale = parsed;
                            lprintln!();
                            break;
                        }
                        _ => {
                            lprintln!("{}", t("invalid_number"));
                        }
                    }
                }
                continue;
            }
            "0" if has_backup => {
                if install_location.is_none() {
                    lprintln!("{}", t("specify_path_first"));
                    match dialog::select_dir_dialog(&t("dialog_restore_title")) {
                        Some(p) => install_location = Some(p),
                        None => {
                            input_discard(&t("operation_cancelled"));
                            continue;
                        }
                    }
                }
                let loc = install_location.clone().unwrap();
                if is_valid_install_location(&loc) {
                    if restore_backup(&loc, None) {
                        finish_execution(0);
                    } else {
                        input_discard(&t("restore_failed_menu"));
                        continue;
                    }
                } else {
                    input_discard(&t("invalid_path_specified"));
                    continue;
                }
            }
            "" => {
                if input_file.is_none() {
                    input_discard(&t("no_font_loaded"));
                    continue;
                }
                let loc = match install_location.as_ref() {
                    Some(l) if is_valid_install_location(l) => l.clone(),
                    _ => {
                        input_discard(&t("path_not_configured"));
                        continue;
                    }
                };
                create_backup(&loc);
                // Break out of menu loop with the values needed for deployment
                deploy(
                    &loc,
                    input_file.as_ref().unwrap(),
                    font_name.as_ref().unwrap(),
                    ui_scale,
                );
            }
            _ => {
                input_discard(&t("unrecognized_command"));
                continue;
            }
        }
    }
}

/// The deployment pipeline — runs after the user presses Enter in the menu.
fn deploy(install_location: &str, input_file: &str, font_name: &str, ui_scale: f64) -> ! {
    let paths = get_fonts_paths(install_location);

    ensure_directory(
        &paths.csgo_fonts,
        t_with(
            "font_dir_missing",
            &[("csgo_fonts", &paths.csgo_fonts.to_string_lossy())],
        ),
        &t("font_dir_created"),
    );
    ensure_directory(
        &paths.core_fonts,
        t_with(
            "core_dir_missing",
            &[("core_fonts", &paths.core_fonts.to_string_lossy())],
        ),
        &t("core_dir_created"),
    );

    remove_existing_fonts(
        &paths.csgo_fonts,
        &paths.ui_font,
        &t("cleaning_ui_font"),
        &t("cleaning_ui_font_failed"),
    );

    let safe_font_name = xml_escape(font_name);
    let target_font = paths.csgo_fonts.join(format!("{}.ttf", font_name));
    match std::fs::copy(input_file, &target_font) {
        Ok(_) => lprintln!("{}", t_with("font_deployed", &[("font_name", font_name)])),
        Err(e) => {
            print_error(&t("deploy_font_failed"), Some(&e));
            finish_execution(1);
        }
    }

    config::write_fonts_conf(
        &paths.csgo_fonts,
        &safe_font_name,
        ui_scale,
        &t("generating_fonts_conf"),
        &t("generating_fonts_conf_failed"),
    );
    config::write_repl_conf(
        &paths.core_fonts,
        &safe_font_name,
        ui_scale,
        &t("generating_repl_conf"),
        &t("generating_repl_conf_failed"),
    );

    if verify_files(&paths.csgo_fonts, font_name) {
        lprintln!("{}", t("config_success"));
        finish_execution(0);
    } else {
        lprintln!("{}", t("config_failed"));
        finish_execution(1);
    }
}
