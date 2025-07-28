use std::{any::Any, collections::HashMap};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

pub mod jsplugin;
pub mod official;
pub mod bandbbs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Author {
    pub name: Option<String>,
    pub author_url: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub name: Option<String>,
    pub restype: Option<String>,
    pub description: Option<String>,
    pub preview: Option<Vec<String>>,
    pub icon: Option<String>,
    pub source_url: Option<String>,
    pub author: Option<Vec<Author>>,
    #[serde(default)]
    pub paid_type: String,
    pub _bandbbs_ext_supported_device: Option<String>,
    pub _bandbbs_ext_resource_id: Option<u32>,
    pub _bandbbs_ext_is_community_paid: Option<bool>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SearchConfig {
    pub filter: Option<String>,
    pub sort: Option<String>,
    pub category: Option<Vec<String>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProgressData {
    pub progress: f32,
    pub status: String,
    pub status_text: String
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProviderState {
    Ready,
    Updating,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceManifestV1 {
    pub item: Item,
    pub downloads: HashMap<String, ResDownloadInfoV1>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResDownloadInfoV1 {
    pub version: String,
    pub file_name: String
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn provider_name(&self) -> String;
    fn as_any(&self) -> &dyn Any;

    async fn refresh(&self) -> anyhow::Result<()> {
        Ok(())
    }

    fn state(&self) -> ProviderState {
        ProviderState::Ready
    }

    async fn get_page(
        &self,
        page: u32,
        limit: u32,
        search: SearchConfig,
    ) -> anyhow::Result<Vec<Item>>;

    async fn get_categories(&self) -> anyhow::Result<Vec<String>>;

    async fn get_item(&self, name: String) -> anyhow::Result<ResourceManifestV1>;

    async fn download(&self, name: String, device: String, progress_cb: Channel<ProgressData>) -> anyhow::Result<String>;

    async fn get_total_items(&self, filter: Option<String>) -> anyhow::Result<u64>;
}
