use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult, JsValue, NativeFunction};
use tauri::Emitter;

pub use crate::pluginsystem::utils::plugin_permission_check;

static INSTALLER_PERMISSION: &str = "installer";

pub fn add_third_party_app_to_queue(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, INSTALLER_PERMISSION.to_string()) {
        return Err(err);
    }

    let file_path = args
        .get(0)
        .ok_or_else(|| js_error!("addThirdPartyAppToQueue args[0] is missing"))?
        .to_string(ctx)
        .map_err(|e| js_error!("{}", e))?
        .to_std_string_lossy();

    if let Some(app) = crate::APP_HANDLE.get() {
        app.emit("plugin-add-third-party-app-to-queue", file_path)
            .map_err(|e| js_error!("{}", e))?;
    }

    Ok(JsValue::undefined())
}

pub fn add_watchface_to_queue(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, INSTALLER_PERMISSION.to_string()) {
        return Err(err);
    }

    let file_path = args
        .get(0)
        .ok_or_else(|| js_error!("addWatchFaceToQueue args[0] is missing"))?
        .to_string(ctx)
        .map_err(|e| js_error!("{}", e))?
        .to_std_string_lossy();

    if let Some(app) = crate::APP_HANDLE.get() {
        app.emit("plugin-add-watch-face-to-queue", file_path)
            .map_err(|e| js_error!("{}", e))?;
    }

    Ok(JsValue::undefined())
}

pub fn add_firmware_to_queue(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, INSTALLER_PERMISSION.to_string()) {
        return Err(err);
    }

    let file_path = args
        .get(0)
        .ok_or_else(|| js_error!("addFirmwareToQueue args[0] is missing"))?
        .to_string(ctx)
        .map_err(|e| js_error!("{}", e))?
        .to_std_string_lossy();

    if let Some(app) = crate::APP_HANDLE.get() {
        app.emit("plugin-add-firmware-to-queue", file_path)
            .map_err(|e| js_error!("{}", e))?;
    }

    Ok(JsValue::undefined())
}

pub fn register_installer(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_fn_ptr(add_third_party_app_to_queue), js_string!("addThirdPartyAppToQueue"), 1)
        .function(NativeFunction::from_fn_ptr(add_watchface_to_queue), js_string!("addWatchFaceToQueue"), 1)
        .function(NativeFunction::from_fn_ptr(add_firmware_to_queue), js_string!("addFirmwareToQueue"), 1)
        .build();

    global.property(js_string!("installer"), jsobj, Attribute::READONLY);
    
    Ok(())
}