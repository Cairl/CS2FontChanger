//! Game-related helpers: install location validation, path layout, process detection.

use std::path::{Path, PathBuf};
use windows::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};

pub struct FontsPaths {
    pub csgo_fonts: PathBuf,
    pub core_fonts: PathBuf,
    pub ui_font: PathBuf,
}

pub fn get_fonts_paths(install_location: &str) -> FontsPaths {
    let base = Path::new(install_location);
    let csgo_fonts = base
        .join("game")
        .join("csgo")
        .join("panorama")
        .join("fonts");
    let core_fonts = base
        .join("game")
        .join("core")
        .join("panorama")
        .join("fonts")
        .join("conf.d");
    let ui_font = csgo_fonts.join("stratum2.uifont");
    FontsPaths {
        csgo_fonts,
        core_fonts,
        ui_font,
    }
}

/// Mirror of Python `is_valid_install_location`:
/// - non-empty, is a directory
/// - normalized path ends with "Counter-Strike Global Offensive"
/// - game\bin\win64\cs2.exe exists (guards against picking an unrelated folder with the same name)
pub fn is_valid_install_location(path: &str) -> bool {
    if path.is_empty() {
        return false;
    }
    let p = Path::new(path);
    if !p.is_dir() {
        return false;
    }
    let normalized = p
        .components()
        .as_path()
        .to_string_lossy()
        .trim_end_matches(['\\', '/'])
        .to_string();
    if !normalized.ends_with("Counter-Strike Global Offensive") {
        return false;
    }
    p.join("game")
        .join("bin")
        .join("win64")
        .join("cs2.exe")
        .is_file()
}

/// Detect cs2.exe via ToolHelp snapshot (replaces the tasklist subprocess, avoids a console flash).
pub fn is_game_running() -> bool {
    unsafe {
        let snapshot = match CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) {
            Ok(h) => h,
            Err(_) => return false,
        };
        let mut entry: PROCESSENTRY32W = std::mem::zeroed();
        entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

        let mut found = false;
        if Process32FirstW(snapshot, &mut entry).is_ok() {
            loop {
                let len = entry
                    .szExeFile
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(entry.szExeFile.len());
                if let Ok(name) = String::from_utf16(&entry.szExeFile[..len]) {
                    if name.eq_ignore_ascii_case("cs2.exe") {
                        found = true;
                        break;
                    }
                }
                if Process32NextW(snapshot, &mut entry).is_err() {
                    break;
                }
            }
        }
        let _ = windows::Win32::Foundation::CloseHandle(snapshot);
        found
    }
}
