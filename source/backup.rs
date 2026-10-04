//! Backup creation and restore, including zip-slip protection and zip integrity validation.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use zip::write::FileOptions;
use zip::{ZipArchive, ZipWriter};

use crate::i18n;
use crate::lprintln;
use crate::game::get_fonts_paths;

pub const BACKUP_FILENAME: &str = "backup_original_fonts.zip";

/// Mirror of Python `create_backup`: walk csgo_fonts and core_fonts, write all files
/// under `csgo_fonts/` and `core_fonts/` arcnames. Skips silently if the backup already exists.
pub fn create_backup(install_location: &str) {
    let backup_path = Path::new(install_location).join(BACKUP_FILENAME);
    if backup_path.exists() {
        return;
    }

    lprintln!("{}", i18n::t("backing_up"));
    let paths = get_fonts_paths(install_location);

    let result = (|| -> io::Result<()> {
        let file = fs::File::create(&backup_path)?;
        let mut zip = ZipWriter::new(file);
        let options: FileOptions = FileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        if paths.csgo_fonts.exists() {
            add_dir_to_zip(&mut zip, &paths.csgo_fonts, "csgo_fonts", options)?;
        }
        if paths.core_fonts.exists() {
            add_dir_to_zip(&mut zip, &paths.core_fonts, "core_fonts", options)?;
        }
        zip.finish()?;
        Ok(())
    })();

    match result {
        Ok(_) => lprintln!(
            "{}",
            i18n::t_with(
                "backup_created",
                &[("backup_path", &backup_path.to_string_lossy())]
            )
        ),
        Err(e) => crate::print_error(&i18n::t("backup_error"), Some(&e)),
    }
}

fn add_dir_to_zip<W: io::Write + io::Seek>(
    zip: &mut ZipWriter<W>,
    dir: &Path,
    prefix: &str,
    options: FileOptions,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            add_dir_to_zip(zip, &path, &format!("{}/{}", prefix, entry.file_name().to_string_lossy()), options)?;
        } else {
            let arcname = format!("{}/{}", prefix, entry.file_name().to_string_lossy());
            zip.start_file(&arcname, options)?;
            let bytes = fs::read(&path)?;
            io::Write::write_all(zip, &bytes)?;
        }
    }
    Ok(())
}

/// Mirror of Python `restore_backup`:
/// - testzip integrity check
/// - extract csgo_fonts/* and core_fonts/* with zip-slip protection
/// - if the backup did not contain `core_fonts/42-repl-global.conf`, remove any leftover one
pub fn restore_backup(install_location: &str, backup_path_override: Option<&str>) -> bool {
    let backup_path: PathBuf = match backup_path_override {
        Some(p) => PathBuf::from(p),
        None => Path::new(install_location).join(BACKUP_FILENAME),
    };

    if !backup_path.exists() {
        lprintln!(
            "{}",
            i18n::t_with(
                "restore_file_missing",
                &[("backup_path", &backup_path.to_string_lossy())]
            )
        );
        return false;
    }

    // Integrity check first — don't delete existing files only to discover a corrupt backup.
    let file = match fs::File::open(&backup_path) {
        Ok(f) => f,
        Err(e) => {
            crate::print_error(&i18n::t("restore_error"), Some(&e));
            return false;
        }
    };
    let mut archive = match ZipArchive::new(file) {
        Ok(a) => a,
        Err(e) => {
            crate::print_error(&i18n::t("restore_error"), Some(&e));
            return false;
        }
    };

    let mut namelist: Vec<String> = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        match archive.by_index(i) {
            Ok(f) => namelist.push(f.name().to_string()),
            Err(e) => {
                crate::print_error(&i18n::t("restore_error"), Some(&e));
                return false;
            }
        }
    }

    lprintln!(
        "{}",
        i18n::t_with(
            "restoring_fonts",
            &[(
                "backup_name",
                &backup_path
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            )]
        )
    );

    let paths = get_fonts_paths(install_location);

    // Wipe csgo_fonts contents (Python removes everything inside, keeps the dir)
    if paths.csgo_fonts.exists() {
        if let Ok(entries) = fs::read_dir(&paths.csgo_fonts) {
            for entry in entries.flatten() {
                let p = entry.path();
                let _ = if p.is_dir() {
                    fs::remove_dir_all(&p)
                } else {
                    fs::remove_file(&p)
                };
            }
        }
    }

    let result = (|| -> io::Result<()> {
        for name in &namelist {
            let (target_base, rel) = if let Some(r) = name.strip_prefix("csgo_fonts/") {
                (&paths.csgo_fonts, r)
            } else if let Some(r) = name.strip_prefix("core_fonts/") {
                (&paths.core_fonts, r)
            } else {
                continue;
            };
            if rel.is_empty() {
                continue;
            }
            // Zip-slip guard: reject absolute paths and any '..' segments (handle both / and \)
            let normalized = rel.replace('\\', "/");
            if Path::new(rel).is_absolute()
                || normalized.starts_with('/')
                || normalized.split('/').any(|seg| seg == "..")
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Unsafe path in backup: {}", name),
                ));
            }
            let target = target_base.join(rel);
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut src = archive
                .by_name(name)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;
            let mut dst = fs::File::create(&target)?;
            io::copy(&mut src, &mut dst)?;
        }
        Ok(())
    })();

    if let Err(e) = result {
        crate::print_error(&i18n::t("restore_error"), Some(&e));
        return false;
    }

    // If the backup didn't include 42-repl-global.conf, remove any custom one still on disk
    let repl_conf = paths.core_fonts.join("42-repl-global.conf");
    if !namelist.iter().any(|n| n == "core_fonts/42-repl-global.conf")
        && repl_conf.exists()
        && fs::remove_file(&repl_conf).is_ok()
    {
        lprintln!("{}", i18n::t("config_cleaned"));
    }

    lprintln!("{}", i18n::t("restore_complete"));
    true
}
