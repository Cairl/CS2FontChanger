//! Native Windows dialogs: GetOpenFileNameW and SHBrowseForFolderW.

use windows::core::PCWSTR;
use windows::Win32::Foundation::{HWND, MAX_PATH};
use windows::Win32::UI::Controls::Dialogs::{GetOpenFileNameW, OPENFILENAMEW, OFN_FILEMUSTEXIST, OFN_NOCHANGEDIR};
use windows::Win32::UI::Shell::{SHBrowseForFolderW, SHGetPathFromIDListW, BROWSEINFOW, BIF_NEWDIALOGSTYLE, BIF_RETURNONLYFSDIRS};

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Build a double-NUL-terminated filter string.
/// `specs` is a list of (label, pattern) pairs, e.g. ("Font Files", "*.ttf;*.otf").
fn build_filter(specs: &[(&str, &str)]) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::new();
    for (label, pattern) in specs {
        out.extend(label.encode_utf16());
        out.push(0);
        out.extend(pattern.encode_utf16());
        out.push(0);
    }
    out.push(0); // final NUL
    out
}

/// Open-file dialog. Returns the selected path with forward slashes converted to backslashes
/// (mirroring Python's `normalize_path`).
pub fn select_file_dialog(title: &str, filters: &[(&str, &str)]) -> Option<String> {
    unsafe {
        let mut file_buf: Vec<u16> = vec![0; MAX_PATH as usize];
        let title_w = to_wide(title);
        let filter_w = build_filter(filters);

        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = HWND(std::ptr::null_mut());
        ofn.lpstrTitle = PCWSTR(title_w.as_ptr());
        ofn.lpstrFilter = PCWSTR(filter_w.as_ptr());
        ofn.lpstrFile = windows::core::PWSTR(file_buf.as_mut_ptr());
        ofn.nMaxFile = MAX_PATH;
        ofn.Flags = OFN_FILEMUSTEXIST | OFN_NOCHANGEDIR;

        if !GetOpenFileNameW(&mut ofn).as_bool() {
            return None;
        }
        let len = file_buf.iter().position(|&c| c == 0).unwrap_or(file_buf.len());
        let path = String::from_utf16(&file_buf[..len]).ok()?;
        if path.is_empty() {
            return None;
        }
        Some(path.replace('/', "\\"))
    }
}

/// Directory picker dialog. Returns normalized path.
pub fn select_dir_dialog(title: &str) -> Option<String> {
    unsafe {
        let title_w = to_wide(title);
        let mut bi: BROWSEINFOW = std::mem::zeroed();
        bi.hwndOwner = HWND(std::ptr::null_mut());
        bi.lpszTitle = PCWSTR(title_w.as_ptr());
        bi.ulFlags = BIF_RETURNONLYFSDIRS | BIF_NEWDIALOGSTYLE;

        let pidl = SHBrowseForFolderW(&bi);
        if pidl.is_null() {
            return None;
        }
        let mut path_buf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
        let ok = SHGetPathFromIDListW(pidl, &mut path_buf);
        windows::Win32::System::Com::CoTaskMemFree(Some(pidl as *const _));
        if !ok.as_bool() {
            return None;
        }
        let len = path_buf.iter().position(|&c| c == 0).unwrap_or(path_buf.len());
        let path = String::from_utf16(&path_buf[..len]).ok()?;
        if path.is_empty() {
            return None;
        }
        Some(path.replace('/', "\\"))
    }
}
