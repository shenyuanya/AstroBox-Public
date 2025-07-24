use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

#[cfg(target_os = "windows")]
#[path = "./win/implementation.rs"]
pub mod imp;

#[cfg(target_os = "macos")]
#[path = "./macos/implementation.rs"]
pub mod imp;

#[cfg(target_os = "linux")]
#[path = "./linux/implementation.rs"]
pub mod imp;

use imp::core;

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<BtclassicSpp<R>> {
    Ok(BtclassicSpp(app.clone()))
}

/// Access to the btclassic-spp APIs.
pub struct BtclassicSpp<R: Runtime>(AppHandle<R>);

impl<R: Runtime> BtclassicSpp<R> {
    pub fn start_scan(&self) -> anyhow::Result<()> {
        core::start_scan_impl()
    }

    pub fn stop_scan(&self) -> anyhow::Result<()> {
        core::stop_scan_impl()
    }

    pub fn get_scanned_devices(&self) -> anyhow::Result<GetScannedDevicesResult> {
        core::get_scanned_devices_impl().map(|devices| GetScannedDevicesResult { ret: devices })
    }

    pub fn connect(&self, addr: &String) -> anyhow::Result<ConnectResult> {
        core::connect_impl(addr).map(|success| ConnectResult { ret: success })
    }

    pub fn get_connected_device_info(&self) -> anyhow::Result<SPPDevice> {
        core::get_connected_device_info_impl()?
            .ok_or_else(|| anyhow::anyhow!("No device connected or info unavailable"))
    }

    pub fn on_connected<F>(&self, cb: F) -> anyhow::Result<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        core::on_connected_impl(Box::new(cb))
    }

    pub fn set_data_listener(
        &self,
        cb: impl FnMut(Result<Vec<u8>, String>) + Send + 'static, // 修改签名以接受 Result
    ) -> anyhow::Result<()> {
        let mut user_cb = cb; // user_cb 现在是 FnMut(Result<Vec<u8>, String>)
        let core_cb = move |result: Result<Vec<u8>, String>| {
            user_cb(result); // 直接传递 Result
        };
        core::set_data_listener_impl(Box::new(core_cb))
    }

    pub fn start_subscription(&self) -> anyhow::Result<()> {
        core::start_subscription_impl()
    }

    pub fn send(&self, data: &[u8]) -> anyhow::Result<()> {
        match core::send_impl(data) {
            Ok(()) => Ok(()),
            Err(err) => {
                log::error!("send msg error {}", err);
                Err(anyhow::anyhow!(err))
            },
        }
    }

    pub fn disconnect(&self) -> anyhow::Result<()> {
        let ret = core::disconnect_impl();
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        imp::cleanup_bluetooth_resources();

        ret
    }
}
