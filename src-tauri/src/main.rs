// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_icon;
mod name_cleaner;

use app_info::get_installed_apps;
use applications::common::SearchPath;
use applications::utils::image::RustImage;
use applications::{AppInfo, AppInfoContext, AppTrait};
use base64::Engine;
use image::imageops::FilterType;
use serde::Serialize;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
// Fix: Safely reference the cross-platform wrapper function or conditional windows items
use crate::app_icon::rgba_to_png_base64;
use crate::name_cleaner::clean_app_name;

#[derive(Serialize, Clone)]
struct FrontEndApp {
    name: String,
    icon_data: Option<String>,
    path: PathBuf,
}

#[tauri::command]
fn open_app(path: String) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", "", &path])
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&path)
            .spawn()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

fn get_search_paths() -> Vec<SearchPath> {
    let mut paths = vec![];

    // 1. Current user's Start Menu (per-user shortcuts)
    if let Ok(appdata) = std::env::var("APPDATA") {
        paths.push(SearchPath::new(
            PathBuf::from(appdata).join("Microsoft\\Windows\\Start Menu\\Programs"),
            3,
        ));
    }

    // 2. All-users Start Menu (system-wide shortcuts — installed by most installers)
    if let Ok(programdata) = std::env::var("PROGRAMDATA") {
        paths.push(SearchPath::new(
            PathBuf::from(programdata).join("Microsoft\\Windows\\Start Menu\\Programs"),
            3,
        ));
    }

    // 3. Current user's Desktop (some apps only put a shortcut here)
    if let Ok(userprofile) = std::env::var("USERPROFILE") {
        paths.push(SearchPath::new(
            PathBuf::from(userprofile.clone()).join("Desktop"),
            1,
        ));
    }

    // 4. Public Desktop (shared shortcuts across all users)
    paths.push(SearchPath::new(
        PathBuf::from("C:\\Users\\Public\\Desktop"),
        1,
    ));

    // 5. Program Files (catches apps without any Start Menu/Desktop shortcut at all)
    if let Ok(program_files) = std::env::var("ProgramFiles") {
        paths.push(SearchPath::new(PathBuf::from(program_files), 2));
    }

    paths.push(SearchPath::new(PathBuf::from("C:\\Program Files"), 4));

    // 6. Program Files (x86) — 32-bit apps on 64-bit Windows
    if let Ok(program_files_x86) = std::env::var("ProgramFiles(x86)") {
        paths.push(SearchPath::new(PathBuf::from(program_files_x86), 2));
    }

    // 7. Per-user local install location (Chrome, Discord, VS Code, Spotify, etc. often live here)
    if let Ok(localappdata) = std::env::var("LOCALAPPDATA") {
        paths.push(SearchPath::new(
            PathBuf::from(localappdata.clone()).join("Programs"),
            2,
        ));
    }

    paths
}

#[tauri::command]
async fn get_apps() -> Vec<FrontEndApp> {
    tauri::async_runtime::spawn_blocking(move || {
        let mut list = vec![];

        let mut ctx = AppInfoContext::new(get_search_paths());
        ctx.refresh_apps().unwrap();

        let apps = ctx.get_all_apps();

        for app in apps {
            let icon_data = app
                .load_icon()
                .ok()
                .and_then(|icon| icon.resize(64, 64, FilterType::Lanczos3).ok())
                .and_then(|resized| resized.to_png().ok())
                .map(|png_buf| {
                    format!(
                        "data:image/png;base64,{}",
                        base64::engine::general_purpose::STANDARD.encode(png_buf.get_bytes())
                    )
                });

            if let Some(path) = app.app_path_exe {
                list.push(FrontEndApp {
                    name: clean_app_name(&clean_app_name(&app.name)),
                    icon_data,
                    path,
                });
            }
        }

        let apps = match get_installed_apps(64) {
            Ok(apps) => apps,
            Err(err) => {
                eprintln!("Failed to get apps: {err}");
                return vec![];
            }
        };

        for app in apps {
            let initial_path = app.path.clone();

            #[cfg(target_os = "windows")]
            let resolved_path = {
                app_icon::windows_icon::resolve_app_path(&app.name, &initial_path).or_else(|| {
                    let fallback_str = app
                        .identifier
                        .clone()
                        .unwrap_or_else(|| initial_path.to_string_lossy().into_owned());
                    app_icon::windows_icon::resolve_app_path(&app.name, Path::new(&fallback_str))
                })
            };

            #[cfg(not(target_os = "windows"))]
            let resolved_path = Some(initial_path);

            let mut icon_data = app
                .icon
                .as_ref()
                .and_then(|icon| rgba_to_png_base64(&icon.pixels, icon.width, icon.height));

            #[cfg(target_os = "windows")]
            {
                if icon_data.is_none() {
                    icon_data = resolved_path
                        .as_deref()
                        .and_then(app_icon::windows_icon::extract_icon)
                        .and_then(|icon| rgba_to_png_base64(&icon.pixels, icon.width, icon.height));
                }
            }

            let is_exe = |p: &Path| {
                p.extension()
                    .map_or(false, |ext| ext == "exe" || ext == "lnk")
            };

            if app.path.exists() && is_exe(&app.path) {
                list.push(FrontEndApp {
                    name: clean_app_name(&clean_app_name(&app.name)),
                    icon_data,
                    path: app.path,
                });
            } else if let Some(path) = resolved_path {
                if path.exists() && is_exe(&path) {
                    list.push(FrontEndApp {
                        name: clean_app_name(&clean_app_name(&app.name)),
                        icon_data,
                        path,
                    });
                }
            }
        }

        println!("Apps completed loading");

        let mut seen_names = HashSet::new();
        let mut seen_paths = HashSet::new();
        list.retain(|app| seen_names.insert(app.name.clone()));
        list.reverse();
        list.retain(|app| {
            let key = app.path.to_string_lossy().to_lowercase();

            seen_paths.insert(key)
        });
        list.sort_by(|a, b| {
            if a.name > b.name {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });

        list
    })
    .await
    .expect("blocking function fail")
}

#[tauri::command]
fn power(action: String) -> Result<(), String> {
    let args = match action.as_str() {
        "shutdown" => vec!["/s", "/t", "0"],
        "restart" => vec!["/r", "/t", "0"],
        "sleep" => vec!["/h"],
        _ => return Err("Unknown power action".into()),
    };

    std::process::Command::new("shutdown")
        .args(args)
        .spawn()
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_apps, open_app, power])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
