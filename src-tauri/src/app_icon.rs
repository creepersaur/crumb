use base64::{engine::general_purpose, Engine as _};
use image::{ImageBuffer, Rgba};

#[cfg(target_os = "windows")]
pub mod windows_icon {
    use std::mem::size_of;
    use std::os::windows::ffi::OsStrExt;
    use std::path::{Path, PathBuf};
    use windows::core::{Interface, PCWSTR};
    use windows::Win32::Graphics::Gdi::{
        CreateCompatibleDC, DeleteDC, DeleteObject, GetDIBits, GetObjectW, SelectObject, BITMAP,
        BITMAPINFO, BITMAPINFOHEADER, BI_RGB, DIB_RGB_COLORS, HGDIOBJ,
    };
    use windows::Win32::System::Com::IPersistFile;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
    };
    use windows::Win32::UI::Shell::IShellLinkW;
    use windows::Win32::UI::Shell::{SHGetFileInfoW, SHFILEINFOW, SHGFI_ICON, SHGFI_LARGEICON};
    use windows::Win32::UI::WindowsAndMessaging::{DestroyIcon, GetIconInfo, ICONINFO};
    use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    use winreg::RegKey;

    pub struct RawIcon {
        pub pixels: Vec<u8>,
        pub width: u32,
        pub height: u32,
    }

    /// Extracts the icon for a given file path safely using Win32 API.
    pub fn extract_icon(path: &Path) -> Option<RawIcon> {
        // Fix: Encode wide path properly to support all Unicode characters without falling back to to_str()
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        let mut file_info = SHFILEINFOW::default();

        let result = unsafe {
            SHGetFileInfoW(
                PCWSTR(wide.as_ptr()),
                windows::Win32::Storage::FileSystem::FILE_FLAGS_AND_ATTRIBUTES::default(),
                Some(&mut file_info),
                size_of::<SHFILEINFOW>() as u32,
                SHGFI_ICON | SHGFI_LARGEICON,
            )
        };

        if result == 0 || file_info.hIcon.is_invalid() {
            return None;
        }

        let icon = file_info.hIcon;
        let result = icon_to_rgba(icon);

        // Fix: Ensure DestroyIcon is ALWAYS run even if parsing partially fails
        unsafe {
            let _ = DestroyIcon(icon);
        }

        result
    }

    fn icon_to_rgba(icon: windows::Win32::UI::WindowsAndMessaging::HICON) -> Option<RawIcon> {
        let mut icon_info = ICONINFO::default();
        unsafe {
            GetIconInfo(icon, &mut icon_info).ok()?;
        }

        let color_bitmap = icon_info.hbmColor;
        let mask_bitmap = icon_info.hbmMask;

        if color_bitmap.is_invalid() {
            if !mask_bitmap.is_invalid() {
                unsafe {
                    let _ = DeleteObject(mask_bitmap.into());
                }
            }
            return None;
        }

        let mut bitmap = BITMAP::default();
        let bitmap_result = unsafe {
            GetObjectW(
                HGDIOBJ::from(color_bitmap),
                size_of::<BITMAP>() as i32,
                Some(&mut bitmap as *mut _ as *mut _),
            )
        };

        if bitmap_result == 0 {
            unsafe {
                let _ = DeleteObject(color_bitmap.into());
                if !mask_bitmap.is_invalid() {
                    let _ = DeleteObject(mask_bitmap.into());
                }
            }
            return None;
        }

        let width = bitmap.bmWidth as u32;
        let height = bitmap.bmHeight as u32;

        if width == 0 || height == 0 {
            unsafe {
                let _ = DeleteObject(color_bitmap.into());
                if !mask_bitmap.is_invalid() {
                    let _ = DeleteObject(mask_bitmap.into());
                }
            }
            return None;
        }

        let dc = unsafe { CreateCompatibleDC(None) };
        if dc.is_invalid() {
            unsafe {
                let _ = DeleteObject(color_bitmap.into());
                if !mask_bitmap.is_invalid() {
                    let _ = DeleteObject(mask_bitmap.into());
                }
            }
            return None;
        }

        let mut info = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: width as i32,
                biHeight: -(height as i32), // Top-down DIB
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB.0,
                ..Default::default()
            },
            ..Default::default()
        };

        let mut bgra = vec![0u8; (width * height * 4) as usize];
        let old_bitmap = unsafe { SelectObject(dc, color_bitmap.into()) };

        let copied = unsafe {
            GetDIBits(
                dc,
                color_bitmap,
                0,
                height,
                Some(bgra.as_mut_ptr() as *mut _),
                &mut info,
                DIB_RGB_COLORS,
            )
        };

        // Fix: Avoid leaking GDI structures on completion/fail
        unsafe {
            SelectObject(dc, old_bitmap);
            let _ = DeleteDC(dc);
            let _ = DeleteObject(color_bitmap.into());
            if !mask_bitmap.is_invalid() {
                let _ = DeleteObject(mask_bitmap.into());
            }
        }

        if copied == 0 {
            return None;
        }

        // Fix: Handle legacy non-alpha images by forcing opaque bounds if transparency block is missing
        let mut has_alpha = false;
        for pixel in bgra.chunks_exact(4) {
            if pixel[3] != 0 {
                has_alpha = true;
                break;
            }
        }

        // Convert Windows BGRA to Standard RGBA
        for pixel in bgra.chunks_exact_mut(4) {
            pixel.swap(0, 2);
            if !has_alpha {
                pixel[3] = 255; // Set to opaque if it completely lacked alpha channels
            }
        }

        Some(RawIcon {
            pixels: bgra,
            width,
            height,
        })
    }

    /// Provides standard fallback location pathways for common application environments.
    pub fn start_menu_dirs() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        if let Ok(appdata) = std::env::var("APPDATA") {
            dirs.push(PathBuf::from(appdata).join(r"Microsoft\Windows\Start Menu\Programs"));
        }
        if let Ok(program_data) = std::env::var("PROGRAMDATA") {
            dirs.push(PathBuf::from(program_data).join(r"Microsoft\Windows\Start Menu\Programs"));
        }
        dirs
    }

    /// Uses Native Windows COM Shell API to cleanly parse shortcuts instead of legacy crate wrappers.
    fn resolve_lnk_via_com(shortcut_path: &Path) -> Option<PathBuf> {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
            let shell_link: IShellLinkW = CoCreateInstance(
                &windows::Win32::UI::Shell::ShellLink,
                None,
                CLSCTX_INPROC_SERVER,
            )
            .ok()?;
            let persist_file: IPersistFile = shell_link.cast().ok()?;

            let wide_path: Vec<u16> = shortcut_path
                .as_os_str()
                .encode_wide()
                .chain(Some(0))
                .collect();
            persist_file
                .Load(
                    PCWSTR(wide_path.as_ptr()),
                    windows::Win32::System::Com::STGM::default(),
                )
                .ok()?;

            let mut buffer = [0u16; 260];
            shell_link
                .GetPath(&mut buffer, std::ptr::null_mut(), 0)
                .ok()?;

            let len = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
            let target_str = String::from_utf16(&buffer[..len]).ok()?;
            let path = PathBuf::from(target_str);

            if path.is_file() {
                Some(path)
            } else {
                None
            }
        }
    }

    /// Scans the start menu for shortcuts matching the application name.
    pub fn find_start_menu_target(app_name: &str) -> Option<PathBuf> {
        let wanted = normalize_name(app_name);

        for root in start_menu_dirs() {
            if !root.exists() {
                continue;
            }
            let mut stack = vec![root];

            while let Some(dir) = stack.pop() {
                let entries = match std::fs::read_dir(&dir) {
                    Ok(entries) => entries,
                    Err(_) => continue,
                };

                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                        continue;
                    }

                    if path.extension().map(|x| x.eq_ignore_ascii_case("lnk")) != Some(true) {
                        continue;
                    }

                    let file_name = match path.file_stem().and_then(|x| x.to_str()) {
                        Some(x) => x,
                        None => continue,
                    };

                    // Fix: Looser Substring matching so "davinci Resolve Studio" matches "davinci Resolve"
                    if !normalize_name(file_name).contains(&wanted) {
                        continue;
                    }

                    if let Some(target) = resolve_lnk_via_com(&path) {
                        return Some(target);
                    }
                }
            }
        }
        None
    }

    /// Crawls the Windows Uninstallation Registries to find explicit App Installation paths.
    pub fn find_via_registry(app_name: &str) -> Option<PathBuf> {
        let wanted = normalize_name(app_name);
        let registry_subkeys = [
            r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall",
            r"SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ];
        let root_keys = [HKEY_LOCAL_MACHINE, HKEY_CURRENT_USER];

        for &root in &root_keys {
            let hk = RegKey::predef(root);
            for subkey in &registry_subkeys {
                if let Ok(uninstall_key) = hk.open_subkey(subkey) {
                    for key_name in uninstall_key.enum_keys().map(|x| x.unwrap_or_default()) {
                        if let Ok(app_key) = uninstall_key.open_subkey(&key_name) {
                            let display_name: String =
                                app_key.get_value("DisplayName").unwrap_or_default();

                            if normalize_name(&display_name).contains(&wanted) {
                                // Attempt 1: Check Install Location directly
                                if let Ok(install_loc) =
                                    app_key.get_value::<String, _>("InstallLocation")
                                {
                                    if !install_loc.is_empty() {
                                        let path = PathBuf::from(&install_loc);
                                        if path.exists() {
                                            return Some(path);
                                        }
                                    }
                                } // Attempt 2: Extract path from the display icon if available
                                if let Ok(display_icon) =
                                    app_key.get_value::<String, _>("DisplayIcon")
                                {
                                    let clean = display_icon
                                        .split(',')
                                        .next()
                                        .unwrap_or_default()
                                        .trim_matches('"');
                                    let path = PathBuf::from(clean);
                                    if path.is_file() {
                                        return Some(path);
                                    }
                                    if path.is_dir() {
                                        return Some(path);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
    pub fn normalize_name(name: &str) -> String {
        name.trim().trim_end_matches(".exe").to_ascii_lowercase()
    }
    /// Master function to systematically find application pathways.
    pub fn resolve_app_path(name: &str, path: &Path) -> Option<PathBuf> {
        // Tier 1: Check if app_info's provided target actually points somewhere valid
        if path.exists() && path.as_os_str().len() > 0 {
            return Some(path.to_path_buf());
        } // Tier 2: Check fuzzy Start Menu Shortcuts
        if let Some(target) = find_start_menu_target(name) {
            return Some(target);
        } // Tier 3: Check Registry Records (Catches davinci Resolve and tricky suites)
        if let Some(target) = find_via_registry(name) {
            // If registry returns a folder path, try looking for the executable pattern inside it
            if target.is_dir() {
                let wanted = normalize_name(name);
                if let Ok(entries) = std::fs::read_dir(&target) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file()
                            && p.extension().map(|e| e.eq_ignore_ascii_case("exe")) == Some(true)
                        {
                            if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                                if wanted.contains(&normalize_name(stem))
                                    || normalize_name(stem).contains(&wanted)
                                {
                                    return Some(p);
                                }
                            }
                        }
                    }
                }
                return Some(target); // Fallback to raw directory path if exact file cannot be pinpointed
            }
            return Some(target);
        } // Tier 4: Global System Environment Variable scanning
        if let Ok(path) = which::which(name) {
            return Some(path);
        }
        None
    }
}

pub fn rgba_to_png_base64(pixels: &[u8], width: u32, height: u32) -> Option<String> {
    let mut buffer = std::io::Cursor::new(Vec::new());

    let img: ImageBuffer<Rgba<u8>, _> = ImageBuffer::from_raw(width, height, pixels.to_vec())?;
    img.write_to(&mut buffer, image::ImageFormat::Png)
        .ok()?;

    let base64_str = general_purpose::STANDARD.encode(buffer.into_inner());
    Some(format!("data:image/png;base64,{}", base64_str))
}