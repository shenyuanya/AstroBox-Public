pub mod provider;

use once_cell::sync::Lazy;
use provider::Provider;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

static PROVIDERS: Lazy<RwLock<HashMap<String, Arc<dyn Provider>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn init() -> Result<(), String> {
    let provider: Arc<dyn Provider> = Arc::new(provider::official::OfficialProvider::new(Some(
        &crate::config::read(|c| c.clone().official_community_provider_cdn),
    )));
    add_provider(provider.clone()).await;
    let bandbbs_provider: Arc<dyn Provider> = Arc::new(provider::bandbbs::BandBBSProvider::new());
    add_provider(bandbbs_provider.clone()).await;

    /*
    provider.refresh().await.map_err(|e| {
        let err_msg = format!("failed to refresh official provider: {e:#}");
        log::error!("{}", &err_msg);
        err_msg
    })?;
    */

    Ok(())
}

pub async fn add_provider(provider: Arc<dyn Provider>) {
    let mut map = PROVIDERS.write().await;
    map.insert(provider.provider_name().to_owned(), provider);
}

pub async fn remove_provider(name: &str) -> Option<Arc<dyn Provider>> {
    let mut map = PROVIDERS.write().await;
    map.remove(name)
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
