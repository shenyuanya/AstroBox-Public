use std::{future::{ready, Future}, pin::Pin, str::FromStr};

use boa_engine::{js_error, js_string, object::ObjectInitializer, property::Attribute, Context, JsResult, JsString, JsValue, NativeFunction};

use crate::{miwear::device::thirdpartyapp::AppInfo, pluginsystem::utils::plugin_permission_check};

pub static THIRD_PARTY_APP_PERMISSION: &str = "thirdpartyapp";

pub fn launch(
    _this: &JsValue,
    args: &[JsValue],
    ctx: &mut Context,

) -> Pin<Box<dyn Future<Output = JsResult<JsValue>> + 'static>> {

    if let Some(err) = plugin_permission_check(ctx, THIRD_PARTY_APP_PERMISSION.to_string()) {
        return Box::pin(ready(Err(err)));
    }

    let app_res: Result<String, boa_engine::JsError> = match args.get(0) {
        Some(v) => v.to_string(ctx).map(|s| s.to_std_string_lossy()).map_err(|e| js_error!("{}", e)),
        None => Err(js_error!("launchQA args[0] appInfo missing")),
    };
    let page_res: Result<String, boa_engine::JsError> = match args.get(1) {
        Some(v) => v.to_string(ctx).map(|s| s.to_std_string_lossy()).map_err(|e| js_error!("{}", e)),
        None => Err(js_error!("launchQA args[1] pageName missing")),
    };

    let info_res: Result<AppInfo, boa_engine::JsError> = app_res
        .and_then(|app| serde_json::from_str(&app).map_err(|e| js_error!("{}", e)));

    Box::pin(async move {
        let appinfo = info_res?;
        let pagename = page_res?;
        crate::miwear::with_connected_device_async(async move |dev| {
            crate::miwear::device::thirdpartyapp::launch_app(dev, appinfo, &pagename).await?;
            anyhow::Ok(())
        })
        .await
        .map_err(|e| js_error!("{}", e))?;

        Ok(JsValue::undefined())
    })
}

pub fn get_thirdparty_app_list(
    _this: &JsValue,
    _args: &[JsValue],
    _ctx: &mut Context,
) -> impl Future<Output = JsResult<JsValue>> {

    let permission_result = plugin_permission_check(_ctx, THIRD_PARTY_APP_PERMISSION.to_string());

    async move {

        if let Some(err) = permission_result {
            return Err(err);
        }

        let list = crate::miwear::with_connected_device_async(|dev| async move {
            anyhow::Ok(crate::miwear::device::thirdpartyapp::get_app_list(dev).await?)
        })
        .await
        .map_err(|e| js_error!("{}", e))?;

        let list_json = serde_json::to_string(&list).map_err(|e| js_error!("{}", e))?;
        Ok(JsValue::String(
            JsString::from_str(&list_json).map_err(|e| js_error!("{}", e))?,
        ))
    }
}

pub fn register_thirdpartyapp(
    global: &mut ObjectInitializer,
) -> Result<(), String> {
    let jsobj = ObjectInitializer::new(global.context())
        .function(NativeFunction::from_async_fn(launch), js_string!("launchQA"), 2)
        .function(NativeFunction::from_async_fn(get_thirdparty_app_list), js_string!("getThirdPartyAppList"), 0)
        .build();

    global.property(js_string!("thirdpartyapp"), jsobj, Attribute::READONLY);
    
    Ok(())
}