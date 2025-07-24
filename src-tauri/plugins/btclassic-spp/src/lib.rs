use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
pub use desktop::BtclassicSpp;
#[cfg(mobile)]
pub use mobile::BtclassicSpp;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the btclassic-spp APIs.
pub trait BtclassicSppExt<R: Runtime> {
    fn btclassic_spp(&self) -> &BtclassicSpp<R>;
}

impl<R: Runtime, T: Manager<R>> crate::BtclassicSppExt<R> for T {
    fn btclassic_spp(&self) -> &BtclassicSpp<R> {
        self.state::<BtclassicSpp<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("btclassic-spp")
        // 不需要任何JS绑定
        //.invoke_handler()
        .setup(|app, api| {
            #[cfg(mobile)]
            let btclassic_spp = mobile::init(app, api)?;
            #[cfg(desktop)]
            let btclassic_spp = {
                #[cfg(target_os = "linux")]
                desktop::imp::init_bluetooth_stack();

                desktop::init(app, api)?
            };
            app.manage(btclassic_spp);
            Ok(())
        })
        .build()
}
