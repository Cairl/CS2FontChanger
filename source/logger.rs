//! Tee-style logger: writes go to stdout and are also captured in memory,
//! so the full session log can be copied to the clipboard on exit.

use std::io::Write;
use std::sync::{Mutex, OnceLock};

static LOG_BUFFER: OnceLock<Mutex<Vec<u8>>> = OnceLock::new();

fn buffer() -> &'static Mutex<Vec<u8>> {
    LOG_BUFFER.get_or_init(|| Mutex::new(Vec::new()))
}

/// Print to stdout and append to the in-memory log.
pub fn log_print(msg: &str) {
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(msg.as_bytes());
    let _ = out.flush();
    if let Ok(mut buf) = buffer().lock() {
        buf.extend_from_slice(msg.as_bytes());
    }
}

/// `print!` equivalent routed through the logger.
#[macro_export]
macro_rules! lprint {
    ($($arg:tt)*) => {{
        $crate::logger::log_print(&format!($($arg)*));
    }};
}

/// `println!` equivalent routed through the logger.
#[macro_export]
macro_rules! lprintln {
    () => { $crate::lprint!("\n") };
    ($($arg:tt)*) => {{
        $crate::lprint!("{}\n", format!($($arg)*));
    }};
}

/// Clear the in-memory log (called each time the menu redraws).
pub fn clear() {
    if let Ok(mut buf) = buffer().lock() {
        buf.clear();
    }
}

/// Return the captured log with ANSI escape sequences stripped.
pub fn get_logs_plain() -> String {
    let raw = match buffer().lock() {
        Ok(buf) => String::from_utf8_lossy(&buf).into_owned(),
        Err(_) => String::new(),
    };
    strip_ansi(&raw)
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\u{1B}' && chars.peek() == Some(&'[') {
            chars.next(); // consume '['
            // Consume until a final byte in 0x40..=0x7E that isn't a parameter digit/;/?
            for c2 in chars.by_ref() {
                if c2.is_ascii_alphabetic() || c2 == '@' || c2 == '~' {
                    break;
                }
            }
            continue;
        }
        out.push(c);
    }
    out
}
