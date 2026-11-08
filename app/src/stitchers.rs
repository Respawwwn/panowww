//! What Settings shows under a Stitcher's row (spec §11.12): whether the
//! program is still there, whether it really is that Stitcher, and which
//! version it is. Windows reads the file's version resource (`ProductName`,
//! `ProductVersion`), macOS the bundle's `CFBundleName` and
//! `CFBundleShortVersionString` through `defaults read`. A program that
//! exists without a readable product name is "not detected", never an
//! error.

use std::path::Path;

use panowww_store::stitchers::Stitcher;

use crate::dto::StitcherCheck;

/// What the program says about itself.
#[derive(Debug, Default, PartialEq, Eq)]
struct ProgramInfo {
    product: Option<String>,
    version: Option<String>,
}

/// Looks at the program on disk; never fails.
pub fn check(stitcher: Stitcher, program: &Path) -> StitcherCheck {
    if !program.exists() {
        return StitcherCheck {
            installed: false,
            genuine: false,
            product: None,
            version: None,
        };
    }
    let info = info(program);
    let product = info.product.filter(|p| !p.is_empty());
    StitcherCheck {
        installed: true,
        genuine: product.as_deref().is_some_and(|p| is_genuine(stitcher, p)),
        product,
        version: info.version.filter(|v| !v.is_empty()),
    }
}

/// `PTGui` for the PTGui row (PTGui and PTGui Pro), any Autopano for the
/// Autopano Giga row; case and spaces do not matter.
fn is_genuine(stitcher: Stitcher, product: &str) -> bool {
    let normalized: String = product
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect::<String>()
        .to_lowercase();
    let needle = match stitcher {
        Stitcher::Ptgui => "ptgui",
        Stitcher::AutopanoGiga => "autopano",
    };
    normalized.contains(needle)
}

/// `ProductName` and `ProductVersion` (else `FileDescription` and
/// `FileVersion`) of the executable's version resource.
#[cfg(target_os = "windows")]
fn info(program: &Path) -> ProgramInfo {
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
        return ProgramInfo::default();
    }
    let mut block = vec![0u8; size as usize];
    // SAFETY: `block` holds exactly the `size` bytes the API asked for.
    if unsafe { GetFileVersionInfoW(&path, None, size, block.as_mut_ptr().cast()) }.is_err() {
        return ProgramInfo::default();
    }

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
    let Some((translation, _)) = query(r"\VarFileInfo\Translation") else {
        return ProgramInfo::default();
    };
    // SAFETY: the table holds at least one 4-byte entry (length > 0 above);
    // the two u16 are read unaligned from inside `block`.
    let (language, code_page) = unsafe {
        let p = translation.cast::<u16>();
        (p.read_unaligned(), p.add(1).read_unaligned())
    };
    let string = |names: &[&str]| -> Option<String> {
        names.iter().find_map(|name| {
            let (text, chars) = query(&format!(
                r"\StringFileInfo\{language:04x}{code_page:04x}\{name}"
            ))?;
            // SAFETY: `text` points at `chars` UTF-16 units inside `block`.
            let units = unsafe { std::slice::from_raw_parts(text.cast::<u16>(), chars as usize) };
            let end = units.iter().position(|&u| u == 0).unwrap_or(units.len());
            Some(String::from_utf16_lossy(&units[..end]).trim().to_owned())
        })
    };
    ProgramInfo {
        product: string(&["ProductName", "FileDescription"]),
        version: string(&["ProductVersion", "FileVersion"]),
    }
}

/// `CFBundleName` (else `CFBundleExecutable`) and
/// `CFBundleShortVersionString` of the `.app`, read by `defaults`, which
/// handles XML and binary property lists alike.
#[cfg(target_os = "macos")]
fn info(program: &Path) -> ProgramInfo {
    let read = |key: &str| -> Option<String> {
        let output = std::process::Command::new("/usr/bin/defaults")
            .arg("read")
            .arg(program.join("Contents").join("Info"))
            .arg(key)
            .output()
            .ok()?;
        output
            .status
            .success()
            .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    };
    ProgramInfo {
        product: read("CFBundleName").or_else(|| read("CFBundleExecutable")),
        version: read("CFBundleShortVersionString"),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn info(_program: &Path) -> ProgramInfo {
    ProgramInfo::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_missing_program_is_not_installed() {
        let dir = tempfile::tempdir().unwrap();
        let check = check(Stitcher::Ptgui, &dir.path().join("PTGui.exe"));
        assert!(!check.installed);
        assert!(!check.genuine);
        assert_eq!(check.version, None);
    }

    #[test]
    fn a_file_without_a_version_resource_is_installed_but_not_detected() {
        let dir = tempfile::tempdir().unwrap();
        let plain = dir.path().join("AutopanoGiga_x64.exe");
        std::fs::write(&plain, b"not a program").unwrap();
        let check = check(Stitcher::AutopanoGiga, &plain);
        assert!(check.installed);
        assert!(!check.genuine);
        assert_eq!(check.product, None);
        assert_eq!(check.version, None);
    }

    #[test]
    fn the_product_name_decides_whether_the_program_is_the_stitcher() {
        assert!(is_genuine(Stitcher::Ptgui, "PTGui"));
        assert!(is_genuine(Stitcher::Ptgui, "PTGui Pro"));
        assert!(is_genuine(Stitcher::AutopanoGiga, "Autopano Giga"));
        assert!(is_genuine(Stitcher::AutopanoGiga, "AutopanoGiga"));
        assert!(!is_genuine(Stitcher::Ptgui, "Autopano Giga"));
        assert!(!is_genuine(Stitcher::AutopanoGiga, "Notepad"));
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn another_windows_program_is_installed_but_not_genuine() {
        let system = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_owned());
        let check = check(
            Stitcher::Ptgui,
            Path::new(&system)
                .join("System32")
                .join("kernel32.dll")
                .as_path(),
        );
        assert!(check.installed);
        assert!(!check.genuine);
        let product = check.product.expect("kernel32 names its product");
        assert!(product.contains("Windows"), "{product}");
        let version = check.version.expect("kernel32 carries a version resource");
        assert!(version.starts_with("10."), "{version}");
    }
}
