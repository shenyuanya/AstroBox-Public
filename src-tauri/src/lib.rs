use btclassic_spp::BtclassicSppExt;
use tauri_plugin_deep_link::DeepLinkExt;
use std::{panic, sync::OnceLock};
use tauri::{AppHandle, Builder, Manager};
use tauri_plugin_blec::init as blec_init;
use tauri_plugin_dialog::{DialogExt, MessageDialogKind};

pub mod bt;
pub mod buildinfo;
pub mod community;
pub mod config;
pub mod crypto;
pub mod account;
pub mod frontapi;
pub mod frontmodels;
pub mod fs;
pub mod interface;
pub mod logger;
pub mod miwear;
pub mod pb;
pub mod pluginstore;
pub mod pluginsystem;
pub mod tools;
pub mod tracker;
pub mod auth;
pub mod net;

pub static APP_HANDLE: OnceLock<AppHandle> = OnceLock::new();

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    println!("Welcome to AstroBox");

    let mut dialogs = Vec::new();

    println!("Building tauri plugins...");
    let mut builder = Builder::default();

    #[cfg(not(debug_assertions))]{
        let sentry_client = tracker::sentry::init_sentry();
        builder = builder.plugin(tauri_plugin_sentry::init(&sentry_client));
    }

    #[cfg(desktop)]
    {
        builder = builder
            .plugin(tauri_plugin_single_instance::init(|app, mut args, _| {
                tauri::async_runtime::block_on(async{
                    let mut open_app = true;
                    if args.len() > 1 {
                        args.remove(0);
                        open_app = interface::cli::handle(app.clone(), &args).await;
                    }
                    if open_app {
                        let _ = app
                            .get_webview_window("main")
                            .expect("no main window")
                            .set_focus();
                    }
                });
            }))
            .plugin(tauri_plugin_window_state::Builder::new().build())
    }

    builder = builder
                .plugin(tauri_plugin_dialog::init())
                .plugin(tauri_plugin_fs::init())
                .plugin(tauri_plugin_geolocation::init())
                .plugin(tauri_plugin_os::init())
                .plugin(tauri_plugin_opener::init())
                .plugin(tauri_plugin_deep_link::init());
    #[cfg(mobile)]
    {
        builder = builder.plugin(tauri_plugin_barcode_scanner::init());
    }

    println!("Setting up app...");
    builder = builder
        .setup(move |app| {
            // 保存全局 AppHandle，若已初始化则返回错误
            APP_HANDLE
                .set(app.handle().clone())
                .map_err(|_| "APP_HANDLE 已初始化")?;

            #[cfg(target_os = "macos")]{
                let menu = interface::macmenu::build_menu(&app.handle().clone())?;
                app.set_menu(menu)?;
            }

            println!("Initializing logger...");
            logger::init()?;
            println!("Initializing configuration system...");
            tauri::async_runtime::block_on(config::init(&app.handle()))?;
            println!("Initializing account store...");
            tauri::async_runtime::block_on(account::init(&app.handle()))?;
            println!("Initializing plugin system...");
            pluginsystem::init(config::read(|c| c.clone().plugin_dir).into())?;

            println!("Initializing deep link...");
            // 对于Linux和Windows，为了方便调试，在运行时重新注册一遍deeplink，即可立即应用修改
            // 对于macOS，必须打release包后将程序放到Applications中启动才能注册
            // 对于Android和iOS，必须重新安装apk/ipa才能生效对deeplink的修改
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            app.deep_link().register_all()?;
            app.deep_link().on_open_url(move |event| {
                for url in event.urls() {
                    interface::deeplink::handle_link(url);
                }
            });

            println!("Initializing account manager...");
            tauri::async_runtime::block_on(account::provider::init_providers());
            println!("Initializing community system...");
            match tauri::async_runtime::block_on(community::init()) {
                Ok(_) => {}
                Err(error) => dialogs.push((
                    "官方源初始化失败".to_string(),
                    format!("AstroBox的官方源初始化失败，这可能是您的网络问题导致的。请在设置中更换官方源CDN，然后重启程序再试。错误信息：{}", error.to_string()),
                    MessageDialogKind::Warning,
                )),
            }
            println!("Initializing plugin store...");
            if let Err(e) = tauri::async_runtime::block_on(pluginstore::init()) {
                dialogs.push(("插件仓库初始化失败".to_string(), e, MessageDialogKind::Warning));
            }

            /*
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }*/

            println!("Initializing blec plugin...");
            let blec_result = panic::catch_unwind(|| blec_init());

            if blec_result.is_err() {
                dialogs.push((
                    "BLEC 初始化失败".to_string(),
                    "未检测到支持 Bluetooth 4.0+ 的适配器，相关蓝牙功能将无法正常工作。"
                        .to_string(),
                    MessageDialogKind::Warning,
                ));
            }
            if let Ok(plugin) = blec_result {
                app.handle().plugin(plugin)?;

                #[cfg(not(target_os = "ios"))]
                {
                    app.handle().plugin(btclassic_spp::init())?;
                    let spp_ref: &btclassic_spp::BtclassicSpp<tauri::Wry> = app.btclassic_spp();
                    let ptr: *const btclassic_spp::BtclassicSpp<tauri::Wry> = spp_ref as *const _;

                    // SAFETY: 这里直接获取btclassic_spp插件的裸指针，是为了更便捷地调用该插件
                    // 调用bt::SPP_PLUGIN本质上是安全的，因为除非tauri窗口被关闭，否则插件指针将一直有效
                    unsafe { bt::SPP_PLUGIN = ptr };
                }
            }

            for (title, message, kind) in dialogs {
                app.dialog()
                    .message(&message)
                    .kind(kind)
                    .title(&title)
                    .show(|_| {});
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // MiWear API
            frontapi::miwear_scan,
            frontapi::miwear_stop_scan,
            frontapi::miwear_connect,
            frontapi::miwear_disconnect,
            frontapi::miwear_remove_device,
            frontapi::miwear_get_state,
            frontapi::miwear_get_device_state,
            frontapi::miwear_start_hello,
            frontapi::miwear_start_auth,
            frontapi::miwear_install_third_app,
            frontapi::miwear_install_watchface,
            frontapi::miwear_install_firmware,
            frontapi::miwear_is_sending_mass,
            frontapi::miwear_get_watchface_list,
            frontapi::miwear_uninstall_watchface,
            frontapi::miwear_set_watchface,
            frontapi::miwear_get_app_list,
            frontapi::miwear_open_quickapp,
            frontapi::miwear_uninstall_quickapp,
            frontapi::miwear_get_codename,
            frontapi::miwear_get_device_info,
            frontapi::miwear_get_unlock_code,
            frontapi::miwear_debug_get_commandpool_json_table,
            // Plugin System API
            frontapi::plugsys_get_list,
            frontapi::plugsys_get_state,
            frontapi::plugsys_is_updated,
            frontapi::plugsys_install_abp,
            frontapi::plugsys_enable,
            frontapi::plugsys_disable,
            frontapi::plugsys_remove,
            frontapi::plugsys_get_settings_ui_nodes,
            frontapi::plugsys_call_registered_func,
            // Community Provider API
            frontapi::commprov_get_providers,
            frontapi::commprov_get_page,
            frontapi::commprov_get_item,
            frontapi::commprov_download,
            frontapi::commprov_get_total_items,
            frontapi::commprov_refresh,
            frontapi::commprov_get_state,
            frontapi::officialprov_get_banners,
            frontapi::officialprov_get_device_map,
            frontapi::plugstore_get_providers,
            frontapi::plugstore_get_page,
            frontapi::plugstore_get_item,
            frontapi::plugstore_download,
            frontapi::plugstore_get_total_items,
            frontapi::plugstore_refresh,
            frontapi::plugstore_get_state,
            // Account Provider API
            frontapi::account_get_providers,
            frontapi::account_get,
            frontapi::account_add,
            frontapi::account_remove,
            frontapi::account_login,
            frontapi::account_logout,
            // Common API
            frontapi::frontend_log,
            frontapi::app_get_config,
            frontapi::app_write_config,
            frontapi::cleanup_before_exit,
            frontapi::get_file_type,
            frontapi::get_build_info,
            frontapi::get_login_mi_account,
            frontapi::get_mi_device_list,
            frontapi::add_new_device,
            frontapi::image_url_to_base64_data_url,
            frontapi::modify_watchface_id
        ]);

        #[cfg(target_os = "macos")]{
            builder = builder.on_menu_event(|app, event| {
                interface::macmenu::process_event(app, event.id().0.clone());
            });
        }

        builder.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
