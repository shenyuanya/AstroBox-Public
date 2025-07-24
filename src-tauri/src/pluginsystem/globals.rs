use std::path::PathBuf;

use anyhow::Result;
use boa_engine::{js_string, property::Attribute};

use crate::pluginsystem::manifest::PluginManifest;

pub fn inject_globals(
    path: &PathBuf,
    ctx: &mut boa_engine::Context,
    manifest: &PluginManifest,
) -> Result<()> {
    ctx.register_global_property(
        js_string!("RUNTIME"),
        js_string!("AstroBox"),
        Attribute::READONLY,
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    ctx.register_global_property(
        js_string!("RUNTIME_VERSION"),
        js_string!(env!("CARGO_PKG_VERSION")),
        Attribute::READONLY,
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    ctx.register_global_property(
        js_string!("PLUGIN_NAME"),
        js_string!(manifest.name.clone()),
        Attribute::READONLY,
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    ctx.register_global_property(
        js_string!("PLUGIN_PATH"),
        js_string!(path.to_string_lossy().to_string()),
        Attribute::READONLY,
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    ctx.register_global_property(
        js_string!("PLUGIN_VERSION"),
        js_string!(manifest.version.clone()),
        Attribute::READONLY,
    )
    .map_err(|e| anyhow::anyhow!(e.to_string()))?;

    Ok(())
}
