use btclassic_spp::BtclassicSpp;
use tauri::Wry;
use tauri_plugin_blec::Handler;

pub mod device;

pub static mut SPP_PLUGIN: *const BtclassicSpp<Wry> = std::ptr::null();

pub fn get_ble_handler() -> &'static Handler {
    tauri_plugin_blec::get_handler().unwrap()
}

pub fn get_spp_handle() -> &'static BtclassicSpp<Wry> {
    // 安全性声明详见lib.rs
    unsafe {
        assert!(!SPP_PLUGIN.is_null(), "SPP_PLUGIN has not been initialized");
        &*SPP_PLUGIN
    }
}
