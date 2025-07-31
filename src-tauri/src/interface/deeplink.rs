use serde_json::json;
use std::collections::HashMap;
use tauri::Emitter;
use url::Url;

use crate::account::provider::get_provider;
use crate::APP_HANDLE;

pub fn handle_link(url: Url) {
    log::info!("[DeepLink] handle link: {}", url.as_str());
    // 若 APP_HANDLE 未初始化则直接忽略
    let Some(app) = APP_HANDLE.get() else {
        log::error!("APP_HANDLE 未初始化，无法处理 deeplink");
        return;
    };

    let params: HashMap<String, String> = url.query_pairs().into_owned().collect();

    if let Some(str) = params.get("source") {
        match str.as_str() {
            "bandbbs" => {
                // Extract the authorization code and state
                if let (Some(code), Some(state)) = (params.get("code"), params.get("state")) {
                    // Clone the values to make them owned
                    let code = code.clone();
                    let state = state.clone();
                    log::debug!("BandBBS OAuth callback: code={}, state={}", code, state);
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        rt.block_on(async {
                            match get_provider("bandbbs").await {
                                Some(provider) => match provider.login_step2(&code, &state).await {
                                    Ok(_) => {
                                        log::info!("[DeepLink] BandBBS authorization successful")
                                    }
                                    Err(e) => log::error!(
                                        "[DeepLink] BandBBS authorization failed: {}",
                                        e
                                    ),
                                },
                                None => log::error!("[DeepLink] BandBBS provider not found"),
                            }
                        });
                    });
                } else {
                    log::error!("[DeepLink] Missing code or state in BandBBS callback");
                }
            }
            "deviceQr" => {
                let _ = app.emit("deviceQr", url.as_str());
            }
            "res" => {
                if let Some(resname) = params.get("res") {
                    if let Some(provider_name) = params.get("provider") {
                        log::info!(
                            "[DeepLink] emitting open-resource: resname={}, provider={}",
                            resname,
                            provider_name
                        );
                        let _ = app.emit(
                            "open-resource",
                            json!({
                            "resname": resname,
                            "provider_name": provider_name
                            }),
                        );
                    }
                }
            }
            "getUa" => {
                if let Some(ua) = params.get("ua") {
                    log::info!("[DeepLink] emitting ua-callback: ua={}", ua);
                    let _ = app.emit(
                        "ua-callback",
                        json!({
                        "ua": ua
                        }),
                    );
                }
            }
            _ => {
                log::warn!("[DeepLink] Unknown source: {}", str);
            }
        }
    }
}
