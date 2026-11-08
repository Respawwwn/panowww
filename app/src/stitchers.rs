//! What Settings shows under a Stitcher's row (spec §11.12): whether the
//! program is still there and which version it is. Windows reads the
//! file's version resource, macOS the bundle's `CFBundleShortVersionString`
//! through `defaults read`; a program that exists without a readable
//! version is "not detected", never an error.

use std::path::Path;

use crate::dto::StitcherCheck;

/// Looks at the program on disk; never fails.
pub fn check(program: &Path) -> StitcherCheck {
    if !program.exists() {
        return StitcherCheck {
            installed: false,
            version: None,
        };
    }
    StitcherCheck {
        installed: true,
        version: version(program).filter(|v| !v.is_empty()),
    }
}

/// `ProductVersion`, else `FileVersion`, of the executable's version
/// resource (`13.10`, `4.4.2.400`).
#[cfg(target_os = "windows")]
fn version(program: &Path) -> Option<String> {
    use std::ffi::c_void;
    use std::ptr::null_mut;

    use windows::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };
    use windows::core::HSTRING;

    let path = HSTRING::from(program.as_os_str());
    // SAFETY: `path` is a valid NUL-terminated wide string; the handle out
    // parameter is optional.
    let size = unsafe { GetFileVersionInfoSizeW(&path, None) };
    if size == 0 {
        return None;
    }
    let mut block = vec![0u8; size as usize];
    // SAFETY: `block` holds exactly the `size` bytes the API asked for.
    unsafe { GetFileVersionInfoW(&path, None, size, block.as_mut_ptr().cast()) }.ok()?;

    let query = |sub_block: &str| -> Option<(*const c_void, u32)> {
        let mut pointer: *mut c_void = null_mut();
        let mut length = 0u32;
        let key = HSTRING::from(sub_block);
        // SAFETY: `block` is a version-info block filled above and outlives
        // `pointer`, which lands inside it; both out parameters are valid.
        let found = unsafe {
            VerQueryValueW(
                block.as_ptr().cast(),
                &key,
                &raw mut pointer,
                &raw mut length,
            )
        };
        (found.as_bool() && !pointer.is_null() && length > 0)
            .then_some((pointer.cast_const(), length))
    };

    // The first language / code page pair of the translation table.
    let (translation, _) = query(r"\VarFileInfo\Translation")?;
    // SAFETY: the table holds at least one 4-byte entry (length > 0 above);
    // the two u16 are read unaligned from inside `block`.
    let (language, code_page) = unsafe {
        let p = translation.cast::<u16>();
        (p.read_unaligned(), p.add(1).read_unaligned())
    };
    ["ProductVersion", "FileVersion"]
        .into_iter()
        .find_map(|name| {
            let (text, chars) = query(&format!(
                r"\StringFileInfo\{language:04x}{code_page:04x}\{name}"
            ))?;
            // SAFETY: `text` points at `chars` UTF-16 units inside `block`.
            let units = unsafe { std::slice::from_raw_parts(text.cast::<u16>(), chars as usize) };
            let end = units.iter().position(|&u| u == 0).unwrap_or(units.len());
            Some(String::from_utf16_lossy(&units[..end]).trim().to_owned())
        })
}

/// `CFBundleShortVersionString` of the `.app`, read by `defaults`, which
/// handles XML and binary property lists alike.
#[cfg(target_os = "macos")]
fn version(program: &Path) -> Option<String> {
    let info = program.join("Contents").join("Info");
    let output = std::process::Command::new("/usr/bin/defaults")
        .arg("read")
        .arg(info)
        .arg("CFBundleShortVersionString")
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn version(_program: &Path) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_program_is_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let check = check(&dir.path().join("PTGui.exe"));
        assert!(!check.installed);
        assert_eq!(check.version, None);
    }

    #[test]
    fn a_file_without_a_version_is_installed_but_not_detected() {
        let dir = tempfile::tempdir().unwrap();
        let plain = dir.path().join("AutopanoGiga_x64.exe");
        std::fs::write(&plain, b"not a program").unwrap();
        let check = check(&plain);
        assert!(check.installed);
        assert_eq!(check.version, None);
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn a_windows_binary_reports_its_product_version() {
        let system = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_owned());
        let check = check(
            Path::new(&system)
                .join("System32")
                .join("kernel32.dll")
                .as_path(),
        );
        assert!(check.installed);
        let version = check.version.expect("kernel32 carries a version resource");
        assert!(version.starts_with("10."), "{version}");
    }
}
