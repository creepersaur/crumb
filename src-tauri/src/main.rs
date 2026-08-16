// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_icon;
mod name_cleaner;

use app_info::get_installed_apps;
use serde::Serialize;
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

#[tauri::command]
async fn get_apps() -> Vec<FrontEndApp> {
    tauri::async_runtime::spawn_blocking(move || {
        let apps = match get_installed_apps(64) {
            Ok(apps) => apps,
            Err(err) => {
                eprintln!("Failed to get apps: {err}");
                return vec![];
            }
        };

        let mut list = vec![];

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

            if let Some(path) = resolved_path {
                list.push(FrontEndApp {
                    name: clean_app_name(&clean_app_name(&app.name)),
                    icon_data,
                    path,
                });
            } else if app.path.exists() {
                list.push(FrontEndApp {
                    name: clean_app_name(&clean_app_name(&app.name)),
                    icon_data,
                    path: app.path,
                });
            }
        }

        println!("Apps completed loading");

        list
    })
    .await
    .expect("blocking function fail")
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![get_apps, open_app])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
