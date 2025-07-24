use crate::{
    account::Account,
    auth::{login_mi_account, mi_service_call_encrypted, DeviceListRespone, MiAccountToken},
    bt::device::{BTDevice, ConnectType},
    buildinfo::BuildInfo,
    community::provider::{
        official::{models::Banner, OfficialProvider},
        Item, ProgressData, ProviderState, ResourceManifestV1, SearchConfig,
    },
    config::AppConfig,
    frontmodels::BTDeviceInfo,
    miwear::device::models::DeviceMap,
    miwear::device::{
        mass::SendMassCallbackData,
        system::{system_get_device_info, system_get_device_status, SystemInfo, SystemStatus},
        MiWearBleCharaUuid, MiWearState,
    },
    pluginstore::provider::StorePluginManifest,
    pluginsystem::{apis::models::PluginUINode, manifest::PluginManifest, plugin::PluginState},
};
use base64::{engine::general_purpose, Engine};
use boa_engine::{js_string, JsValue};
use mime_guess::MimeGuess;
use serde_json::{json, Value};
use std::{collections::HashMap, fs, io::Write, path::Path, sync::Arc};
use tauri::{ipc::Channel, Emitter};
use uuid::Uuid;

// 前端API：扫描设备
// 在Windows上使用SPP模式时仅返回支持经典蓝牙的设备
#[tauri::command]
pub fn miwear_scan(cb: Channel<Vec<BTDeviceInfo>>) -> Result<(), String> {
    let scan_type = crate::config::read(|c| c.clone().connect_type);

    BTDevice::scan(15_000, scan_type, move |devices| {
        let retinfo: Vec<_> = devices
            .into_iter()
            .map(|device| match device.handle {
                crate::bt::device::Connection::BLE(ble) => BTDeviceInfo {
                    name: ble.name,
                    addr: ble.address,
                    connect_type: "BLE".into(),
                },
                crate::bt::device::Connection::SPP(spp) => BTDeviceInfo {
                    name: spp.name.unwrap_or_default(),
                    addr: spp.address,
                    connect_type: "SPP".into(),
                },
            })
            .collect();

        let _ = cb.send(retinfo);
    })
    .expect("Error when scan bluetooth devices");

    Ok(())
}

// 前端API：停止扫描
#[tauri::command]
pub async fn miwear_stop_scan() -> Result<(), String> {
    if crate::config::read(|c| c.clone().connect_type) == ConnectType::BLE {
        crate::bt::get_ble_handler()
            .stop_scan()
            .await
            .map_err(|e| e.to_string())?;
    } else {
        crate::bt::get_spp_handle()
            .stop_scan()
            .map_err(|e| e.to_string())?;
    }

    Ok(())
}

// 前端API：连接设备
#[tauri::command]
pub async fn miwear_connect(addr: String, name: String) -> Result<(), String> {
    if let Some(current_device) = crate::miwear::CONNECTED_DEVICE.read().await.as_ref() {
        if current_device.state.read().await.addr != addr {
            log::info!("Switching device, disconnecting from the old one first.");
            current_device
                .disconnect()
                .await
                .map_err(|e| e.to_string())?;
            // 等待一小段时间确保资源完全释放
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        } else {
            log::info!("Already connected to this device. No action needed.");
            return Ok(());
        }
    }

    let device = crate::miwear::device::MiWearDevice::connect(addr, name)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(app) = crate::APP_HANDLE.get() {
        let _ = app.emit("device-connected", ());
    }
    crate::miwear::set_connected_device(Some(device)).await;
    Ok(())
}

// 前端API：断连设备
#[tauri::command]
pub async fn miwear_disconnect() -> Result<(), String> {
    crate::miwear::with_connected_device_async(|device| async move {
        device.disconnect().await?;
        anyhow::Ok(())
    })
    .await
}

#[tauri::command]
pub async fn miwear_remove_device(addr: String) -> Result<(), String> {
    let current_device = crate::config::read(|c| c.current_device.clone());

    if let Some(dev) = &current_device {
        if dev.addr == addr {
            let _ = miwear_disconnect().await;
        }
    }

    crate::config::write(|c| {
        c.paired_devices.retain(|dev| dev.addr != addr);
        if let Some(dev) = &c.current_device {
            if dev.addr == addr {
                c.current_device = None;
            }
        }
    });

    Ok(())
}

// 前端API：获取已连接设备的信息
#[tauri::command]
pub async fn miwear_get_state() -> Result<MiWearState, String> {
    crate::miwear::with_connected_device_async(|device| async move {
        let state = device.state.read().await;
        anyhow::Ok(state.clone())
    })
    .await
}

// 前端API：获取已连接设备的电量等信息
#[tauri::command]
pub async fn miwear_get_device_state() -> Result<SystemStatus, String> {
    let result = crate::miwear::with_connected_device_async(|device| async move {
        system_get_device_status(device).await
    })
    .await;
    result.map_err(|e| e.to_string())
}

//获取设备详细信息
#[tauri::command]
pub async fn miwear_get_device_info() -> Result<SystemInfo, String> {
    let result = crate::miwear::with_connected_device_async(|device| async move {
        system_get_device_info(device).await
    })
    .await;
    result.map_err(|e| e.to_string())
}

// 前端API：给设备发送hello和sc
#[tauri::command]
pub async fn miwear_start_hello() -> Result<(), String> {
    crate::miwear::with_connected_device_async(|device| async move { device.start_hello().await })
        .await
}

// 前端API：开始设备auth
#[tauri::command]
pub async fn miwear_start_auth(auth_key: String) -> Result<(), String> {
    crate::miwear::with_connected_device_async(
        |device| async move { device.start_auth(auth_key).await },
    )
    .await
}

// 前端API：安装third app (MassDataType=64)
#[tauri::command]
pub async fn miwear_install_third_app(
    file_path: String,
    package_name: String,
    version_code: u32,
    on_progress: Channel<SendMassCallbackData>,
) -> Result<(), String> {
    crate::miwear::with_connected_device_async(|device| async move {
        crate::miwear::device::thirdpartyapp::install_app(
            device,
            &file_path,
            &package_name,
            version_code,
            move |data| {
                let _ = on_progress.send(data);
            },
        )
        .await?;
        anyhow::Ok(())
    })
    .await
}

// 前端API：安装表盘 (MassDataType=16)
#[tauri::command]
pub async fn miwear_install_watchface(
    file_path: String,
    on_progress: Channel<SendMassCallbackData>,
) -> Result<(), String> {
    crate::miwear::with_connected_device_async(|device| async move {
        let id = crate::miwear::device::resutils::get_watchface_id(&file_path)
            .await
            .unwrap_or("000000000000".to_string());
        crate::miwear::device::watchface::install_watchface(device, &file_path, &id, move |data| {
            let _ = on_progress.send(data);
        })
        .await?;
        anyhow::Ok(())
    })
    .await
}

// 前端API：获取表盘列表
#[tauri::command]
pub async fn miwear_get_watchface_list(
) -> Result<Vec<crate::miwear::device::watchface::WatchfaceInfo>, String> {
    let watchface_list = crate::miwear::with_connected_device_async(|device| async {
        crate::miwear::device::watchface::get_watchface_list(device).await
    })
    .await?;
    Ok(watchface_list)
}

// 前端API：卸载表盘
#[tauri::command]
pub async fn miwear_uninstall_watchface(
    watchface: crate::miwear::device::watchface::WatchfaceInfo,
) -> Result<(), String> {
    crate::miwear::with_connected_device_async(|device| async {
        crate::miwear::device::watchface::uninstall_watchface(device, watchface).await
    })
    .await?;
    Ok(())
}

// 前端API：启用表盘
#[tauri::command]
pub async fn miwear_set_watchface(
    watchface: crate::miwear::device::watchface::WatchfaceInfo,
) -> Result<(), String> {
    crate::miwear::with_connected_device_async(|device| async {
        crate::miwear::device::watchface::set_watchface(device, watchface).await
    })
    .await?;
    Ok(())
}

// 前端API：安装固件 (MassDataType=32)
#[tauri::command]
pub async fn miwear_install_firmware(
    file_path: String,
    on_progress: Channel<SendMassCallbackData>,
) -> Result<(), String> {
    crate::miwear::with_connected_device_async(|device| async move {
        crate::miwear::device::firmware::install_firmware(device, &file_path, move |data| {
            let _ = on_progress.send(data);
        })
        .await?;
        anyhow::Ok(())
    })
    .await
}

//获取设备应用列表
#[tauri::command]
pub async fn miwear_get_app_list(
) -> Result<Vec<crate::miwear::device::thirdpartyapp::AppInfo>, String> {
    let app_list = crate::miwear::with_connected_device_async(|device| async {
        crate::miwear::device::thirdpartyapp::get_app_list(device).await
    })
    .await?;
    Ok(app_list)
}

//删除快应用
#[tauri::command]
pub async fn miwear_uninstall_quickapp(
    app: crate::miwear::device::thirdpartyapp::AppInfo,
) -> Result<(), String> {
    crate::miwear::with_connected_device_async(|device| async {
        crate::miwear::device::thirdpartyapp::uninstall_app(device, app).await
    })
    .await?;
    Ok(())
}

//打开快应用
#[tauri::command]
pub async fn miwear_open_quickapp(
    app: crate::miwear::device::thirdpartyapp::AppInfo,
    page: String,
) -> Result<(), String> {
    crate::miwear::with_connected_device_async(|device| async {
        crate::miwear::device::thirdpartyapp::launch_app(device, app, &page).await
    })
    .await?;
    Ok(())
}

#[tauri::command]
pub async fn miwear_is_sending_mass() -> Result<bool, String> {
    crate::miwear::with_connected_device_async(|device| async move {
        let stat = device.is_sending_mass.lock().await.clone();
        anyhow::Ok(stat)
    })
    .await
}

#[tauri::command]
pub async fn miwear_get_codename() -> Result<String, String> {
    crate::miwear::with_connected_device_async(|device| async move {
        device.get_codename().await.map_err(|e| e.to_string())
    })
    .await
}

#[tauri::command]
pub async fn miwear_debug_get_commandpool_json_table() -> Result<Vec<serde_json::Value>, String> {
    crate::miwear::with_connected_device_async(|device| async move {
        let table = device.cmd_pool.to_json_table().await;
        anyhow::Ok(table)
    })
    .await
}

// 插件系统API
#[tauri::command]
pub async fn plugsys_get_list() -> Vec<PluginManifest> {
    crate::pluginsystem::with_plugin_manager_async(|pm| pm.list())
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn plugsys_get_state(name: String) -> PluginState {
    crate::pluginsystem::with_plugin_manager_async(move |pm| {
        pm.get(&name)
            .map(|p| p.state.clone())
            .unwrap_or(PluginState {
                disabled: true,
                icon_b64: String::new(),
            })
    })
    .await
    .unwrap_or(PluginState {
        disabled: true,
        icon_b64: String::new(),
    })
}

#[tauri::command]
pub async fn plugsys_is_updated() -> Result<bool, String> {
    crate::pluginsystem::with_plugin_manager_async(move |pm| pm.is_updated())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn plugsys_install_abp(name: String, path: String) -> Result<(), String> {
    crate::pluginsystem::with_plugin_manager_async(move |pm| {
        tokio::task::block_in_place(|| {
            tauri::async_runtime::block_on(pm.add_from_abp(&name, &path))
        })
        .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn plugsys_enable(name: String) -> bool {
    crate::pluginsystem::with_plugin_manager_async(move |pm| pm.enable(&name))
        .await
        .unwrap_or(false)
}

#[tauri::command]
pub async fn plugsys_disable(name: String) -> bool {
    crate::pluginsystem::with_plugin_manager_async(move |pm| pm.disable(&name))
        .await
        .unwrap_or(false)
}

#[tauri::command]
pub async fn plugsys_remove(name: String) -> bool {
    crate::pluginsystem::with_plugin_manager_async(move |pm| pm.remove(&name))
        .await
        .unwrap_or(false)
}

#[tauri::command]
pub async fn plugsys_get_settings_ui_nodes(name: String) -> Vec<PluginUINode> {
    crate::pluginsystem::with_plugin_manager_async(move |pm| {
        pm.get(&name)
            .map(|p| p.js_env_data.settings_ui.clone())
            .unwrap_or_default()
    })
    .await
    .unwrap_or_default()
}

#[tauri::command]
pub async fn plugsys_call_registered_func(
    name: String,
    func_id: String,
    payload: String,
) -> String {
    crate::pluginsystem::with_plugin_manager_async(move |pm| {
        let Some(plug) = pm.get(&name) else {
            return "Target plugin not found".to_string();
        };
        for func in &plug.js_env_data.registered_functions {
            if func.0 == &func_id {
                let result = func.1.call(
                    &JsValue::undefined(),
                    &[js_string!(payload).into()],
                    &mut plug.js_context,
                );
                plug.js_context.run_jobs();
                match result {
                    Ok(res) => {
                        return res
                            .to_string(&mut plug.js_context)
                            .unwrap()
                            .to_std_string_lossy();
                    }
                    Err(err) => {
                        return err.to_string();
                    }
                }
            }
        }
        return "Target function not found".to_string();
    })
    .await
    .unwrap_or_else(|e| e.to_string())
}

// 社区Provider API
#[tauri::command]
pub async fn commprov_get_providers() -> Vec<String> {
    crate::community::list_providers().await
}

#[tauri::command]
pub async fn commprov_get_page(
    name: String,
    page: u32,
    limit: u32,
    search: SearchConfig,
) -> Result<Vec<Item>, String> {
    match crate::community::get_provider(&name).await {
        Some(prov) => {
            let items = prov
                .get_page(page, limit, search)
                .await
                .map_err(|e| e.to_string())?;
            Ok(items)
        }
        None => Err("Provider not found".to_string()),
    }
}

#[tauri::command]
pub async fn commprov_get_item(
    name: String,
    resname: String,
) -> Result<ResourceManifestV1, String> {
    match crate::community::get_provider(&name).await {
        Some(prov) => {
            let item = prov.get_item(resname).await.map_err(|e| e.to_string())?;
            Ok(item)
        }
        None => Err("Provider not found".to_string()),
    }
}

#[tauri::command]
pub async fn commprov_download(
    name: String,
    resname: String,
    device: String,
    progress_cb: Channel<ProgressData>,
) -> Result<String, String> {
    match crate::community::get_provider(&name).await {
        Some(prov) => {
            let path = prov
                .download(resname, device, progress_cb)
                .await
                .map_err(|e| e.to_string())?;
            Ok(path)
        }
        None => Err("Provider not found".to_string()),
    }
}

#[tauri::command]
pub async fn commprov_get_total_items(name: String, filter: Option<String>) -> Result<u64, String> {
    match crate::community::get_provider(&name).await {
        Some(prov) => {
            let path = prov
                .get_total_items(filter)
                .await
                .map_err(|e| e.to_string())?;
            Ok(path)
        }
        None => Err("Provider not found".to_string()),
    }
}

#[tauri::command]
pub async fn commprov_refresh(name: String) -> Result<(), String> {
    match crate::community::get_provider(&name).await {
        Some(prov) => prov.refresh().await.map_err(|e| e.to_string()),
        None => Err("Provider not found".to_string()),
    }
}

#[tauri::command]
pub async fn commprov_get_state(name: String) -> Option<ProviderState> {
    crate::community::get_provider(&name)
        .await
        .map(|p| p.state())
}

#[tauri::command]
pub async fn officialprov_get_banners() -> Result<Vec<Banner>, String> {
    let provider = crate::community::get_provider("official").await.unwrap();
    if provider.as_any().is::<OfficialProvider>() {
        let ptr = Arc::into_raw(provider) as *const OfficialProvider;

        // SAFETY: 名为official的Provider类型一定是OfficialProvider
        // 因此强行转换是安全的
        let official_provider = unsafe { Arc::from_raw(ptr) };
        let banners = official_provider
            .get_banner()
            .await
            .map_err(|e| e.to_string())?;

        return Ok(banners);
    }

    Err("Provider Error".to_string())
}

#[tauri::command]
pub async fn officialprov_get_device_map() -> Result<DeviceMap, String> {
    let provider = crate::community::get_provider("official").await.unwrap();
    if provider.as_any().is::<OfficialProvider>() {
        let ptr = Arc::into_raw(provider) as *const OfficialProvider;

        // SAFETY: 名为official的Provider类型一定是OfficialProvider
        let official_provider = unsafe { Arc::from_raw(ptr) };
        let map = official_provider
            .get_device_map()
            .await
            .map_err(|e| e.to_string())?;

        return Ok(map);
    }

    Err("Provider Error".to_string())
}

// 账户Provider API
// Plugin Store API
#[tauri::command]
pub async fn plugstore_get_providers() -> Vec<String> {
    crate::pluginstore::list_providers().await
}

#[tauri::command]
pub async fn plugstore_get_page(
    name: String,
    page: u32,
    limit: u32,
    filter: Option<String>,
) -> Result<Vec<StorePluginManifest>, String> {
    match crate::pluginstore::get_provider(&name).await {
        Some(prov) => prov
            .get_page(page, limit, filter)
            .await
            .map_err(|e| e.to_string()),
        None => Err("Provider not found".to_string()),
    }
}

#[tauri::command]
pub async fn plugstore_get_item(
    name: String,
    plugin: String,
) -> Result<StorePluginManifest, String> {
    match crate::pluginstore::get_provider(&name).await {
        Some(prov) => prov.get_item(plugin).await.map_err(|e| e.to_string()),
        None => Err("Provider not found".to_string()),
    }
}

#[tauri::command]
pub async fn plugstore_download(
    name: String,
    plugin: String,
    progress_cb: Channel<ProgressData>,
) -> Result<String, String> {
    match crate::pluginstore::get_provider(&name).await {
        Some(prov) => prov
            .download(plugin, progress_cb)
            .await
            .map_err(|e| e.to_string()),
        None => Err("Provider not found".to_string()),
    }
}

#[tauri::command]
pub async fn plugstore_get_total_items(
    name: String,
    filter: Option<String>,
) -> Result<u64, String> {
    match crate::pluginstore::get_provider(&name).await {
        Some(prov) => prov
            .get_total_items(filter)
            .await
            .map_err(|e| e.to_string()),
        None => Err("Provider not found".to_string()),
    }
}

#[tauri::command]
pub async fn plugstore_refresh(name: String) -> Result<(), String> {
    match crate::pluginstore::get_provider(&name).await {
        Some(prov) => prov.refresh().await.map_err(|e| e.to_string()),
        None => Err("Provider not found".to_string()),
    }
}

#[tauri::command]
pub async fn plugstore_get_state(name: String) -> Option<ProviderState> {
    crate::pluginstore::get_provider(&name)
        .await
        .map(|p| p.state())
}
#[tauri::command]
pub async fn account_get_providers() -> Vec<String> {
    crate::account::provider::list_providers().await
}

#[tauri::command]
pub fn account_get(name: String) -> Vec<Account> {
    crate::account::list_accounts(&name)
}

#[tauri::command]
pub fn account_add(name: String, account: Account) -> Result<(), String> {
    crate::account::add_account(&name, account);
    Ok(())
}

#[tauri::command]
pub fn account_remove(name: String, index: usize) -> Result<(), String> {
    crate::account::remove_account(&name, index);
    Ok(())
}

#[tauri::command]
pub async fn account_login(name: String) -> Account {
    let prov = crate::account::provider::get_provider(&name)
        .await
        .expect("target provider not found");
    let account = prov.login().await.expect("failed to login");
    crate::account::add_account(&name, account.clone());
    account
}

#[tauri::command]
pub async fn account_logout(name: String, account: Account) -> Result<(), String> {
    let prov = crate::account::provider::get_provider(&name)
        .await
        .expect("target provider not found");
    prov.logout(&account).await.expect("failed to logout");
    Ok(())
}
// 通用API

// 前端API：打印日志
#[tauri::command]
pub fn frontend_log(level: String, message: String) {
    match level.as_str() {
        "info" => {
            log::info!("[FrontEnd] {}", message);
        }
        "warn" => {
            log::warn!("[FrontEnd] {}", message);
        }
        "error" => {
            log::error!("[FrontEnd] {}", message);
        }
        _ => {}
    }
}

// 前端API：获取应用Config
#[tauri::command]
pub fn app_get_config() -> Result<AppConfig, String> {
    let config = crate::config::read(|c| c.clone());
    Ok(config)
}

#[tauri::command]
pub fn app_write_config(patch: serde_json::Value) -> Result<AppConfig, String> {
    let new_cfg: AppConfig = {
        let cur_cfg = crate::config::read(|c| c.clone());
        let mut val = serde_json::to_value(cur_cfg).expect("serialize AppConfig to JSON");
        crate::config::merge(&mut val, &patch);
        serde_json::from_value(val).map_err(|e| format!("invalid config patch: {e}"))?
    };
    crate::config::write(|cfg| *cfg = new_cfg.clone());
    Ok(new_cfg)
}

// 前端API：执行清理并退出
#[tauri::command]
pub async fn cleanup_before_exit() {
    log::info!("Cleaning up before exit...");
    #[cfg(target_os = "windows")]
    if let Err(e) = crate::bt::get_spp_handle().disconnect() {
        log::error!("断开 SPP 连接失败: {}", e);
    }

    std::process::exit(0);
}

// 前端API：获取文件类型
#[tauri::command]
pub async fn get_file_type(path: String) -> Result<String, String> {
    match crate::miwear::device::resutils::get_file_type(&path).await {
        Ok(tp) => {
            log::info!("{} is {}", path, tp);
            Ok(tp)
        }
        Err(e) => Err(e.to_string()),
    }
}

// 前端API：获取BuildInfo
#[tauri::command]
pub fn get_build_info() -> Result<Value, String> {
    Ok(json!({
        "GIT_COMMIT_HASH": BuildInfo::GIT_COMMIT_HASH,
        "BUILD_TIME": BuildInfo::BUILD_TIME,
        "BUILD_USER": BuildInfo::BUILD_USER,
        "VERSION": env!("CARGO_PKG_VERSION")
    }))
}

#[tauri::command]
pub async fn get_login_mi_account(
    username: String,
    password: String,
    ua: String,
) -> Result<MiAccountToken, String> {
    let account = login_mi_account(username, password, ua).await;
    match account {
        Ok(res) => Ok(res),
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub async fn get_mi_device_list(
    token: MiAccountToken,
    ua: String,
) -> Result<DeviceListRespone, String> {
    let mut params = HashMap::<String, String>::new();
    params.insert(
        "data".to_string(),
        "{\"page_size\":50,\"status\":1}".to_string(),
    );

    let result = mi_service_call_encrypted(
        token,
        "".to_string(),
        "https://hlth.io.mi.com/app/v1/source/get_source_list".to_string(),
        params,
        ua,
    )
    .await;

    match result {
        Ok(res) => {
            let rsp: DeviceListRespone = serde_json::from_str(&res).unwrap();
            log::info!("[MiAccount.DeviceList] got devices: {}", &res);
            Ok(rsp)
        }
        Err(err) => Err(err.to_string()),
    }
}

#[tauri::command]
pub fn add_new_device(name: String, addr: String, authkey: String) {
    let state: MiWearState = MiWearState {
        name: name,
        addr: addr.clone(),
        authkey: authkey.clone(),
        bleservice: MiWearBleCharaUuid {
            recv: Uuid::nil(),
            sent: Uuid::nil(),
        },
        max_frame_size: 0,
        sec_keys: None,
        network_mtu: 800, /* 默认800 此值过大会导致表端buffer溢出 极限大概在900左右 设置为900会导致不稳定 */
        codename: String::new(),
    };

    crate::config::write(|c| {
        // 如果存在相同addr的设备则更新authkey，否则添加新设备
        if let Some(existing_device) = c.paired_devices.iter_mut().find(|d| d.addr == addr) {
            existing_device.authkey = authkey;
        } else {
            c.paired_devices.push(state);
        }
    });
}

#[tauri::command]
pub fn miwear_get_unlock_code(sn: String, mac: String) -> String {
    crate::tools::calc_unlock_code(mac, sn)
}

// 前端API：图片URL转Base64 Data URL，防止跨域请求
#[tauri::command]
pub async fn image_url_to_base64_data_url(url: &str) -> Result<String, String> {
    let client = crate::net::default_client();
    let resp = client.get(url).send().await.map_err(|e| e.to_string())?;
    let bytes = resp.bytes().await.map_err(|e| e.to_string())?;
    let mime_type = MimeGuess::from_path(url)
        .first_or_octet_stream()
        .to_string();

    let b64 = general_purpose::STANDARD.encode(&bytes);

    let data_url = format!("data:{};base64,{}", mime_type, b64);

    Ok(data_url)
}

// 十六进制修改表盘ID
#[tauri::command]
pub async fn modify_watchface_id(original_path: String, new_id: String) -> Result<(), String> {
    let path = Path::new(&original_path);

    let mut content = fs::read(path).map_err(|e| format!("Read file failed: {}", e))?;

    if content.len() < 0x28 + 12 {
        return Err("File too small to modify".into());
    }

    let mut new_id_bytes = new_id.as_bytes().to_vec();

    if new_id_bytes.len() < 12 {
        let padding = 12 - new_id_bytes.len();
        new_id_bytes.extend(std::iter::repeat(0x00).take(padding));
    }

    for i in 0..12 {
        content[0x28 + i] = new_id_bytes[i];
    }

    let temp_path = {
        let mut temp = path.to_path_buf();
        temp.set_extension("tmp");
        temp
    };

    let mut temp_file = fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temporary file: {}", e))?;

    temp_file
        .write_all(&content)
        .map_err(|e| format!("Write failed: {}", e))?;

    temp_file
        .sync_all()
        .map_err(|e| format!("Sync failed: {}", e))?;

    fs::rename(&temp_path, path).map_err(|e| format!("File replacement failed: {}", e))?;

    Ok(())
}
