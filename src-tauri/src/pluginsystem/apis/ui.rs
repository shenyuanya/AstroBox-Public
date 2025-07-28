use crate::pluginsystem::{apis::models::PluginUINode, utils::plugin_permission_check};
use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult, JsValue, NativeFunction};
use tauri::Emitter;

pub static UI_PERMISSION: &str = "ui";

pub fn update_plugin_settings_ui(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, UI_PERMISSION.to_string()) {
        return Err(err);
    }

    let ui_str = args
        .get(0)
        .ok_or_else(|| js_error!("updatePluginSettingsUI args[0] is missing"))?;
    let ui: Vec<PluginUINode> = serde_json::from_str(
        &ui_str
            .to_string(ctx)
            .map_err(|e| js_error!("{}", e))?
            .to_std_string_lossy(),
    )
    .map_err(|e| js_error!("{}", e))?;

    let _ = crate::pluginsystem::with_plugin_manager_sync(|pm| {
        let plugname = crate::pluginsystem::utils::get_plugin_name(ctx);
        pm.set_plugin_data(&plugname, |data| {
            data.settings_ui = ui;
        })
    })
    .map_err(|e| js_error!("{}", e));

    if let Some(app) = crate::APP_HANDLE.get() {
        app.emit("plugin-update-settings-page", "")
            .map_err(|e| js_error!("{}", e))?;
    }

    Ok(JsValue::undefined())
}

pub fn open_page_with_nodes(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, UI_PERMISSION.to_string()) {
        return Err(err);
    }

    let ui_str = args
        .get(0)
        .ok_or_else(|| js_error!("openPageWithNodes args[0] is missing"))?;
    let ui: Vec<PluginUINode> = serde_json::from_str(
        &ui_str
            .to_string(ctx)
            .map_err(|e| js_error!("{}", e))?
            .to_std_string_lossy(),
    )
    .map_err(|e| js_error!("{}", e))?;

    if let Some(app) = crate::APP_HANDLE.get() {
        app.emit("plugin-open-page-with-nodes", ui)
            .map_err(|e| js_error!("{}", e))?;
    }

    Ok(JsValue::undefined())
}

pub fn open_page_with_url(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,
) -> JsResult<JsValue> {

    if let Some(err) = plugin_permission_check(ctx, UI_PERMISSION.to_string()) {
        return Err(err);
    }

    let url_str = args
        .get(0)
        .ok_or_else(|| js_error!("openPageWithUrl args[0] is missing"))?
        .to_string(ctx)
        .map_err(|e| js_error!("{}", e))?
        .to_std_string_lossy();

    crate::tools::open_url_with_default_browser(url_str).map_err(|e| js_error!("{}", e))?;

    Ok(JsValue::undefined())
}

pub fn register_ui(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_fn_ptr(update_plugin_settings_ui), js_string!("updatePluginSettingsUI"), 1)
        .function(NativeFunction::from_fn_ptr(open_page_with_nodes), js_string!("openPageWithNodes"), 1)
        .function(NativeFunction::from_fn_ptr(open_page_with_url), js_string!("openPageWithUrl"), 1)
        .build();

    global.property(js_string!("ui"), jsobj, Attribute::READONLY);
    
    Ok(())
}