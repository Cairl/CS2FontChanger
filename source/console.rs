//! Console helpers: ANSI enablement, screen clear, and msvcrt.getch()-style single-key reads.

use std::io::Write;
use windows::Win32::System::Console::{
    GetStdHandle, ReadConsoleInputW, SetConsoleMode, CONSOLE_MODE,
    ENABLE_PROCESSED_INPUT, ENABLE_VIRTUAL_TERMINAL_PROCESSING, INPUT_RECORD, KEY_EVENT,
    STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};

use crate::i18n;
use crate::lprint;

/// Enable ANSI/VT escape processing on stdout so `\x1b[96m` etc. render as colors.
pub fn init_console() {
    unsafe {
        if let Ok(h) = GetStdHandle(STD_OUTPUT_HANDLE) {
            let mut mode = CONSOLE_MODE(0);
            if windows::Win32::System::Console::GetConsoleMode(h, &mut mode).is_ok() {
                let _ = SetConsoleMode(
                    h,
                    mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING | ENABLE_PROCESSED_INPUT,
                );
            }
        }
    }
}

/// Clear the console (equivalent of `cls`) and scroll the buffer.
pub fn clear_screen() {
    // Simple approach: emit the VT sequence. Works once init_console() ran.
    lprint!("\x1b[2J\x1b[H");
    let _ = std::io::stdout().flush();
}

/// Read a single key press from the console, returning the character it produced.
/// Returns `'\r'` for Enter. Ignores key-up events and non-character keys.
/// Mirrors `msvcrt.getch()` semantics closely enough for menu use.
pub fn getch() -> Option<char> {
    unsafe {
        let h = GetStdHandle(STD_INPUT_HANDLE).ok()?;
        let mut record: [INPUT_RECORD; 1] = [std::mem::zeroed()];
        let mut read: u32 = 0;
        loop {
            if ReadConsoleInputW(h, &mut record, &mut read).is_err() {
                return None;
            }
            if record[0].EventType as u32 != KEY_EVENT {
                continue;
            }
            let ke = record[0].Event.KeyEvent;
            // Only respond to key-down (bKeyDown != 0); ignore key repeats beyond first.
            if !ke.bKeyDown.as_bool() {
                continue;
            }
            let ch = char::from_u32(ke.uChar.UnicodeChar as u32)?;
            if ch == '\0' {
                continue;
            }
            return Some(ch);
        }
    }
}

/// Python `read_menu_key`: loop until one of `valid` chars is pressed, echo it back in green,
/// return "" for Enter, otherwise the character.
pub fn read_menu_key(valid: &[char], enter_label: &str) -> String {
    loop {
        let Some(c) = getch() else { continue };
        let c = if c == '\n' { '\r' } else { c };
        if !valid.contains(&c) {
            continue;
        }
        if c == '\r' {
            lprint!("\x1b[92m[{}]\x1b[0m\n\n", enter_label);
            return String::new();
        }
        lprint!("\x1b[92m{}\x1b[0m\n\n", c);
        return c.to_string();
    }
}

/// Python `wait_for_enter`: print prompt, wait for Enter, then a newline.
pub fn wait_for_enter(prompt: &str) {
    lprint!("{}", prompt);
    let _ = std::io::stdout().flush();
    loop {
        if let Some(c) = getch() {
            if c == '\r' || c == '\n' {
                lprint!("\n");
                break;
            }
        }
    }
    lprint!("\n");
}

/// Read a line of text from stdin (for ui_scale input). Trims trailing newline.
pub fn read_line() -> String {
    let mut s = String::new();
    let _ = std::io::stdin().read_line(&mut s);
    s.trim_end_matches(['\r', '\n']).to_string()
}

/// Stub matching the Python `input(t(...))` idiom: print prompt, read a line, discard result.
pub fn input_discard(prompt: &str) {
    lprint!("{}", prompt);
    let _ = std::io::stdout().flush();
    let _ = read_line();
    // Mimic Python input() adding a newline after user presses Enter
    lprint!("\n");
}

pub fn enter_label() -> String {
    i18n::t("enter_key")
}
