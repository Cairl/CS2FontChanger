//! TTF/OTF parsing: extract the font family name (nameID=1, platformID=3) and XML escaping.

use std::fs;

/// Read big-endian primitives from a byte slice.
fn be_u16(b: &[u8], off: usize) -> Option<u16> {
    b.get(off..off + 2).map(|s| u16::from_be_bytes([s[0], s[1]]))
}
fn be_u32(b: &[u8], off: usize) -> Option<u32> {
    b.get(off..off + 4)
        .map(|s| u32::from_be_bytes([s[0], s[1], s[2], s[3]]))
}

/// Locate the offset of the first font face in the file.
/// Handles plain sfnt (TTF/OTF) and TrueType Collections (TTC, reads face 0).
fn first_face_offset(data: &[u8]) -> Option<usize> {
    let tag = be_u32(data, 0)?;
    match tag {
        // 'ttcf' — TrueType Collection
        0x7474_6366 => {
            // TTC header: tag(4) version(4) numFonts(4) offsets...
            let num_fonts = be_u32(data, 8)?;
            if num_fonts == 0 {
                return None;
            }
            be_u32(data, 12).map(|v| v as usize)
        }
        // 0x00010000 (TTF), 'OTTO' (OTF/CFF), 'true', 'typ1' — all plain sfnt
        _ => Some(0),
    }
}

/// Find the `name` table within a face and return its (offset, length).
fn find_name_table(data: &[u8], face_off: usize) -> Option<(usize, usize)> {
    let num_tables = be_u16(data, face_off + 4)? as usize;
    let table_dir = face_off + 12;
    for i in 0..num_tables {
        let rec = table_dir + i * 16;
        let tag = be_u32(data, rec)?;
        // 'name'
        if tag == 0x6E61_6D65 {
            let off = be_u32(data, rec + 8)? as usize;
            let len = be_u32(data, rec + 12)? as usize;
            return Some((off, len));
        }
    }
    None
}

/// Extract the family name: first nameID=1 record with platformID=3 (Windows).
/// Falls back to platformID=1 (Macintosh, roman) if no Windows record is found —
/// matching the spirit of `fontTools`' lookup but keeping the Windows-first bias.
pub fn get_font_name(path: &str) -> Option<String> {
    let data = fs::read(path).ok()?;
    let face_off = first_face_offset(&data)?;
    let (name_off, name_len) = find_name_table(&data, face_off)?;
    let name_table = data.get(name_off..name_off + name_len)?;

    let _format = be_u16(name_table, 0)?;
    let count = be_u16(name_table, 2)? as usize;
    let string_off = be_u16(name_table, 4)? as usize;

    // Pass 1: platformID=3 (Windows), nameID=1
    for i in 0..count {
        let rec = 6 + i * 12;
        let platform_id = be_u16(name_table, rec)?;
        let name_id = be_u16(name_table, rec + 6)?;
        let length = be_u16(name_table, rec + 8)? as usize;
        let offset = be_u16(name_table, rec + 10)? as usize;

        if platform_id == 3 && name_id == 1 {
            let start = string_off + offset;
            let bytes = name_table.get(start..start + length)?;
            return decode_utf16be(bytes);
        }
    }

    // Pass 2: platformID=1 (Mac), nameID=1, single-byte (MacRoman) encoding
    for i in 0..count {
        let rec = 6 + i * 12;
        let platform_id = be_u16(name_table, rec)?;
        let name_id = be_u16(name_table, rec + 6)?;
        let length = be_u16(name_table, rec + 8)? as usize;
        let offset = be_u16(name_table, rec + 10)? as usize;

        if platform_id == 1 && name_id == 1 {
            let start = string_off + offset;
            let bytes = name_table.get(start..start + length)?;
            // MacRoman is mostly ASCII for typical font names; lossy conversion is fine.
            let s: String = bytes.iter().map(|&b| b as char).collect();
            let trimmed = s.trim().to_string();
            if !trimmed.is_empty() {
                return Some(trimmed);
            }
        }
    }

    None
}

fn decode_utf16be(bytes: &[u8]) -> Option<String> {
    if !bytes.len().is_multiple_of(2) {
        return None;
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|c| u16::from_be_bytes([c[0], c[1]]))
        .collect();
    let s = String::from_utf16(&units).ok()?;
    let trimmed = s.trim().to_string();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    }
}

/// Mirror of Python `xml.sax.saxutils.escape` with default entities: & < >
pub fn xml_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}
