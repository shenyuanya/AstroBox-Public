use base64::{engine::general_purpose, Engine};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tauri::{
    ipc::{Channel, InvokeResponseBody},
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_btclassic_spp);

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<BtclassicSpp<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(
        "com.astralsight.astrobox.plugin.btclassic_spp",
        "BtClassicSPPPlugin",
    )?;
    // ios插件并没有实际实现，只是为了通过编译
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_btclassic_spp)?;
    Ok(BtclassicSpp(handle))
}

/// 访问 btclassic-spp API
pub struct BtclassicSpp<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> BtclassicSpp<R> {
    /* ---------- 无返回值调用 ---------- */
    pub fn start_scan(&self) -> anyhow::Result<()> {
        self.0
            .run_mobile_plugin::<()>("startScan", ())
            .map_err(Into::into)
    }

    pub fn stop_scan(&self) -> anyhow::Result<()> {
        self.0
            .run_mobile_plugin::<()>("stopScan", ())
            .map_err(Into::into)
    }

    pub fn connect(&self, addr: &String) -> anyhow::Result<ConnectResult> {
        let arg = ConnectArg {
            addr: addr.to_owned(),
        };
        self.0.run_mobile_plugin("connect", arg).map_err(Into::into)
    }

    pub fn disconnect(&self) -> anyhow::Result<()> {
        self.0
            .run_mobile_plugin::<()>("disconnect", ())
            .map_err(Into::into)
    }

    pub fn start_subscription(&self) -> anyhow::Result<()> {
        self.0
            .run_mobile_plugin::<()>("startSubscription", ())
            .map_err(Into::into)
    }

    pub fn send(&self, data: &[u8]) -> anyhow::Result<()> {
        self.0
            .run_mobile_plugin::<()>(
                "send",
                SPPSendPayload {
                    b64data: general_purpose::STANDARD.encode(data),
                },
            )
            .map_err(Into::into)
    }

    /* ---------- 有返回值调用 ---------- */
    pub fn get_scanned_devices(&self) -> anyhow::Result<GetScannedDevicesResult> {
        self.0
            .run_mobile_plugin("getScannedDevices", ())
            .map_err(Into::into)
    }

    pub fn get_connected_device_info(&self) -> anyhow::Result<SPPDevice> {
        self.0
            .run_mobile_plugin("getConnectedDeviceInfo", ())
            .map_err(Into::into)
    }

    /* ---------- 事件回调 ---------- */
    pub fn on_connected<F>(&self, cb: F) -> anyhow::Result<()>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let cb_arc = Arc::new(cb);
        let channel = Channel::<Value>::new(move |_raw| {
            (cb_arc)();
            Ok(())
        });

        self.0.run_mobile_plugin::<()>("onConnected", channel)?;
        Ok(())
    }

    pub fn set_data_listener<F>(&self, cb: F) -> anyhow::Result<()>
    where
        F: FnMut(Result<Vec<u8>, String>) + Send + 'static,
    {
        let cb_arc = Arc::new(Mutex::new(cb));

        let channel = Channel::<()>::new({
            let cb_arc = Arc::clone(&cb_arc);
            move |raw: InvokeResponseBody| {
                let msg: SetDataListenerResult = match raw.deserialize() {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("setDataListener (mobile): deserialize error: {e}");
                        if let Ok(mut f) = cb_arc.lock() {
                            (f)(Err(format!("Deserialize error from mobile: {}", e)));
                        }
                        return Ok(());
                    }
                };

                if let Some(err_msg) = msg.err {
                    eprintln!("SPP read error (Android): {}", err_msg);
                    if let Ok(mut f) = cb_arc.lock() {
                        (f)(Err(err_msg));
                    }
                    return Ok(());
                }

                let bytes = match general_purpose::STANDARD.decode(msg.ret) {
                    Ok(b) => b,
                    Err(e) => {
                        eprintln!("Base64 decode error (mobile): {e}");
                        if let Ok(mut f) = cb_arc.lock() {
                            (f)(Err(format!("Base64 decode error from mobile: {}", e)));
                        }
                        return Ok(());
                    }
                };

                if let Ok(mut f) = cb_arc.lock() {
                    (f)(Ok(bytes));
                }
                Ok(())
            }
        });

        self.0.run_mobile_plugin::<()>("setDataListener", channel)?;
        Ok(())
    }
}
