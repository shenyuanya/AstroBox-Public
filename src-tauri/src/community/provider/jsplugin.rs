use std::{any::Any, str::FromStr};
use anyhow::{Context, Result};
use boa_engine::{JsString, JsValue};
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use crate::community::provider::{
    Item, ProgressData, Provider, SearchConfig, ResourceManifestV1
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JSPluginProvider {
    pub name: String,
    pub plugin_name: String,
    pub fn_get_categories: String,
    pub fn_get_page: String,
    pub fn_get_item: String,
    pub fn_download: String,
}

impl JSPluginProvider {
    async fn call_js(&self, fn_name: &str, args: Vec<String>) -> anyhow::Result<String> {
        let plugin_name = self.plugin_name.clone();
        let fn_name = fn_name.to_owned();

        let ret = crate::pluginsystem::with_plugin_manager_async(move |pm| {
            let plug = pm
                .plugins
                .get_mut(&plugin_name)
                .with_context(|| format!("plugin '{plugin_name}' gone"))?;

            let ctx: &mut boa_engine::Context = &mut plug.js_context;
            let js_func = plug
                .js_env_data
                .registered_functions
                .iter()
                .find(|(n, _)| *n == &fn_name)
                .map(|(_, f)| f)
                .with_context(|| format!("function '{fn_name}' not found"))?;

            let mut value_args = vec![];
            for arg in args {
                value_args.push(JsValue::String(
                    JsString::from_str(&arg).map_err(|e| anyhow::anyhow!("{e}"))?,
                ));
            }

            let result = js_func
                .call(&JsValue::Undefined, value_args.as_slice(), ctx)
                .map_err(|e| anyhow::anyhow!("JS error: {e:?}"))?;

            ctx.run_jobs();

            anyhow::Ok(
                result
                    .to_string(ctx)
                    .map_err(|e| anyhow::anyhow!("{e}"))?
                    .to_std_string_lossy(),
            )
        })
        .await??;

        Ok(ret)
    }
}

#[async_trait::async_trait]
impl Provider for JSPluginProvider {
    fn provider_name(&self) -> String {
        self.name.to_string()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    async fn get_categories(&self) -> anyhow::Result<Vec<String>> {
        let ret = self
            .call_js(&self.fn_get_categories, vec![])
            .await?;

        Ok(serde_json::from_str(&ret)?)
    }

    async fn get_page(&self, p: u32, l: u32, s: SearchConfig) -> Result<Vec<Item>> {
        let ret = self
            .call_js(
                &self.fn_get_page,
                vec![
                    serde_json::to_string(&p)?,
                    serde_json::to_string(&l)?,
                    serde_json::to_string(&s)?,
                ],
            )
            .await?;

        Ok(serde_json::from_str(&ret)?)
    }

    async fn get_item(&self, name: String) -> Result<ResourceManifestV1> {
        let ret = self
            .call_js(&self.fn_get_item, vec![serde_json::to_string(&name)?])
            .await?;

        Ok(serde_json::from_str(&ret)?)
    }

    async fn download(&self, name: String, device: String, progress_cb: Channel<ProgressData>) -> Result<String> {
        let plugin_name = self.plugin_name.clone();
        let channel_id = crate::tools::random_string(10);
        let channel_id_clone = channel_id.clone();
        crate::pluginsystem::with_plugin_manager_async(move |pm| -> anyhow::Result<()> {
            let plug = pm
                .plugins
                .get_mut(&plugin_name.clone())
                .with_context(|| format!("plugin '{}' gone", plugin_name.clone()))?;

            plug.js_env_data
                .registered_progress_channels
                .insert(channel_id, progress_cb);
            Ok(())
        })
        .await
        .ok();

        let path = self.call_js(
            &self.fn_download,
            vec![
                serde_json::to_string(&name)?,
                serde_json::to_string(&device)?,
                serde_json::to_string(&channel_id_clone)?,
            ],
        )
        .await?;

        Ok(path)
    }

    async fn get_total_items(&self, filter: Option<String>) -> Result<u64> {
        let ret = self
            .call_js(
                &self.fn_get_page,
                vec![serde_json::to_string(&filter)?],
            )
            .await?;

        Ok(serde_json::from_str(&ret)?)
    }
}
