use std::any::Any;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

use crate::community::provider::{ProgressData, ProviderState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorePluginManifest {
    pub name: String,
    pub icon: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub website: String,
    pub entry: String,
    pub api_level: u32,
    pub permissions: Vec<String>,
    #[serde(default)]
    pub additional_files: Vec<String>,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn provider_name(&self) -> String;
    fn as_any(&self) -> &dyn Any;

    async fn refresh(&self) -> anyhow::Result<()>;
    fn state(&self) -> ProviderState;

    async fn get_page(
        &self,
        page: u32,
        limit: u32,
        filter: Option<String>,
    ) -> anyhow::Result<Vec<StorePluginManifest>>;

    async fn get_item(&self, name: String) -> anyhow::Result<StorePluginManifest>;

    async fn download(
        &self,
        name: String,
        progress_cb: Channel<ProgressData>,
    ) -> anyhow::Result<String>;

    async fn get_total_items(&self, filter: Option<String>) -> anyhow::Result<u64>;
}

pub mod official;
