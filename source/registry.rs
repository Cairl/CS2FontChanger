//! Registry: locate the CS2 install path via the same three registry probes as the Python version.

use std::path::Path;
use windows::core::PCWSTR;
use windows::Win32::System::Registry::{RegOpenKeyExW, RegQueryValueExW, HKEY, HKEY_LOCAL_MACHINE, KEY_READ, REG_SZ, REG_EXPAND_SZ, REG_VALUE_TYPE};

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn read_reg_string(subkey: &str, value_name: &str) -> Option<String> {
    unsafe {
        let mut hkey: HKEY = HKEY::default();
        let subkey_w = to_wide(subkey);
        if RegOpenKeyExW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(subkey_w.as_ptr()),
            0,
            KEY_READ,
            &mut hkey,
        )
        .is_err()
        {
            return None;
        }

        let value_w = to_wide(value_name);
        let mut data_type: REG_VALUE_TYPE = REG_VALUE_TYPE(0);
        let mut data_size: u32 = 0;

        // First query: get required buffer size.
        let status = RegQueryValueExW(
            hkey,
            PCWSTR(value_w.as_ptr()),
            None,
            Some(&mut data_type),
            None,
            Some(&mut data_size),
        );
        if status.is_err() || data_size == 0 {
            let _ = windows::Win32::System::Registry::RegCloseKey(hkey);
            return None;
        }
        if data_type != REG_SZ && data_type != REG_EXPAND_SZ {
            let _ = windows::Win32::System::Registry::RegCloseKey(hkey);
            return None;
        }

        let mut buf: Vec<u16> = vec![0; (data_size as usize).div_ceil(2)];
        let status = RegQueryValueExW(
            hkey,
            PCWSTR(value_w.as_ptr()),
            None,
            None,
            Some(buf.as_mut_ptr() as *mut u8),
            Some(&mut data_size),
        );
        let _ = windows::Win32::System::Registry::RegCloseKey(hkey);
        if status.is_err() {
            return None;
        }
        // Trim trailing NUL
        let len = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
        String::from_utf16(&buf[..len]).ok()
    }
}

/// Mirror of Python `get_auto_install_location()`:
/// 1) Valve\cs2 (both 64-bit and WOW6432Node views)
/// 2) Uninstall\Steam App 730 (legacy)
/// 3) Valve\Steam\InstallPath + steamapps\common\Counter-Strike Global Offensive
pub fn get_auto_install_location() -> Option<String> {
    // 1) Direct cs2 key
    for path in &[r"SOFTWARE\WOW6432Node\Valve\cs2", r"SOFTWARE\Valve\cs2"] {
        if let Some(v) = read_reg_string(path, "installpath") {
            if !v.is_empty() && Path::new(&v).is_dir() {
                return Some(v);
            }
        }
    }

    // 2) Legacy Steam App 730 uninstall info
    for path in &[
        r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 730",
        r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall\Steam App 730",
    ] {
        if let Some(v) = read_reg_string(path, "InstallLocation") {
            if !v.is_empty() && Path::new(&v).is_dir() {
                return Some(v);
            }
        }
    }

    // 3) Steam InstallPath + default library layout
    for path in &[r"SOFTWARE\Valve\Steam", r"SOFTWARE\WOW6432Node\Valve\Steam"] {
        if let Some(steam_path) = read_reg_string(path, "InstallPath") {
            let candidate = Path::new(&steam_path)
                .join("steamapps")
                .join("common")
                .join("Counter-Strike Global Offensive");
            if candidate.is_dir() {
                return candidate.to_string_lossy().into_owned().into();
            }
        }
    }

    None
}
