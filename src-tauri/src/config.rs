use anyhow::{Context, Result};
use once_cell::sync::{Lazy, OnceCell};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use tauri::{AppHandle, Manager};

use crate::{bt::device::ConnectType, miwear::device::MiWearState};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub connect_type: ConnectType,
    pub fragments_send_delay: u32,
    pub official_community_provider_cdn: String,
    pub plugin_dir: String,
    pub disabled_plugins: Vec<String>,
    pub current_device: Option<MiWearState>,
    pub paired_devices: Vec<MiWearState>,
    pub plugin_configs: HashMap<String, HashMap<String, String>>,
    pub auto_install: bool,
    pub disable_auto_clean: bool,
    pub debug_window: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_install: true,
            disable_auto_clean: false,
            debug_window: cfg!(debug_assertions),
            #[cfg(target_os = "ios")]
            connect_type: ConnectType::BLE,
            #[cfg(not(target_os = "ios"))]
            connect_type: ConnectType::SPP,
            fragments_send_delay: 5,
            official_community_provider_cdn: "ghfast".to_string(),
            plugin_dir: {
                let base = crate::APP_HANDLE
                    .get()
                    .and_then(|h| h.path().app_data_dir().ok())
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_else(|| "./data".into());
                format!("{}/plugins", base)
            },
            disabled_plugins: vec![],
            current_device: None,
            paired_devices: vec![],
            plugin_configs: HashMap::new()
        }
    }
}

static CONFIG: Lazy<RwLock<AppConfig>> = Lazy::new(|| RwLock::new(AppConfig::default()));
static CONFIG_PATH: OnceCell<PathBuf> = OnceCell::new();

pub async fn init(app: &AppHandle) -> Result<()> {
    let path = config_file_path(app)?;
    CONFIG_PATH.set(path.clone()).ok();

    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }

    if path.exists() {
        let json = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("read config: {}", path.display()))?;
        let cfg: AppConfig = serde_json::from_str(&json)?;
        *CONFIG.write() = cfg;
    } else {
        persist(&CONFIG.read())?;
    }

    Ok(())
}

pub async fn save(app: &AppHandle) -> Result<()> {
    let guard = CONFIG.read();
    let data = serde_json::to_string_pretty(&*guard)?;

    let path = config_file_path(app)?;
    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }

    tokio::fs::write(&path, data)
        .await
        .with_context(|| format!("write config: {}", path.display()))?;

    Ok(())
}

pub fn read<R>(f: impl FnOnce(&AppConfig) -> R) -> R {
    let guard = CONFIG.read();
    f(&guard)
}

pub fn write<R>(f: impl FnOnce(&mut AppConfig) -> R) -> R {
    let mut guard = CONFIG.write();
    let result = f(&mut guard);

    let snapshot: AppConfig = guard.clone();
    drop(guard);

    if let Err(e) = persist(&snapshot) {
        log::error!("failed to save config: {e:#}");
    }

    result
}

pub fn merge(dst: &mut serde_json::Value, patch: &serde_json::Value) {
    match (dst, patch) {
        (serde_json::Value::Object(d), serde_json::Value::Object(p)) => {
            for (k, v) in p {
                match d.get_mut(k) {
                    Some(dv) => merge(dv, v),
                    None => { d.insert(k.clone(), v.clone()); }
                }
            }
        }
        (dst_slot, src_val) => *dst_slot = src_val.clone(),
    }
}

fn persist(cfg: &AppConfig) -> Result<()> {
    let path = CONFIG_PATH
        .get()
        .expect("CONFIG_PATH not set; did you forget to call init()?");

    let data = serde_json::to_string_pretty(cfg)?;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    std::fs::write(path, data).with_context(|| format!("write config: {}", path.display()))?;

    Ok(())
}

fn config_file_path(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .context("app_config_dir unavailable; did you call after app was ready?")?;

    Ok(dir.join("config.json"))
}
