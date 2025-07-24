use crate::{community::provider::ProgressData, pluginsystem::apis::models::PluginUINode};
use super::manifest::PluginManifest;
use anyhow::{bail, Context, Result};
use base64::{engine::general_purpose, Engine};
use boa_engine::{object::builtins::JsFunction, JsValue, Source};
use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;
use std::{collections::HashMap, fs, path::PathBuf};
pub struct Plugin {
    pub manifest: PluginManifest,
    pub path: PathBuf,
    pub state: PluginState,
    pub js_context: boa_engine::Context,
    pub js_env_data: JsEnvData,
}

impl Plugin {
    pub fn load(path: PathBuf) -> Result<Self> {
        let manifest_path = path.join("manifest.json");
        let manifest_content = fs::read_to_string(&manifest_path)
            .with_context(|| format!("read manifest failed: {:?}", manifest_path))?;
        let manifest: PluginManifest = serde_json::from_str(&manifest_content)
            .with_context(|| "failed to deserialize manifest")?;

        let icon_bytes = fs::read(path.join(&manifest.icon))
            .with_context(|| "Plugin icon read failed")?;
        let icon_b64_str = general_purpose::STANDARD.encode(&icon_bytes);

        let mut ctx = boa_engine::Context::default();

        super::console::register_console(&mut ctx, &manifest.name);
        if let Err(e) = super::timers::register(&mut ctx) {
            anyhow::bail!("register timers failed: {}", e);
        }
        super::globals::inject_globals(&path, &mut ctx, &manifest)?;
        if let Err(e) = super::apis::register_apis(&mut ctx) {
            anyhow::bail!("register apis failed: {}", e);
        }

        Ok(Plugin {
            manifest: manifest.clone(),
            path,
            state: PluginState {
                disabled: crate::config::read(|c| c.clone().disabled_plugins)
                    .contains(&manifest.name),
                icon_b64: format!("data:image/png;base64,{}", icon_b64_str),
            },
            js_context: ctx,
            js_env_data: JsEnvData::default(),
        })
    }

    pub fn run(&mut self) -> Result<()> {
        let entry_path = self.path.join(&self.manifest.entry);
        let script = fs::read_to_string(&entry_path)
            .with_context(|| format!("failed to read entry js: {:?}", entry_path))?;

        let eval_result = self
            .js_context
            .eval(Source::from_bytes(script.as_bytes()))
            .map_err(|e| e.to_string());
        match eval_result {
            Ok(_val) => {
                match &self.js_env_data.on_load_function {
                    Some(cb) => {
                        match cb.call(&JsValue::Undefined, &[], &mut self.js_context) {
                            Ok(_) => {}
                            Err(e) => {
                                log::error!("Plugin onLoad function error: {}", e);
                            }
                        }
                        self.js_context.run_jobs();
                    }
                    None => {}
                }

                Ok(())
            }
            Err(err) => {
                log::error!(
                    "Plugin js (at {}) eval error: {}",
                    self.path.to_string_lossy(),
                    err
                );
                bail!("Plugin js eval error: {}", err);
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginState {
    pub disabled: bool,
    pub icon_b64: String,
}

#[derive(Default)]
pub struct JsEnvData {
    pub on_load_function: Option<JsFunction>,
    pub event_listeners: HashMap<String, JsFunction>,
    pub timeouts: HashMap<u32, JsFunction>,
    pub intervals: HashMap<u32, (JsFunction, u64)>,
    pub settings_ui: Vec<PluginUINode>,
    pub registered_functions: HashMap<String, JsFunction>,
    pub registered_progress_channels: HashMap<String, Channel<ProgressData>>
}
