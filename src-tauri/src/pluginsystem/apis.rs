use anyhow::Result;
use boa_engine::{js_string, object::ObjectInitializer, property::Attribute, Context};

pub mod models;
pub mod lifecycle;
pub mod event;
pub mod network;
pub mod config;
pub mod device;
pub mod ui;
pub mod native;
pub mod installer;
pub mod interconnect;
pub mod provider;
pub mod debug;
pub mod thirdpartyapp;
pub mod filesystem;

pub fn register_apis(context: &mut Context) -> Result<(), String> {
    let mut initializer = ObjectInitializer::new(context);
    
    lifecycle::register_lifecycle(&mut initializer)?;
    event::register_event(&mut initializer)?;
    network::register_network(&mut initializer)?;
    config::register_config(&mut initializer)?;
    device::register_device(&mut initializer)?;
    ui::register_ui(&mut initializer)?;
    native::register_native(&mut initializer)?;
    installer::register_installer(&mut initializer)?;
    interconnect::register_interconnect(&mut initializer)?;
    provider::register_provider(&mut initializer)?;
    #[cfg(debug_assertions)]
    debug::register_debug(&mut initializer)?;
    thirdpartyapp::register_thirdpartyapp(&mut initializer)?;
    filesystem::register_filesystem(&mut initializer)?;

    let jsobj = initializer.build();

    context
        .register_global_property(js_string!("AstroBox"), jsobj, Attribute::READONLY)
        .map_err(|e| e.to_string())?;

    Ok(())
}
