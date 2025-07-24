use anyhow::{Context, Result};
use once_cell::sync::Lazy;
use parking_lot::RwLock as SyncRwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use tauri::{AppHandle, Manager};

pub mod provider;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub avatar: String,
    pub data: HashMap<String, String>,
}

// -- Accounts and persistence logic (保留原逻辑，未动) --
static ACCOUNTS: Lazy<SyncRwLock<HashMap<String, Vec<Account>>>> =
    Lazy::new(|| SyncRwLock::new(HashMap::new()));
static ACCOUNTS_PATH: once_cell::sync::OnceCell<PathBuf> = once_cell::sync::OnceCell::new();

pub async fn init(app: &AppHandle) -> Result<()> {
    let path = account_file_path(app)?;
    ACCOUNTS_PATH.set(path.clone()).ok();

    if let Some(dir) = path.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }

    if path.exists() {
        let json = tokio::fs::read_to_string(&path)
            .await
            .with_context(|| format!("read accounts: {}", path.display()))?;
        let map: HashMap<String, Vec<Account>> = serde_json::from_str(&json)?;
        *ACCOUNTS.write() = map;
    } else {
        persist(&ACCOUNTS.read())?;
    }

    Ok(())
}

pub fn read<R>(f: impl FnOnce(&HashMap<String, Vec<Account>>) -> R) -> R {
    let guard = ACCOUNTS.read();
    f(&guard)
}

pub fn write<R>(f: impl FnOnce(&mut HashMap<String, Vec<Account>>) -> R) -> R {
    let mut guard = ACCOUNTS.write();
    let result = f(&mut guard);
    let snapshot = guard.clone();
    drop(guard);
    if let Err(e) = persist(&snapshot) {
        log::error!("failed to save accounts: {e:#}");
    }
    result
}

pub fn add_account(name: &str, account: Account) {
    write(|map| {
        let accounts = map.entry(name.to_string()).or_default();
        // 移除已存在的相同id账户
        accounts.retain(|a| a.id != account.id);
        accounts.push(account);
    });
}

pub fn remove_account(name: &str, index: usize) -> Option<Account> {
    write(|map| {
        map.get_mut(name).and_then(|vec| {
            if index < vec.len() {
                Some(vec.remove(index))
            } else {
                None
            }
        })
    })
}

pub fn list_accounts(name: &str) -> Vec<Account> {
    read(|map| map.get(name).cloned().unwrap_or_default())
}

// 这个 list_providers 函数建议重命名为 list_account_types，避免和上面 provider 冲突
pub fn list_account_types() -> Vec<String> {
    read(|map| map.keys().cloned().collect())
}

fn persist(data: &HashMap<String, Vec<Account>>) -> Result<()> {
    let path = ACCOUNTS_PATH
        .get()
        .expect("ACCOUNTS_PATH not set; did you forget to call init()?");

    let json = serde_json::to_string_pretty(data)?;

    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }

    std::fs::write(path, json).with_context(|| format!("write accounts: {}", path.display()))?;
    Ok(())
}

fn account_file_path(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_config_dir()
        .context("app_config_dir unavailable; did you call after app was ready?")?;

    Ok(dir.join("account.json"))
}
