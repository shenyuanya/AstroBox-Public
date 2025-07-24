use crate::pluginsystem::plugin::JsEnvData;
use anyhow::Result;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::{collections::HashMap, io::Cursor};
use zip::ZipArchive;

use super::{manifest::PluginManifest, plugin::Plugin};

pub struct PluginManager {
    pub plugins: HashMap<String, Plugin>,
    pub updated: bool,
}

impl PluginManager {
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            updated: false,
        }
    }

    pub fn add(&mut self, path: &PathBuf) -> Result<()> {
        let plugin = Plugin::load(path.clone())?;
        let name = plugin.manifest.name.clone();
        if crate::config::read(|c| c.clone().plugin_configs)
            .get(&name)
            .is_none()
        {
            crate::config::write(|c| c.plugin_configs.insert(name.clone(), HashMap::new()));
        }

        self.plugins.insert(name.clone(), plugin);

        if let Some(pl) = self.plugins.get_mut(&name) {
            if !pl.state.disabled {
                pl.run()?;
            }
        }
        Ok(())
    }

    pub async fn add_from_abp(&mut self, name: &String, path: &String) -> Result<()> {
        self.updated = true;
        let package_raw = crate::fs::read_file_cross_platform(path).await?;
        let reader = Cursor::new(package_raw);
        let mut archive = ZipArchive::new(reader)?;

        let dest_dir_path = format!("{}/{}", crate::config::read(|c| c.clone().plugin_dir), name);
        let dest_dir = Path::new(&dest_dir_path);

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = dest_dir.join(file.mangled_name());

            if file.name().ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(parent) = outpath.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent)?;
                    }
                }
                let mut outfile = File::create(&outpath)?;
                std::io::copy(&mut file, &mut outfile)?;
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }

        Ok(())
    }

    pub fn enable(&mut self, name: &String) -> bool {
        log::info!("Enable plugin {}", name);
        self.updated = true;
        if crate::config::read(|c| c.clone().disabled_plugins).contains(name) {
            crate::config::write(|c| c.disabled_plugins.retain(|v| v != name));

            log::info!("Enable successful");
            return true;
        }
        false
    }

    pub fn disable(&mut self, name: &String) -> bool {
        log::info!("Disable plugin {}", name);
        self.updated = true;
        match self.plugins.get_mut(name) {
            Some(plug) => {
                plug.js_context = boa_engine::Context::default();
                plug.js_env_data = JsEnvData::default();
                plug.state.disabled = true;

                if !crate::config::read(|c| c.clone().disabled_plugins).contains(name) {
                    crate::config::write(|c| c.disabled_plugins.push(name.to_string()));
                }

                log::info!("Disable successful");
                true
            }
            None => false,
        }
    }

    pub fn remove(&mut self, name: &String) -> bool {
        self.updated = true;
        let dir = match self.plugins.get(name) {
            Some(p) => p.path.clone(),
            None => {
                log::error!("Plugin {} not found", name);
                return false;
            }
        };
        match self.plugins.remove(name) {
            Some(_) => {
                match fs::remove_dir_all(dir) {
                    Ok(_) => true,
                    Err(e) => {
                        log::error!("Failed to remove plugin: {:?} error: {:?}", name, e);
                        false
                    }
                }
            }
            None => false,
        }
    }

    pub fn load_from_dir(&mut self, dir: PathBuf) -> Result<()> {
        fs::create_dir_all(&dir)?;

        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                match self.add(&path) {
                    Ok(()) => {}
                    Err(e) => {
                        log::error!("Failed to load plugin: {:?} error: {:?}", path, e);
                    }
                }
            }
        }
        Ok(())
    }

    pub fn set_plugin_data<F>(&mut self, name: &str, f: F) -> Result<()>
    where
        F: FnOnce(&mut JsEnvData),
    {
        if let Some(plugin) = self.plugins.get_mut(name) {
            f(&mut plugin.js_env_data);
            Ok(())
        } else {
            anyhow::bail!("Plugin '{}' not found", name);
        }
    }

    pub fn get(&mut self, name: &str) -> Option<&mut Plugin> {
        self.plugins.get_mut(name)
    }

    pub fn list(&self) -> Vec<PluginManifest> {
        let plugs = self.plugins
            .values()
            .map(|pl| pl.manifest.clone())
            .collect();

        match serde_json::to_string(&plugs) {
            Ok(s) => log::info!("Get plugin list: {}", s),
            Err(e) => log::error!("Serialize plugin list failed: {}", e),
        }
        plugs
    }

    pub fn is_updated(&self) -> bool {
        self.updated
    }
}
