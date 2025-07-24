pub mod provider;
use once_cell::sync::Lazy;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use provider::{Provider, official::OfficialPluginStore};

static PROVIDERS: Lazy<RwLock<HashMap<String, Arc<dyn Provider>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn init() -> Result<(), String> {
    let prov: Arc<dyn Provider> = Arc::new(OfficialPluginStore::new(
        "https://raw.githubusercontent.com/AstralSightStudios/AstroBox-Plugin-Repo/refs/heads/main/",
        Some(&crate::config::read(|c| c.clone().official_community_provider_cdn)),
    ));
    add_provider(prov).await;
    Ok(())
}

pub async fn add_provider(provider: Arc<dyn Provider>) {
    let mut map = PROVIDERS.write().await;
    map.insert(provider.provider_name(), provider);
}

pub async fn get_provider(name: &str) -> Option<Arc<dyn Provider>> {
    let map = PROVIDERS.read().await;
    map.get(name).cloned()
}

pub async fn list_providers() -> Vec<String> {
    let map = PROVIDERS.read().await;
    let mut names: Vec<String> = map.keys().cloned().collect();
    names.sort();
    names
}

