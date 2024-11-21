use serde::Serialize;
use std::collections::HashSet;
use std::time::{Duration, Instant};
use tauri::plugin::TauriPlugin;
use tauri::{Emitter, LogicalPosition, LogicalSize, Manager, Runtime};
use tauri_plugin_shell::ShellExt;
use theseus::settings;
use tokio::sync::RwLock;

pub struct AdsState {
    pub shown: bool,
    pub size: Option<LogicalSize<f32>>,
    pub position: Option<LogicalPosition<f32>>,
    pub last_click: Option<Instant>,
    pub malicious_origins: HashSet<String>,
}

const AD_LINK: &str = "https://modrinth.com/wrapper/app-ads-cookie";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("ads")
        .setup(|app, _api| {
            app.manage(RwLock::new(AdsState {
                shown: true,
                size: None,
                position: None,
                last_click: None,
                malicious_origins: HashSet::new(),
            }));

            // We refresh the ads window every 5 minutes for performance
            let app = app.clone();
            tauri::async_runtime::spawn(async move {
                loop {
                    if let Some(webview) = app.webviews().get_mut("ads-window")
                    {
                        let _ = webview.navigate(AD_LINK.parse().unwrap());
                    }

                    tokio::time::sleep(std::time::Duration::from_secs(60 * 5))
                        .await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            init_ads_window,
            hide_ads_window,
            scroll_ads_window,
            show_ads_window,
            record_ads_click,
            open_link,
            get_ads_personalization,
        ])
        .build()
}

#[tauri::command]
#[cfg(not(target_os = "linux"))]
pub async fn init_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    override_shown: bool,
) -> crate::api::Result<()> {
    Ok(())
}

// TODO: make ads work on linux
#[tauri::command]
#[cfg(target_os = "linux")]
pub async fn init_ads_window() {}

#[tauri::command]
pub async fn show_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> crate::api::Result<()> {
    Ok(())
}

#[tauri::command]
pub async fn hide_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    reset: Option<bool>,
) -> crate::api::Result<()> {
    Ok(())
}

#[derive(Serialize, Clone)]
struct ScrollEvent {
    scroll: f32,
}

#[tauri::command]
pub async fn scroll_ads_window<R: Runtime>(
    app: tauri::AppHandle<R>,
    scroll: f32,
) -> crate::api::Result<()> {
    Ok(())
}

#[tauri::command]
pub async fn record_ads_click<R: Runtime>(
    app: tauri::AppHandle<R>,
) -> crate::api::Result<()> {
    Ok(())
}

#[tauri::command]
pub async fn open_link<R: Runtime>(
    app: tauri::AppHandle<R>,
    path: String,
    origin: String,
) -> crate::api::Result<()> {
    let state = app.state::<RwLock<AdsState>>();
    let mut state = state.write().await;

    if url::Url::parse(&path).is_ok()
        && !state.malicious_origins.contains(&origin)
    {
        if let Some(last_click) = state.last_click {
            if last_click.elapsed() < Duration::from_millis(100) {
                let _ = app.shell().open(&path, None);
                state.last_click = None;

                return Ok(());
            }
        }
    }

    tracing::info!("Malicious click: {path} origin {origin}");
    state.malicious_origins.insert(origin);

    Ok(())
}

#[tauri::command]
pub async fn get_ads_personalization() -> crate::api::Result<bool> {
    let res = settings::get().await?;
    Ok(res.personalized_ads)
}
