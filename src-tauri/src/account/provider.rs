use async_trait::async_trait;
use anyhow::Result;
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use std::{collections::HashMap, sync::Arc};

use super::Account;

pub mod bandbbs;

pub async fn init_providers() {
    add_provider(Arc::new(bandbbs::BandBBSProvider::new()) as Arc<dyn AccountProvider>).await;
}

#[async_trait]
pub trait AccountProvider: Send + Sync {
    async fn login(&self) -> Result<Account>;
    async fn login_step2(&self,code:&str,state:&str) -> Result<()>;
    async fn logout(&self, account: &Account) -> Result<()>;
    async fn refresh_token(&self, account: &Account) -> Result<()>;

    fn provider_name(&self) -> &str;
}

static PROVIDERS: Lazy<RwLock<HashMap<String, Arc<dyn AccountProvider>>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

pub async fn add_provider(provider: Arc<dyn AccountProvider>) {
    let mut map = PROVIDERS.write().await;
    map.insert(provider.provider_name().to_owned(), provider);
}

pub async fn remove_provider(name: &str) -> Option<Arc<dyn AccountProvider>> {
    let mut map = PROVIDERS.write().await;
    map.remove(name)
}

pub async fn get_provider(name: &str) -> Option<Arc<dyn AccountProvider>> {
    let map = PROVIDERS.read().await;
    map.get(name).cloned()
}

pub async fn list_providers() -> Vec<String> {
    let map = PROVIDERS.read().await;
    map.keys().cloned().collect()
}
